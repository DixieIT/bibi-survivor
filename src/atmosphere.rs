use macroquad::prelude::*;
use macroquad::rand::gen_range;

/// Light map resolution: one texel covers this many map pixels (smoothed by linear filtering).
const TEXEL: f32 = 4.;
/// Fog texture size in texels, and how many map pixels each texel covers.
const FOG_SIZE: usize = 64;
const FOG_SCALE: f32 = 6.;
const NIGHT_COLOR: (f32, f32, f32) = (0.05, 0.06, 0.09);
const FIRE_COLOR: (f32, f32, f32) = (1., 0.55, 0.22);

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// A campfire: big, flickering, throws embers.
    Fire,
    /// A torch on a post: smaller flame, flickers.
    Torch,
    /// Smouldering embers (in a burnt house): a steady dim glow, no flame.
    Glow,
}

struct Light {
    pos: Vec2,
    radius: f32,
    kind: Kind,
    /// Phase offset, so lights don't flicker in step.
    seed: f32,
}

struct Mote {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    ember: bool,
}

/// Night, firelight, drifting fog and ash, as set by the map's map.txt:
/// `night <0..1>`, `fog <0..1>`, `ash <0..1>`, `light <x> <y> <radius> [fire|torch|glow]`.
pub struct Atmosphere {
    night: f32,
    fog: f32,
    ash: f32,
    lights: Vec<Light>,
    time: f32,
    shade: Image,
    shade_tex: Texture2D,
    fog_tex: Texture2D,
    motes: Vec<Mote>,
}

impl Atmosphere {
    pub fn new(settings: &str, view: Vec2) -> Self {
        let (mut night, mut fog, mut ash) = (0., 0., 0.);
        let mut lights = Vec::new();
        for line in settings.lines() {
            let parts: Vec<&str> = line.split('#').next().unwrap_or("").split_whitespace().collect();
            let num = |i: usize| parts.get(i).and_then(|p| p.parse::<f32>().ok());
            match parts.first() {
                Some(&"night") => night = num(1).unwrap_or(0.).clamp(0., 1.),
                Some(&"fog") => fog = num(1).unwrap_or(0.).clamp(0., 1.),
                Some(&"ash") => ash = num(1).unwrap_or(0.).clamp(0., 1.),
                Some(&"light") => {
                    if let (Some(x), Some(y), Some(radius)) = (num(1), num(2), num(3)) {
                        let kind = match parts.get(4) {
                            Some(&"torch") => Kind::Torch,
                            Some(&"glow") => Kind::Glow,
                            _ => Kind::Fire,
                        };
                        lights.push(Light { pos: vec2(x, y), radius, kind, seed: gen_range(0., 100.) });
                    }
                }
                _ => {}
            }
        }
        let cols = (view.x / TEXEL) as u16 + 3;
        let rows = (view.y / TEXEL) as u16 + 3;
        let shade = Image::gen_image_color(cols, rows, BLANK);
        let shade_tex = Texture2D::from_image(&shade);
        shade_tex.set_filter(FilterMode::Linear);
        let fog_tex = Texture2D::from_image(&fog_image());
        fog_tex.set_filter(FilterMode::Linear);
        Atmosphere { night, fog, ash, lights, time: 0., shade, shade_tex, fog_tex, motes: Vec::new() }
    }

    fn active(&self) -> bool {
        self.night > 0. || self.fog > 0. || self.ash > 0. || !self.lights.is_empty()
    }

    /// Brightness multiplier of a light right now.
    fn flicker(&self, light: &Light) -> f32 {
        let t = self.time + light.seed;
        match light.kind {
            Kind::Fire => 0.9 + 0.06 * (t * 9.).sin() + 0.04 * (t * 23.).sin(),
            Kind::Torch => 0.88 + 0.08 * (t * 13.).sin() + 0.04 * (t * 31.).sin(),
            Kind::Glow => 0.85 + 0.15 * (t * 1.7).sin(),
        }
    }

