use std::borrow::Borrow;
use std::ops::{BitAnd, Deref, DerefMut, Range};
use crate::axis::{in_range, overlapping, Offset};
use crate::axis::zone::{Walkable, Zone};
use crate::{some, Position};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Rect {
    width: usize,
    height: usize
}

impl Rect {
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub const fn new_sqr(side: usize) -> Self {
        Self::new(side, side)
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn fold_x_first(&self, index: usize) -> Option<Offset> {
        some! { if index < self.size() => (index % self.width, index / self.width).try_into().ok()? }
    }

    pub fn fold_y_first(&self, index: usize) -> Option<Offset> {
        some! { if index < self.size() => (index / self.height, index % self.height).try_into().ok()? }
    }

    pub fn flatten_x_first(&self, offset: &Offset) -> Option<usize> {
        some! { if self.contains_offset(offset) => self.width * offset.get_y().cast_unsigned() + offset.get_x().cast_unsigned() }
    }

    pub fn flatten_y_first(&self, offset: &Offset) -> Option<usize> {
        some! { if self.contains_offset(offset) => self.height * offset.get_x().cast_unsigned() + offset.get_y().cast_unsigned() }
    }

    pub fn contains_offset(&self, offset: &Offset) -> bool {
        let x_contained = !offset.get_x().is_negative() && in_range(offset.get_x().unsigned_abs(), self.width, 0);
        let y_contained = !offset.get_y().is_negative() && in_range(offset.get_y().unsigned_abs(), self.height, 0);
        x_contained && y_contained
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

pub struct RectZone {
    pos: Position,
    rect: Rect
}

impl RectZone {
    pub const fn new(width: usize, height: usize) -> Self {
        Self::with_pos(width, height, Position::new(0, 0))
    }

    pub const fn with_pos(width: usize, height: usize, pos: Position) -> Self {
        Self { pos, rect: Rect::new(width, height) }
    }

    pub fn get_width(&self) -> usize {
        self.rect.width
    }

    pub fn get_height(&self) -> usize {
        self.rect.height
    }

    pub fn pos_x(&self) -> usize {
        self.pos.pos_x()
    }

    pub fn pos_y(&self) -> usize {
        self.pos.pos_y()
    }

    fn x_shadow(&self) -> Range<usize> {
        self.pos_x()..self.pos_x() + self.get_width()
    }

    fn y_shadow(&self) -> Range<usize> {
        self.pos_y()..self.pos_y() + self.get_height()
    }
}

impl Zone for RectZone {
    fn get_anchor(&self) -> Position {
        self.pos
    }

    #[inline]
    fn contains_offset(&self, query: &impl Borrow<Offset>) -> bool {
        self.rect.contains_offset(query.borrow())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.rect.size())
    }
}

impl BitAnd for &RectZone {
    type Output = Option<RectZone>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let x_overlap = overlapping(self.x_shadow(), rhs.x_shadow())?;
        let y_overlap = overlapping(self.y_shadow(), rhs.y_shadow())?;

        Some(RectZone::with_pos(
            x_overlap.len(),
            y_overlap.len(),
            Position::new(x_overlap.start, y_overlap.start)
        ))
    }
}

pub struct RectWalker {
    rect: Rect,
    cur_index: usize
}

impl RectWalker {
    pub const fn new(rect: Rect) -> Self {
        Self { rect, cur_index: 0 }
    }
}

impl Iterator for RectWalker {
    type Item = Offset;
    fn next(&mut self) -> Option<Self::Item> {
        let cur_offset = self.rect.fold_x_first(self.cur_index)?;
        self.cur_index += 1;
        Some(cur_offset)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.rect.size();
        (size - self.cur_index, Some(size))
    }
}

impl Walkable for RectZone {
    type Walker = RectWalker;
    fn walk_through(&self) -> Self::Walker {
        RectWalker::new(self.rect)
    }
}

pub enum MajoredRect {
    RowMajored(Rect),
    ColMajored(Rect)
}

impl Deref for MajoredRect {
    type Target = Rect;
    fn deref(&self) -> &Self::Target {
        match self {
            MajoredRect::RowMajored(rect) => rect,
            MajoredRect::ColMajored(rect) => rect
        }
    }
}

impl DerefMut for MajoredRect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MajoredRect::RowMajored(rect) => rect,
            MajoredRect::ColMajored(rect) => rect
        }
    }
}

impl MajoredRect {
    pub fn new_row(width: usize, height: usize) -> Self {
        MajoredRect::RowMajored(Rect::new(width, height))
    }

    pub fn new_col(width: usize, height: usize) -> Self {
        MajoredRect::ColMajored(Rect::new(width, height))
    }
    
    pub fn fold_majored(&self, index: usize) -> Option<Offset> {
        match self {
            MajoredRect::RowMajored(rect) => rect.fold_x_first(index),
            MajoredRect::ColMajored(rect) => rect.fold_y_first(index)
        }
    }

