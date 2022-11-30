use ggez::{
    conf, event,
    glam::*,
    graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use mapgen::Generator;
use rand_seeder::{Seeder, SipRng};
use scene::{Scene, SceneStack, Transition};

mod fov;
mod game;
mod geom;
mod gfx;
mod mapgen;
mod scene;

const SCREEN_WIDTH_TILES: i32 = 60;
const SCREEN_HEIGHT_TILES: i32 = 35;

struct App {
    state: game::GameState,
    scenes: SceneStack<game::GameState>,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let image =
            graphics::Image::from_path(ctx, "/nice-curses.png").expect("unable to load resource");
        let sprite_set = gfx::SpriteSet::new(image, 16, 16, 12, 12);
        let world = hecs::World::new();
        let hero = world.reserve_entity();
        let mut state = game::GameState::new(
            world,
            hero,
            sprite_set,
            game::Map::new(SCREEN_WIDTH_TILES, SCREEN_HEIGHT_TILES),
        );

        let mut rng: SipRng = Seeder::from("helloworld").make_rng();
        let mut map_gen_visualizer =
            mapgen::SimpleMapGenerator::new(state.map.tiles.width, state.map.tiles.height);
        map_gen_visualizer.run(&mut rng, &mut state.map);
        let mut scenes = SceneStack::new(Box::new(game::Game::new(
            ctx,
            &mut state,
            SCREEN_WIDTH_TILES,
            SCREEN_HEIGHT_TILES,
        )));
        scenes.push(Box::new(MapGenViewer {
            history: map_gen_visualizer.timeline(),
            cur: 0,
        }));

        Ok(App { state, scenes })
    }
}

impl event::EventHandler<ggez::GameError> for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            self.scenes.update(ctx, &mut self.state);

            // Clear input
            self.state.input.key = None;
            self.state.input.mods = None;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.scenes.draw(ctx, &mut self.state);
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeat: bool) -> GameResult {
        self.state.input.key = input.keycode;
        self.state.input.mods = Some(input.mods);
        self.state.input.repeat = repeat;

        self.scenes.input(ctx, input, repeat);

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

    let state = App::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

struct MapGenViewer {
    history: Vec<geom::Grid<graphics::Color>>,
    cur: usize,
}

impl MapGenViewer {
    fn grid_to_pixels(g: &geom::Grid<graphics::Color>) -> Vec<u8> {
        g.iter()
            .flat_map(|c| vec![c.to_rgba().0, c.to_rgba().1, c.to_rgba().2, c.to_rgba().3])
            .collect()
    }
}

impl Scene<game::GameState> for MapGenViewer {
    fn update(
        &mut self,
        _ctx: &mut Context,
        _state: &mut game::GameState,
    ) -> scene::Transition<game::GameState> {
        Transition::None
    }

    fn draw(&mut self, ctx: &mut Context, _state: &mut game::GameState) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, gfx::BACKGROUND);

        // Currently broken, https://github.com/ggez/ggez/issues/1127
        canvas.set_sampler(graphics::Sampler::nearest_clamp());
        let img = graphics::Image::from_pixels(
            ctx,
            &MapGenViewer::grid_to_pixels(&self.history[self.cur]),
            graphics::ImageFormat::Rgba8UnormSrgb,
            SCREEN_WIDTH_TILES as u32,
            SCREEN_HEIGHT_TILES as u32,
        );
        let scale = Vec2::splat(
            (canvas.scissor_rect().w / (SCREEN_WIDTH_TILES as f32))
                .min(canvas.scissor_rect().h / (SCREEN_HEIGHT_TILES as f32)),
        );
        canvas.draw(&img, graphics::DrawParam::new().scale(scale));
        canvas.finish(ctx)
    }

    fn key_down(&mut self, input: KeyInput, _repeat: bool) -> scene::Transition<game::GameState> {
        match input.keycode {
            Some(KeyCode::Escape) => Transition::Pop,
            Some(KeyCode::Right) => {
                if self.cur < self.history.len() - 1 {
                    self.cur += 1;
                };
                Transition::None
            }
            Some(KeyCode::Left) => {
                if self.cur > 0 {
                    self.cur -= 1;
                }
                Transition::None
            }
            _ => Transition::None,
        }
    }
}
