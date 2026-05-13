use std::borrow::Borrow;
use super::{Offset, Position};

mod rect;

pub use rect::{Rect, RectZone, MajoredRect};
pub(crate) use rect::chunking;

/// Zone stand for an area which provide:
/// - `anchor`: an absolute Position of Grid
/// - size hint: provide optional size hint, it stands for a total num of positions contains inside
/// - contains check: check a specific position is inside Current zone
pub trait Zone {
    fn get_anchor(&self) -> Position;
    fn contains_offset(&self, query: &impl Borrow<Offset>) -> bool;
    fn size_hint(&self) -> Option<usize> { None }
    // default impls
    fn contains_position(&self, query: &impl Borrow<Position>) -> bool {
        self.contains_offset(&(self.get_anchor() - *query.borrow()))
    }
}