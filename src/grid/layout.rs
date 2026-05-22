mod loose_layout;
pub use loose_layout::LooseLayout;
mod tight_layout;
use crate::Position;
pub use tight_layout::TightLayout;

#[derive(Debug)]
pub enum AccessError {
    CannotAccess(Position),
    NoValue(Position)
}

pub type AccessResult<T> = Result<T, AccessError>;

pub trait Layout {
    type Item;
    
    fn get(&self, pos: &Position) -> AccessResult<&Self::Item>;
    fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut Self::Item>;
    fn set(&mut self, pos: &Position, item: Self::Item) -> AccessResult<Option<Self::Item>>;
    fn rmv(&mut self, pos: &Position) -> AccessResult<Option<Self::Item>>;

    fn has(&self, pos: &Position) -> bool {
        self.get(pos).is_ok()
    }
}