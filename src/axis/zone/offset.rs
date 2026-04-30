use std::mem;
use crate::short_cut;
use super::super::Offset;

pub struct OffsetWalker {
    delta_x: isize,
    delta_y: isize,
    current: Offset
}

impl OffsetWalker {
    pub const fn new(delta_x: isize, delta_y: isize) -> Self {
        Self::with_offset(delta_x, delta_y, Offset::zero())
    }

    pub const fn with_offset(delta_x: isize, delta_y: isize, current: Offset) -> Self {
        Self { delta_x, delta_y, current }
    }

    pub fn set_delta_x(&mut self, delta_x: isize) {
        self.delta_x = delta_x;
    }

    pub fn set_delta_y(&mut self, delta_y: isize) {
        self.delta_y = delta_y;
    }

    pub(super) fn set_current(&mut self, current: Offset) {
        self.current = current;
    }

    pub(super) fn backward(&mut self) -> bool {
        self.backward_n(1)
    }

    pub(super) fn backward_n(&mut self, n: isize) -> bool {
        if n.is_negative() { return false; }
        if n == 0 { return true; }
        let back_x = short_cut!(Some(self.current.get_x().checked_sub(self.delta_x * n))?false);
        let back_y = short_cut!(Some(self.current.get_y().checked_sub(self.delta_y * n))?false);
        self.set_current(Offset::new(back_x, back_y));
        true
    }
}

impl Iterator for OffsetWalker {
    type Item = Offset;

    fn next(&mut self) -> Option<Self::Item> {
        let next_x = self.current.get_x().checked_add(self.delta_x)?;
        let next_y = self.current.get_y().checked_add(self.delta_y)?;
        Some(mem::replace(&mut self.current, Offset::new(next_x, next_y)))
    }
}