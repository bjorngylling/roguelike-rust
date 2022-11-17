use euclid::{point2, Point2D};

pub type Point = Point2D<i32, i32>;
pub fn pt(x: i32, y: i32) -> Point {
    point2(x, y)
}

#[derive(Clone, Debug)]
pub struct Grid<T: std::clone::Clone> {
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

impl<T: std::clone::Clone> std::ops::Index<Point2D<i32, i32>> for Grid<T> {
    type Output = T;

    fn index(&self, p: Point2D<i32, i32>) -> &Self::Output {
        self.storage.index(self.idx((p.x, p.y)) as usize)
    }
}

impl<T: std::clone::Clone> std::ops::IndexMut<Point> for Grid<T> {
    fn index_mut(&mut self, p: Point) -> &mut T {
        self.storage.index_mut(self.idx((p.x, p.y)) as usize)
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
        assert!(m[point2(0, 1)]);
    }
}
