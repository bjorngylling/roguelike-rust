use crate::game::Tile;
use crate::geom::{pt, Grid};
use euclid::Box2D;
use image::RgbaImage;

pub trait Generator {
    fn generate(&mut self) -> Grid<Tile>;
    fn timeline(&self) -> Vec<RgbaImage>;
}

pub struct SimpleMapGenerator {
    timeline: Vec<RgbaImage>,
    bounds: Box2D<i32, i32>,
    m: Grid<u8>,
}

impl SimpleMapGenerator {
    pub fn new(width: i32, height: i32) -> Self {
        SimpleMapGenerator {
            timeline: vec![],
            bounds: Box2D {
                min: pt(0, 0),
                max: pt(width, height),
            },
            m: Grid::new(width, height, 0),
        }
    }

    fn snapshot(&mut self) {
        let mut img = RgbaImage::new(self.bounds.width() as u32, self.bounds.height() as u32);

        for (x, y, px) in img.enumerate_pixels_mut() {
            let i = self.m[(x as i32, y as i32)] * 255;
            *px = image::Rgba([i, i, i, 255]);
        }

        self.timeline.push(img);
    }
}

impl Generator for SimpleMapGenerator {
    fn generate(&mut self) -> Grid<Tile> {
        for x in [0, 1, 2, 3, 5, 7, 9, 12, 13, 14] {
            self.m[(x, 2)] = 1;
        }
        self.snapshot();
        let mut m = Grid::new(self.bounds.width(), self.bounds.height(), Tile::Floor);
        for y in 0..self.bounds.height() {
            for x in 0..self.bounds.width() {
                if self.m[(x, y)] == 1 {
                    m[(x, y)] = Tile::Wall;
                }
            }
        }
        m
    }

    fn timeline(&self) -> Vec<RgbaImage> {
        self.timeline.clone()
    }
}
