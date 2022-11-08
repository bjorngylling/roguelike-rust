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

// SpriteSet contains metadata about the tilesheet but not the actual image
pub struct SpriteSet {
    cols: u32,
    rows: u32,
    tile_width: f32,
    tile_height: f32,
}

impl SpriteSet {
    pub fn new(cols: u32, rows: u32, tile_width: u32, tile_height: u32) -> SpriteSet {
        let wf = tile_width as f32;
        let hf = tile_height as f32;
        let w = wf / (cols as f32 * wf);
        let h = hf / (cols as f32 * hf);
        SpriteSet {
            cols,
            rows,
            tile_width: w,
            tile_height: h,
        }
    }

    pub fn src_by_idx(&self, idx: u32) -> graphics::Rect {
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

    pub fn src(&self, x: u32, y: u32) -> graphics::Rect {
        if x >= self.rows || y >= self.cols {
            panic!("accessing sprite outside sheet bounds at {},{}", x, y)
        } else {
            graphics::Rect::new(
                self.tile_width * x as f32,
                self.tile_height * y as f32,
                self.tile_width,
                self.tile_height,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spriteset_src() {
        let t = SpriteSet::new(4, 4, 10, 10);
        assert_eq!(t.src(0, 0), graphics::Rect::new(0., 0., 0.25, 0.25));
        assert_eq!(t.src(3, 0), graphics::Rect::new(0.75, 0., 0.25, 0.25));
        assert_eq!(t.src(0, 1), graphics::Rect::new(0., 0.25, 0.25, 0.25));
    }

    #[test]
    #[should_panic(expected = "accessing sprite outside sheet bounds at 4,0")]
    fn spriteset_src_requesting_tile_outside_bounds() {
        let t = SpriteSet::new(4, 4, 10, 10);
        t.src(4, 0);
    }

    #[test]
    fn spriteset_src_by_idx() {
        let t = SpriteSet::new(4, 4, 10, 10);
        assert_eq!(
            t.src_by_idx(0),
            graphics::Rect::new(0., 0., 0.25, 0.25)
        );
        assert_eq!(
            t.src_by_idx(3),
            graphics::Rect::new(0.75, 0., 0.25, 0.25)
        );
        assert_eq!(
            t.src_by_idx(4),
            graphics::Rect::new(0., 0.25, 0.25, 0.25)
        );
    }

    #[test]
    #[should_panic(expected = "accessing sprite by idx outside sheet bounds at 16")]
    fn spriteset_src_by_idx_requesting_tile_outside_bounds() {
        let t = SpriteSet::new(4, 4, 10, 10);
        t.src_by_idx(16);
    }
}
