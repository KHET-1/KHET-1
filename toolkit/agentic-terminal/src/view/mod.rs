//! Focus stack rendering and input routing (`View trait`).

use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::Frame;

pub mod palette;
pub mod search;

pub(crate) fn point_in_rect(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

pub(crate) fn inner_rect(rect: Rect) -> Rect {
    if rect.width <= 2 || rect.height <= 2 {
        return Rect::default();
    }
    Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width - 2,
        height: rect.height - 2,
    }
}

pub enum ViewResult {
    Consumed,
    Pop,
    Push(Box<dyn View>),
    Quit,
}

pub trait View: Send {
    fn title(&self) -> &'static str;

    fn on_event(&mut self, event: &Event, ctx: &mut crate::app::ViewCtx<'_>) -> ViewResult;

    fn render(&mut self, area: Rect, frame: &mut Frame<'_>, ctx: &mut crate::app::ViewCtx<'_>);

    fn layers_as_overlay(&self) -> bool {
        false
    }
}

pub struct ViewStack {
    layers: Vec<Box<dyn View>>,
}

impl Default for ViewStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewStack {
    #[must_use]
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    #[must_use]
    pub fn with_base(view: Box<dyn View>) -> Self {
        Self { layers: vec![view] }
    }

    pub fn push(&mut self, view: Box<dyn View>) {
        self.layers.push(view);
    }

    pub fn top_mut(&mut self) -> Option<&mut (dyn View + '_)> {
        let layer = self.layers.last_mut()?;
        Some(Box::as_mut(layer))
    }

    pub fn apply_outcome(&mut self, outcome: ViewResult) {
        match outcome {
            ViewResult::Consumed | ViewResult::Quit => {}
            ViewResult::Pop => {
                let _ = self.layers.pop();
            }
            ViewResult::Push(v) => {
                self.layers.push(v);
            }
        }
    }

    #[must_use]
    pub(crate) fn has_overlay_top(&self) -> bool {
        self.layers
            .last()
            .is_some_and(|layer| View::layers_as_overlay(layer.as_ref()))
    }

    pub(crate) fn render_all(
        &mut self,
        area: Rect,
        frame: &mut Frame<'_>,
        ctx: &mut crate::app::ViewCtx<'_>,
    ) {
        if self.layers.is_empty() {
            return;
        }

        if self.has_overlay_top() {
            if let Some(base) = self.layers.first_mut() {
                (*base).render(area, frame, ctx);
            }
            for layered in self
                .layers
                .iter_mut()
                .skip(1)
                .filter(|layer| View::layers_as_overlay(layer.as_ref()))
            {
                layered.render(area, frame, ctx);
            }
        } else if let Some(top) = self.layers.last_mut() {
            top.render(area, frame, ctx);
        }
    }
}
