use crate::{
    fov,
    geom::{pt, Grid, Point},
    gfx,
};
use ggez::{
    glam::*,
    graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use hecs;
use std::collections::HashSet;

pub struct MainState {
    world: hecs::World,
    hero: hecs::Entity,
    instances: ggez::graphics::InstanceArray,
    sprite_set: gfx::SpriteSet,
    map: Map,
}

impl MainState {
    pub fn new(
        ctx: &mut Context,
        width: i32,
        height: i32,
        map_layer: Grid<Tile>,
    ) -> GameResult<MainState> {
        let sprite_set = gfx::SpriteSet::new(16, 16, 12, 12);
        let image = graphics::Image::from_path(ctx, "/nice-curses.png")?;
        let mut instances = graphics::InstanceArray::new(ctx, image);
        instances.resize(ctx, (width * height) as u32 + 50); // mapsize + 50 entities

        let mut hero_pos = pt(10, 10);
        for x in 0..map_layer.width {
            for y in 0..map_layer.height {
                if map_layer[(x, y)] == Tile::StairUp {
                    hero_pos.x = x;
                    hero_pos.y = y;
                }
            }
        }
        let mut world = hecs::World::new();
        let hero = world.spawn((
            Player,
            Name("Hero".to_string()),
            Physics { pos: hero_pos },
            gfx::Renderable {
                spr: sprite_set.src_by_idx(gfx::CP437::ChAt as i32),
                color: gfx::WHITE_BRIGHT,
            },
            /*Viewshed {
                visible_tiles: HashSet::new(),
                range: 7,
                dirty: true,
            },*/
        ));
        world.spawn((
            Name("Giant Ant".to_string()),
            Physics { pos: pt(20, 14) },
            gfx::Renderable {
                spr: sprite_set.src_by_idx(gfx::CP437::Cha as i32),
                color: gfx::BLUE_BRIGHT,
            },
        ));
        Ok(MainState {
            world,
            hero,
            instances,
            sprite_set,
            map: Map { tiles: map_layer },
        })
    }

    pub fn update(&mut self) -> GameResult {
        move_handler(&mut self.world, &self.map.tiles);
        fov_handler(&mut self.world, &self.map.tiles);

        Ok(())
    }

    pub fn draw(&mut self, _ctx: &mut Context) -> &graphics::InstanceArray {
        self.instances.clear();
        let map_layer = &self.map.tiles;
        let viewshed = self.world.get::<&Viewshed>(self.hero);
        for x in 0..map_layer.width {
            for y in 0..map_layer.height {
                let pos = pt(x as i32, y as i32);
                let draw = if let Ok(v) = &viewshed {
                    v.visible_tiles.contains(&pos)
                } else {
                    true
                };
                if draw {
                    let t = map_layer[(x, y)];
                    let d: Vec2 = pos.to_f32().to_array().into();
                    let spr = match t {
                        Tile::Floor => self.sprite_set.src_by_idx(gfx::CP437::ChDot as i32),
                        Tile::StairUp => self.sprite_set.src_by_idx(gfx::CP437::LessThan as i32),
                        _ => self.sprite_set.src_by_idx(gfx::CP437::Pillar as i32),
                    };
                    self.instances
                        .push(graphics::DrawParam::new().dest(d * 12.).src(spr));
                }
            }
        }
        for (_, (phys, renderable)) in self
            .world
            .query::<(&Physics, &gfx::Renderable)>()
            .into_iter()
        {
            let draw = if let Ok(v) = &viewshed {
                v.visible_tiles
                    .contains(&Point::new(phys.pos.x as i32, phys.pos.y as i32))
            } else {
                true
            };
            if draw {
                let d: Vec2 = phys.pos.to_f32().to_array().into();
                self.instances.push(
                    graphics::DrawParam::new()
                        .dest(d * 12.)
                        .src(renderable.spr)
                        .color(renderable.color),
                );
            }
        }

        &self.instances
    }

    pub fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeat: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => {
                if let Some(action) = handle_input(input) {
                    self.world.insert_one(self.hero, action);
                }
                ()
            }
        };
        Ok(())
    }
}

fn handle_input(input: KeyInput) -> Option<Move> {
    match input.keycode {
        Some(KeyCode::Up) => Some(pt(0, -1)),
        Some(KeyCode::Down) => Some(pt(0, 1)),
        Some(KeyCode::Left) => Some(pt(-1, 0)),
        Some(KeyCode::Right) => Some(pt(1, 0)),
        //Some(KeyCode::Period) => Some(Action::Rest()),
        _ => None,
    }
}

fn ai_handler(world: &mut hecs::World, m: &Grid<Tile>) {}

fn move_handler(world: &mut hecs::World, m: &Grid<Tile>) {
    let mut moved: Vec<hecs::Entity> = vec![];
    for (e, (phys, d, viewshed)) in
        world.query_mut::<(&mut Physics, &Move, Option<&mut Viewshed>)>()
    {
        let n = phys.pos + d.to_vector();
        let t = m[n];
        if t.blocked() {
            continue;
        }
        phys.pos = n;
        if let Some(v) = viewshed {
            v.dirty = true;
        }
        moved.push(e);
    }
    for e in moved {
        world.remove_one::<Move>(e);
    }
}

fn fov_handler(world: &mut hecs::World, m: &Grid<Tile>) {
    let opaque_at = |p: Point| {
        if p.x >= 0 && p.x < m.width as i32 && p.y >= 0 && p.y < m.height as i32 {
            m[p].opaque()
        } else {
            false
        }
    };
    for (_, (phys, viewshed)) in world.query_mut::<(&Physics, &mut Viewshed)>() {
        if viewshed.dirty {
            viewshed.visible_tiles = fov::calculate(phys.pos, viewshed.range, opaque_at);
            viewshed.dirty = false;
        }
    }
}

struct Physics {
    pos: Point,
}

struct Viewshed {
    visible_tiles: HashSet<Point>,
    range: i32,
    dirty: bool,
}

#[derive(Debug)]
struct Name(String);

struct Player;

type Move = Point;

struct Map {
    tiles: Grid<Tile>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
    StairUp,
}

impl Tile {
    fn blocked(&self) -> bool {
        match self {
            Tile::Wall => true,
            _ => false,
        }
    }

    fn opaque(&self) -> bool {
        match self {
            Tile::Wall => true,
            _ => false,
        }
    }
}
