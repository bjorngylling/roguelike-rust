use ggez::{
    conf, event,
    glam::*,
    graphics,
    input::keyboard::{KeyCode, KeyInput},
    timer, Context, GameResult,
};

mod game;
mod gfx;
mod fov;
mod geom;

const SCREEN_WIDTH_TILES: i32 = 60;
const SCREEN_HEIGHT_TILES: i32 = 35;

struct App {
    game: game::MainState,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        match game::MainState::new(ctx, SCREEN_WIDTH_TILES, SCREEN_HEIGHT_TILES) {
            Ok(game) => Ok(App { game }),
            Err(e) => Err(e),
        }
    }
}

impl event::EventHandler<ggez::GameError> for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            timer::sleep(std::time::Duration::from_secs(0));
        }

        self.game.update()
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let game_view = self.game.draw(ctx);

        let mut canvas = graphics::Canvas::from_frame(ctx, gfx::BACKGROUND);

        // Currently broken, https://github.com/ggez/ggez/issues/1127
        canvas.set_sampler(graphics::Sampler::nearest_clamp());

        let scale = Vec2::splat(
            (canvas.scissor_rect().w / (SCREEN_WIDTH_TILES as f32 * 12.))
                .min(canvas.scissor_rect().h / (SCREEN_HEIGHT_TILES as f32 * 12.)),
        );
        canvas.draw(
            game_view,
            graphics::DrawParam::new()
                .dest(Vec2::new(
                    (canvas.scissor_rect().w - (SCREEN_WIDTH_TILES as f32 * 12.) * scale.x) / 2.,
                    (canvas.scissor_rect().h - (SCREEN_HEIGHT_TILES as f32 * 12.) * scale.y) / 2.,
                ))
                .scale(scale),
        );

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::Escape) => {
                ctx.request_quit();
                Ok(())
            }
            _ => self.game.key_down_event(ctx, input, _repeat),
        }
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

    let state = App::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
