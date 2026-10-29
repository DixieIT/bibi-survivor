use crate::assets::MapAssets;
use macroquad::prelude::*;
use std::collections::VecDeque;

/// Collision grid resolution, in map pixels.
const CELL: f32 = 4.;
const UNREACHABLE: u16 = u16::MAX;

/// A sword swing: hitbox, swing id (each swing hits a target once), push direction, damage.
pub struct Swing {
    pub hitbox: Rect,
    pub id: u32,
    pub push: Vec2,
    pub damage: i32,
}

/// A cut-out of the map (e.g. a tree crown) that covers characters whose feet are
/// above its baseline, i.e. standing behind it.
pub struct Overlay {
    baseline: (Vec2, Vec2),
    bounds: Rect,
}

impl Overlay {
    pub fn hides(&self, feet: Vec2) -> bool {
        let (a, b) = self.baseline;
        let t = if (b.x - a.x).abs() < 0.01 { 0. } else { (feet.x - a.x) / (b.x - a.x) };
        feet.y < a.y + (b.y - a.y) * t
    }

    /// Whether a character standing at `feet` can overlap it at all.
    pub fn near(&self, feet: Vec2) -> bool {
        let b = self.bounds;
        Rect::new(b.x - 32., b.y, b.w + 64., b.h + 40.).contains(feet)
    }

    pub fn draw(&self, assets: &MapAssets) {
        let b = self.bounds;
        draw_texture_ex(&assets.overlay, b.x, b.y, WHITE, DrawTextureParams { source: Some(b), ..Default::default() });
    }
}

pub struct Level {
    size: Vec2,
    cols: i32,
    rows: i32,
    solid: Vec<bool>,
    /// Cells where a small body fits (no solid cell right next to it): what enemies path through.
    roomy: Vec<bool>,
    /// Steps from each cell to the player, for enemies to follow.
    flow: Vec<u16>,
    flow_target: Option<(i32, i32)>,
    /// Feet position of the player start.
    pub spawn: Vec2,
    pub overlays: Vec<Overlay>,
}

impl Level {
    pub fn new(assets: &MapAssets) -> Self {
        let size = assets.map.size();
        let cols = (size.x / CELL).ceil() as i32;
        let rows = (size.y / CELL).ceil() as i32;
        let mask = &assets.collision;
        let (mw, mh) = (mask.width as i32, mask.height as i32);
        // A cell is solid if the collision image is opaque at its center.
        let solid: Vec<bool> = (0..rows * cols)
            .map(|i| {
                let x = (((i % cols) as f32 + 0.5) * CELL) as i32;
                let y = (((i / cols) as f32 + 0.5) * CELL) as i32;
                x >= mw || y >= mh || mask.bytes[((y * mw + x) * 4 + 3) as usize] > 0
            })
            .collect();

        let mut level = Level {
            size,
            cols,
            rows,
            roomy: Vec::new(),
            flow: vec![UNREACHABLE; (rows * cols) as usize],
            flow_target: None,
            solid,
            spawn: size / 2.,
            overlays: Vec::new(),
        };
        level.roomy = (0..rows * cols)
            .map(|i| {
                let (cx, cy) = (i % cols, i / cols);
                (-1..=1).all(|dy| (-1..=1).all(|dx| !level.solid_cell(cx + dx, cy + dy)))
            })
            .collect();

        for line in assets.overlays.lines() {
            let parts: Vec<f32> = line.split('#').next().unwrap_or("").split_whitespace().skip(1).filter_map(|p| p.parse().ok()).collect();
            if let [x0, y0, x1, y1, bx, by, bw, bh] = parts[..] {
                level.overlays.push(Overlay { baseline: (vec2(x0, y0), vec2(x1, y1)), bounds: Rect::new(bx, by, bw, bh) });
            }
        }
        for line in assets.settings.lines() {
            let parts: Vec<&str> = line.split('#').next().unwrap_or("").split_whitespace().collect();
            if let ["spawn", x, y] = parts.as_slice() {
                if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                    level.spawn = vec2(x, y);
                }
            }
        }
        level
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    fn cell_of(&self, p: Vec2) -> (i32, i32) {
        ((p.x / CELL).floor() as i32, (p.y / CELL).floor() as i32)
    }