    pub fn flat_majored(&self, offset: &Offset) -> Option<usize> {
        match self {
            MajoredRect::RowMajored(rect) => rect.flatten_x_first(offset),
            MajoredRect::ColMajored(rect) => rect.flatten_y_first(offset)
        }
    }
}

/// calculate a minimum rect to fill the [width] * [height] grid without exceed [max] counts
pub(crate) fn chunking(width: usize, height: usize, max: usize) -> Option<Rect> {
    match (width, height, max) {
        (0, ..) | (_, 0, ..) | (_, _, 0) => None,
        _ => match width.checked_mul(height) {
            Some(total_size) if total_size <= max => Some(Rect::new(1, 1)),
            _ => {
                let (f_width, f_height) = (width as f32, height as f32);
                let ratio = f_width / f_height;
                if ratio.is_nan() {
                    return None;
                }
                let average_chunk_count = f_width * f_height / max as f32;
                let calculated_chunk_height = (average_chunk_count / ratio).sqrt();
                let calculated_chunk_width = average_chunk_count / calculated_chunk_height;

                let (mut chunk_width, mut chunk_height) = match (calculated_chunk_width.round(), calculated_chunk_height.round()) {
                    (0.0, 0.0) => {
                        return None;
                    }
                    // extremely ratio might cause cal_height greater than avg_count,
                    // which will cause the final chunk been larger than avg_count expected.
                    // these two branch fix it with simply re-calculate with height 1 or width 1 at least
                    (0.0, _) => {
                        return Some(Rect::new(1, average_chunk_count.ceil() as usize));
                    }
                    (_, 0.0) => {
                        return Some(Rect::new(average_chunk_count.ceil() as usize, 1));
                    }
                    (c_width, c_height) => (c_width, c_height)
                };

                // since the cal_width and cal_height satisfied cal_width * cal_height = avg_count
                // the cal_width.ceil() * cal_height.ceil() will always greater than or equal to avg_count.
                // the cal_*.round() might act as cal_*.floor(), so use cmp to adjust if the value is floored

                if width == height && chunk_height * chunk_width < average_chunk_count {
                    // for the square shape, the chunk_height and width might change together
                    chunk_height += 1.0;
                    chunk_width += 1.0;
                } else if width > height {
                    if chunk_height * chunk_width < average_chunk_count && chunk_width < calculated_chunk_width {
                        chunk_width += 1.0;
                    }
                    if chunk_height * chunk_width < average_chunk_count && chunk_height < calculated_chunk_height {
                        chunk_height += 1.0;
                    }
                } else /* width < height */ {
                    // the only difference between width < height and width > height is
                    // this part will check and increase height first
                    if chunk_height * chunk_width < average_chunk_count && chunk_height < calculated_chunk_height {
                        chunk_height += 1.0;
                    }
                    if chunk_height * chunk_width < average_chunk_count && chunk_width < calculated_chunk_width {
                        chunk_width += 1.0;
                    }
                }
                Some(Rect::new(chunk_width as usize, chunk_height as usize))
            }
        }
    }
}

#[test]
fn test_chunking() {
    let single = Rect::new(1, 1);
    assert_eq!(chunking(1, 1, 256), Some(single));
    assert_eq!(chunking(15, 15, 256), Some(single));
    assert_eq!(chunking(1, 256, 256), Some(single));
    assert_eq!(chunking(0, 0, 256), None);
    assert_eq!(chunking(16, 16, 16_usize.pow(2)), Some(single));
    assert_eq!(chunking(256, 256, 256_usize.pow(2)), Some(single));
    assert_eq!(chunking(65536, 65536, 65536_usize.pow(2)), Some(single));

    let rect_1x2 = Rect::new(1, 2);
    assert_eq!(chunking(16, 17, 256), Some(rect_1x2));
    assert_eq!(chunking(17, 18, 256), Some(rect_1x2));
    assert_eq!(chunking(16, 32, 256), Some(rect_1x2));
    assert_eq!(chunking(1, 257, 256), Some(rect_1x2));
    assert_eq!(chunking(1, 512, 256), Some(rect_1x2));

    let rect_2x1 = Rect::new(2, 1);
    assert_eq!(chunking(17, 16, 256), Some(rect_2x1));
    assert_eq!(chunking(18, 17, 256), Some(rect_2x1));
    assert_eq!(chunking(32, 16, 256), Some(rect_2x1));
    assert_eq!(chunking(257, 1, 256), Some(rect_2x1));
    assert_eq!(chunking(512, 1, 256), Some(rect_2x1));

    // some case
    assert_eq!(chunking(100, 100, 256), Some(Rect::new(7, 7)));
    assert_eq!(chunking(101, 99, 256), Some(Rect::new(7, 6)));
    assert_eq!(chunking(129, 30, 256), Some(Rect::new(8, 2)));
}