    pub fn update(&mut self, dt: f32, view: Rect) {
        if !self.active() {
            return;
        }
        self.time += dt;
        let near = Rect::new(view.x - 40., view.y - 40., view.w + 80., view.h + 80.);
        for light in self.lights.iter().filter(|l| l.kind != Kind::Glow && near.contains(l.pos)) {
            let rate = if light.kind == Kind::Fire { 14. } else { 3. };
            if gen_range(0., 1.) < rate * dt {
                let spread = if light.kind == Kind::Fire { 6. } else { 1.5 };
                let from = light.pos + vec2(gen_range(-spread, spread), -gen_range(2., 6.));
                let vel = vec2(gen_range(-8., 8.), -gen_range(18., 34.));
                self.motes.push(Mote { pos: from, vel, life: gen_range(0.8, 1.8), ember: true });
            }
        }
        // Ash keeps drifting down across the view.
        if gen_range(0., 1.) < self.ash * 40. * dt {
            let from = vec2(gen_range(view.x - 40., view.right()), view.y - 4.);
            let vel = vec2(gen_range(4., 12.), gen_range(8., 16.));
            self.motes.push(Mote { pos: from, vel, life: gen_range(6., 12.), ember: false });
        }
        for m in &mut self.motes {
            let sway = if m.ember { (self.time * 5. + m.pos.y * 0.2).sin() * 10. } else { (self.time * 1.3 + m.pos.x * 0.05).sin() * 6. };
            m.pos += (m.vel + vec2(sway, 0.)) * dt;
            m.life -= dt;
        }
        self.motes.retain(|m| m.life > 0. && near.contains(m.pos));
    }

    /// Draws fog, darkness and fire over the world, inside a world-space camera
    /// looking at `view`. `glows` are moving lights: (position, radius).
    pub fn draw(&mut self, view: Rect, glows: &[(Vec2, f32)]) {
        if !self.active() {
            return;
        }
        if self.night > 0. {
            self.draw_darkness(view, glows);
        }
        // Over the dark, so it reads as pale mist.
        self.draw_fog(view);
        for light in &self.lights {
            if light.kind != Kind::Glow && Rect::new(view.x - 20., view.y - 30., view.w + 40., view.h + 60.).contains(light.pos) {
                self.draw_flame(light);
            }
        }
        for m in &self.motes {
            let color = if m.ember {
                let hot = (m.life * 2.).min(1.);
                Color::new(1., 0.45 + 0.4 * hot, 0.15, hot)
            } else {
                Color::new(0.62, 0.62, 0.6, 0.45 * (m.life / 2.).min(1.))
            };
            draw_rectangle(m.pos.x.round(), m.pos.y.round(), 1., 1., color);
        }
    }

    /// Two layers of soft fog, sliding slowly in different directions.
    fn draw_fog(&self, view: Rect) {
        if self.fog <= 0. {
            return;
        }
        let tile = FOG_SIZE as f32 * FOG_SCALE;
        let layers = [(vec2(7., 2.), 1., 0.32), (vec2(-5., 3.), 0.7, 0.22)];
        for (i, &(speed, scale, alpha)) in layers.iter().enumerate() {
            let size = tile * scale;
            let drift = speed * self.time + vec2(i as f32 * 97., i as f32 * 53.);
            let offset = vec2(drift.x.rem_euclid(size), drift.y.rem_euclid(size));
            let x0 = ((view.x - offset.x) / size).floor() * size + offset.x;
            let y0 = ((view.y - offset.y) / size).floor() * size + offset.y;
            let tint = Color::new(0.58, 0.62, 0.64, alpha * self.fog);
            let mut y = y0;
            while y < view.bottom() {
                let mut x = x0;
                while x < view.right() {
                    let params = DrawTextureParams { dest_size: Some(vec2(size, size)), flip_x: i == 1, ..Default::default() };
                    draw_texture_ex(&self.fog_tex, x, y, tint, params);
                    x += size;
                }
                y += size;
            }
        }
    }

