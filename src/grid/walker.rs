use crate::grid::Layout;
use crate::{Offset, Position};

pub mod rect_walker;

pub trait Walker {
    fn next_step(&mut self) -> Option<Offset>;
}

impl <I> Walker for I
where
    I: Iterator<Item: Into<Offset>>
{
    fn next_step(&mut self) -> Option<Offset> {
        self.next().map(Into::into)
    }
}

/// this is a marker for [Walker] implementor
///
/// impl this trait means the walker will **NEVER** access same [Position]
/// with specified [Offset] order provided by itself.
///
/// for a more mathematical description:
/// Any continuous sub chain of [Offset] produce by the [Walker] should never have a sum equals to [Offset::zero]
pub unsafe trait NeverAcross : Walker {}

pub struct Walkthrough<'l, W, L> {
    stand: Option<Position>,
    walker: W,
    layout_ref: &'l L
}

impl <'l, W, L> Walkthrough<'l, W, L> {
    pub(crate) fn new(walker: W, layout_ref: &'l L) -> Self {
        Self::with_start(Default::default(), walker, layout_ref)
    }

    pub(crate) fn with_start(start: Position, walker: W, layout_ref: &'l L) -> Self {
        Self {
            stand: Some(start),
            walker,
            layout_ref
        }
    }
}

impl <'l, W, L> Iterator for Walkthrough<'l, W, L>
where
    W: Walker,
    L: Layout + 'l
{
    type Item = Option<&'l L::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let stand = self.stand.take()?;
        if let Some(offset) = self.walker.next_step() {
            let _ = self.stand.insert(stand + offset);
        }
        match self.layout_ref.get(&stand) {
            Ok(value) => Some(Some(value)),
            Err(_) => Some(None)
        }
    }
}

pub struct WalkWith<'l, W, L> {
    stand: Option<Position>,
    walker: W,
    layout_mut: &'l mut L
}

impl <'l, W, L> WalkWith<'l, W, L> {
    pub(crate) fn new(walker: W, layout_mut: &'l mut L) -> Self {
        Self::with_start(Default::default(), walker, layout_mut)
    }

    pub(crate) fn with_start(start: Position, walker: W, layout_mut: &'l mut L) -> Self {
        Self {
            stand: Some(start),
            walker,
            layout_mut
        }
    }
}

impl <'l, W, L> Iterator for WalkWith<'l, W, L>
where
    W: NeverAcross,
    L: Layout
{
    type Item = Option<&'l mut L::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let stand = self.stand.take()?;
        if let Some(offset) = self.walker.next_step() {
            let _ = self.stand.insert(stand + offset);
        }
        match self.layout_mut.get_mut(&stand) {
            Ok(value) => {
                let ptr: *mut L::Item = value;
                // SAFETY: walker impls NeverAcross promise that the get_mut will never
                // access twice by same walker
                Some(Some(unsafe { &mut *ptr }))
            }
            Err(_) => Some(None)
        }
    }
}

