use crate::{
    fov,
    geom::{pt, Grid, Point},
    gfx::{self, Renderable},
    scene::{Scene, Transition},
};
use ggez::{
    glam::*,
    graphics,
    input::keyboard::{KeyCode, KeyInput, KeyMods},
    Context, GameResult,
};
use core::time;
use std::collections::HashSet;

pub struct GameState {
    world: hecs::World,
    hero: hecs::Entity,
    sprite_set: gfx::SpriteSet,
    chan: shrev::EventChannel<Event>,
    pub map: Map,
    pub input: KeyState,
}

impl GameState {
    pub fn new(
        world: hecs::World,
        hero: hecs::Entity,
        sprite_set: gfx::SpriteSet,
        map: Map,
    ) -> Self {
        GameState {
            world,
            hero,
            sprite_set,
            map,
            input: KeyState::default(),
            chan: shrev::EventChannel::new(),
        }
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
    move_reader: shrev::ReaderId<Event>,
    collision_reader: shrev::ReaderId<Event>,
    damage_reader: shrev::ReaderId<Event>,
    dt: time::Duration,
}

impl Game {
    pub fn new(ctx: &mut Context, state: &mut GameState, width: i32, height: i32) -> Self {
        let mut instances = graphics::InstanceArray::new(ctx, state.sprite_set.img.clone());
        instances.resize(ctx, (width * height) as u32 + 50); // mapsize + 50 entities

        state
            .world
            .insert(
                state.hero,
                (
                    Player,
                    Name("Hero".to_string()),
                    Position(state.map.entrance),
                    BlocksTile,
                    gfx::Renderable {
                        spr: gfx::CP437::ChAt,
                        color: gfx::WHITE_BRIGHT,
                    },
                    Viewshed {
                        visible_tiles: HashSet::new(),
                        range: 7,
                        dirty: true,
                    },
                ),
            )
            .expect("hero entity missing");
        state.world.spawn((
            Name("Giant Ant".to_string()),
            AI,
            Position(pt(20, 13)),
            BlocksTile,
            gfx::Renderable {
                spr: gfx::CP437::Cha,
                color: gfx::BLUE_BRIGHT,
            },
        ));
        state.world.spawn((
            Name("Giant Ant".to_string()),
            AI,
            Position(state.map.entrance + pt(10, -1).to_vector()),
            BlocksTile,
            gfx::Renderable {
                spr: gfx::CP437::Cha,
                color: gfx::BLUE_BRIGHT,
            },
        ));
        state.world.spawn((
            Name("Exploding Flask".to_string()),
            Position(state.map.entrance + pt(1, -1).to_vector()),
            gfx::Renderable {
                spr: gfx::CP437::Trap,
                color: gfx::YELLOW_BRIGHT,
            },
            Explosive { radius: 3 },
        ));
        Game {
            instances,
            width,
            height,
            move_reader: state.chan.register_reader(),
            collision_reader: state.chan.register_reader(),
            damage_reader: state.chan.register_reader(),
            dt: time::Duration::default(),
        }
    }
}

impl Scene<GameState> for Game {
    fn update(&mut self, ctx: &mut Context, state: &mut GameState) -> Transition<GameState> {
        map_indexing_handler(&state.world, &mut state.map);
        if input_handler(&state.input, state.hero, &mut state.chan) {
            // Monsters only act when the player acts
            ai_handler(&state.world, &mut state.chan);
        }
        move_handler(
            &mut state.world,
            &state.map,
            &mut state.chan,
            &mut self.move_reader,
        );
        collision_handler(
            &mut state.world,
            &state.map,
            &mut state.chan,
            &mut self.collision_reader,
        );
        damage_handler(&mut state.world, &state.chan, &mut self.damage_reader);
        fov_handler(&mut state.world, &state.map.tiles, &mut state.map.explored);
        
        self.dt += ctx.time.delta();
        if self.dt > time::Duration::new(0, 100000000) {
            explosion_handler(&mut state.world);
            self.dt = time::Duration::default();
        }

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
                let in_los = if let Ok(v) = &viewshed {
                    v.visible_tiles.contains(&pos)
                } else {
                    true
                };
                let explored = state.map.explored[pos];
                if in_los || explored {
                    let t = map_layer[(x, y)];
                    let d: Vec2 = pos.to_f32().to_array().into();
                    let spr = match t {
                        Tile::Floor => gfx::CP437::ChDot,
                        Tile::StairUp => gfx::CP437::LessThan,
                        _ => gfx::CP437::Pillar,
                    };
                    let mut draw = graphics::DrawParam::new()
                        .dest(d * 12.)
                        .src(state.sprite_set.src(spr));
                    if explored && !in_los {
                        draw.color.a *= 0.2;
                    }
                    self.instances.push(draw);
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
                        .src(state.sprite_set.src(renderable.spr))
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

fn input_handler(input: &KeyState, hero: hecs::Entity, chan: &mut EventChan) -> bool {
    match input.key {
        Some(KeyCode::Up) => chan.single_write(Event::Move(hero, pt(0, -1))),
        Some(KeyCode::Down) => chan.single_write(Event::Move(hero, pt(0, 1))),
        Some(KeyCode::Left) => chan.single_write(Event::Move(hero, pt(-1, 0))),
        Some(KeyCode::Right) => chan.single_write(Event::Move(hero, pt(1, 0))),
        Some(KeyCode::Period) => (),
        _ => return false,
    };
    true
}

fn ai_handler(world: &hecs::World, chan: &mut EventChan) {
    world
        .query::<()>()
        .with::<&AI>()
        .iter()
        .map(|(e, _)| Event::Move(e, pt(-1, 0)))
        .for_each(|e| chan.single_write(e));
}

fn move_handler(
    world: &mut hecs::World,
    map: &Map,
    chan: &mut EventChan,
    r: &mut shrev::ReaderId<Event>,
) {
    let mut collisions: Vec<Event> = vec![];
    for ev in chan.read(r) {
        if let Event::Move(e, m) = ev {
            if let Ok((pos, viewshed)) =
                world.query_one_mut::<(&mut Position, Option<&mut Viewshed>)>(*e)
            {
                let n = pos.0 + m.to_vector();
                for other in &map.entities[n] {
                    collisions.push(Event::Collision(*e, *other));
                }
                if map.blocked[n] {
                    continue;
                }
                pos.0 = n;
                if let Some(v) = viewshed {
                    v.dirty = true;
                }
            }
        };
    }
    chan.drain_vec_write(&mut collisions);
}

fn fov_handler(world: &mut hecs::World, m: &Grid<Tile>, explored: &mut Grid<bool>) {
    let opaque_at = |p: Point| {
        if p.x >= 0 && p.x < m.width as i32 && p.y >= 0 && p.y < m.height as i32 {
            m[p].opaque()
        } else {
            false
        }
    };
    for (_, (pos, viewshed, player)) in
        world.query_mut::<(&Position, &mut Viewshed, Option<&Player>)>()
    {
        if viewshed.dirty {
            viewshed.visible_tiles = fov::calculate(pos.0, viewshed.range, opaque_at);
            viewshed.dirty = false;
            if player.is_some() {
                for p in &viewshed.visible_tiles {
                    explored[*p] = true;
                }
            }
        }
    }
}

fn map_indexing_handler(world: &hecs::World, m: &mut Map) {
    m.clear_entities();
    m.calc_blocked_from_tile();

    for (e, (pos, blocks)) in world.query::<(&Position, Option<&BlocksTile>)>().iter() {
        m.entities[pos.0].push(e);

        if blocks.is_some() {
            m.blocked[pos.0] = true;
        }
    }
}

fn collision_handler(
    world: &mut hecs::World,
    map: &Map,
    chan: &mut EventChan,
    r: &mut shrev::ReaderId<Event>,
) {
    let mut events: Vec<Event> = vec![];
    for ev in chan.read(r) {
        if let Event::Collision(a, b) = ev {
            {
                let mut e = world.query_one::<&Name>(*a).unwrap();
                let mut other = world.query_one::<&Name>(*b).unwrap();
                println!(
                    "{} collided with {}.",
                    e.get().unwrap_or(&Name("unnamed".to_string())).0,
                    other.get().unwrap_or(&Name("unnamed".to_string())).0
                );
            }

            // Player attacks AI
            if let Ok(true) = world.satisfies::<&Player>(*a) {
                if let Ok(true) = world.satisfies::<&AI>(*b) {
                    events.push(Event::TakeDamage(*b, 1));
                }
            }

            // Colliding with Explosive sets it off
            let mut cmd = hecs::CommandBuffer::new();
            if let Ok(pos) = world.query_one_mut::<hecs::With<&Position, &Explosive>>(*b) {
                for p in [pt(0, 0), pt(1, 0), pt(-1, 0), pt(0, -1), pt(0, 1)] {
                    let n = pos.0 + p.to_vector();
                    cmd.spawn((
                        Explosion { duration_left: 6 },
                        Position(n),
                        Renderable {
                            spr: gfx::CP437::Filled3,
                            color: gfx::RED,
                        },
                    ));
                    for other in &map.entities[n] {
                        if other != b {
                            events.push(Event::TakeDamage(*other, 1))
                        }
                    }
                }
                cmd.despawn(*b);
            }
            cmd.run_on(world);
        }
    }
    chan.drain_vec_write(&mut events);
}

fn explosion_handler(world: &mut hecs::World) {
    let mut cmd = hecs::CommandBuffer::new();
    for (e, (exp, rd)) in world.query_mut::<(&mut Explosion, &mut Renderable)>() {
        exp.duration_left -= 1;
        match exp.duration_left {
            3 => rd.spr = gfx::CP437::Filled3,
            2 => rd.spr = gfx::CP437::Filled2,
            1 => {
                rd.spr = gfx::CP437::Filled1;
                rd.color = gfx::YELLOW_BRIGHT;
            },
            0 => cmd.despawn(e),
            _ => (),
        }
    }
    cmd.run_on(world);
}

fn damage_handler(world: &mut hecs::World, chan: &EventChan, r: &mut shrev::ReaderId<Event>) {
    for ev in chan.read(r) {
        if let Event::TakeDamage(e, _dmg) = ev {
            world.despawn(*e).expect("failed to despawn entity");
        }
    }
}

type EventChan = shrev::EventChannel<Event>;
#[derive(Clone)]
enum Event {
    Move(hecs::Entity, Point),
    Collision(hecs::Entity, hecs::Entity),
    TakeDamage(hecs::Entity, i32),
}

// Components
struct Position(Point);
struct BlocksTile;
struct Viewshed {
    visible_tiles: HashSet<Point>,
    range: i32,
    dirty: bool,
}
#[derive(Debug, Clone)]
struct Name(String);
struct Player;
struct AI;
struct Explosive {
    radius: u8,
}
struct Explosion {
    duration_left: u8,
}

// Map
pub struct Map {
    pub entrance: Point,
    pub tiles: Grid<Tile>,
    pub entities: Grid<Vec<hecs::Entity>>,
    pub blocked: Grid<bool>,
    pub explored: Grid<bool>,
}

impl Map {
    pub fn new(w: i32, h: i32) -> Self {
        let tiles = Grid::new(w, h, Tile::Wall);
        let entities = Grid::new(w, h, vec![]);
        let blocked = Grid::new(w, h, false);
        let explored = Grid::new(w, h, false);
        Map {
            entrance: pt(0, 0),
            tiles,
            entities,
            blocked,
            explored,
        }
    }

    fn clear_entities(&mut self) {
        for v in self.entities.iter_mut() {
            v.clear()
        }
    }

    fn calc_blocked_from_tile(&mut self) {
        for y in 0..self.tiles.height {
            for x in 0..self.tiles.width {
                self.blocked[(x, y)] = self.tiles[(x, y)].blocked()
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    StairUp,
}

impl Tile {
    fn blocked(&self) -> bool {
        matches!(self, Tile::Wall)
    }

    fn opaque(&self) -> bool {
        matches!(self, Tile::Wall)
    }
}