    /// A light map on a coarse grid: black-blue where it's dark, a warm tint near fire.
    fn draw_darkness(&mut self, view: Rect, glows: &[(Vec2, f32)]) {
        let (cols, rows) = (self.shade.width as i32, self.shade.height as i32);
        let origin = vec2((view.x / TEXEL).floor() - 1., (view.y / TEXEL).floor() - 1.) * TEXEL;
        // (position, radius, strength, warm)
        let mut lights: Vec<(Vec2, f32, f32, bool)> = self
            .lights
            .iter()
            .map(|l| {
                let f = self.flicker(l);
                let strength = if l.kind == Kind::Glow { 0.85 } else { 1. };
                (l.pos, l.radius * f, strength * f, true)
            })
            .collect();
        lights.extend(glows.iter().map(|&(p, r)| (p, r, 0.75, r < 80.)));
        lights.retain(|&(p, r, _, _)| {
            p.x + r > view.x - TEXEL * 2. && p.x - r < view.right() + TEXEL * 2. && p.y + r > view.y - TEXEL * 2. && p.y - r < view.bottom() + TEXEL * 2.
        });
        let center = view.center();
        let half = vec2(view.w, view.h) / 2.;
        let (nr, ng, nb) = NIGHT_COLOR;
        let (fr, fg, fb) = FIRE_COLOR;
        for ty in 0..rows {
            for tx in 0..cols {
                let p = origin + (vec2(tx as f32, ty as f32) + 0.5) * TEXEL;
                let (mut light, mut warm) = (0f32, 0f32);
                for &(lp, r, strength, hot) in &lights {
                    let d = p.distance_squared(lp) / (r * r);
                    if d < 1. {
                        let k = (1. - d) * (1. - d) * strength;
                        light = light + k - light * k;
                        if hot {
                            warm = warm.max(k);
                        }
                    }
                }
                let edge = ((p - center) / half).length();
                let vignette = ((edge - 0.5) / 0.6).clamp(0., 1.) * 0.35;
                let alpha = ((self.night + vignette) * (1. - light)).max(warm * 0.12).clamp(0., 0.95);
                let mix = |a: f32, b: f32| a + (b - a) * warm;
                let i = ((ty * cols + tx) * 4) as usize;
                let bytes = &mut self.shade.bytes[i..i + 4];
                bytes[0] = (mix(nr, fr) * 255.) as u8;
                bytes[1] = (mix(ng, fg) * 255.) as u8;
                bytes[2] = (mix(nb, fb) * 255.) as u8;
                bytes[3] = (alpha * 255.) as u8;
            }
        }
        self.shade_tex.update(&self.shade);
        let size = vec2(cols as f32, rows as f32) * TEXEL;
        // Texel centers sit on the grid points sampled above.
        draw_texture_ex(&self.shade_tex, origin.x, origin.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
    }

    fn draw_flame(&self, light: &Light) {
        let t = self.time + light.seed;
        let (count, width, height) = if light.kind == Kind::Fire { (6, 5., 13.) } else { (4, 2.5, 7.) };
        for k in 0..count {
            let up = k as f32 / (count - 1) as f32;
            let sway = ((t * 14. + k as f32 * 1.3).sin() * (1. + up * 1.5)).round();
            let h = height * (0.85 + 0.15 * (t * 11.).sin());
            let r = (width * (1. - up * 0.7)).max(1.);
            let color = Color::new(1., 0.3 + 0.6 * up, 0.08 + 0.35 * up, 0.9 - 0.3 * up);
            draw_circle((light.pos.x + sway).round(), (light.pos.y - h * up).round(), r, color);
        }
        draw_circle(light.pos.x.round(), (light.pos.y - 1.).round(), (width * 0.5).max(1.), Color::new(1., 0.92, 0.6, 0.95));
    }
}

/// Tileable value noise, shaped into soft patches of fog.
fn fog_image() -> Image {
    let n = FOG_SIZE;
    let lattice = |cells: usize, seed: u32| -> Vec<f32> {
        (0..cells * cells)
            .map(|i| {
                let mut h = (i as u32).wrapping_mul(2654435761) ^ seed.wrapping_mul(40503);
                h ^= h >> 15;
                h = h.wrapping_mul(2246822519);
                h ^= h >> 13;
                (h & 0xffff) as f32 / 65535.
            })
            .collect()
    };
    let sample = |grid: &[f32], cells: usize, x: f32, y: f32| {
        let (fx, fy) = (x * cells as f32, y * cells as f32);
        let (x0, y0) = (fx.floor() as usize % cells, fy.floor() as usize % cells);
        let (x1, y1) = ((x0 + 1) % cells, (y0 + 1) % cells);
        let s = |t: f32| t * t * (3. - 2. * t);
        let (tx, ty) = (s(fx.fract()), s(fy.fract()));
        let top = grid[y0 * cells + x0] * (1. - tx) + grid[y0 * cells + x1] * tx;
        let bottom = grid[y1 * cells + x0] * (1. - tx) + grid[y1 * cells + x1] * tx;
        top * (1. - ty) + bottom * ty
    };
    let (coarse, fine) = (lattice(4, 1), lattice(8, 2));
    let mut image = Image::gen_image_color(n as u16, n as u16, BLANK);
    for y in 0..n {
        for x in 0..n {
            let (u, v) = (x as f32 / n as f32, y as f32 / n as f32);
            let v = sample(&coarse, 4, u, v) * 0.65 + sample(&fine, 8, u, v) * 0.35;
            let a = ((v - 0.42) / 0.38).clamp(0., 1.);
            image.set_pixel(x as u32, y as u32, Color::new(1., 1., 1., a * a * (3. - 2. * a)));
        }
    }
    image
}
