use std::collections::HashSet;

use crate::{fov, gfx};
use ggez::glam::*;
use ggez::{
    graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use ndarray::Array2;

type EntityId = usize;
pub struct MainState {
    instances: ggez::graphics::InstanceArray,
    sprite_set: gfx::SpriteSet,
    hero_id: EntityId,
    entities: Vec<Entity>,
    map_layer: MapLayer,
}

impl MainState {
    pub fn new(ctx: &mut Context, width: u32, height: u32) -> GameResult<MainState> {
        let sprite_set = gfx::SpriteSet::new(16, 16, 12, 12);
        let image = graphics::Image::from_path(ctx, "/nice-curses.png")?;
        let mut instances = graphics::InstanceArray::new(ctx, image);
        instances.resize(ctx, (width * height) + 50); // mapsize + 50 entities

        let mut map_layer = MapLayer::from_elem(
            (width as usize, height as usize),
            Tile {
                block: false,
                renderable: Renderable {
                    spr: sprite_set.src(14, 2),
                    color: gfx::WHITE,
                },
            },
        );
        for x in [0, 1, 2, 3, 5, 7, 9, 12, 13, 14] {
            map_layer[[x as usize, 2]].renderable.spr = sprite_set.src(3, 2);
            map_layer[[x as usize, 2]].block = true;
        }
        let entities = vec![
            Entity {
                name: "Hero".to_string(),
                physics: Physics {
                    pos: vec2(10., 10.),
                },
                renderable: Renderable {
                    spr: sprite_set.src(0, 4),
                    color: gfx::WHITE_BRIGHT,
                },
                next_action: Some(Action::UpdateViewshed(0)), // queue a viewshed update
                player: true,
                viewshed: Some(Viewshed {
                    visible_tiles: HashSet::new(),
                    range: 5,
                }),
            },
            Entity {
                name: "Giant Ant".to_string(),
                physics: Physics {
                    pos: vec2(20., 14.),
                },
                renderable: Renderable {
                    spr: sprite_set.src(1, 6),
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
                            UpdateViewshed(id) => {
                                fov_handler(id, &mut self.entities, &self.map_layer)
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn draw(&mut self, _ctx: &mut Context) -> &graphics::InstanceArray {
        self.instances.clear();
        let map_layer = &self.map_layer.view();
        let viewshed = &self.entities[self.hero_id].viewshed;
        for x in 0..map_layer.nrows() {
            for y in 0..map_layer.ncols() {
                let draw = if let Some(v) = viewshed {
                    v.visible_tiles
                        .contains(&fov::Point::new(x as i32, y as i32))
                } else {
                    true
                };
                if draw {
                    let t = map_layer[[x, y]];
                    self.instances.push(
                        graphics::DrawParam::new()
                            .dest(Vec2::new(x as f32 * 12., y as f32 * 12.))
                            .src(t.renderable.spr),
                    );
                }
            }
        }
        self.entities.iter().for_each(|m| {
            let draw = if let Some(v) = viewshed {
                v.visible_tiles.contains(&fov::Point::new(
                    m.physics.pos.x as i32,
                    m.physics.pos.y as i32,
                ))
            } else {
                true
            };
            if draw {
                self.instances.push(
                    graphics::DrawParam::new()
                        .dest(m.physics.pos * 12.)
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
        Some(KeyCode::Up) => Some(Action::Move(id, vec2(0., -1.))),
        Some(KeyCode::Down) => Some(Action::Move(id, vec2(0., 1.))),
        Some(KeyCode::Left) => Some(Action::Move(id, vec2(-1., 0.))),
        Some(KeyCode::Right) => Some(Action::Move(id, vec2(1., 0.))),
        Some(KeyCode::Period) => Some(Action::Rest(id)),
        _ => None,
    }
}

enum Action {
    Move(EntityId, Vec2),
    Rest(EntityId),
    Attack(EntityId, EntityId),
    UpdateViewshed(EntityId),
}

fn ai_handler(id: EntityId, _ent: &Entity) -> Option<Action> {
    Some(Action::Move(id, vec2(-1., 0.)))
}

fn move_handler(id: usize, entities: &mut Vec<Entity>, d: Vec2, m: &MapLayer) -> Option<Action> {
    let n = entities[id].physics.pos + d;
    let t = m[[n.x as usize, n.y as usize]];
    if t.block {
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

fn attack_handler(id: EntityId, target: EntityId, entities: &Vec<Entity>) -> Option<Action> {
    println!(
        "The {} strikes the {}",
        entities[id].name.to_lowercase(),
        entities[target].name.to_lowercase()
    );
    None
}

fn fov_handler(id: usize, entities: &mut [Entity], m: &MapLayer) -> Option<Action> {
    let m = m.view();
    let opaque_at = |p: fov::Point| {
        if p.x >= 0 && p.x < m.ncols() as i32 && p.y >= 0 && p.y < m.nrows() as i32 {
            m[[p.x as usize, p.y as usize]].block
        } else {
            false
        }
    };
    if let Some(v) = &mut entities[id].viewshed {
        let p = fov::Point::new(
            entities[id].physics.pos.x as i32,
            entities[id].physics.pos.y as i32,
        );
        v.visible_tiles = fov::calculate(p, v.range, opaque_at);
    }
    None
}

struct Physics {
    pos: Vec2,
}

#[derive(Copy, Clone)]
struct Renderable {
    spr: graphics::Rect,
    color: graphics::Color,
}

struct Viewshed {
    visible_tiles: HashSet<fov::Point>,
    range: i32,
}

struct Entity {
    name: String,
    physics: Physics,
    next_action: Option<Action>,
    renderable: Renderable,
    player: bool,
    viewshed: Option<Viewshed>,
}

type MapLayer = Array2<Tile>;

#[derive(Copy, Clone)]
struct Tile {
    renderable: Renderable,
    block: bool,
}
