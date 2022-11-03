use ggez::graphics;

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

    pub fn src_by_idx(&self, idx: u32) -> Option<graphics::Rect> {
        if idx >= self.rows * self.cols {
            None
        } else {
            Some(graphics::Rect::new(
                (idx % self.cols) as f32 * self.tile_width,
                (idx / self.cols) as f32 * self.tile_width,
                self.tile_width,
                self.tile_height,
            ))
        }
    }

    pub fn src(&self, x: u32, y: u32) -> Option<graphics::Rect> {
        if x >= self.rows || y >= self.cols {
            None
        } else {
            Some(graphics::Rect::new(
                self.tile_width * x as f32,
                self.tile_height * y as f32,
                self.tile_width,
                self.tile_height,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spriteset_src() {
        let t = SpriteSet::new(4, 4, 10, 10);
        assert_eq!(t.src(0, 0), Some(graphics::Rect::new(0., 0., 0.25, 0.25)));
        assert_eq!(t.src(3, 0), Some(graphics::Rect::new(0.75, 0., 0.25, 0.25)));
        assert_eq!(t.src(0, 1), Some(graphics::Rect::new(0., 0.25, 0.25, 0.25)));
    }

    #[test]
    fn spriteset_src_requesting_tile_outside_bounds() {
        let t = SpriteSet::new(4, 4, 10, 10);
        assert_eq!(t.src(4,0), None);
        assert_eq!(t.src(0,4), None);
    }

    #[test]
    fn spriteset_src_by_idx() {
        let t = SpriteSet::new(4, 4, 10, 10);
        assert_eq!(t.src_by_idx(0), Some(graphics::Rect::new(0., 0., 0.25, 0.25)));
        assert_eq!(t.src_by_idx(3), Some(graphics::Rect::new(0.75, 0., 0.25, 0.25)));
        assert_eq!(t.src_by_idx(4), Some(graphics::Rect::new(0., 0.25, 0.25, 0.25)));
    }

    #[test]
    fn spriteset_src_by_idx_requesting_tile_outside_bounds() {
        let t = SpriteSet::new(4, 4, 10, 10);
        assert_eq!(t.src_by_idx(16), None);
        assert_eq!(t.src_by_idx(17), None);
    }
}
