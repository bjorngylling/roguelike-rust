use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{conf, event, graphics, timer, Context, GameResult};
use ggez::{glam::*, GameError};

mod gfx;

const SCREEN_WIDTH_TILES: u32 = 60;
const SCREEN_HEIGHT_TILES: u32 = 35;

struct MainState {
    instances: ggez::graphics::InstanceArray,
    sprite_set: gfx::SpriteSet,
    hero: Hero,
    monsters: Vec<Monster>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sprite_set = gfx::SpriteSet::new(16, 16, 12, 12);
        let image = graphics::Image::from_path(ctx, "/nice-curses.png")?;
        let mut instances = graphics::InstanceArray::new(ctx, image);
        instances.resize(ctx, SCREEN_WIDTH_TILES * SCREEN_HEIGHT_TILES);
        let hero = Hero {
            physics: Physics {
                pos: vec2(10., 10.),
            },
            spr: sprite_set
                .src(0, 4)
                .ok_or_else(|| GameError::CustomError(String::from("invalid sprite")))?,
            next_action: None,
        };
        let monster = Monster {
            physics: Physics {
                pos: vec2(20., 14.),
            },
            spr: sprite_set
                .src(1, 6)
                .ok_or_else(|| GameError::CustomError(String::from("invalid sprite")))?,
        };
        Ok(MainState {
            instances,
            sprite_set,
            hero,
            monsters: vec![monster],
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            timer::sleep(std::time::Duration::from_secs(0));
        }

        let mut action = self.hero.get_action();
        let player_took_action = action.is_some();
        while let Some(a) = action {
            action = a.perform(&mut self.hero.physics)
        }
        if player_took_action {
            for m in &mut self.monsters {
                let mut action = m.get_action();
                while let Some(a) = action {
                    action = a.perform(&mut m.physics)
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, gfx::BACKGROUND);

        // Currently broken, https://github.com/ggez/ggez/issues/1127
        canvas.set_sampler(graphics::Sampler::nearest_clamp());

        let tilesheet = &self.sprite_set;
        self.instances.set((0..SCREEN_WIDTH_TILES).flat_map(|x| {
            (0..SCREEN_HEIGHT_TILES).map(move |y| {
                let x = x as f32;
                let y = y as f32;
                graphics::DrawParam::new()
                    .dest(Vec2::new(x * 12., y * 12.))
                    .src(tilesheet.src(14, 2).unwrap())
            })
        }));
        self.instances.push(
            graphics::DrawParam::new()
                .dest(self.hero.physics.pos * 12.)
                .src(self.hero.spr),
        );
        self.monsters.iter().for_each(|m| {
            self.instances.push(
                graphics::DrawParam::new()
                    .dest(m.physics.pos * 12.)
                    .src(m.spr),
            );
        });

        let scale = Vec2::splat(
            (canvas.scissor_rect().w / (SCREEN_WIDTH_TILES as f32 * 12.))
                .min(canvas.scissor_rect().h / (SCREEN_HEIGHT_TILES as f32 * 12.)),
        );
        canvas.draw(
            &self.instances,
            graphics::DrawParam::new()
                .dest(Vec2::new(
                    (canvas.scissor_rect().w - (SCREEN_WIDTH_TILES as f32 * 12.) * scale.x) / 2.,
                    (canvas.scissor_rect().h - (SCREEN_HEIGHT_TILES as f32 * 12.) * scale.y) / 2.,
                ))
                .scale(scale),
        );

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => self.hero.handle_input(input),
        };
        Ok(())
    }
}

struct Move {
    d: Vec2,
}

impl Action for Move {
    fn perform(&self, p: &mut Physics) -> Option<Box<dyn Action>> {
        p.pos += self.d;
        None
    }
}

trait Action {
    fn perform(&self, actor: &mut Physics) -> Option<Box<dyn Action>>;
}

struct Physics {
    pos: Vec2,
}

struct Hero {
    physics: Physics,
    spr: graphics::Rect,
    next_action: Option<Box<dyn Action>>,
}

impl Hero {
    fn handle_input(&mut self, input: KeyInput) {
        self.next_action = match input.keycode {
            Some(KeyCode::Up) => Some(Box::new(Move { d: vec2(0., -1.) })),
            Some(KeyCode::Down) => Some(Box::new(Move { d: vec2(0., 1.) })),
            Some(KeyCode::Left) => Some(Box::new(Move { d: vec2(-1., 0.) })),
            Some(KeyCode::Right) => Some(Box::new(Move { d: vec2(1., 0.) })),
            _ => None,
        }
    }

    fn get_action(&mut self) -> Option<Box<dyn Action>> {
        self.next_action.take()
    }
}

struct Monster {
    physics: Physics,
    spr: graphics::Rect,
}

impl Monster {
    fn get_action(&mut self) -> Option<Box<dyn Action>> {
        Some(Box::new(Move { d: vec2(0., -1.) }))
    }
}

fn main() -> GameResult {
    use std::{env, path};
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("roguelike-rust", "bjorngylling")
        .window_mode(
            conf::WindowMode::default()
                .resizable(true)
                .dimensions(1440.0, 960.0)
                .min_dimensions(1440.0, 960.0),
        )
        .window_setup(conf::WindowSetup::default().title("roguelike-rust"))
        .add_resource_path(resource_dir);

    let (mut ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("roguelike-rust");
    ctx.gfx.set_resizable(true)?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
