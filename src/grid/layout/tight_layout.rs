use crate::axis::rect::Rect;
use crate::grid::layout::{AccessError, AccessResult, Layout};
use crate::{Position, RectBounded};
use std::iter::{repeat_with, Enumerate};
use std::ops::ControlFlow;
use std::vec;

pub struct TightLayout<T> {
    data: Vec<Option<T>>,
    rect: Rect
}

impl <T> TightLayout<T> {

    /// Take data to build layout
    pub fn with_rect_and_data(rect: Rect, mut data: Vec<Option<T>>) -> Self {
        let rect_size = rect.size();
        let data_len = data.len();
        if rect_size < data_len {
            // this will drop rest value, only keep first rect_size elements
            // it will not cause capacity shrink
            data.truncate(rect_size);
        } else if rect_size > data_len {
            // if capacity is enough, this calling will do nothing
            data.reserve_exact(rect_size - data_len);
            // extend the collection to rect_size
            data.extend(repeat_with(|| None).take(rect_size - data_len));
        };
        // now data has exactly same length as rect_size
        Self { data, rect }
    }

    pub fn with_rect_and_src(rect: Rect, src: impl FnMut() -> Option<T>) -> Self {
        let mut data = Vec::with_capacity(rect.size());
        data.extend(repeat_with(src).take(rect.size()));
        Self::with_rect_and_data(rect, data)
    }

    pub fn with_default(width: usize, height: usize) -> Self
    where
        T: Default
    {
        Self::with_rect_and_src(
            Rect::new(width, height),
            || Some(Default::default())
        )
    }

    pub fn with_elem(width: usize, height: usize, elem: T) -> Self
    where
        T: Clone
    {
        Self::with_rect_and_src(
            Rect::new(width, height),
            || Some(elem.clone())
        )
    }

    pub fn fill_with(&mut self, mut f: impl FnMut(Position) -> T) {
        self.data.iter_mut()
            .enumerate()
            .for_each(|(index, slot)| {
                if let Some(offset) = self.rect.fold_x_first(index) {
                    let fill_elem = f(offset);
                    let _ = slot.insert(fill_elem);
                }
            })
    }

    pub fn fill(&mut self, elem: T)
    where
        T: Clone
    {
        self.fill_with(|_| elem.clone())
    }

    pub fn new(width: usize, height: usize) -> Self
    {
        Self::with_rect(
            Rect::new(width, height)
        )
    }

    pub fn with_rect(rect: Rect) -> Self
    {
        Self::with_rect_and_src(
            rect,
            || None
        )
    }

    pub fn get_rect(&self) -> &Rect {
        &self.rect
    }

    fn map_data_index(&self, pos: &Position) -> AccessResult<usize> {
        self.rect.flatten_x_first(pos).ok_or(AccessError::CannotAccess(*pos))
    }
}

impl <T> From<Rect> for TightLayout<T> {
    fn from(value: Rect) -> Self {
        Self::with_rect(value)
    }
}

impl <T> Layout for TightLayout<T> {
    type Item = T;
    fn get(&self, pos: &Position) -> AccessResult<&Self::Item> {
        let index = self.map_data_index(pos)?;
        self.data[index].as_ref().ok_or(AccessError::NoValue(*pos))
    }

    fn get_mut(&mut self, pos: &Position) -> AccessResult<&mut Self::Item> {
        let index = self.map_data_index(pos)?;
        self.data[index].as_mut().ok_or(AccessError::NoValue(*pos))
    }

    fn set(&mut self, pos: &Position, item: Self::Item) -> AccessResult<Option<Self::Item>> {
        let index = self.map_data_index(pos)?;
        Ok(self.data[index].replace(item))
    }

    fn rmv(&mut self, pos: &Position) -> AccessResult<Option<Self::Item>> {
        let index = self.map_data_index(pos)?;
        Ok(self.data[index].take())
    }
}

impl <T> RectBounded for TightLayout<T> {
    fn boundary(&self) -> &Rect {
        &self.rect
    }
}

pub struct IntoIter<T> {
    remaining: Enumerate<vec::IntoIter<Option<T>>>,
    rect: Rect
}

impl <T> Iterator for IntoIter<T> {
    type Item = (Position, T);
    fn next(&mut self) -> Option<Self::Item> {
        let (index, value) = self.remaining.try_fold((), |_, remains| match remains {
            (index, Some(value)) => ControlFlow::Break((index, value)),
            // skip all None slots
            _ => ControlFlow::Continue(())
        }).break_value()?;
        Some((self.rect.fold_x_first(index)?, value))
    }
}

impl <T> IntoIterator for TightLayout<T> {
    type Item = (Position, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remaining: self.data.into_iter().enumerate(),
            rect: self.rect
        }
    }
}