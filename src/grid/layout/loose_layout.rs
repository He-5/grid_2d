//! 最宽松的存储布局，存储离散数据

use crate::grid::layout::{AccessError, AccessResult, Layout};
use std::cmp::max;
use std::collections::{hash_map, HashMap};
use crate::axis::rect::Rect;
use crate::Position;

impl <T> Layout for HashMap<Position, T> {
    type Item = T;
    fn get(&self, pos: &Position) -> AccessResult<&Self::Item> {
        self.get(pos).ok_or(AccessError::NoValue(*pos))
    }

    fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut Self::Item> {
        self.get_mut(pos).ok_or(AccessError::NoValue(*pos))
    }

    fn set(&mut self, pos: &Position, item: Self::Item) -> AccessResult<Option<Self::Item>> {
        Ok(self.insert(*pos, item))
    }

    fn rmv(&mut self, pos: &Position) -> AccessResult<Option<Self::Item>> {
        Ok(self.remove(pos))
    }

    fn has(&self, pos: &Position) -> bool {
        self.contains_key(pos)
    }
}

pub struct LooseLayout<T> {
    data_map: HashMap<Position, T>,
    rect: Rect
}

impl <T> LooseLayout<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_rect(Rect::new(width, height))
    }

    pub fn with_rect(rect: Rect) -> Self {
        Self {
            data_map: HashMap::with_capacity(max(rect.size() >> 1, 10)),
            rect
        }
    }

    pub fn unbound(self) -> HashMap<Position, T> {
        self.data_map
    }

    fn contains_check(&self, pos: &Position) -> AccessResult<()> {
        if self.rect.contains_pos(pos) {
            Ok(())
        } else {
            Err(AccessError::CannotAccess(*pos))
        }
    }
}

impl <T> Layout for LooseLayout<T> {
    type Item = T;

    fn get(&self, pos: &Position) -> AccessResult<&Self::Item> {
        self.contains_check(pos)?;
        Layout::get(&self.data_map, pos)
    }

    fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut Self::Item> {
        self.contains_check(pos)?;
        Layout::get_mut(&mut self.data_map, pos)
    }

    fn set(&mut self, pos: &Position, item: Self::Item) -> AccessResult<Option<Self::Item>> {
        self.contains_check(pos)?;
        Layout::set(&mut self.data_map, pos, item)
    }

    fn rmv(&mut self, pos: &Position) -> AccessResult<Option<Self::Item>> {
        self.contains_check(pos)?;
        Layout::rmv(&mut self.data_map, pos)
    }
}

impl <T> From<Rect> for LooseLayout<T> {
    fn from(value: Rect) -> Self {
        Self::with_rect(value)
    }
}

pub struct IntoIter<T> {
    remaining: hash_map::IntoIter<Position, T>,
}

impl <T> Iterator for IntoIter<T> {
    type Item = (Position, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.next()
    }
}

impl <T> IntoIterator for LooseLayout<T> {
    type Item = (Position, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remaining: self.data_map.into_iter()
        }
    }
}
