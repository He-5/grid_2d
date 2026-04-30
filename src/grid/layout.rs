use std::cmp::{max, min};
use crate::axis::{Offset, Rect};
use std::mem;

mod loose_layout;
pub use loose_layout::LooseLayout;
mod compress_layout;
pub use compress_layout::CompressLayout;
mod tight_layout;
pub use tight_layout::TightLayout;
use crate::grid::layout::compress_layout::{Compressible, Guarded};

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

pub enum Chunk<T> {
    Loose(LooseLayout<T>),
    Tight(TightLayout<T>)
}

impl <T> Chunk<T> {
    /// Create Loose Chunk with Rect
    pub fn new_loose(rect: Rect) -> Self {
        Self::Loose(LooseLayout::with_rect(rect))
    }

    /// Create Tight Chunk with Rect
    pub fn new_tight(rect: Rect) -> Self {
        Self::Tight(TightLayout::new(rect.get_width(), rect.get_height()))
    }
}

pub struct ChunkedLayout<T, I> {
    compressed: Compressed<T, I>,
    chunk_rect: Rect,
    rect: Rect
}

pub enum Compressed<T, I> {
    Chunked(CompressLayout<Chunk<T>, I>),
    Single(CompressLayout<T, I>)
}

impl <T, I> Compressed<T, I> {
    pub fn new_single(rect: Rect) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        Ok(Self::Single(CompressLayout::new(rect)?))
    }

    pub fn new_chunked(rect: Rect) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        Ok(Self::Chunked(CompressLayout::new(rect)?))
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

    pub fn with_chunk_rect_and_rect(chunk_rect: Rect, rect: Rect) -> Result<Self, CreateError>
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
                    Compressed::new_chunked(Rect::new(chunk_width_count, chunk_height_count))
                }
            }?;
        Ok(Self { compressed, chunk_rect, rect })
    }

    pub fn new(width: usize, height: usize) -> Result<Self, CreateError>
    where
        I: Guarded,
        usize: Compressible<I>
    {
        let chunk_rect = match (width, height) {
            (0, _) | (_, 0) => { return Err(CreateError::InvalidSize(0)); }
            (width, height) => {
                match width.checked_mul(height) {
                    Some(total_size) if total_size.is_compressible() => Rect::new(1, 1),
                    Some(big_size) => {
                        todo!("get unhandled big size {}", big_size)
                    },
                    _ => {
                        todo!("get extra big size")
                    }
                }
            }
        };
        Self::with_chunk_rect_and_rect(chunk_rect, Rect::new(width, height))
    }
}