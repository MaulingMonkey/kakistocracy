use winapi::shared::ntdef::*;
use winapi::shared::windef::RECT;

use std::ops::Range;



/// Extension methods for [`RECT`].
pub trait RectExt {
    /// [`LONG`] for [`RECT`].  A component of a position in the coordinate space.  Likely signed.
    type Pos;

    /// `left` .. `right`
    fn xrange(&self) -> Range<Self::Pos>;

    /// `top` .. `bottom`
    fn yrange(&self) -> Range<Self::Pos>;

    /// [`ULONG`] for [`RECT`].  A component of a size that should cover the entire coordinate space.  Unsigned.
    type Size;

    /// `right` - `left` (or `0` if `right` < `left`)
    fn width(&self) -> Self::Size;

    /// `bottom` - `top` (or `0` if `bottom` < `top`)
    fn height(&self) -> Self::Size;

    /// `(width(), height())`
    fn size(&self) -> (Self::Size, Self::Size) { (self.width(), self.height()) }
}



impl RectExt for RECT {
    type Pos = LONG;
    fn xrange(&self) -> Range<LONG> { self.left .. self.right }
    fn yrange(&self) -> Range<LONG> { self.top .. self.bottom }

    type Size = ULONG;

    fn width(&self) -> ULONG {
        if self.left > self.right {
            0
        } else {
            self.right.wrapping_sub(self.left) as ULONG
        }
    }

    fn height(&self) -> ULONG {
        if self.top > self.bottom {
            0
        } else {
            self.bottom.wrapping_sub(self.top) as ULONG
        }
    }
}



pub trait IntoRect {
    fn into(self) -> RECT;
}

impl IntoRect for RECT {
    fn into(self) -> RECT { self }
}

impl IntoRect for Range<(LONG, LONG)> {
    fn into(self) -> RECT { RECT {
        left:   self.start.0,
        top:    self.start.1,
        right:  self.end.0,
        bottom: self.end.1,
    }}
}

impl IntoRect for Range<[LONG; 2]> {
    fn into(self) -> RECT { RECT {
        left:   self.start[0],
        top:    self.start[1],
        right:  self.end[0],
        bottom: self.end[1],
    }}
}

impl IntoRect for (Range<LONG>, Range<LONG>) {
    fn into(self) -> RECT { RECT {
        left:   self.0.start,
        top:    self.1.start,
        right:  self.0.end,
        bottom: self.1.end,
    }}
}

impl IntoRect for [Range<LONG>; 2] {
    fn into(self) -> RECT { RECT {
        left:   self[0].start,
        top:    self[1].start,
        right:  self[0].end,
        bottom: self[1].end,
    }}
}
