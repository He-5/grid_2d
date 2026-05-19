use std::ops::Range;
use super::{in_range, Position};
use crate::some;

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

    pub fn checked_size(&self) -> Option<usize> {
        self.width.checked_mul(self.height)
    }

    pub fn fold_x_first(&self, index: usize) -> Option<Position> {
        some! { if index < self.size() => (index % self.width, index / self.width).into() }
    }

    pub fn fold_y_first(&self, index: usize) -> Option<Position> {
        some! { if index < self.size() => (index / self.height, index % self.height).into() }
    }

    pub fn flatten_x_first(&self, pos: &Position) -> Option<usize> {
        some! { if self.contains_pos(pos) => self.width * pos.pos_y() + pos.pos_x() }
    }

    pub fn flatten_y_first(&self, pos: &Position) -> Option<usize> {
        some! { if self.contains_pos(pos) => self.height * pos.pos_x() + pos.pos_y() }
    }

    pub fn contains_pos(&self, pos: &Position) -> bool {
        in_range(pos.pos_x(), self.width, 0) && in_range(pos.pos_y(), self.height, 0)
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

pub trait RectBounded {
    fn boundary(&self) -> &Rect;

    fn inside(&self, pos: &Position) -> bool {
        self.boundary().contains_pos(pos)
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