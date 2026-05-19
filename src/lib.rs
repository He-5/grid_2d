mod grid;
pub use grid::{
    Grid,
    TightLayout,
    LooseLayout,
    RectWalker,
    OffsetWalker,
    PathWalker,
    Movement,
    D8Step,
    D4Step,
    WalkWith,
    Walkthrough
};

mod axis;
pub use axis::{
    Position,
    Offset,
    Rect,
    RectZone,
    zone::Zone
};

mod macros;

