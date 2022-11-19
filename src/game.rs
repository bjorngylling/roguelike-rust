use crate::{
    fov,
    geom::{pt, Grid, Point},
    gfx,
    scene::{Scene, Transition},
};
use ggez::{
    glam::*,
    graphics,
    input::keyboard::{KeyCode, KeyInput, KeyMods},
    Context, GameResult,
};
use hecs;
use std::collections::HashSet;

pub struct GameState {
    world: hecs::World,
    hero: hecs::Entity,
    sprite_set: gfx::SpriteSet,
    map: Map,
    pub input: KeyState,
}

impl GameState {
    pub fn new(
        world: hecs::World,
        hero: hecs::Entity,
        sprite_set: gfx::SpriteSet,
        map: Map,
    ) -> Self {
        GameState { world, hero, sprite_set, map, input: KeyState::default() }
    }
}

#[derive(Debug, Default)]
pub struct KeyState {
    pub key: Option<KeyCode>,
    pub mods: Option<KeyMods>,
    pub repeat: bool,
}

pub struct Game {
    instances: graphics::InstanceArray,
    width: i32,
    height: i32,
}

impl Game {
    pub fn new(ctx: &mut Context, state: &mut GameState, width: i32, height: i32) -> Self {
        let mut instances = graphics::InstanceArray::new(ctx, state.sprite_set.img.clone());
        instances.resize(ctx, (width * height) as u32 + 50); // mapsize + 50 entities

        let mut hero_pos = pt(10, 10);
        for x in 0..state.map.tiles.width {
            for y in 0..state.map.tiles.height {
                if state.map.tiles[(x, y)] == Tile::StairUp {
                    hero_pos.x = x;
                    hero_pos.y = y;
                }
            }
        }
        state.world.insert(state.hero, (
            Player,
            Name("Hero".to_string()),
            Position(hero_pos),
            gfx::Renderable {
                spr: state.sprite_set.src_by_idx(gfx::CP437::ChAt as i32),
                color: gfx::WHITE_BRIGHT,
            },
            /*Viewshed {
                visible_tiles: HashSet::new(),
                range: 7,
                dirty: true,
            },*/
        ));
        state.world.spawn((
            Name("Giant Ant".to_string()),
            AI,
            Position(pt(20, 14)),
            gfx::Renderable {
                spr: state.sprite_set.src_by_idx(gfx::CP437::Cha as i32),
                color: gfx::BLUE_BRIGHT,
            },
        ));
        Game {
            instances,
            width,
            height,
        }
    }
}

impl Scene<GameState> for Game {
    fn update(&mut self, ctx: &mut Context, state: &mut GameState) -> Transition<GameState> {
        if input_handler(&mut state.world, state.hero, &state.input) {
            // Monsters only act when the player acts
            ai_handler(&mut state.world, &state.map.tiles);
        }
        move_handler(&mut state.world, &state.map.tiles);
        fov_handler(&mut state.world, &state.map.tiles);

        Transition::None
    }

    fn draw(&mut self, ctx: &mut Context, state: &mut GameState) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, gfx::BACKGROUND);

        // Currently broken, https://github.com/ggez/ggez/issues/1127
        canvas.set_sampler(graphics::Sampler::nearest_clamp());

        self.instances.clear();
        let map_layer = &state.map.tiles;
        let viewshed = state.world.get::<&Viewshed>(state.hero);
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
                        Tile::Floor => state.sprite_set.src_by_idx(gfx::CP437::ChDot as i32),
                        Tile::StairUp => state.sprite_set.src_by_idx(gfx::CP437::LessThan as i32),
                        _ => state.sprite_set.src_by_idx(gfx::CP437::Pillar as i32),
                    };
                    self.instances
                        .push(graphics::DrawParam::new().dest(d * 12.).src(spr));
                }
            }
        }
        for (_, (pos, renderable)) in state
            .world
            .query::<(&Position, &gfx::Renderable)>()
            .into_iter()
        {
            let draw = if let Ok(v) = &viewshed {
                v.visible_tiles
                    .contains(&Point::new(pos.0.x as i32, pos.0.y as i32))
            } else {
                true
            };
            if draw {
                let d: Vec2 = pos.0.to_f32().to_array().into();
                self.instances.push(
                    graphics::DrawParam::new()
                        .dest(d * 12.)
                        .src(renderable.spr)
                        .color(renderable.color),
                );
            }
        }
        let scale = Vec2::splat(
            (canvas.scissor_rect().w / (self.width as f32 * 12.))
                .min(canvas.scissor_rect().h / (self.height as f32 * 12.)),
        );
        canvas.draw(
            &self.instances,
            graphics::DrawParam::new()
                .dest(Vec2::new(
                    (canvas.scissor_rect().w - (self.width as f32 * 12.) * scale.x) / 2.,
                    (canvas.scissor_rect().h - (self.height as f32 * 12.) * scale.y) / 2.,
                ))
                .scale(scale),
        );

        canvas.finish(ctx)
    }

    fn key_down(&mut self, input: KeyInput, _repeat: bool) -> Transition<GameState> {
        match input.keycode {
            Some(KeyCode::Escape) => Transition::Pop,
            _ => Transition::None,
        }
    }
}

fn input_handler(world: &mut hecs::World, hero: hecs::Entity, input: &KeyState) -> bool {
    match input.key {
        Some(KeyCode::Up) => world.insert_one(hero, Move(pt(0, -1))),
        Some(KeyCode::Down) => world.insert_one(hero, Move(pt(0, 1))),
        Some(KeyCode::Left) => world.insert_one(hero, Move(pt(-1, 0))),
        Some(KeyCode::Right) => world.insert_one(hero, Move(pt(1, 0))),
        Some(KeyCode::Period) => Ok(()),
        _ => return false,
    };
    true
}

fn ai_handler(world: &mut hecs::World, _m: &Grid<Tile>) {
    let moves: Vec<_> = world
        .query::<()>()
        .with::<&AI>()
        .iter()
        .map(|(e, _)| (e, Move(pt(-1, 0))))
        .collect();
    for (e, mv) in moves {
        world.insert_one(e, mv);
    }
}

fn move_handler(world: &mut hecs::World, m: &Grid<Tile>) {
    let mut moved: Vec<hecs::Entity> = vec![];
    for (e, (pos, d, viewshed)) in
        world.query_mut::<(&mut Position, &Move, Option<&mut Viewshed>)>()
    {
        let n = pos.0 + d.0.to_vector();
        let t = m[n];
        if t.blocked() {
            continue;
        }
        pos.0 = n;
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
    for (_, (pos, viewshed)) in world.query_mut::<(&Position, &mut Viewshed)>() {
        if viewshed.dirty {
            viewshed.visible_tiles = fov::calculate(pos.0, viewshed.range, opaque_at);
            viewshed.dirty = false;
        }
    }
}

struct Position(Point);

struct Viewshed {
    visible_tiles: HashSet<Point>,
    range: i32,
    dirty: bool,
}

#[derive(Debug)]
struct Name(String);

struct Player;
struct AI;

struct Move(Point);

pub struct Map {
    pub tiles: Grid<Tile>,
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
