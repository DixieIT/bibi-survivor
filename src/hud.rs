//! The medieval HUD: iron frames, cloth banners, parchment cards, all drawn as
//! pixel art on the same grid as the game, with a small hand-made pixel font.

use crate::assets::{MapAssets, MapInfo};
use crate::player::{Player, MAX_HP};
use crate::upgrades::Offer;
use macroquad::miniquad::{BlendFactor, BlendState, Equation, PipelineParams, ShaderSource};
use macroquad::prelude::*;

const OUTLINE: Color = rgb(14, 10, 12);
const IRON: Color = rgb(74, 68, 66);
const IRON_LIGHT: Color = rgb(122, 112, 102);
const IRON_DARK: Color = rgb(44, 40, 40);
const PANEL: Color = Color::new(0.11, 0.085, 0.09, 0.88);
const GOLD: Color = rgb(214, 168, 72);
const GOLD_LIGHT: Color = rgb(250, 222, 140);
const GOLD_DARK: Color = rgb(132, 92, 36);
const PARCHMENT: Color = rgb(236, 220, 180);
const PARCHMENT_DIM: Color = rgb(168, 154, 126);
const INK: Color = rgb(64, 42, 28);
const CLOTH: Color = rgb(112, 28, 30);
const CLOTH_LIGHT: Color = rgb(150, 44, 42);
const CLOTH_DARK: Color = rgb(70, 16, 20);
const TROUGH: Color = rgb(30, 20, 22);
const BLOOD: Color = rgb(196, 44, 40);

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.)
}

fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

