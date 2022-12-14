#![allow(dead_code)]

use ggez::graphics;

pub const BLACK: graphics::Color = graphics::Color {
    r: 0.184,
    g: 0.18,
    b: 0.176,
    a: 1.,
};
pub const BLUE: graphics::Color = graphics::Color {
    r: 0.498,
    g: 0.647,
    b: 0.741,
    a: 1.,
};
pub const BLACK_BRIGHT: graphics::Color = graphics::Color {
    r: 0.29,
    g: 0.282,
    b: 0.271,
    a: 1.,
};
pub const BLUE_BRIGHT: graphics::Color = graphics::Color {
    r: 0.631,
    g: 0.741,
    b: 0.808,
    a: 1.,
};
pub const CYAN_BRIGHT: graphics::Color = graphics::Color {
    r: 0.694,
    g: 0.906,
    b: 0.867,
    a: 1.,
};
pub const GREEN_BRIGHT: graphics::Color = graphics::Color {
    r: 0.686,
    g: 0.745,
    b: 0.635,
    a: 1.,
};
pub const MAGENTA_BRIGHT: graphics::Color = graphics::Color {
    r: 0.843,
    g: 0.745,
    b: 0.855,
    a: 1.,
};
pub const RED_BRIGHT: graphics::Color = graphics::Color {
    r: 0.843,
    g: 0.529,
    b: 0.529,
    a: 1.,
};
pub const WHITE_BRIGHT: graphics::Color = graphics::Color {
    r: 0.937,
    g: 0.937,
    b: 0.937,
    a: 1.,
};
pub const YELLOW_BRIGHT: graphics::Color = graphics::Color {
    r: 0.894,
    g: 0.788,
    b: 0.686,
    a: 1.,
};
pub const CYAN: graphics::Color = graphics::Color {
    r: 0.541,
    g: 0.859,
    b: 0.706,
    a: 1.,
};
pub const GREEN: graphics::Color = graphics::Color {
    r: 0.565,
    g: 0.647,
    b: 0.49,
    a: 1.,
};
pub const MAGENTA: graphics::Color = graphics::Color {
    r: 0.78,
    g: 0.62,
    b: 0.769,
    a: 1.,
};
pub const RED: graphics::Color = graphics::Color {
    r: 0.639,
    g: 0.4,
    b: 0.4,
    a: 1.,
};
pub const WHITE: graphics::Color = graphics::Color {
    r: 0.816,
    g: 0.816,
    b: 0.816,
    a: 1.,
};
pub const YELLOW: graphics::Color = graphics::Color {
    r: 0.843,
    g: 0.686,
    b: 0.529,
    a: 1.,
};
pub const BACKGROUND: graphics::Color = graphics::Color {
    r: 0.11,
    g: 0.11,
    b: 0.11,
    a: 1.,
};

#[derive(Copy, Clone)]
pub enum CP437 {
    Pillar = 35,
    ChDot = 46,
    LessThan = 60,
    GreaterThan = 62,
    ChAt = 64,
    ChA = 65,
    Trap = 94,
    Cha = 97,
    Filled1 = 176,
    Filled2 = 177,
    Filled3 = 178,
    Filled4 = 219,
}

#[derive(Copy, Clone)]
pub struct Renderable {
    pub spr: CP437,
    pub color: graphics::Color,
}

// SpriteSet contains metadata about the tilesheet but not the actual image
pub struct SpriteSet {
    pub img: graphics::Image,
    cols: i32,
    rows: i32,
    tile_width: f32,
    tile_height: f32,
}

impl SpriteSet {
    pub fn new(
        img: graphics::Image,
        cols: i32,
        rows: i32,
        tile_width: i32,
        tile_height: i32,
    ) -> SpriteSet {
        let wf = tile_width as f32;
        let hf = tile_height as f32;
        let w = wf / (cols as f32 * wf);
        let h = hf / (cols as f32 * hf);
        SpriteSet {
            img,
            cols,
            rows,
            tile_width: w,
            tile_height: h,
        }
    }

    pub fn src(&self, t: CP437) -> graphics::Rect {
        let idx = t as i32;
        if idx >= self.rows * self.cols {
            panic!("accessing sprite by idx outside sheet bounds at {}", idx)
        } else {
            graphics::Rect::new(
                (idx % self.cols) as f32 * self.tile_width,
                (idx / self.cols) as f32 * self.tile_width,
                self.tile_width,
                self.tile_height,
            )
        }
    }
}
