use crate::gfx;

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

pub fn pt(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub renderable: gfx::Renderable,
    pub block: bool,
}

pub struct Map {
    pub width: i32,
    pub height: i32,
    storage: Vec<Tile>,
}

impl Map {
    pub fn new(width: i32, height: i32, fill_tile: Tile) -> Self {
        Map {
            width,
            height,
            storage: vec![fill_tile; (width * height) as usize],
        }
    }

    const fn tile_index(&self, (x, y): (i32, i32)) -> i32 {
        x + y * self.width
    }
}

impl std::ops::Index<(i32, i32)> for Map {
    type Output = Tile;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        self.storage.index(self.tile_index(index) as usize)
    }
}

impl std::ops::IndexMut<(i32, i32)> for Map {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        self.storage.index_mut(self.tile_index(index) as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::map;
    use ggez::graphics;

    #[test]
    fn map_access_by_point() {
        let m = Map::new(
            10,
            10,
            map::Tile {
                block: false,
                renderable: gfx::Renderable {
                    spr: graphics::Rect::new(0.1, 0.1, 0., 0.),
                    color: gfx::WHITE,
                },
            },
        );
        assert!(!m[pt(0, 0).into()].block);
    }
}