    fn index(&self, cx: i32, cy: i32) -> Option<usize> {
        (cx >= 0 && cy >= 0 && cx < self.cols && cy < self.rows).then(|| (cy * self.cols + cx) as usize)
    }

    /// Outside the map counts as solid.
    fn solid_cell(&self, cx: i32, cy: i32) -> bool {
        self.index(cx, cy).map_or(true, |i| self.solid[i])
    }

    fn roomy_cell(&self, cx: i32, cy: i32) -> bool {
        self.index(cx, cy).is_some_and(|i| self.roomy[i])
    }

    /// True if a feet-sized box overlaps anything solid or leaves the map.
    pub fn blocked(&self, r: Rect) -> bool {
        let (x0, y0) = self.cell_of(r.point());
        let (x1, y1) = self.cell_of(vec2(r.right() - 0.01, r.bottom() - 0.01));
        (y0..=y1).any(|cy| (x0..=x1).any(|cx| self.solid_cell(cx, cy)))
    }

    /// True if nothing solid lies on the straight line between two points.
    pub fn line_clear(&self, a: Vec2, b: Vec2) -> bool {
        let steps = (a.distance(b) / CELL).ceil().max(1.) as i32;
        (0..=steps).all(|i| {
            let (cx, cy) = self.cell_of(a.lerp(b, i as f32 / steps as f32));
            !self.solid_cell(cx, cy)
        })
    }

    /// Somewhere an enemy can stand that leads to the player.
    pub fn can_spawn_at(&self, p: Vec2) -> bool {
        let (cx, cy) = self.cell_of(p);
        self.roomy_cell(cx, cy) && self.index(cx, cy).is_some_and(|i| self.flow[i] != UNREACHABLE)
    }

    /// Recomputes the distance field when the player enters a new cell.
    pub fn update_flow(&mut self, target: Vec2) {
        let start = self.cell_of(target);
        if self.flow_target == Some(start) {
            return;
        }
        self.flow_target = Some(start);
        self.flow.fill(UNREACHABLE);
        let Some(i) = self.index(start.0, start.1) else { return };
        self.flow[i] = 0;
        let mut queue = VecDeque::from([start]);
        while let Some((cx, cy)) = queue.pop_front() {
            let d = self.flow[self.index(cx, cy).unwrap()];
            let next: Vec<_> = self.neighbours(cx, cy).collect();
            for (nx, ny) in next {
                let n = self.index(nx, ny).unwrap();
                if self.flow[n] == UNREACHABLE {
                    self.flow[n] = d + 1;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    /// Roomy neighbours, including diagonals that don't cut a corner.
    fn neighbours(&self, cx: i32, cy: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)]
            .into_iter()
            .filter(move |&(dx, dy)| {
                self.roomy_cell(cx + dx, cy + dy)
                    && (dx == 0 || dy == 0 || (self.roomy_cell(cx + dx, cy) && self.roomy_cell(cx, cy + dy)))
            })
            .map(move |(dx, dy)| (cx + dx, cy + dy))
    }

    /// Where to walk next from `from` to get closer to the player, if there's a path.
    pub fn next_step(&self, from: Vec2) -> Option<Vec2> {
        let (cx, cy) = self.cell_of(from);
        let here = self.index(cx, cy).map_or(UNREACHABLE, |i| self.flow[i]);
        let best = self
            .neighbours(cx, cy)
            .min_by_key(|&(nx, ny)| self.flow[self.index(nx, ny).unwrap()])?;
        let d = self.flow[self.index(best.0, best.1).unwrap()];
        (d != UNREACHABLE && d <= here).then(|| (vec2(best.0 as f32, best.1 as f32) + 0.5) * CELL)
    }

    pub fn draw(&self, assets: &MapAssets) {
        draw_texture(&assets.map, 0., 0., WHITE);
    }
}
