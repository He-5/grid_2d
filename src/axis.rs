//! graphic utils of axis
pub mod zone;

use std::cmp::{max, min};
pub use zone::{Rect, RectZone, RectWalker, OffsetWalker, MajoredRect};

use std::ops::{Add, AddAssign, Mul, Neg, Range, Sub, SubAssign};
use crate::some;
// pub type Position = (usize, usize);

/// the `Position` represent a 2-Dim grid pos
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Position(
    /// Pos x
    usize,
    /// Pos y
    usize
);

impl Position {
    pub const fn new(pos_x: usize, pos_y: usize) -> Self {
        Self(pos_x, pos_y)
    }

    pub const fn pos_x(&self) -> usize {
        self.0
    }

    pub const fn pos_y(&self) -> usize {
        self.1
    }
}

impl From<(usize, usize)> for Position {
    fn from((x, y): (usize, usize)) -> Self {
        Self(x, y)
    }
}

impl Into<(usize, usize)> for Position {
    fn into(self) -> (usize, usize) {
        (self.0, self.1)
    }
}

impl TryFrom<(isize, isize)> for Position {
    type Error = <usize as TryFrom<isize>>::Error;
    fn try_from((x, y): (isize, isize)) -> Result<Self, Self::Error> {
        let u_x = x.try_into()?;
        let u_y = y.try_into()?;
        Ok(Self::new(u_x, u_y))
    }
}

impl TryFrom<Offset> for Position {
    type Error = <usize as TryFrom<isize>>::Error;
    fn try_from(value: Offset) -> Result<Self, Self::Error> {
        let offset_x = value.get_x();
        let offset_y = value.get_y();
        (offset_x, offset_y).try_into()
    }
}

impl Sub<Position> for Position {
    type Output = Offset;
    fn sub(self, rhs: Position) -> Self::Output {
        let x_diff = self.pos_x().checked_signed_diff(rhs.pos_x()).expect("x diff out of bound");
        let y_diff = self.pos_y().checked_signed_diff(rhs.pos_y()).expect("y diff out of bound");
        Offset::new(x_diff, y_diff)
    }
}

impl AddAssign<Offset> for Position {
    fn add_assign(&mut self, rhs: Offset) {
        self.0 = self.pos_x().checked_add_signed(rhs.get_x()).expect("overflow with x adding");
        self.1 = self.pos_y().checked_add_signed(rhs.get_y()).expect("overflow with y adding");
    }
}

impl Add<Offset> for Position {
    type Output = Position;
    fn add(self, rhs: Offset) -> Self::Output {
        let mut from = self.clone();
        from += rhs;
        from
    }
}

impl SubAssign<Offset> for Position {
    fn sub_assign(&mut self, rhs: Offset) {
        self.0 = self.pos_x().checked_sub_signed(rhs.get_x()).expect("overflow with x adding");
        self.1 = self.pos_y().checked_sub_signed(rhs.get_y()).expect("overflow with y adding");
    }
}

impl Sub<Offset> for Position {
    type Output = Position;
    fn sub(self, rhs: Offset) -> Self::Output {
        let mut from = self.clone();
        from -= rhs;
        from
    }
}

/// the `Offset` represent the diff from pos to pos
#[derive(Debug, Copy, Clone, Default, Hash, PartialEq, Eq)]
pub struct Offset(
    /// offset_x
    isize,
    /// offset_y
    isize
);

impl Offset {
    pub const fn new(offset_x: isize, offset_y: isize) -> Self {
        Self(offset_x, offset_y)
    }

    pub const fn with_x(x: isize) -> Self {
        Self::new(x, 0)
    }

    pub const fn with_y(y: isize) -> Self {
        Self::new(0, y)
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn get_x(&self) -> isize {
        self.0
    }

    pub fn get_y(&self) -> isize {
        self.1
    }

    pub(crate) fn get_mut_x(&mut self) -> &mut isize {
        &mut self.0
    }

    pub(crate) fn get_mut_y(&mut self) -> &mut isize {
        &mut self.1
    }

    pub fn is_zero(&self) -> bool {
        self.get_x() == 0 && self.get_y() == 0
    }
}

impl From<(isize, isize)> for Offset {
    fn from((x, y): (isize, isize)) -> Self {
        Self(x, y)
    }
}

impl TryFrom<(usize, usize)> for Offset {
    type Error = <isize as TryFrom<usize>>::Error;
    fn try_from((x, y): (usize, usize)) -> Result<Self, Self::Error> {
        let i_x = x.try_into()?;
        let i_y = y.try_into()?;
        Ok(Self::new(i_x, i_y))
    }
}

impl TryFrom<Position> for Offset {
    type Error = <isize as TryFrom<usize>>::Error;
    fn try_from(value: Position) -> Result<Self, Self::Error> {
        let pos_x = value.pos_x();
        let pos_y = value.pos_y();
        (pos_x, pos_y).try_into()
    }
}

impl Neg for Offset {
    type Output = Offset;
    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

impl Add<Offset> for Offset {
    type Output = Self;
    fn add(self, rhs: Offset) -> Self::Output {
        Self(self.get_x() + rhs.get_x(), self.get_y() + rhs.get_y())
    }
}

impl AddAssign<Offset> for Offset {
    fn add_assign(&mut self, rhs: Offset) {
        *self.get_mut_x() += rhs.get_x();
        *self.get_mut_y() += rhs.get_y();
    }
}

impl Sub<Offset> for Offset {
    type Output = Self;
    fn sub(self, rhs: Offset) -> Self::Output {
        Self(self.get_x() - rhs.get_x(), self.get_y() - rhs.get_y())
    }
}

impl SubAssign<Offset> for Offset {
    fn sub_assign(&mut self, rhs: Offset) {
        *self.get_mut_x() -= rhs.get_x();
        *self.get_mut_y() -= rhs.get_y();
    }
}

impl Mul<Rect> for Offset {
    type Output = Offset;

    fn mul(self, rhs: Rect) -> Self::Output {
        let multiplied_x = self.get_x().checked_mul(
            rhs.get_width().try_into().expect("rect's width is not safe for isize")
        ).expect(&format!("{} * {} is overflow", self.get_x(), rhs.get_width()));
        let multiplied_y = self.get_y().checked_mul(
            rhs.get_height().try_into().expect("rect's height is not safe for isize")
        ).expect(&format!("{} * {} is overflow", self.get_y(), rhs.get_height()));
        Offset::new(multiplied_x, multiplied_y)
    }
}

impl Mul<usize> for Offset {
    type Output = Offset;
    fn mul(self, rhs: usize) -> Self::Output {
        self * Rect::new_sqr(rhs)
    }
}

pub fn in_range<T, R>(point: T, range: R, start: T) -> bool
where T: Ord + Add<R, Output = T> {
    point >= start && point < start + range
}

pub fn overlapping<R>(range_1: Range<R>, range_2: Range<R>) -> Option<Range<R>>
where R: Ord
{
    if range_1.is_empty() || range_2.is_empty() {
        return None;
    }
    some! {
        if range_1.start < range_2.end && range_2.start < range_1.end => {
            max(range_1.start, range_2.start)..min(range_1.end, range_2.end)
        }
    }
}