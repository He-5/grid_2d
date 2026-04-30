use std::mem;
use crate::axis::{Offset, Rect};

mod loose_layout;
pub use loose_layout::LooseLayout;
mod tight_layout;
pub use tight_layout::TightLayout;

pub enum AccessError<P> {
    CannotAccess(P),
    NoValue(P)
}

impl <P> AccessError<P> {
    pub fn reload<P1>(self, payload: P1) -> AccessError<P1> {
        match self {
            Self::CannotAccess(_) => AccessError::CannotAccess(payload),
            Self::NoValue(_) => AccessError::NoValue(payload)
        }
    }
}

pub type AccessResult<T> = Result<T, AccessError<Offset>>;

fn from_option<T>(option: Option<T>) -> Result<T, AccessError<()>> {
    match option {
        Some(value) => Ok(value),
        None => Err(AccessError::NoValue(()))
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

macro_rules! delegate_to_layout {
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

    pub fn repeat(width: usize, height: usize, repeated: T) -> Self
    where
        T: Clone
    {
        Self::Tight(TightLayout::with_elem(width, height, &repeated))
    }

    /// Tighten Current Layout
    ///
    /// turn current layout into [GlobalLayout::Tight] in-place
    ///
    /// it's a heavy operation with calling set on each exist element
    #[cold]
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
    #[cold]
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
        delegate_to_layout! {
            |layout| { layout.get_rect() } by self
        }
    }

    // CURD
    pub fn get(&self, offset: &Offset) -> AccessResult<&T> {
        delegate_to_layout!{
            |layout| { layout.get(offset) } by self
        }
    }

    pub fn get_mut(&mut self, offset: &Offset) -> AccessResult<&mut T> {
        delegate_to_layout! {
            |layout| { layout.get_mut(offset) } by self
        }
    }

    pub fn set(&mut self, offset: &Offset, item: T) -> AccessResult<Option<T>> {
        delegate_to_layout! {
            |layout| { layout.set(offset, item) } by self
        }
    }

    pub fn rmv(&mut self, offset: &Offset) -> AccessResult<Option<T>> {
        delegate_to_layout! {
            |layout| { layout.rmv(offset) } by self
        }
    }
}