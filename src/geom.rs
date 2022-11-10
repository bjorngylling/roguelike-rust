use crate::gfx;
use ggez::glam::Vec2;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn length(self) -> f32 {
        f32::sqrt(self.dot(self) as f32)
    }

    pub fn dot(self, other: Self) -> i32 {
        (self.x * other.x) + (self.y * other.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Mul for Point {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl std::ops::Div for Point {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl From<Point> for (i32, i32) {
    fn from(p: Point) -> (i32, i32) {
        (p.x, p.y)
    }
}

impl From<Point> for Vec2 {
    fn from(p: Point) -> Vec2 {
        Vec2::new(p.x as f32, p.y as f32)
    }
}

pub fn pt(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

pub struct Grid<T> {
    pub width: i32,
    pub height: i32,
    storage: Vec<T>,
}

impl<T: std::clone::Clone> Grid<T> {
    pub fn new(width: i32, height: i32, fill: T) -> Self {
        Grid {
            width,
            height,
            storage: vec![fill; (width * height) as usize],
        }
    }

    const fn idx(&self, (x, y): (i32, i32)) -> i32 {
        x + y * self.width
    }
}

impl<T: std::clone::Clone> std::ops::Index<(i32, i32)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        self.storage.index(self.idx(index) as usize)
    }
}

impl<T: std::clone::Clone> std::ops::IndexMut<(i32, i32)> for Grid<T> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut T {
        self.storage.index_mut(self.idx(index) as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn map_access_by_point() {
        let m = {
            let mut m = Grid::new(10, 10, false);
            m[(0, 1)] = true;
            m
        };
        assert!(m[pt(0, 1).into()]);
    }
}
