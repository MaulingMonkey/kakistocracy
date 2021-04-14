use winapi::shared::windef::RECT;

use std::ops::Range;



pub trait RectExt {
    type Pos;
    fn xrange(&self) -> Range<Self::Pos>;
    fn yrange(&self) -> Range<Self::Pos>;

    type Size;
    fn width(&self) -> Self::Size;
    fn height(&self) -> Self::Size;
    fn size(&self) -> (Self::Size, Self::Size) { (self.width(), self.height()) }
}

impl RectExt for RECT {
    type Pos = i32;
    fn xrange(&self) -> Range<i32> { self.left .. self.right }
    fn yrange(&self) -> Range<i32> { self.top .. self.bottom }

    type Size = u32;

    fn width(&self) -> u32 {
        if self.left > self.right {
            0
        } else {
            self.right.wrapping_sub(self.left) as u32
        }
    }

    fn height(&self) -> u32 {
        if self.top > self.bottom {
            0
        } else {
            self.bottom.wrapping_sub(self.top) as u32
        }
    }
}



pub trait IntoRect {
    fn into(self) -> RECT;
}

impl IntoRect for RECT {
    fn into(self) -> RECT { self }
}

impl IntoRect for Range<(i32, i32)> {
    fn into(self) -> RECT { RECT {
        left:   self.start.0,
        top:    self.start.1,
        right:  self.end.0,
        bottom: self.end.1,
    }}
}

impl IntoRect for Range<[i32; 2]> {
    fn into(self) -> RECT { RECT {
        left:   self.start[0],
        top:    self.start[1],
        right:  self.end[0],
        bottom: self.end[1],
    }}
}

impl IntoRect for (Range<i32>, Range<i32>) {
    fn into(self) -> RECT { RECT {
        left:   self.0.start,
        top:    self.1.start,
        right:  self.0.end,
        bottom: self.1.end,
    }}
}

impl IntoRect for [Range<i32>; 2] {
    fn into(self) -> RECT { RECT {
        left:   self[0].start,
        top:    self[1].start,
        right:  self[0].end,
        bottom: self[1].end,
    }}
}
