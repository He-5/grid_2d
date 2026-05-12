//! 压缩存储，仅存储有值的部分

use crate::axis::{Offset, Rect};
use crate::grid::layout::{AccessError, AccessResult, CreateError};
use std::iter::repeat_with;
use std::{mem, vec};
use std::ops::{Range, RangeInclusive};
use crate::some;

enum DataSlot {
    Invalid,
    Free(Offset, /** Map Index */usize),
    Occurred(Offset, /** Data Index */usize)
}

impl DataSlot {
    pub fn get_data_index(&self) -> Option<usize> {
        match self {
            Self::Occurred(_, data_index) => Some(*data_index),
            _ => None
        }
    }

    pub fn get_data_from<'vec, T>(&self, data_container: &'vec Vec<T>) -> Option<&'vec T> {
        data_container.get(self.get_data_index()?)
    }

    pub fn get_mut_data_from<'vec, T>(&self, data_container: &'vec mut Vec<T>) -> Option<&'vec mut T> {
        data_container.get_mut(self.get_data_index()?)
    }
}

pub trait Guarded {
    fn guardian() -> Self;
}

macro_rules! impl_guard_for_int {
    ($($int:ident)*) => {
        $(
        impl Guarded for $int {
            fn guardian() -> Self {
                Self::MAX
            }
        }
        )*
    };
}

impl_guard_for_int! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}

pub trait Compressible<Target>: Sized {
    fn decompress(target: Target) -> Self;

    fn compress(self) -> Option<Target>;

    fn is_compressible(&self) -> bool;
}

pub trait RangeCompressible<Target>: Compressible<Target> {
    fn get_range() -> RangeInclusive<Self>;
}

macro_rules! impl_compressible_for_int {
    ($to_compress:ident => [$($compressed:ident)*]) => {
        $(
        impl Compressible<$compressed> for $to_compress {
            fn decompress(target: $compressed) -> Self {
                target as $to_compress
            }
            fn compress(self) -> Option<$compressed> {
                some! { if <Self as Compressible<$compressed>>::is_compressible(&self) => self as $compressed }
            }
            fn is_compressible(&self) -> bool {
                const MAX: $to_compress = $compressed::MAX as $to_compress;
                const MIN: $to_compress = $compressed::MIN as $to_compress;
                match self {
                    MIN..=MAX => true,
                    _ => false
                }
            }
        }
        impl RangeCompressible<$compressed> for $to_compress {
            fn get_range() -> RangeInclusive<Self> {
                const MAX: $to_compress = $compressed::MAX as $to_compress;
                const MIN: $to_compress = $compressed::MIN as $to_compress;
                MIN..=MAX
            }
        }
        )*
    };
}

impl_compressible_for_int! { usize => [u8 u16] }
#[cfg(target_pointer_width = "64")]
impl_compressible_for_int! { usize => [u32] }

pub struct CompressLayout<T, I> {
    data_store: Vec<(I, T)>,
    index_map: Vec<I>,
    rect: Rect
}

impl <T, I> CompressLayout<T, I> {
    pub fn is_full(&self) -> bool
    {
        self.rect.size() <= self.data_store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data_store.is_empty()
    }

    unsafe fn new_unchecked(rect: Rect) -> Self
    where
        I: Guarded
    {
        Self {
            data_store: Vec::new(),
            index_map: repeat_with(I::guardian).take(rect.size()).collect(),
            rect
        }
    }

