use crate::geom::{pt, Point};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Angles {
    /// the angle closest to the horizontal/vertical line
    start: f32,
    /// the center angle
    center: f32,
    /// the angle furthest from the horizontal/vertical line
    end: f32,
}

impl Angles {
    const ZERO: Angles = Angles {
        start: 0.,
        center: 0.,
        end: 0.,
    };

    fn new(start: f32, center: f32, end: f32) -> Angles {
        Angles { start, center, end }
    }

    fn contains(self, angle: f32) -> bool {
        self.start < angle && angle < self.end
    }
}

fn at(center: Point, quad: Point, step: i32, it: i32, vert: bool) -> Point {
    if vert {
        pt(center.x + step * quad.x, center.y + it * quad.y)
    } else {
        pt(center.x + it * quad.x, center.y + step * quad.y)
    }
}

pub fn calculate<F>(center: Point, radius: i32, opaque_at: F) -> HashSet<Point>
where
    F: Fn(Point) -> bool,
{
    let mut vis = fov_quadrant(pt(-1, -1), center, radius, false, &opaque_at);
    vis.extend(fov_quadrant(pt(-1, -1), center, radius, true, &opaque_at));
    vis.extend(fov_quadrant(pt(1, -1), center, radius, false, &opaque_at));
    vis.extend(fov_quadrant(pt(1, -1), center, radius, true, &opaque_at));
    vis.extend(fov_quadrant(pt(1, 1), center, radius, false, &opaque_at));
    vis.extend(fov_quadrant(pt(1, 1), center, radius, true, &opaque_at));
    vis.extend(fov_quadrant(pt(-1, 1), center, radius, false, &opaque_at));
    vis.extend(fov_quadrant(pt(-1, 1), center, radius, true, &opaque_at));
    vis.insert(center);
    vis
}

fn fov_quadrant<F>(
    quad_dir: Point,
    center: Point,
    radius: i32,
    vert: bool,
    opaque_at: F,
) -> HashSet<Point>
where
    F: Fn(Point) -> bool,
{
    let mut vis = HashSet::new();
    let mut obstructions: Vec<Angles> = Vec::new();
    for it in 1..=radius {
        let angle_range = 1. / (it + 1) as f32;
        for stp in 0..(it + 1) {
            let p = at(center, quad_dir, stp, it, vert);
            if center.distance(p) <= (radius as f32) + 1. / 3. {
                let f_stp = stp as f32;
                let angles = Angles::new(
                    f_stp * angle_range,
                    (f_stp + 0.5) * angle_range,
                    (f_stp + 1.) * angle_range,
                );
                if is_visible(angles, &obstructions) {
                    vis.insert(p);
                    if opaque_at(p) {
                        obstructions.push(angles);
                    }
                } else {
                    obstructions.push(angles);
                }
            }
        }
    }

    vis
}

fn is_visible(a: Angles, obstructions: &Vec<Angles>) -> bool {
    let mut start_vis = true;
    let mut center_vis = true;
    let mut end_vis = true;
    for o in obstructions {
        if o.contains(a.start) {
            start_vis = false
        }
        if o.contains(a.center) {
            center_vis = false
        }
        if o.contains(a.end) {
            end_vis = false
        }
    }

    (end_vis || start_vis) && center_vis
}

#[cfg(test)]
mod test {
    use super::*;

    fn print(s: &HashSet<Point>) {
        for y in 0..11 {
            for x in 0..11 {
                print!("{}", if s.contains(&pt(x, y)) { "X" } else { "." });
            }
            println!()
        }
    }

    #[test]
    fn test_fov() {
        let vis = calculate(pt(5, 5), 5, |_| false);
        assert!(vis.contains(&pt(5, 5)));
    }

    #[test]
    fn test_quadrant_nw_no_obstacles() {
        let vis = fov_quadrant(pt(-1, -1), pt(5, 5), 4, true, |_| false);
        assert_eq!(
            vis,
            vec![
                pt(4, 1),
                pt(5, 1),
                pt(2, 2),
                pt(3, 2),
                pt(4, 2),
                pt(5, 2),
                pt(3, 3),
                pt(4, 3),
                pt(5, 3),
                pt(4, 4),
                pt(5, 4)
            ]
            .into_iter()
            .collect()
        );
        let vis = fov_quadrant(pt(-1, -1), pt(5, 5), 4, false, |_| false);
        assert_eq!(
            vis,
            vec![
                pt(1, 4),
                pt(1, 5),
                pt(2, 2),
                pt(2, 3),
                pt(2, 4),
                pt(2, 5),
                pt(3, 3),
                pt(3, 4),
                pt(3, 5),
                pt(4, 4),
                pt(4, 5)
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn test_quadrant_se_no_obstacles() {
        let vis = fov_quadrant(pt(1, 1), pt(5, 5), 4, true, |_| false);
        assert_eq!(
            vis,
            vec![
                pt(5, 6),
                pt(6, 6),
                pt(5, 7),
                pt(6, 7),
                pt(7, 7),
                pt(5, 8),
                pt(6, 8),
                pt(7, 8),
                pt(8, 8),
                pt(5, 9),
                pt(6, 9)
            ]
            .into_iter()
            .collect()
        );
        let vis = fov_quadrant(pt(1, 1), pt(5, 5), 4, false, |_| false);
        print(&vis);
        assert_eq!(
            vis,
            vec![
                pt(6, 5),
                pt(7, 5),
                pt(8, 5),
                pt(9, 5),
                pt(6, 6),
                pt(7, 6),
                pt(8, 6),
                pt(9, 6),
                pt(7, 7),
                pt(8, 7),
                pt(8, 8)
            ]
            .into_iter()
            .collect()
        );
    }
}
