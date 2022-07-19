use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl AddAssign<Point> for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<Point> for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Rect {
    pub fn zero() -> Self {
        Self {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }
    }

    pub fn shrink(&self, amount: i16) -> Self {
        Rect {
            left: self.left.wrapping_add(amount),
            top: self.top.wrapping_add(amount),
            bottom: self.bottom.wrapping_sub(amount),
            right: self.right.wrapping_sub(amount),
        }
    }

    pub fn inflate(&self, dx: i16, dy: i16) -> Self {
        Rect {
            left: self.left.wrapping_sub(dx),
            top: self.top.wrapping_sub(dy),
            bottom: self.bottom.wrapping_add(dy),
            right: self.right.wrapping_add(dx),
        }
    }

    pub fn offset(&self, dx: i16, dy: i16) -> Self {
        Rect {
            left: self.left.wrapping_add(dx),
            top: self.top.wrapping_add(dy),
            bottom: self.bottom.wrapping_add(dy),
            right: self.right.wrapping_add(dx),
        }
    }
}
