use crate::axis::{chunking, Offset, Rect};
use std::mem;

mod loose_layout;
pub use loose_layout::LooseLayout;
mod compress_layout;
pub use compress_layout::CompressLayout;
mod tight_layout;
pub use tight_layout::TightLayout;
use crate::grid::layout::compress_layout::{Compressible, Guarded, RangeCompressible};

#[derive(Debug)]
pub enum AccessError {
    CannotAccess(Offset),
    NoValue(Offset)
}

pub type AccessResult<T> = Result<T, AccessError>;

#[derive(Debug)]
pub enum CreateError {
    InvalidSize(usize),
    InvalidShape(Rect)
}

pub trait Layout {
    type Item;
    
    fn get(&self, offset: &Offset) -> AccessResult<&Self::Item>;
    fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut Self::Item>;
    fn set(&mut self, offset: &Offset, item: Self::Item) -> AccessResult<Option<Self::Item>>;
    fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<Self::Item>>;

    fn has(&self, offset: &Offset) -> bool {
        self.get(offset).is_ok()
    }
}

/// # Global Layout
///
/// Global Layout is design for a single-alloc area,
/// all location is open for access
/// and any set or rmv will not cause re-alloc in layout-tier(but might happen in inner container)
///
/// use it if you are **NOT** going to manage a heavy task
pub enum GlobalLayout<T> {
    /// see [LooseLayout]
    Loose(LooseLayout<T>),
    /// see [TightLayout]
    Tight(TightLayout<T>)
}

macro_rules! delegate_to_global_layout {
    (|$layout:ident| $block:block by $expr:expr) => {
        match $expr {
            GlobalLayout::Loose($layout) => $block,
            GlobalLayout::Tight($layout) => $block
        }
    };
}

impl <T> GlobalLayout<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Self::Loose(LooseLayout::new(width, height))
    }

    pub fn new_tight(width: usize, height: usize) -> Self {
        Self::Tight(TightLayout::new(width, height))
    }

    pub fn with_default(width: usize, height: usize) -> Self
    where
        T: Default
    {
        Self::Tight(TightLayout::with_default(width, height))
    }

    pub fn with_repeat(width: usize, height: usize, repeated: T) -> Self
    where
        T: Clone
    {
        Self::Tight(TightLayout::with_elem(width, height, repeated))
    }

    /// Tighten Current Layout
    ///
    /// turn current layout into [GlobalLayout::Tight] in-place
    ///
    /// it's a heavy operation with calling set on each exist element
    pub fn tighten(&mut self) {
        let tighten = match self {
            Self::Tight(_) => { return; },
            Self::Loose(internal) => {
                let width = internal.get_rect().get_width();
                let height = internal.get_rect().get_height();
                mem::replace(internal, LooseLayout::new(0, 0))
                    .into_iter()
                    .fold(
                        TightLayout::new(width, height),
                        |mut layout, (ref offset, item)| {
                            let _ = layout.set(offset, item);
                            layout
                        }
                    )
            }
        };
        *self = Self::Tight(tighten);
    }

    /// Loosen Current Layout
    /// 
    /// turn current layout into [GlobalLayout::Loose] in-place
    /// 
    /// it's a heavy operation with calling set on each exist element
    pub fn loosen(&mut self) {
        let loosen = match self {
            Self::Loose(_) => { return; }
            Self::Tight(internal) => {
                let width = internal.get_rect().get_width();
                let height = internal.get_rect().get_height();
                mem::replace(internal, TightLayout::new(0, 0))
                    .into_iter()
                    .fold(
                        LooseLayout::new(width, height),
                        |mut layout, (ref offset, item)| {
                            let _ = layout.set(offset, item);
                            layout
                        }
                    )
            }
        };
        *self = Self::Loose(loosen)
    }

    pub(crate) fn get_rect(&self) -> &Rect {
        delegate_to_global_layout! {
            |layout| { layout.get_rect() } by self
        }
    }

    // CURD
    pub fn get(&self, offset: &Offset) -> AccessResult<&T> {
        delegate_to_global_layout!{
            |layout| { layout.get(offset) } by self
        }
    }

    pub fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut T> {
        delegate_to_global_layout! {
            |layout| { layout.get_mut(offset) } by self
        }
    }

    pub fn set(&mut self, offset: &Offset, item: T) -> AccessResult<Option<T>> {
        delegate_to_global_layout! {
            |layout| { layout.set(offset, item) } by self
        }
    }

    pub fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<T>> {
        delegate_to_global_layout! {
            |layout| { layout.rmv(offset) } by self
        }
    }
}

pub struct ChunkedLayout<T, I> {
    compressed: Compressed<T, I>,
    rect: Rect
}

pub enum Compressed<T, I> {
    Chunked(CompressLayout<TightLayout<T>, I>, Rect),
    Single(CompressLayout<T, I>)
}

fn split_offset_by_rect(offset: &Offset, rect: &Rect) -> Option<(Offset, Offset)>
{
    if offset.get_x() < 0 || offset.get_y() < 0 {
        return None;
    }
    let (abs_x, abs_y) = (offset.get_x() as usize, offset.get_y() as usize);
    let (chunk_width, chunk_height) = (rect.get_width(), rect.get_height());
    let (chunk_x, in_chunk_x) = (abs_x.div_euclid(chunk_width), abs_x.rem_euclid(chunk_width));
    let (chunk_y, in_chunk_y) = (abs_y.div_euclid(chunk_height), abs_y.rem_euclid(chunk_height));
    let chunk_offset = (chunk_x, chunk_y).try_into().ok()?;
    let in_chunk_offset = (in_chunk_x, in_chunk_y).try_into().ok()?;
    Some((chunk_offset, in_chunk_offset))
}