/// 5x7 pixel font, uppercase only; glyphs can be narrower than 5.
const GLYPHS: &[(char, [&str; 7])] = &[
    ('A', [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('B', ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."]),
    ('C', [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."]),
    ('D', ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."]),
    ('E', ["#####", "#....", "#....", "####.", "#....", "#....", "#####"]),
    ('F', ["#####", "#....", "#....", "####.", "#....", "#....", "#...."]),
    ('G', [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".####"]),
    ('H', ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('I', ["###", ".#.", ".#.", ".#.", ".#.", ".#.", "###"]),
    ('J', ["..###", "...#.", "...#.", "...#.", "#..#.", "#..#.", ".##.."]),
    ('K', ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"]),
    ('L', ["#....", "#....", "#....", "#....", "#....", "#....", "#####"]),
    ('M', ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"]),
    ('N', ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"]),
    ('O', [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('P', ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."]),
    ('Q', [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"]),
    ('R', ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"]),
    ('S', [".####", "#....", "#....", ".###.", "....#", "....#", "####."]),
    ('T', ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."]),
    ('U', ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('V', ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."]),
    ('W', ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"]),
    ('X', ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"]),
    ('Y', ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."]),
    ('Z', ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"]),
    ('0', [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."]),
    ('1', [".#.", "##.", ".#.", ".#.", ".#.", ".#.", "###"]),
    ('2', [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"]),
    ('3', ["####.", "....#", "....#", ".###.", "....#", "....#", "####."]),
    ('4', ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."]),
    ('5', ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."]),
    ('6', [".###.", "#....", "#....", "####.", "#...#", "#...#", ".###."]),
    ('7', ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."]),
    ('8', [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."]),
    ('9', [".###.", "#...#", "#...#", ".####", "....#", "....#", ".###."]),
    ('.', [".", ".", ".", ".", ".", ".", "#"]),
    (',', ["..", "..", "..", "..", "..", ".#", "#."]),
    (':', [".", ".", "#", ".", ".", "#", "."]),
    ('!', ["#", "#", "#", "#", "#", ".", "#"]),
    ('?', [".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#.."]),
    ('/', ["....#", "....#", "...#.", "..#..", ".#...", "#....", "#...."]),
    ('-', ["...", "...", "...", "###", "...", "...", "..."]),
    ('+', [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."]),
    ('%', ["##..#", "##..#", "...#.", "..#..", ".#...", "#..##", "#..##"]),
    ('(', [".#", "#.", "#.", "#.", "#.", "#.", ".#"]),
    (')', ["#.", ".#", ".#", ".#", ".#", ".#", "#."]),
    ('\'', ["#", "#", ".", ".", ".", ".", "."]),
    ('*', ["...", "...", ".#.", "###", ".#.", "...", "..."]),
    ('[', ["##", "#.", "#.", "#.", "#.", "#.", "##"]),
    (']', ["##", ".#", ".#", ".#", ".#", ".#", "##"]),
];
const SPACE: f32 = 3.;

fn glyph(c: char) -> Option<&'static [&'static str; 7]> {
    let c = c.to_ascii_uppercase();
    GLYPHS.iter().find(|g| g.0 == c).map(|g| &g.1)
}

/// Width of `s` in font pixels.
pub fn text_width(s: &str) -> f32 {
    let w: f32 = s.chars().map(|c| glyph(c).map_or(SPACE, |g| g[0].len() as f32) + 1.).sum();
    (w - 1.).max(0.)
}

/// Draws `s` with its top-left at (x, y); each font pixel is `px` wide.
pub fn text(s: &str, x: f32, y: f32, px: f32, color: Color) {
    let mut cx = x;
    for c in s.chars() {
        let Some(g) = glyph(c) else {
            cx += (SPACE + 1.) * px;
            continue;
        };
        for (row, line) in g.iter().enumerate() {
            for (col, b) in line.bytes().enumerate() {
                if b == b'#' {
                    draw_rectangle(cx + col as f32 * px, y + row as f32 * px, px, px, color);
                }
            }
        }
        cx += (g[0].len() as f32 + 1.) * px;
    }
}

/// Text with a hard drop shadow, one font pixel down and right.
pub fn text_shadowed(s: &str, x: f32, y: f32, px: f32, color: Color) {
    text(s, x + px, y + px, px, alpha(OUTLINE, color.a));
    text(s, x, y, px, color);
}

// Icons: one char per pixel. k outline, w white, s steel, y gold, b brown,
// r red, R dark red, p pink, c blue, C dark blue.
const HEART: [&str; 8] = [".kkk.kkk.", "kprrkrrrk", "kprrrrrrk", "krrrrrrRk", ".krrrrRk.", "..krrRk..", "...kRk...", "....k...."];
const BOLT: [&str; 9] = ["...kkk.", "..kyyk.", ".kyyk..", "kyyyykk", "kkyyyyk", "..kyyk.", "..kyk..", ".kyk...", ".kk...."];
const SHIELD: [&str; 8] = ["kkkkkkk", "kwcccck", "kwcccCk", "kcccCCk", ".kccCk.", ".kcCCk.", "..kCk..", "...k..."];
const SKULL: [&str; 7] = [".kkkkk.", "kwwwwwk", "kwkwkwk", "kwwwwwk", ".kwkwk.", ".kwwwk.", "..kkk.."];
const SWORD: [&str; 9] = [
    ".......kk", "......kwk", ".....kwsk", "....kwsk.", "kk.kwsk..", "kykwsk...", ".kyyk....", "kbkyyk...", "kk..kk...",
];
const DASH: [&str; 7] = ["kk..kk...", "kck.kck..", ".kck.kck.", "..kck.kck", ".kck.kck.", "kck.kck..", "kk..kk..."];
const DROP: [&str; 8] = ["...k...", "..krk..", ".krrrk.", ".kprrk.", "kprrrRk", "krrrrRk", ".kRRRk.", "..kkk.."];
const SWIRL: [&str; 9] = ["..kkkkk..", ".kwwwwwk.", "kwk...kwk", "kwk....k.", "kwk..kkkk", "kwk...kwk", ".kwk..kk.", "..kwwwk..", "...kkk..."];

fn icon_for(offer: Offer) -> &'static [&'static str] {
    match offer {
        Offer::Slash => &SWORD,
        Offer::Shield => &SHIELD,
        Offer::Dash => &DASH,
        Offer::LifeSteal => &DROP,
        Offer::Vortex | Offer::VortexCost | Offer::VortexDamage => &SWIRL,
        Offer::Stamina => &BOLT,
    }
}

fn roman(n: u32) -> &'static str {
    ["", "I", "II", "III", "IV", "V", "VI"][n.min(6) as usize]
}

/// Draws on the game's pixel grid: one UI pixel is `u` screen pixels, (0, 0) is the
/// top-left of the game view and the view is 640 x 360 UI pixels.
pub struct Ui {
    u: f32,
    origin: Vec2,
    pub w: f32,
    pub h: f32,
}

impl Ui {
    pub fn new(view: Rect, w: f32) -> Self {
        let u = (view.w / w).floor().max(1.);
        Ui { u, origin: view.point(), w, h: view.h / u }
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let u = self.u;
        draw_rectangle(self.origin.x + (x * u).round(), self.origin.y + (y * u).round(), (w * u).round(), (h * u).round(), c);
    }

    fn text(&self, s: &str, x: f32, y: f32, scale: f32, c: Color) {
        text_shadowed(s, self.origin.x + (x * self.u).round(), self.origin.y + (y * self.u).round(), scale * self.u, c);
    }

    fn text_centered(&self, s: &str, cx: f32, y: f32, scale: f32, c: Color) {
        self.text(s, (cx - text_width(s) * scale / 2.).round(), y, scale, c);
    }

    fn icon(&self, rows: &[&str], x: f32, y: f32, scale: f32, dim: bool) {
        for (j, row) in rows.iter().enumerate() {
            for (i, b) in row.bytes().enumerate() {
                let c = match b {
                    b'k' => OUTLINE,
                    b'w' => rgb(236, 232, 220),
                    b's' => rgb(150, 156, 166),
                    b'y' => GOLD,
                    b'b' => rgb(110, 72, 40),
                    b'r' => BLOOD,
                    b'R' => rgb(120, 22, 26),
                    b'p' => rgb(250, 150, 140),
                    b'c' => rgb(130, 196, 250),
                    b'C' => rgb(60, 110, 180),
                    _ => continue,
                };
                let c = if dim && b != b'k' { rgb(64, 60, 62) } else { c };
                self.rect(x + i as f32 * scale, y + j as f32 * scale, scale, scale, c);
            }
        }
    }

    /// A wrought-iron frame with gold rivets in the corners.
    fn frame(&self, x: f32, y: f32, w: f32, h: f32) {
        self.rect(x, y, w, h, OUTLINE);
        self.rect(x + 1., y + 1., w - 2., h - 2., IRON);
        self.rect(x + 1., y + 1., w - 2., 1., IRON_LIGHT);
        self.rect(x + 1., y + 1., 1., h - 2., IRON_LIGHT);
        self.rect(x + 1., y + h - 2., w - 2., 1., IRON_DARK);
        self.rect(x + 2., y + 2., w - 4., h - 4., OUTLINE);
        self.rect(x + 3., y + 3., w - 6., h - 6., PANEL);
        for (rx, ry) in [(x, y), (x + w - 4., y), (x, y + h - 4.), (x + w - 4., y + h - 4.)] {
            self.rect(rx, ry, 4., 4., OUTLINE);
            self.rect(rx + 1., ry + 1., 2., 2., GOLD);
            self.rect(rx + 1., ry + 1., 1., 1., GOLD_LIGHT);
        }
    }

    /// A bar in a dark trough, shaded top and bottom, split into `segments`.
    fn bar(&self, x: f32, y: f32, w: f32, h: f32, frac: f32, fill: [Color; 3], segments: u32) {
        self.rect(x - 1., y - 1., w + 2., h + 2., OUTLINE);
        self.rect(x, y, w, h, TROUGH);
        let fw = (w * frac.clamp(0., 1.)).round();
        if fw > 0. {
            self.rect(x, y, fw, h, fill[0]);
            self.rect(x, y, fw, 1., fill[1]);
            self.rect(x, y + h - 1., fw, 1., fill[2]);
        }
        for i in 1..segments {
            let sx = (x + w * i as f32 / segments as f32).round();
            self.rect(sx, y, 1., h, alpha(OUTLINE, 0.55));
        }
    }

    /// A hanging cloth ribbon with notched tails and a gold trim.
    fn banner(&self, cx: f32, y: f32, w: f32, h: f32) {
        let x = (cx - w / 2.).round();
        // Tails behind the ends, with a V cut out of their outer edge.
        let mid = (h / 2.).floor();
        for row in 0..h as i32 {
            let j = row as f32;
            let cut = (5. - (j - mid).abs()).max(0.);
            let edge = row == 0 || row == h as i32 - 1;
            let fill = if edge { OUTLINE } else { CLOTH_DARK };
            let ty = y + 3. + j;
            self.rect(x - 12. + cut, ty, 12. - cut, 1., OUTLINE);
            self.rect(x - 11. + cut, ty, 11. - cut, 1., fill);
            self.rect(x + w, ty, 12. - cut, 1., OUTLINE);
            self.rect(x + w, ty, 11. - cut, 1., fill);
        }
        self.rect(x, y, w, h, OUTLINE);
        self.rect(x + 1., y + 1., w - 2., h - 2., CLOTH);
        self.rect(x + 1., y + 1., w - 2., 1., CLOTH_LIGHT);
        self.rect(x + 1., y + 2., w - 2., 1., GOLD_DARK);
        self.rect(x + 1., y + h - 3., w - 2., 1., GOLD_DARK);
        self.rect(x + 1., y + h - 2., w - 2., 1., CLOTH_DARK);
        // Wear: dark stains, and loose threads along the edges.
        for i in 0..(w as i32 / 6) {
            let k = hash(i, w as i32);
            let (sx, sy) = (x + 2. + (k % (w as u32 - 4)) as f32, y + 4. + ((k >> 8) % (h as u32 - 8)) as f32);
            self.rect(sx, sy, 1. + ((k >> 16) % 3) as f32, 1., alpha(CLOTH_DARK, 0.8));
        }
        for c in 0..w as i32 {
            match hash(c, 77) % 9 {
                0 => self.rect(x + c as f32, y + h, 1., 1. + (hash(c, 78) % 2) as f32, CLOTH_DARK),
                1 => self.rect(x + c as f32, y - 1., 1., 1., CLOTH_DARK),
                _ => {}
            }
        }
    }

    /// Wraps `s` into lines no wider than `width` font pixels.
    fn wrap(s: &str, width: f32) -> Vec<String> {
        let mut lines = vec![String::new()];
        for word in s.split(' ') {
            let line = lines.last_mut().unwrap();
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if text_width(&candidate) > width && !line.is_empty() {
                lines.push(word.to_string());
            } else {
                *line = candidate;
            }
        }
        lines
    }
}

/// What the HUD shows about the fight.
pub struct Status<'a> {
    pub player: &'a Player,
    pub wave: u32,
    pub remaining: usize,
    pub defeated: u32,
    /// Seconds until the next wave, during the breather.
    pub next_wave: Option<f32>,
    pub time: f32,
}

pub fn draw_hud(ui: &Ui, s: &Status) {
    let p = s.player;
    // Health, stamina and shield in an iron frame, top left.
    let shields = p.upgrades.shield_hits();
    let (fx, fy, fw) = (6., 6., 160.);
    let fh = if shields > 0 { 46. } else { 34. };
    ui.frame(fx, fy, fw, fh);
    let low = p.hp > 0 && p.hp <= 3;
    let beat = if low { ((s.time * 6.).sin() > 0.) as i32 as f32 } else { 0. };
    ui.icon(&HEART, fx + 6. - beat * 0.5, fy + 7. - beat * 0.5, 1., p.hp <= 0);
    let hp_fill = if low && beat > 0. { [rgb(236, 70, 60), rgb(255, 150, 130), rgb(150, 30, 30)] } else { [BLOOD, rgb(236, 110, 96), rgb(118, 22, 26)] };
    let (bx, bw) = (fx + 19., fw - 26.);
    ui.bar(bx, fy + 7., bw, 9., p.hp.max(0) as f32 / MAX_HP as f32, hp_fill, MAX_HP as u32);
    let hp = format!("{}/{}", p.hp.max(0), MAX_HP);
    ui.text(&hp, bx + bw - text_width(&hp) - 2., fy + 8., 1., PARCHMENT);
    // Stamina flashes red when an attack was refused for lack of it.
    let refused = p.out_of_stamina > 0. && (p.out_of_stamina * 12.) as i32 % 2 == 0;
    let st_fill = if refused { [rgb(230, 60, 50), rgb(255, 140, 120), rgb(140, 30, 30)] } else { [rgb(214, 170, 60), GOLD_LIGHT, GOLD_DARK] };
    ui.icon(&BOLT, fx + 7., fy + 20., 1., false);
    ui.bar(bx, fy + 22., bw, 5., p.stamina / p.max_stamina(), st_fill, 1);
    for i in 0..shields {
        ui.icon(&SHIELD, bx + i as f32 * 10., fy + 32., 1., i >= p.shield);
    }

    // The wave on a red banner, top center, and the count below it.
    let title = format!("WAVE {}", s.wave);
    let tw = text_width(&title) * 2. + 28.;
    ui.banner(ui.w / 2., 4., tw, 22.);
    ui.title_text(&title, (ui.w / 2. - text_width(&title)).round(), 8., 2.);
    let info = match s.next_wave {
        Some(t) => format!("NEXT WAVE IN {}", t.ceil()),
        None => format!("{} LEFT", s.remaining),
    };
    let slain = format!("{} SLAIN", s.defeated);
    let iw = text_width(&info) + text_width(&slain) + 36.;
    let ix = (ui.w / 2. - iw / 2.).round();
    ui.rect(ix, 29., iw, 11., alpha(OUTLINE, 0.75));
    ui.icon(&SKULL, ix + 3., 31., 1., false);
    ui.text(&info, ix + 12., 31., 1., PARCHMENT);
    let sx = ix + 12. + text_width(&info) + 7.;
    ui.icon(&SWORD, sx, 30., 1., false);
    ui.text(&slain, sx + 11., 31., 1., PARCHMENT_DIM);

    // Power-ups as slots, top right; dash and vortex show their key and recharge.
    let up = &p.upgrades;
    let mut slots: Vec<(Offer, u32, Option<&str>, f32)> = vec![(Offer::Slash, 1 + up.slash, None, 0.)];
    if up.shield > 0 {
        slots.push((Offer::Shield, up.shield, None, 0.));
    }
    if up.dash > 0 {
        slots.push((Offer::Dash, up.dash, Some("E"), (p.dash_cooldown / up.dash_cooldown()).clamp(0., 1.)));
    }
    if up.life_steal > 0 {
        slots.push((Offer::LifeSteal, up.life_steal, None, 0.));
    }
    if up.vortex > 0 {
        slots.push((Offer::Vortex, 1 + up.vortex_cost + up.vortex_damage, Some("Q"), 0.));
    }
    if up.stamina > 0 {
        slots.push((Offer::Stamina, up.stamina, None, 0.));
    }
    let size = 20.;
    let mut x = ui.w - 6. - slots.len() as f32 * (size + 3.) + 3.;
    for (offer, level, key, cooldown) in slots {
        ui.rect(x, 6., size, size, OUTLINE);
        ui.rect(x + 1., 7., size - 2., size - 2., IRON);
        ui.rect(x + 1., 7., size - 2., 1., IRON_LIGHT);
        ui.rect(x + 2., 8., size - 4., size - 4., PANEL);
        let rows = icon_for(offer);
        let (iw, ih) = (rows[0].len() as f32, rows.len() as f32);
        ui.icon(rows, x + ((size - iw) / 2.).floor(), 6. + ((size - ih) / 2.).floor() - 1., 1., false);
        if cooldown > 0. {
            let h = ((size - 4.) * cooldown).round();
            ui.rect(x + 2., 8., size - 4., h, Color::new(0., 0., 0., 0.65));
        }
        ui.text_centered(roman(level), x + size / 2., 6. + size + 2., 1., GOLD);
        if let Some(k) = key {
            ui.rect(x - 2., 4., 9., 11., OUTLINE);
            ui.rect(x - 1., 5., 7., 9., IRON_DARK);
            ui.text(k, x, 6., 1., PARCHMENT);
        }
        x += size + 3.;
    }

    // Controls along the bottom, quiet.
    let help = "WASD MOVE * SHIFT SPRINT * J/SPACE ATTACK * E DASH * Q VORTEX * F5 RESTART * ESC MENU";
    ui.text_centered(help, ui.w / 2., ui.h - 11., 1., alpha(PARCHMENT_DIM, 0.6));
}

/// The end-of-wave pick: three parchment cards in wooden frames, each with its key on a wax seal.
pub fn draw_choice(ui: &Ui, wave: u32, offers: &[Offer], player: &Player, time: f32) {
    draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(0.02, 0.01, 0.02, 0.72));
    let title = format!("WAVE {wave} CLEARED");
    let tw = text_width(&title) * 2. + 40.;
    ui.banner(ui.w / 2., 46., tw, 24.);
    ui.title_text(&title, (ui.w / 2. - text_width(&title)).round(), 51., 2.);
    ui.text_centered("CHOOSE A BOON", ui.w / 2., 80., 1., GOLD);
    let (cw, ch, gap) = (132., 150., 10.);
    let total = offers.len() as f32 * cw + (offers.len() as f32 - 1.) * gap;
    for (i, &offer) in offers.iter().enumerate() {
        let x = (ui.w / 2. - total / 2. + i as f32 * (cw + gap)).round();
        let y = 96. + ((time * 2. + i as f32).sin() * 1.).round();
        ui.wood_frame(x, y, cw, ch);
        ui.parchment(x + 6., y + 6., cw - 12., ch - 12., i as i32 * 7 + 3);
        // Iron medallion with the icon.
        ui.medallion((x + cw / 2. - 14.).round(), y + 12., 28.);
        let rows = icon_for(offer);
        let (iw, ih) = (rows[0].len() as f32 * 2., rows.len() as f32 * 2.);
        ui.icon(rows, x + ((cw - iw) / 2.).floor(), y + 26. - (ih / 2.).floor(), 2., false);
        let name = offer.title().to_uppercase();
        ui.text_bold(&name, x + cw / 2. - (text_width(&name) + 1.) / 2., y + 46., INK);
        // Slash starts at rank I; the vortex upgrades improve a power she has.
        let rank = player.upgrades.next_level(offer) + (offer == Offer::Slash) as u32;
        let rank = match offer {
            Offer::VortexCost | Offer::VortexDamage => "UPGRADE".to_string(),
            _ if rank == 1 => "NEW".to_string(),
            _ => format!("RANK {}", roman(rank)),
        };
        ui.text_bold(&rank, x + cw / 2. - (text_width(&rank) + 1.) / 2., y + 56., GOLD_DARK);
        ui.rect(x + 14., y + 67., cw - 28., 1., rgb(150, 124, 84));
        ui.rect(x + 14., y + 68., cw - 28., 1., rgb(226, 208, 164));
        let mut ly = y + 74.;
        for line in Ui::wrap(&player.upgrades.describe(offer).to_uppercase(), cw - 24.) {
            ui.text_ink(&line, x + 12., ly, INK);
            ly += 10.;
        }
        ui.wax_seal(x + cw / 2., y + ch - 20., &format!("{}", i + 1));
    }
}

/// Cheap integer hash, for wear and stains that stay put from frame to frame.
fn hash(a: i32, b: i32) -> u32 {
    let mut h = (a as u32).wrapping_mul(374761393) ^ (b as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

const RUST: Color = rgb(124, 50, 32);
const RUST_LIGHT: Color = rgb(170, 82, 50);

impl Ui {
    /// Ink on parchment: no drop shadow.
    fn text_ink(&self, s: &str, x: f32, y: f32, c: Color) {
        text(s, self.origin.x + (x * self.u).round(), self.origin.y + (y * self.u).round(), self.u, c);
    }

    /// Ink drawn twice, a pixel apart, for a heavier stroke.
    fn text_bold(&self, s: &str, x: f32, y: f32, c: Color) {
        self.text_ink(s, x.round(), y, c);
        self.text_ink(s, x.round() + 1., y, c);
    }

    /// Engraved gold lettering: a dark outline, then light-to-dark gold by row.
    fn title_text(&self, s: &str, x: f32, y: f32, scale: f32) {
        let u = self.u;
        let (sx, sy) = (self.origin.x + (x * u).round(), self.origin.y + (y * u).round());
        let px = scale * u;
        text(s, sx + u, sy + 2. * u, px, alpha(OUTLINE, 0.8));
        for (dx, dy) in [(-1., 0.), (1., 0.), (0., -1.), (0., 1.), (-1., -1.), (1., 1.), (-1., 1.), (1., -1.)] {
            text(s, sx + dx * u, sy + dy * u, px, OUTLINE);
        }
        let rows = [GOLD_LIGHT, rgb(244, 210, 120), rgb(232, 190, 96), GOLD, rgb(196, 148, 62), rgb(168, 120, 48), GOLD_DARK];
        let mut cx = sx;
        for c in s.chars() {
            let Some(g) = glyph(c) else {
                cx += (SPACE + 1.) * px;
                continue;
            };
            for (row, line) in g.iter().enumerate() {
                for (col, b) in line.bytes().enumerate() {
                    if b == b'#' {
                        draw_rectangle(cx + col as f32 * px, sy + row as f32 * px, px, px, rows[row]);
                    }
                }
            }
            cx += (g[0].len() as f32 + 1.) * px;
        }
    }

    fn rivet(&self, x: f32, y: f32) {
        self.rect(x, y, 3., 3., OUTLINE);
        self.rect(x, y, 2., 2., rgb(150, 142, 132));
        self.rect(x, y, 1., 1., rgb(214, 206, 192));
    }

    /// A blob of colour with a pixel-round outline.
    fn blob(&self, cx: f32, cy: f32, r: f32, c: Color) {
        let r = r.round();
        for dy in -r as i32..=r as i32 {
            let half = (r * r - (dy * dy) as f32).sqrt().round();
            self.rect(cx - half, cy + dy as f32, half * 2., 1., c);
        }
    }

    /// Dark planks with grain, iron plates and rivets at the corners.
    fn wood_frame(&self, x: f32, y: f32, w: f32, h: f32) {
        self.rect(x, y, w, h, OUTLINE);
        self.rect(x + 1., y + 1., w - 2., h - 2., rgb(66, 46, 32));
        for i in 0..((w + h) as i32 / 3) {
            let k = hash(i, x as i32);
            let c = if hash(i, 5) % 3 == 0 { rgb(88, 64, 44) } else { rgb(48, 33, 24) };
            let len = 4. + (k % 6) as f32;
            if hash(i, 3) % 2 == 0 {
                let gx = x + 2. + (k % (w as u32 - 12)) as f32;
                self.rect(gx, if hash(i, 9) % 2 == 0 { y + 2. } else { y + h - 4. }, len, 1., c);
            } else {
                let gy = y + 2. + (k % (h as u32 - 12)) as f32;
                self.rect(if hash(i, 9) % 2 == 0 { x + 2. } else { x + w - 4. }, gy, 1., len, c);
            }
        }
        self.rect(x + 1., y + 1., w - 2., 1., rgb(104, 78, 54));
        self.rect(x + 5., y + 5., w - 10., h - 10., OUTLINE);
        for (px, py) in [(x, y), (x + w - 9., y), (x, y + h - 9.), (x + w - 9., y + h - 9.)] {
            self.rect(px, py, 9., 9., OUTLINE);
            self.rect(px + 1., py + 1., 7., 7., IRON);
            self.rect(px + 1., py + 1., 7., 1., IRON_LIGHT);
            self.rivet(px + 3., py + 3.);
        }
    }

    /// Old paper: worn darker edges and a few stains.
    fn parchment(&self, x: f32, y: f32, w: f32, h: f32, seed: i32) {
        self.rect(x, y, w, h, rgb(214, 194, 150));
        for i in 0..7 {
            let k = hash(seed, i);
            let cx = x + 8. + (k % 1000) as f32 / 1000. * (w - 16.);
            let cy = y + 8. + ((k >> 10) % 1000) as f32 / 1000. * (h - 16.);
            let r = 3. + ((k >> 20) % 7) as f32;
            for s in 0..3 {
                self.blob(cx, cy, r - s as f32 * r / 3., Color::new(0.45, 0.33, 0.2, 0.07));
            }
        }
        for i in 0..5 {
            let c = Color::new(0.4, 0.28, 0.16, 0.12 - i as f32 * 0.02);
            let f = i as f32;
            self.rect(x + f, y + f, w - 2. * f, 1., c);
            self.rect(x + f, y + h - 1. - f, w - 2. * f, 1., c);
            self.rect(x + f, y + 1. + f, 1., h - 2. - 2. * f, c);
            self.rect(x + w - 1. - f, y + 1. + f, 1., h - 2. - 2. * f, c);
        }
    }

    /// A square iron mount with rusted corners, for an icon.
    fn medallion(&self, x: f32, y: f32, s: f32) {
        self.rect(x, y, s, s, OUTLINE);
        self.rect(x + 1., y + 1., s - 2., s - 2., IRON);
        self.rect(x + 1., y + 1., s - 2., 1., IRON_LIGHT);
        self.rect(x + 1., y + s - 2., s - 2., 1., IRON_DARK);
        self.rect(x + 4., y + 4., s - 8., s - 8., OUTLINE);
        self.rect(x + 5., y + 5., s - 10., s - 10., rgb(44, 34, 32));
        for (cx, cy) in [(x + 1., y + 1.), (x + s - 5., y + 1.), (x + 1., y + s - 5.), (x + s - 5., y + s - 5.)] {
            self.rect(cx, cy, 4., 4., RUST);
            self.rect(cx, cy, 2., 2., RUST_LIGHT);
        }
    }

    fn wax_seal(&self, cx: f32, cy: f32, key: &str) {
        let mut edge = 0f32;
        while edge < std::f32::consts::TAU {
            self.blob(cx + edge.cos() * 8., cy + edge.sin() * 8., 2., OUTLINE);
            edge += 0.5;
        }
        self.blob(cx, cy, 9., OUTLINE);
        self.blob(cx, cy, 8., CLOTH);
        self.blob(cx - 1., cy - 1., 6., CLOTH_LIGHT);
        self.blob(cx, cy, 5., CLOTH_DARK);
        self.blob(cx, cy, 4., CLOTH);
        self.text_bold(key, cx - (text_width(key) + 1.) / 2., cy - 3., rgb(236, 190, 170));
    }
}

pub fn draw_game_over(ui: &Ui, wave: u32, defeated: u32, time: f32) {
    draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(0.08, 0.01, 0.02, 0.72));
    let y = ui.h / 2. - 40.;
    ui.text_centered("YOU HAVE FALLEN", ui.w / 2., y, 4., rgb(186, 34, 34));
    ui.rect(ui.w / 2. - 90., y + 36., 180., 1., GOLD_DARK);
    ui.text_centered(&format!("REACHED WAVE {wave} * {defeated} SKELETONS SLAIN"), ui.w / 2., y + 44., 1., PARCHMENT);
    let pulse = 0.6 + 0.4 * (time * 4.).sin();
    ui.text_centered("PRESS F5 TO RISE AGAIN", ui.w / 2., y + 62., 1., alpha(GOLD, pulse));
}

/// Where things sit on menu.png, as fractions of its size: the torches' flames and
/// the paper inside the iron frame, where the map preview goes.
const MENU_TORCHES: [Vec2; 2] = [vec2(0.042, 0.37), vec2(0.447, 0.38)];
const MENU_PAPER: Rect = Rect { x: 0.535, y: 0.245, w: 0.383, h: 0.514 };

/// The menu's backdrop, and a soft glow to make its torches flicker.
pub struct MenuArt {
    backdrop: Texture2D,
    glow: Texture2D,
}

impl MenuArt {
    pub fn new(backdrop: Texture2D) -> Self {
        let mut glow = Image::gen_image_color(64, 64, BLANK);
        for y in 0..64 {
            for x in 0..64 {
                let d = (vec2(x as f32, y as f32) - 31.5).length() / 32.;
                let a = (1. - d).clamp(0., 1.);
                glow.set_pixel(x, y, Color::new(1., 1., 1., a * a));
            }
        }
        let glow = Texture2D::from_image(&glow);
        glow.set_filter(FilterMode::Linear);
        MenuArt { backdrop, glow }
    }
}

pub fn draw_menu(art: &MenuArt, maps: &[MapInfo], selected: usize, preview: &MapAssets) {
    clear_background(BLACK);
    let (sw, sh) = (screen_width(), screen_height());
    let u = (sw / 640.).min(sh / 360.).floor().max(1.);
    let view = Rect::new(((sw - 640. * u) / 2.).floor(), ((sh - 360. * u) / 2.).floor(), 640. * u, 360. * u);
    let ui = Ui::new(view, 640.);
    let time = get_time() as f32;
    // The backdrop covers the whole window, cropped to keep its shape.
    let size = art.backdrop.size();
    let cover = (sw / size.x).max(sh / size.y);
    let back = Rect::new((sw - size.x * cover) / 2., (sh - size.y * cover) / 2., size.x * cover, size.y * cover);
    draw_texture_ex(&art.backdrop, back.x, back.y, WHITE, DrawTextureParams { dest_size: Some(back.size()), ..Default::default() });
    let on_back = |f: Vec2| vec2(back.x + f.x * back.w, back.y + f.y * back.h);

    for (i, &torch) in MENU_TORCHES.iter().enumerate() {
        let t = time + i as f32 * 2.3;
        let flicker = 0.85 + 0.1 * (t * 9.).sin() + 0.05 * (t * 23.).sin();
        let size = 120. * flicker * u;
        let at = on_back(torch);
        let params = DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() };
        draw_texture_ex(&art.glow, at.x - size / 2., at.y - size / 2., Color::new(1., 0.55, 0.22, 0.22 * flicker), params);
    }

    // Title, a divider with a diamond, the list.
    let cx = 150.;
    let title = "BIBI-SURVIVOR";
    ui.title_text(title, (cx - text_width(title) * 1.5).round(), 60., 3.);
    let (dl, dr, dy) = (cx - 100., cx + 100., 90.);
    ui.rect(dl, dy, dr - dl, 3., OUTLINE);
    ui.rect(dl + 1., dy + 1., dr - dl - 2., 1., GOLD_DARK);
    for ex in [dl - 4., dr] {
        ui.rect(ex, dy - 2., 4., 7., OUTLINE);
        ui.rect(ex + 1., dy - 1., 2., 5., GOLD);
    }
    ui.blob(cx, dy + 1., 4., OUTLINE);
    ui.blob(cx, dy + 1., 3., GOLD);
    ui.rect(cx - 1., dy - 1., 1., 1., GOLD_LIGHT);
    ui.text_centered("CHOOSE THY DESTINATION", cx, 106., 1., PARCHMENT_DIM);
    for (i, map) in maps.iter().enumerate() {
        let y = 128. + i as f32 * 26.;
        let name = map.title.to_uppercase();
        if i == selected {
            ui.banner(cx, y - 5., text_width(&name) * 2. + 40., 22.);
            ui.text_centered(&name, cx, y, 2., PARCHMENT);
        } else {
            ui.text_centered(&name, cx, y, 2., alpha(PARCHMENT_DIM, 0.85));
        }
    }

    // The highlighted map on the paper inside the frame, keeping its shape.
    let paper = Rect::new(back.x + MENU_PAPER.x * back.w, back.y + MENU_PAPER.y * back.h, MENU_PAPER.w * back.w, MENU_PAPER.h * back.h);
    let map = preview.map.size();
    let fit = (paper.w / map.x).min(paper.h / map.y);
    let dest = map * fit;
    let (mx, my) = ((paper.center().x - dest.x / 2.).round(), (paper.center().y - dest.y / 2.).round());
    draw_rectangle(mx - u, my - u, dest.x + 2. * u, dest.y + 2. * u, OUTLINE);
    draw_texture_ex(&preview.map, mx, my, Color::new(0.92, 0.88, 0.8, 1.), DrawTextureParams { dest_size: Some(dest), ..Default::default() });

    ui.text("W/S OR ARROWS CHOOSE * ENTER/SPACE PLAY * ESC QUIT", 22., ui.h - 22., 1., alpha(PARCHMENT_DIM, 0.7));
}

/// Translucent drawing lowers the window's alpha, and Windows can then show the desktop
/// through; this writes alpha 1 everywhere and leaves the colours alone.
pub struct AlphaFix(Material);

impl AlphaFix {
    pub fn new() -> Self {
        const VERTEX: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1); }";
        const FRAGMENT: &str = "#version 100
precision lowp float;
void main() { gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0); }";
        let pipeline_params = PipelineParams {
            color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
            alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero)),
            ..Default::default()
        };
        let shader = ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT };
        let material = load_material(shader, MaterialParams { pipeline_params, ..Default::default() }).expect("alpha fix shader compiles");
        AlphaFix(material)
    }

    /// Call last thing in the frame, with the default camera.
    pub fn apply(&self) {
        gl_use_material(&self.0);
        draw_rectangle(0., 0., screen_width(), screen_height(), WHITE);
        gl_use_default_material();
    }
}
