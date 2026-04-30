use crate::axis::Offset;
use crate::grid::layout::{AccessError, AccessResult, GlobalLayout};
use crate::{Position, Rect};

mod layout;

pub struct Grid<T> {
    layout: GlobalLayout<T>
}

impl <T> Grid<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            layout: GlobalLayout::new(width, height)
        }
    }

    pub fn with_default(width: usize, height: usize) -> Self
    where
        T: Default
    {
        Self {
            layout: GlobalLayout::with_default(width, height)
        }
    }

    pub fn get_rect(&self) -> &Rect {
        self.layout.get_rect()
    }

    fn get_offset_in_grid(&self, position: &Position) -> AccessResult<Offset> {
        let offset = Offset::try_from(*position)
            .map_err(|_| AccessError::CannotAccess(Offset::new(-1, -1)))?;
        if self.layout.get_rect().contains_offset(&offset) {
            Ok(offset)
        } else {
            Err(AccessError::CannotAccess(offset))
        }
    }

    // CURD
    pub fn get(&self, pos: &Position) -> AccessResult<&T> {
        self.layout.get(&self.get_offset_in_grid(pos)?)
    }

    pub fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut T> {
        self.layout.get_mut(&self.get_offset_in_grid(pos)?)
    }

    pub fn set(&mut self, pos: &Position, item: T) -> AccessResult<Option<T>> {
        self.layout.set(&self.get_offset_in_grid(pos)?, item)
    }

    pub fn rmv(&mut self, pos: &Position) -> AccessResult<Option<T>> {
        self.layout.rmv(&self.get_offset_in_grid(pos)?)
    }
}