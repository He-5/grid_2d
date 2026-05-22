use std::ops::{Index, IndexMut};
use crate::{Position, Rect, RectWalker, Walker};

mod layout;
pub use layout::{
    AccessError,
    AccessResult,
    Layout,
    LooseLayout,
    TightLayout
};

pub mod walker;
use walker::{WalkWith, Walkthrough};
use crate::axis::rect::RectBounded;

pub struct Grid<L> {
    layout: L,
    bound_rect: Rect
}

impl <L> Grid<L> {
    pub fn with_rect(rect: Rect) -> Self
    where
        L: From<Rect>
    {
        let layout = L::from(rect.clone());
        unsafe { Self::with_layout_and_rect(layout, rect) }
    }

    pub fn with_layout(layout: L) -> Self
    where
        L: RectBounded
    {
        let bound = *layout.boundary();
        unsafe { Self::with_layout_and_rect(layout, bound) }
    }

    pub unsafe fn with_layout_and_rect(layout: L, bound_rect: Rect) -> Self {
        Self { layout, bound_rect }
    }

    pub fn walkthrough<W>(&self, walker: W) -> Walkthrough<'_, W, Self>
    where
        W: Walker
    {
        Walkthrough::new(walker, self)
    }

    pub fn scan_through(&self) -> Walkthrough<'_, RectWalker, Self> {
        self.walkthrough(RectWalker::new(self.bound_rect))
    }

    pub fn walk_with<W>(&mut self, walker: W) -> WalkWith<'_, W, Self>
    where
        W: Walker
    {
        WalkWith::new(walker, self)
    }

    pub fn scan_with(&mut self) -> WalkWith<'_, RectWalker, Self> {
        self.walk_with(RectWalker::new(self.bound_rect))
    }
}

impl <L> From<Rect> for Grid<L>
where
    L: From<Rect>
{
    fn from(value: Rect) -> Self {
        Self::with_rect(value)
    }
}

impl <L> Layout for Grid<L>
where
    L: Layout
{
    type Item = L::Item;
    fn get(&self, pos: &Position) -> AccessResult<&L::Item> {
        self.layout.get(pos)
    }
    fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut L::Item>
    {
        self.layout.get_mut(pos)
    }
    fn set(&mut self, pos: &Position, item: L::Item) -> AccessResult<Option<L::Item>>
    {
        self.layout.set(pos, item)
    }
    fn rmv(&mut self, pos: &Position) -> AccessResult<Option<L::Item>>
    {
        self.layout.rmv(pos)
    }
    fn has(&self, pos: &Position) -> bool {
        self.layout.has(pos)
    }
}

impl <L> RectBounded for Grid<L> {
    fn boundary(&self) -> &Rect {
        &self.bound_rect
    }
}

impl <L> Index<&Position> for Grid<L>
where
    L: Layout
{
    type Output = L::Item;
    fn index(&self, index: &Position) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl <L> IndexMut<&Position> for Grid<L>
where
    L: Layout
{
    fn index_mut(&mut self, index: &Position) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}