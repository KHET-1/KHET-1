//! Bounded crossbeam channels + filter work off the TUI thread.

use crossbeam_channel::{Receiver, Sender};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};
use std::cmp::Reverse;
use std::thread;
use std::thread::JoinHandle;

use crate::model::ToolId;

const CHANNEL_CAP: usize = 32;

#[derive(Debug, Clone)]
pub struct FilterJob {
    pub epoch: u64,
    pub query: String,
    /// `(id, fuzzy haystack)` — stable id preserved across re-ranks.
    pub items: Vec<(ToolId, String)>,
}

#[derive(Debug, Clone)]
pub struct FilterResult {
    pub epoch: u64,
    pub ordered_ids: Vec<ToolId>,
}

/// Owning handle: dropping the sender ends the worker `recv` loop; [`shutdown`](Self::shutdown) joins the thread.
pub struct FilterWorker {
    job_tx: Sender<FilterJob>,
    pub result_rx: Receiver<FilterResult>,
    join: JoinHandle<()>,
}

impl FilterWorker {
    pub fn job_tx(&self) -> &Sender<FilterJob> {
        &self.job_tx
    }

    pub fn shutdown(self) -> thread::Result<()> {
        drop(self.job_tx);
        self.join.join()
    }
}

pub fn spawn_filter_worker() -> FilterWorker {
    let (job_tx, job_rx): (Sender<FilterJob>, Receiver<FilterJob>) =
        crossbeam_channel::bounded(CHANNEL_CAP);
    let (res_tx, res_rx): (Sender<FilterResult>, Receiver<FilterResult>) =
        crossbeam_channel::bounded(CHANNEL_CAP);

    let join = thread::spawn(move || {
        let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
        let mut buf = Vec::new();

        while let Ok(job) = job_rx.recv() {
            let ordered = rank(&job.query, &job.items, &mut matcher, &mut buf);
            let _ = res_tx.send(FilterResult {
                epoch: job.epoch,
                ordered_ids: ordered,
            });
        }
    });

    FilterWorker {
        job_tx,
        result_rx: res_rx,
        join,
    }
}

fn rank(
    query: &str,
    items: &[(ToolId, String)],
    matcher: &mut Matcher,
    buf: &mut Vec<char>,
) -> Vec<ToolId> {
    if query.is_empty() {
        return items.iter().map(|(id, _)| *id).collect();
    }

    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    if pattern.atoms.is_empty() {
        return items.iter().map(|(id, _)| *id).collect();
    }

    let mut scored: Vec<(ToolId, u32)> = items
        .iter()
        .filter_map(|(id, text)| {
            pattern
                .score(Utf32Str::new(text.as_str(), buf), matcher)
                .map(|score| (*id, score))
        })
        .collect();

    scored.sort_by_key(|(_, score)| Reverse(*score));
    scored.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_empty_query_returns_stable_order() {
        let mut m = Matcher::new(nucleo_matcher::Config::DEFAULT);
        let mut buf = Vec::new();
        let items = vec![(1u32, "git".into()), (2u32, "cargo".into())];
        let out = rank("", &items, &mut m, &mut buf);
        assert_eq!(out, vec![1, 2]);
    }

    #[test]
    fn worker_shutdown_joins() {
        let w = spawn_filter_worker();
        w.shutdown().unwrap();
    }
}
