use std::ops::{Deref, DerefMut};
use crate::axis::Offset;
use crate::Position;

mod layout;
pub use layout::{AccessError, AccessResult, CompressLayout, GlobalLayout, Layout};

mod walker;
pub use walker::{
    WalkWith, Walkthrough,
    RectWalker, OffsetWalker,
    PathWalker, Movement, D4Step, D8Step
};

pub struct Grid<L> {
    layout: L
}

impl <L> Grid<L> {
    pub fn with_layout(layout: L) -> Self {
        Self { layout }
    }

    fn get_offset_in_grid(&self, position: &Position) -> AccessResult<Offset> {
        Offset::try_from(*position)
            .map_err(|_| AccessError::CannotAccess(Offset::new(-1, -1)))
    }

    // CURD
    pub fn get(&self, pos: &Position) -> AccessResult<&L::Item>
    where
        L: Layout
    {
        self.layout.get(&self.get_offset_in_grid(pos)?)
    }

    pub fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut L::Item>
    where
        L: Layout
    {
        self.layout.get_mut(&self.get_offset_in_grid(pos)?)
    }

    pub fn set(&mut self, pos: &Position, item: L::Item) -> AccessResult<Option<L::Item>>
    where
        L: Layout
    {
        self.layout.set(&self.get_offset_in_grid(pos)?, item)
    }

    pub fn rmv(&mut self, pos: &Position) -> AccessResult<Option<L::Item>>
    where
        L: Layout
    {
        self.layout.rmv(&self.get_offset_in_grid(pos)?)
    }

    pub fn walkthrough<W>(&self, walker: W) -> Walkthrough<'_, W, L> {
        Walkthrough::new(walker, &self.layout)
    }

    pub fn walk_with<W>(&mut self, walker: W) -> WalkWith<'_, W, L>
    {
        WalkWith::new(walker, &mut self.layout)
    }
}

impl <L> Deref for Grid<L> {
    type Target = L;
    fn deref(&self) -> &Self::Target {
        &self.layout
    }
}

impl <L> DerefMut for Grid<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.layout
    }
}