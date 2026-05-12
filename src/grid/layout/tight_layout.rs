use crate::axis::{MajoredRect, Offset, Rect};
use crate::grid::layout::{AccessError, AccessResult};
use std::iter::repeat_with;
use std::vec;

pub struct TightLayout<T> {
    data: Vec<Option<T>>,
    rect: MajoredRect
}

impl <T> TightLayout<T> {

    /// Take data to build layout
    pub fn with_rect_and_data(rect: MajoredRect, mut data: Vec<Option<T>>) -> Self {
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

    pub fn with_rect_and_src(rect: MajoredRect, src: impl FnMut() -> Option<T>) -> Self {
        Self {
            data: repeat_with(src).take(rect.size()).collect(),
            rect
        }
    }

    pub fn with_default(width: usize, height: usize) -> Self
    where
        T: Default
    {
        Self::with_rect_and_src(
            MajoredRect::new_row(width, height),
            || Some(Default::default())
        )
    }

    pub fn with_elem(width: usize, height: usize, elem: T) -> Self
    where
        T: Clone
    {
        Self::with_rect_and_src(
            MajoredRect::new_row(width, height),
            || Some(elem.clone())
        )
    }

    pub fn fill_with(&mut self, mut f: impl FnMut(Offset) -> T) {
        self.data.iter_mut()
            .enumerate()
            // only care about those empty slot
            .filter(|(_, slot)| slot.is_none())
            .for_each(|(index, slot)| {
                if let Some(offset) = self.rect.fold_majored(index) {
                    let fill_elem = f(offset);
                    let _ = slot.insert(fill_elem);
                }
            })
    }

    pub fn fill_default(&mut self)
    where
        T: Default
    {
        self.fill_with(|_| Default::default())
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
            MajoredRect::RowMajored(rect),
            || None
        )
    }

    pub(crate) fn get_rect(&self) -> &Rect {
        &self.rect
    }

    fn map_data_index(&self, offset: &Offset) -> AccessResult<usize> {
        self.rect.flat_majored(offset).ok_or(AccessError::CannotAccess(*offset))
    }

    // CURD
    pub fn get(&self, offset: &Offset) -> AccessResult<&T> {
        let index = self.map_data_index(offset)?;
        self.data[index].as_ref().ok_or(AccessError::NoValue(*offset))
    }

    pub fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut T> {
        let index = self.map_data_index(offset)?;
        self.data[index].as_mut().ok_or(AccessError::NoValue(*offset))
    }

    pub fn set(&mut self, offset: &Offset, item: T) -> AccessResult<Option<T>> {
        let index = self.map_data_index(offset)?;
        Ok(self.data[index].replace(item))
    }

    pub fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<T>> {
        let index = self.map_data_index(offset)?;
        Ok(self.data[index].take())
    }
}

pub struct IntoIter<T> {
    remaining: vec::IntoIter<Option<T>>,
    stored_next: Option<Option<T>>,
    consumed: usize,
    rect: MajoredRect
}

impl <T> IntoIter<T> {
    fn store_next_one(&mut self) -> Option<Option<T>> {
        if let Some(slot) = self.remaining.next() {
            self.stored_next.replace(slot)
        } else {
            None
        }
    }

    fn store_next_some(&mut self) {
        while matches!(self.stored_next, None | Some(None)) {
            // 这里主要是区分None和Some(None)
            // None表示stored中的值要么从未存在，要么已经计算过consumed了，不能重复计算
            // 其他的Some(None)，表示从remaining取得了None值，但是又消耗掉了，要计算consumed值
            if let Some(_) = self.store_next_one() {
                self.consumed += 1;
            }
        }
    }

    fn consume_stored(&mut self) -> Option<Option<T>> {
        match self.stored_next.take() {
            Some(stored) => {
                self.consumed += 1;
                Some(stored)
            },
            None => None
        }
    }
}

impl <T> Iterator for IntoIter<T> {
    type Item = (Offset, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.store_next_some();
        match self.consume_stored() {
            Some(slot) => Some((
                // 这里consumed值域在[1, rect.size()], 要-1变成0开始的下标
                self.rect.fold_majored(self.consumed - 1)?,
                slot?
            )),
            None => None
        }
    }
}

impl <T> IntoIterator for TightLayout<T> {
    type Item = (Offset, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remaining: self.data.into_iter(),
            rect: self.rect,
            consumed: 0,
            stored_next: None
        }
    }
}