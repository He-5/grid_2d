mod grid;
pub use grid::{
    walker::{
        rect_walker::RectWalker, WalkWith, Walker, NeverAcross,
        Walkthrough
    },
    AccessError,
    AccessResult,
    Grid,
    LooseLayout,
    TightLayout,
    Layout
};

mod axis;
pub use axis::{
    Offset,
    Position,
    rect::{Rect, RectBounded}
};

mod macros;

