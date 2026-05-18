use crate::Offset;
use crate::axis::rect::Rect;
use crate::grid::walker::{NeverAcross, Walker};

pub struct RectWalker {
    count: usize,
    rect: Rect
}

impl RectWalker {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            count: 0
        }
    }
}

impl Walker for RectWalker {
    fn next_step(&mut self) -> Option<Offset> {
        if self.count >= self.rect.checked_size().unwrap_or_else(|| usize::MAX) {
            return None;
        }
        self.count += 1;
        let offset = if (self.count - 1) % self.rect.get_width() == self.rect.get_width() - 1 {
            // on new line
            Offset::new(1 - self.rect.get_width().cast_signed(), 1)
        } else {
            // row major move
            Offset::new(1, 0)
        };
        Some(offset)
    }
}

unsafe impl NeverAcross for RectWalker {}