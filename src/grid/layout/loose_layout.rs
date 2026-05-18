//! 最宽松的存储布局，存储离散数据

use crate::axis::{Offset, Rect};
use crate::grid::layout::{AccessError, AccessResult, Layout};
use std::cmp::max;
use std::collections::{hash_map, HashMap};

pub struct LooseLayout<T> {
    data_map: HashMap<Offset, T>,
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

    pub fn get_rect(&self) -> &Rect {
        &self.rect
    }

    fn contains_check(&self, offset: &Offset) -> AccessResult<()> {
        if self.rect.contains_offset(offset) {
            Ok(())
        } else {
            Err(AccessError::CannotAccess(*offset))
        }
    }
}

impl <T> Layout for LooseLayout<T> {
    type Item = T;

    fn get(&self, offset: &Offset) -> AccessResult<&Self::Item> {
        self.contains_check(offset)?;
        self.data_map.get(offset).ok_or(AccessError::NoValue(*offset))
    }

    fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut Self::Item> {
        self.contains_check(offset)?;
        self.data_map.get_mut(offset).ok_or(AccessError::NoValue(*offset))
    }

    fn set(&mut self, offset: &Offset, item: Self::Item) -> AccessResult<Option<Self::Item>> {
        self.contains_check(offset)?;
        Ok(self.data_map.insert(*offset, item))
    }

    fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<Self::Item>> {
        self.contains_check(offset)?;
        Ok(self.data_map.remove(offset))
    }
}

pub struct IntoIter<T> {
    remaining: hash_map::IntoIter<Offset, T>,
}

impl <T> Iterator for IntoIter<T> {
    type Item = (Offset, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.next()
    }
}

impl <T> IntoIterator for LooseLayout<T> {
    type Item = (Offset, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remaining: self.data_map.into_iter()
        }
    }
}
