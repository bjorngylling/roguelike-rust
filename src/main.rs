use ggez::glam::*;
use ggez::{conf, event, graphics, timer, Context, GameResult};

mod gfx;

const SCREEN_WIDTH_TILES: u32 = 60;
const SCREEN_HEIGHT_TILES: u32 = 35;

struct MainState {
    instances: ggez::graphics::InstanceArray,
    tilesheet: gfx::SpriteSet,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let tilesheet = gfx::SpriteSet::new(16, 16, 12, 12);
        let image = graphics::Image::from_path(ctx, "/nice-curses.png")?;
        let mut instances = graphics::InstanceArray::new(ctx, image);
        instances.resize(ctx, SCREEN_WIDTH_TILES * SCREEN_HEIGHT_TILES);
        Ok(MainState {
            instances,
            tilesheet,
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            timer::sleep(std::time::Duration::from_secs(0));
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Currently broken, https://github.com/ggez/ggez/issues/1127
        canvas.set_sampler(graphics::Sampler::nearest_clamp());

        let tilesheet = &self.tilesheet;
        self.instances.set((0..SCREEN_WIDTH_TILES).flat_map(|x| {
            (0..SCREEN_HEIGHT_TILES).map(move |y| {
                let x = x as f32;
                let y = y as f32;
                graphics::DrawParam::new()
                    .dest(Vec2::new(x * 12.0, y * 12.0))
                    // 16x16 tiles, tile: 12x12 px
                    .src(tilesheet.src(0, 4).unwrap())
            })
        }));

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
