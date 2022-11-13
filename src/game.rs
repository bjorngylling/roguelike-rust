use std::collections::HashSet;

use crate::{
    fov,
    geom::{pt, Grid, Point},
    gfx,
};
use ggez::glam::*;
use ggez::{
    graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};

type EntityId = usize;
pub struct MainState {
    instances: ggez::graphics::InstanceArray,
    sprite_set: gfx::SpriteSet,
    hero_id: EntityId,
    entities: Vec<Entity>,
    map_layer: Grid<Tile>,
}

impl MainState {
    pub fn new(ctx: &mut Context, width: i32, height: i32, map_layer: Grid<Tile>) -> GameResult<MainState> {
        let sprite_set = gfx::SpriteSet::new(16, 16, 12, 12);
        let image = graphics::Image::from_path(ctx, "/nice-curses.png")?;
        let mut instances = graphics::InstanceArray::new(ctx, image);
        instances.resize(ctx, (width * height) as u32 + 50); // mapsize + 50 entities

        let entities = vec![
            Entity {
                name: "Hero".to_string(),
                physics: Physics { pos: pt(10, 10) },
                renderable: gfx::Renderable {
                    spr: sprite_set.src_by_idx(gfx::CP437::ChAt as i32),
                    color: gfx::WHITE_BRIGHT,
                },
                next_action: Some(Action::UpdateViewshed(0)), // queue a viewshed update
                player: true,
                viewshed: Some(Viewshed {
                    visible_tiles: HashSet::new(),
                    range: 7,
                }),
            },
            Entity {
                name: "Giant Ant".to_string(),
                physics: Physics { pos: pt(20, 14) },
                renderable: gfx::Renderable {
                    spr: sprite_set.src_by_idx(gfx::CP437::Cha as i32),
                    color: gfx::BLUE_BRIGHT,
                },
                next_action: None,
                player: false,
                viewshed: None,
            },
        ];
        Ok(MainState {
            instances,
            sprite_set,
            hero_id: 0,
            entities,
            map_layer,
        })
    }

    pub fn update(&mut self) -> GameResult {
        let hero = &mut self.entities[self.hero_id];
        let mut action = hero.next_action.take();
        let player_took_action = action.is_some();
        while let Some(a) = action {
            use Action::*;
            action = match a {
                Move(id, d) => move_handler(id, &mut self.entities, d, &self.map_layer),
                Rest(_) => None,
                Attack(id, target) => attack_handler(id, target, &self.entities),
                UpdateViewshed(id) => fov_handler(id, &mut self.entities, &self.map_layer),
            }
        }
        if player_took_action {
            for id in 0..self.entities.len() {
                if !self.entities[id].player {
                    let mut action = ai_handler(id, &self.entities[id]);
                    while let Some(a) = action {
                        use Action::*;
                        action = match a {
                            Move(id, d) => move_handler(id, &mut self.entities, d, &self.map_layer),
                            Rest(_) => None,
                            Attack(id, target) => attack_handler(id, target, &self.entities),
                            UpdateViewshed(id) => fov_handler(id, &mut self.entities, &self.map_layer)
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, _ctx: &mut Context) -> &graphics::InstanceArray {
        self.instances.clear();
        let map_layer = &self.map_layer;
        let viewshed = &self.entities[self.hero_id].viewshed;
        for x in 0..map_layer.width {
            for y in 0..map_layer.height {
                let pos = pt(x as i32, y as i32);
                let draw = if let Some(v) = viewshed {
                    v.visible_tiles.contains(&pos)
                } else {
                    true
                };
                if draw {
                    let t = map_layer[(x, y)];
                    let d: Vec2 = pos.to_f32().to_array().into();
                    let spr = if t == Tile::Floor {
                        self.sprite_set.src_by_idx(gfx::CP437::ChDot as i32)
                    } else {
                        self.sprite_set.src_by_idx(gfx::CP437::Pillar as i32)
                    };
                    self.instances
                        .push(graphics::DrawParam::new().dest(d * 12.).src(spr));
                }
            }
        }
        self.entities.iter().for_each(|m| {
            let draw = if let Some(v) = viewshed {
                v.visible_tiles
                    .contains(&Point::new(m.physics.pos.x as i32, m.physics.pos.y as i32))
            } else {
                true
            };
            if draw {
                let d: Vec2 = m.physics.pos.to_f32().to_array().into();
                self.instances.push(
                    graphics::DrawParam::new()
                        .dest(d * 12.)
                        .src(m.renderable.spr)
                        .color(m.renderable.color),
                );
            }
        });

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
            _ => self.entities[self.hero_id].next_action = handle_input(input, self.hero_id),
        };
        Ok(())
    }
}

fn handle_input(input: KeyInput, id: EntityId) -> Option<Action> {
    match input.keycode {
        Some(KeyCode::Up) => Some(Action::Move(id, pt(0, -1))),
        Some(KeyCode::Down) => Some(Action::Move(id, pt(0, 1))),
        Some(KeyCode::Left) => Some(Action::Move(id, pt(-1, 0))),
        Some(KeyCode::Right) => Some(Action::Move(id, pt(1, 0))),
        Some(KeyCode::Period) => Some(Action::Rest(id)),
        _ => None,
    }
}

enum Action {
    Move(EntityId, Point),
    Rest(EntityId),
    Attack(EntityId, EntityId),
    UpdateViewshed(EntityId),
}

fn ai_handler(id: EntityId, _ent: &Entity) -> Option<Action> {
    Some(Action::Move(id, pt(-1, 0)))
}

fn move_handler(id: usize, entities: &mut [Entity], d: Point, m: &Grid<Tile>) -> Option<Action> {
    let n = entities[id].physics.pos + d.to_vector();
    let t = m[n];
    if t.blocked() {
        return None;
    }
    if let Some(other) = entities.iter().position(|e| e.physics.pos == n) {
        Some(Action::Attack(id, other))
    } else {
        entities[id].physics.pos = n;
        if entities[id].viewshed.is_some() {
            Some(Action::UpdateViewshed(id))
        } else {
            None
        }
    }
}

fn attack_handler(id: EntityId, target: EntityId, entities: &[Entity]) -> Option<Action> {
    println!(
        "The {} strikes the {}",
        entities[id].name.to_lowercase(),
        entities[target].name.to_lowercase()
    );
    None
}

fn fov_handler(id: usize, entities: &mut [Entity], m: &Grid<Tile>) -> Option<Action> {
    let opaque_at = |p: Point| {
        if p.x >= 0 && p.x < m.width as i32 && p.y >= 0 && p.y < m.height as i32 {
            m[p].opaque()
        } else {
            false
        }
    };
    if let Some(v) = &mut entities[id].viewshed {
        v.visible_tiles = fov::calculate(entities[id].physics.pos, v.range, opaque_at);
    }
    None
}

struct Physics {
    pos: Point,
}

struct Viewshed {
    visible_tiles: HashSet<Point>,
    range: i32,
}

struct Entity {
    name: String,
    physics: Physics,
    next_action: Option<Action>,
    renderable: gfx::Renderable,
    player: bool,
    viewshed: Option<Viewshed>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
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