impl <T, I> Compressed<T, I> {
    pub fn new_single(rect: Rect) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        Ok(Self::Single(CompressLayout::new(rect)?))
    }

    pub fn new_chunked(rect: Rect, chunk_rect: Rect) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        Ok(Self::Chunked(CompressLayout::new(rect)?, chunk_rect))
    }

    fn get(&self, offset: &Offset) -> AccessResult<&T>
    where
        I: Clone,
        usize: Compressible<I>
    {
        match self {
            Compressed::Single(layout) => layout.get(offset),
            Compressed::Chunked(layout, chunk) => {
                let (chunk_offset, in_chunk_offset) = split_offset_by_rect(offset, chunk).ok_or(AccessError::NoValue(*offset))?;
                let chunk = layout.get(&chunk_offset)?;
                chunk.get(&in_chunk_offset)
            }
        }
    }

    fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut T>
    where
        I: Clone,
        usize: Compressible<I>
    {
        match self {
            Compressed::Single(layout) => layout.get_mut(offset),
            Compressed::Chunked(layout, chunk) => {
                let (chunk_offset, in_chunk_offset) = split_offset_by_rect(offset, chunk).ok_or(AccessError::NoValue(*offset))?;
                let chunk = layout.get_mut(&chunk_offset)?;
                chunk.get_mut(&in_chunk_offset)
            }
        }
    }

    fn set(&mut self, offset: &Offset, item: T) -> AccessResult<Option<T>>
    where
        I: Clone,
        usize: Compressible<I>
    {
        match self {
            Compressed::Single(layout) => layout.set(offset, item),
            Compressed::Chunked(layout, chunk_rect) => {
                let (chunk_offset, in_chunk_offset) = split_offset_by_rect(offset, chunk_rect).ok_or(AccessError::NoValue(*offset))?;
                match layout.get_mut(&chunk_offset) {
                    Ok(chunk) => chunk.set(&in_chunk_offset, item),
                    Err(AccessError::NoValue(_)) => {
                        let mut new_chunk = TightLayout::with_rect(*chunk_rect);
                        new_chunk.set(&in_chunk_offset, item)?;
                        layout.set(&chunk_offset, new_chunk)?;
                        Ok(None)
                    }
                    Err(err) => Err(err)
                }
            }
        }
    }

    fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<T>>
    where
        I: Clone + Guarded,
        usize: Compressible<I>
    {
        match self {
            Compressed::Single(layout) => layout.rmv(offset),
            Compressed::Chunked(layout, chunk_rect) => {
                let (chunk_offset, in_chunk_offset) = split_offset_by_rect(offset, chunk_rect).ok_or(AccessError::NoValue(*offset))?;
                match layout.get_mut(&chunk_offset) {
                    Ok(chunk) => chunk.rmv(&in_chunk_offset),
                    Err(AccessError::NoValue(_)) => Ok(None),
                    Err(err) => Err(err)
                }
            }
        }
    }
}

impl <T, I> ChunkedLayout<T, I> {
    fn safe_count(total: usize, unit: usize) -> usize {
        if unit == 0 {
            return 0;
        }
        match total.checked_add(unit - 1) {
            Some(safe_sum) => safe_sum / unit,
            _ => {
                let main = total / unit;
                if total % unit != 0 {
                    main + 1
                } else {
                    main
                }
            }
        }
    }

    fn with_chunk_rect_and_rect(chunk_rect: Rect, rect: Rect) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        let compressed =
            match (chunk_rect.get_width(), chunk_rect.get_height()) {
                (0, _) | (_, 0) => { return Err(CreateError::InvalidShape(chunk_rect)); }
                (1, 1) => Compressed::new_single(rect),
                _ => {
                    let chunk_width_count = Self::safe_count(rect.get_width(), chunk_rect.get_width());
                    let chunk_height_count = Self::safe_count(rect.get_height(), chunk_rect.get_height());
                    Compressed::new_chunked(Rect::new(chunk_width_count, chunk_height_count), chunk_rect)
                }
            }?;
        Ok(Self { compressed, rect })
    }

    pub fn new(width: usize, height: usize) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: RangeCompressible<I>
    {
        let (_, max_index) = usize::get_range().into_inner();
        let chunk_rect = chunking(width, height, max_index + 1)
            .ok_or(CreateError::InvalidShape(Rect::new(width, height)))?;
        Self::with_chunk_rect_and_rect(chunk_rect, Rect::new(width, height))
    }

    // CURD
    pub fn get(&self, offset: &Offset) -> AccessResult<&T>
    where
        I: Clone,
        usize: Compressible<I>
    {
        if !self.rect.contains_offset(offset) {
            return Err(AccessError::CannotAccess(*offset));
        }
        self.compressed.get(offset)
    }

    pub fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut T>
    where
        I: Clone,
        usize: Compressible<I>
    {
        if !self.rect.contains_offset(offset) {
            return Err(AccessError::CannotAccess(*offset));
        }
        self.compressed.get_mut(offset)
    }

    pub fn set(&mut self, offset: &Offset, item: T) -> AccessResult<Option<T>>
    where
        I: Clone,
        usize: Compressible<I>
    {
        if !self.rect.contains_offset(offset) {
            return Err(AccessError::CannotAccess(*offset));
        }
        self.compressed.set(offset, item)
    }

    pub fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<T>>
    where
        I: Clone + Guarded,
        usize: Compressible<I>
    {
        if !self.rect.contains_offset(offset) {
            return Err(AccessError::CannotAccess(*offset));
        }
        self.compressed.rmv(offset)
    }
}