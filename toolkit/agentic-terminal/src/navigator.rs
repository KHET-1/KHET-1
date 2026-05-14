//! Tool list navigation backed by nucleo.

use std::collections::HashMap;
use std::sync::Arc;

use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config as NucleoConfig, Matcher, Nucleo, Utf32String};
use ratatui::widgets::ListState;

use crate::types::Tool;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Match {
    pub tool_idx: usize,
    pub score: u32,
}

pub struct ToolNavigator {
    nucleo: Nucleo<Tool>,
    tools: Vec<Tool>,
    name_index: HashMap<String, usize>,
    matches: Vec<Match>,
    pub list_state: ListState,
}

impl ToolNavigator {
    pub fn new(tools: Vec<Tool>) -> Self {
        let name_index = tools
            .iter()
            .enumerate()
            .map(|(i, t)| (t.name.clone(), i))
            .collect::<HashMap<_, _>>();

        let notify = Arc::new(|| {});
        let mut nucleo = Nucleo::new(NucleoConfig::DEFAULT, notify, Some(1), 1);
        let injector = nucleo.injector();
        for t in tools.iter().cloned() {
            injector.push(t, |tool, cols| {
                cols[0] = Utf32String::from(tool.name.as_str());
            });
        }
        nucleo.pattern.reparse(0, "", CaseMatching::Smart, Normalization::Smart, false);
        let mut nav = Self {
            nucleo,
            tools,
            name_index,
            matches: Vec::new(),
            list_state: ListState::default(),
        };
        nav.refresh_matches(/* reset_selection_top */ true);
        nav
    }

    fn rebuild_matches_snapshot(&mut self) {
        let snap = self.nucleo.snapshot();
        let count = snap.matched_item_count() as usize;
        self.matches.clear();
        self.matches.reserve(count);

        let mut matcher = Matcher::new(NucleoConfig::DEFAULT);
        for i in 0..snap.matched_item_count() {
            let Some(item) = snap.get_matched_item(i) else {
                continue;
            };
            let Some(&tool_idx) = self.name_index.get(item.data.name.as_str()) else {
                continue;
            };
            let score = snap
                .pattern()
                .score(item.matcher_columns, &mut matcher)
                .unwrap_or(0);
            self.matches.push(Match { tool_idx, score });
        }
    }

    pub fn refresh_matches(&mut self, reset_selection_top: bool) {
        loop {
            let status = self.nucleo.tick(10);
            self.rebuild_matches_snapshot();
            if reset_selection_top {
                self.pick_top_or_none();
            } else {
                self.stabilize_selection();
            }

            if !status.running {
                break;
            }
        }
    }

    fn pick_top_or_none(&mut self) {
        if self.matches.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn stabilize_selection(&mut self) {
        let len = self.matches.len();
        if len == 0 {
            self.list_state.select(None);
            return;
        }
        match self.list_state.selected() {
            Some(i) if i < len => {}
            Some(i) => self.list_state.select(Some(i.min(len.saturating_sub(1)))),
            None => self.list_state.select(Some(0)),
        }
    }

    pub fn set_query(&mut self, query: &str) {
        let trimmed = query.trim();
        let pat = if trimmed.is_empty() { "" } else { query };
        self.nucleo.pattern.reparse(0, pat, CaseMatching::Smart, Normalization::Smart, false);
        self.refresh_matches(/* reset_selection_top */ true);
    }

    pub fn tick_frame(&mut self) {
        self.refresh_matches(false);
    }

    pub fn selected_tool(&self) -> Option<&Tool> {
        let i = self.list_state.selected()?;
        let idx = self.matches.get(i)?.tool_idx;
        self.tools.get(idx)
    }

    pub fn next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.matches.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new = current.saturating_sub(page_size);
        self.list_state.select(Some(new));
    }

    pub fn page_down(&mut self, page_size: usize) {
        if self.matches.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let max = self.matches.len().saturating_sub(1);
        let new = (current + page_size).min(max);
        self.list_state.select(Some(new));
    }

    pub fn selected_score(&self) -> Option<u32> {
        let i = self.list_state.selected()?;
        self.matches.get(i).map(|m| m.score)
    }

    pub fn filtered_len(&self) -> usize {
        self.matches.len()
    }

    pub fn list_state_offset(&self) -> usize {
        self.list_state.offset()
    }
}