    pub fn new(rect: Rect) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        let width = rect.get_width();
        let height = rect.get_height();
        if width == 0 || height == 0 {
            return Err(CreateError::InvalidSize(0));
        }
        match width.checked_mul(height) {
            None => Err(CreateError::InvalidShape(rect)),
            // SAFETY: parts above add validate of rect.size and rect shape,
            // which make it's safe for unchecked creation
            // the size - 1 is for 0-based index
            Some(size) if (size - 1).is_compressible() => Ok(unsafe { Self::new_unchecked(rect) }),
            Some(invalid_size) => Err(CreateError::InvalidSize(invalid_size))
        }
    }

    // internal mapping
    fn get_slot(&self, offset: &Offset) -> DataSlot
    where
        usize: Compressible<I>,
        I: Clone
    {
        let map_index =
            match self.rect.flatten_x_first(offset) {
                Some(index) => index,
                None => { return DataSlot::Invalid; }
            };
        match usize::decompress(self.index_map[map_index].clone()) {
            index if index < self.data_store.len() => DataSlot::Occurred(*offset, index),
            // free must assert data_store is not full at this time
            _ if !self.is_full() => DataSlot::Free(*offset, map_index),
            _ => DataSlot::Invalid
        }
    }

    /// Read from layout
    pub fn get(&self, offset: &Offset) -> AccessResult<&T>
    where
        I: Clone,
        usize: Compressible<I>
    {
        match self.get_slot(offset) {
            DataSlot::Invalid => Err(AccessError::CannotAccess(*offset)),
            DataSlot::Free(offset, ..) => Err(AccessError::NoValue(offset)),
            DataSlot::Occurred(offset, data_index) =>
                match self.data_store.get(data_index) {
                    Some((_, item)) => Ok(item),
                    None => Err(AccessError::NoValue(offset))
                }
        }
    }

    /// Get mut ref from layout
    pub fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut T>
    where
        I: Clone,
        usize: Compressible<I>
    {
        match self.get_slot(offset) {
            DataSlot::Invalid => Err(AccessError::CannotAccess(*offset)),
            DataSlot::Free(offset, ..) => Err(AccessError::NoValue(offset)),
            DataSlot::Occurred(offset, data_index) =>
                match self.data_store.get_mut(data_index) {
                    Some((_, item)) => Ok(item),
                    None => Err(AccessError::NoValue(offset))
                }
        }
    }

    /// Set item and replaced exist item(if any)
    pub fn set(&mut self, offset: &Offset, item: T) -> AccessResult<Option<T>>
    where
        I: Clone,
        usize: Compressible<I>
    {
        match self.get_slot(offset) {
            DataSlot::Invalid => Err(AccessError::CannotAccess(*offset)),
            DataSlot::Occurred(_, data_index) => {
                let (_, stored) = &mut self.data_store[data_index];
                Ok(Some(mem::replace(stored, item)))
            }
            DataSlot::Free(_, map_index) => {
                let compressed = self.data_store.len().compress().expect("fail to compress index");
                self.data_store.push((compressed.clone(), item));
                self.index_map[map_index] = compressed;
                Ok(None)
            }
        }
    }

    /// Remove the item at offset
    pub fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<T>>
    where
        I: Clone + Guarded,
        usize: Compressible<I>
    {
        match self.get_slot(offset) {
            DataSlot::Invalid => Err(AccessError::CannotAccess(*offset)),
            DataSlot::Free(..) => Ok(None),
            DataSlot::Occurred(_, data_index) => {
                let (compressed_index, removed_data) = self.data_store.swap_remove(data_index);
                // clear map_index stored
                self.index_map[usize::decompress(compressed_index)] = I::guardian();
                if data_index < self.data_store.len() { // index validate
                    let (saved_index, _) = &self.data_store[data_index];
                    // TODO: add extra error and roll-back instead of panic
                    self.index_map[usize::decompress(saved_index.clone())] = data_index.compress().expect("fail to compress index");
                }
                Ok(Some(removed_data))
            }
        }
    }
}

pub struct IntoIter<T, I> {
    remaining: vec::IntoIter<(I, T)>,
    rect: Rect
}

impl <T, I> Iterator for IntoIter<T, I>
where
    usize: Compressible<I>
{
    type Item = (Offset, T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining.next() {
            Some((compressed_index, item)) => {
                self.rect.fold_x_first(usize::decompress(compressed_index))
                    .map(|offset| (offset, item))
            },
            None => None
        }
    }
}

impl <T, I> IntoIterator for CompressLayout<T, I>
where
    usize: Compressible<I>
{
    type Item = (Offset, T);
    type IntoIter = IntoIter<T, I>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remaining: self.data_store.into_iter(),
            rect: self.rect
        }
    }
}
