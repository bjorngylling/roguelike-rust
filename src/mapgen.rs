use crate::game::{Map, Tile};
use crate::geom::{pt, Grid, Point};
use crate::gfx;
use euclid::Box2D;
use ggez::graphics;
use rand::Rng;
use std::collections::HashSet;

pub trait Generator {
    fn run(&mut self, rng: &mut impl Rng, map: &mut Map);
    fn timeline(&self) -> Vec<Grid<graphics::Color>>;
}

pub struct SimpleMapGenerator {
    timeline: Vec<Grid<graphics::Color>>,
    colors: Vec<graphics::Color>,
    cur: usize,
    bounds: Box2D<i32, i32>,
    m: Grid<u8>,
}

impl SimpleMapGenerator {
    pub fn new(width: i32, height: i32) -> Self {
        let colors: Vec<graphics::Color> = vec![
            gfx::BLACK_BRIGHT,
            gfx::BLUE_BRIGHT,
            gfx::RED_BRIGHT,
            gfx::CYAN_BRIGHT,
            gfx::GREEN_BRIGHT,
            gfx::WHITE_BRIGHT,
            gfx::YELLOW_BRIGHT,
            gfx::MAGENTA_BRIGHT,
            gfx::BLACK,
            gfx::BLUE,
            gfx::RED,
            gfx::CYAN,
            gfx::GREEN,
            gfx::WHITE,
            gfx::YELLOW,
            gfx::MAGENTA,
        ];
        SimpleMapGenerator {
            timeline: vec![Grid::new(width, height, gfx::BACKGROUND)],
            colors,
            cur: 0,
            bounds: Box2D {
                min: pt(0, 0),
                max: pt(width, height),
            },
            m: Grid::new(width, height, 1),
        }
    }

    fn snapshot_room(&mut self, room: Box2D<i32, i32>) {
        let mut img = self.timeline().last().unwrap().clone();

        let c = self.colors[self.cur];
        self.cur = (self.cur + 1) % self.colors.len();
        for y in room.y_range() {
            for x in room.x_range() {
                if self.m[(x, y)] == 0 {
                    img[(x, y)] = c;
                }
            }
        }

        self.timeline.push(img);
    }

    fn snapshot_corridor(&mut self, corridor: Vec<Point>) {
        let mut img = self.timeline().last().unwrap().clone();

        let c = self.colors[self.cur];
        self.cur = (self.cur + 1) % self.colors.len();
        for w in corridor.windows(2) {
            let (a, b) = (w[0], w[1]);
            if a.x != b.x {
                for x in a.x..=b.x {
                    if self.m[(x, a.y)] == 0 {
                        img[(x, a.y)] = c;
                    }
                }
            } else if a.y != b.y {
                for y in a.y..=b.y {
                    if self.m[(a.x, y)] == 0 {
                        img[(a.x, y)] = c;
                    }
                }
            }
        }

        self.timeline.push(img);
    }
}

impl Generator for SimpleMapGenerator {
    fn run(&mut self, rng: &mut impl Rng, map: &mut Map) {
        let mut rooms: Vec<Box2D<i32, i32>> = vec![];
        for _ in 0..30 {
            let w = rng.gen_range(5..=12);
            let h = rng.gen_range((w - 3)..=(w + 3));
            let x = rng.gen_range(1..self.bounds.width() - w);
            let y = rng.gen_range(1..self.bounds.height() - h);
            let room = Box2D {
                min: pt(x, y),
                max: pt(x + w, y + h),
            };

            let outer = room.inflate(1, 1);
            if !rooms.iter().any(|&r| r.intersects(&outer)) {
                rooms.push(room);
                for y in room.y_range() {
                    for x in room.x_range() {
                        self.m[(x, y)] = 0;
                    }
                }
                self.snapshot_room(room);
            }
        }

        let mut connected: HashSet<Point> = HashSet::new();
        for room in rooms.iter() {
            if connected.len() == rooms.len() - 1 {
                break;
            }

            // find closest other room
            let tar = rooms
                .iter()
                .filter(|&r| r.center() != room.center())
                .filter(|&r| !connected.contains(&r.center()))
                .min_by(|a, b| {
                    room.center()
                        .to_f32()
                        .distance_to(a.center().to_f32())
                        .partial_cmp(&room.center().to_f32().distance_to(b.center().to_f32()))
                        .unwrap()
                })
                .unwrap();

            // connect the rooms
            for x in room.center().x.min(tar.center().x)..=room.center().x.max(tar.center().x) {
                self.m[(x, room.center().y)] = 0;
            }
            for y in room.center().y.min(tar.center().y)..=room.center().y.max(tar.center().y) {
                self.m[(tar.center().x, y)] = 0;
            }
            connected.insert(room.center());
            self.snapshot_corridor(vec![
                pt(room.center().x.min(tar.center().x), room.center().y),
                pt(room.center().x.max(tar.center().x), room.center().y),
                pt(tar.center().x, room.center().y.min(tar.center().y)),
                pt(tar.center().x, room.center().y.max(tar.center().y)),
            ]);
        }

        for y in 0..self.bounds.height() {
            for x in 0..self.bounds.width() {
                if self.m[(x, y)] == 0 {
                    map.tiles[(x, y)] = Tile::Floor;
                }
            }
        }
        // Put the entrance in the center of the first room
        map.tiles[rooms.first().unwrap().center()] = Tile::StairUp;
        map.entrance = rooms.first().unwrap().center();
    }

    fn timeline(&self) -> Vec<Grid<graphics::Color>> {
        self.timeline.clone()
    }
}
