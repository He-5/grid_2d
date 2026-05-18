use crate::axis::{Offset, Rect};
use crate::grid::layout::{AccessError, AccessResult, Layout};
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

    pub fn fill_with(&mut self, mut f: impl FnMut(Offset) -> T) {
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

    fn map_data_index(&self, offset: &Offset) -> AccessResult<usize> {
        self.rect.flatten_x_first(offset).ok_or(AccessError::CannotAccess(*offset))
    }
}

impl <T> Layout for TightLayout<T> {
    type Item = T;
    fn get(&self, offset: &Offset) -> AccessResult<&Self::Item> {
        let index = self.map_data_index(offset)?;
        self.data[index].as_ref().ok_or(AccessError::NoValue(*offset))
    }

    fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut Self::Item> {
        let index = self.map_data_index(offset)?;
        self.data[index].as_mut().ok_or(AccessError::NoValue(*offset))
    }

    fn set(&mut self, offset: &Offset, item: Self::Item) -> AccessResult<Option<Self::Item>> {
        let index = self.map_data_index(offset)?;
        Ok(self.data[index].replace(item))
    }

    fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<Self::Item>> {
        let index = self.map_data_index(offset)?;
        Ok(self.data[index].take())
    }
}

pub struct IntoIter<T> {
    remaining: Enumerate<vec::IntoIter<Option<T>>>,
    rect: Rect
}

impl <T> Iterator for IntoIter<T> {
    type Item = (Offset, T);
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
    type Item = (Offset, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remaining: self.data.into_iter().enumerate(),
            rect: self.rect
        }
    }
}