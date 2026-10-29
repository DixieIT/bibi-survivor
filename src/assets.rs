use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

// Built into the exe, so it runs without an assets/ folder; files in assets/ override these.
const DEFAULT_PLAYER_LAYOUT: &str = include_str!("../assets/player.txt");
const DEFAULT_PLAYER: &[u8] = include_bytes!("../assets/player.png");
const DEFAULT_SKELETON_LAYOUT: &str = include_str!("../assets/skeleton.txt");
const DEFAULT_SKELETON: &[u8] = include_bytes!("../assets/skeleton.png");
const DEFAULT_SHADOW: &[u8] = include_bytes!("../assets/shadow.png");

/// A map built into the exe; a folder assets/maps/<id>/ overrides it file by file.
struct BuiltinMap {
    id: &'static str,
    map: &'static [u8],
    collision: &'static [u8],
    settings: &'static str,
    overlay: &'static [u8],
    overlays: &'static str,
}

macro_rules! builtin_map {
    ($id:literal) => {
        BuiltinMap {
            id: $id,
            map: include_bytes!(concat!("../assets/maps/", $id, "/map.png")),
            collision: include_bytes!(concat!("../assets/maps/", $id, "/collision.png")),
            settings: include_str!(concat!("../assets/maps/", $id, "/map.txt")),
            overlay: include_bytes!(concat!("../assets/maps/", $id, "/overlay.png")),
            overlays: include_str!(concat!("../assets/maps/", $id, "/overlays.txt")),
        }
    };
}

const BUILTIN_MAPS: [BuiltinMap; 2] = [builtin_map!("village"), builtin_map!("town")];

/// A run of frames in a sprite sheet.
#[derive(Clone, Copy)]
pub struct Anim {
    /// Grid cell of the first frame.
    pub cell: (u32, u32),
    pub frames: u32,
    pub fps: f32,
    pub looping: bool,
}

impl Anim {
    pub fn duration(&self) -> f32 {
        self.frames as f32 / self.fps
    }

    pub fn frame_at(&self, t: f32) -> u32 {
        let f = (t * self.fps) as u32;
        if self.looping {
            f % self.frames
        } else {
            f.min(self.frames - 1)
        }
    }
}

pub struct SpriteSheet {
    pub texture: Texture2D,
    pub frame: Vec2,
    pub feet: f32,
    step_down: bool,
    anims: HashMap<String, Anim>,
}

impl SpriteSheet {
    pub fn anim(&self, name: &str) -> Anim {
        self.anims
            .get(name)
            .or_else(|| self.anims.get("idle"))
            .copied()
            .unwrap_or(Anim { cell: (0, 0), frames: 1, fps: 1., looping: true })
    }

    pub fn source(&self, anim: Anim, frame: u32) -> Rect {
        let (mut x, mut y) = anim.cell;
        if self.step_down {
            y += frame;
        } else {
            x += frame;
        }
        Rect::new(x as f32 * self.frame.x, y as f32 * self.frame.y, self.frame.x, self.frame.y)
    }

    pub fn draw_rotated(&self, anim: Anim, frame: u32, center: Vec2, angle: f32, tint: Color) {
        draw_texture_ex(
            &self.texture,
            (center.x - self.frame.x / 2.).round(),
            (center.y - self.frame.y / 2.).round(),
            tint,
            DrawTextureParams {
                source: Some(self.source(anim, frame)),
                rotation: angle,
                pivot: Some(center.round()),
                ..Default::default()
            },
        );
    }

    pub fn draw(&self, anim: Anim, frame: u32, feet: Vec2, facing: f32, tint: Color) {
        draw_texture_ex(
            &self.texture,
            (feet.x - self.frame.x / 2.).round(),
            (feet.y - (self.frame.y - self.feet)).round(),
            tint,
            DrawTextureParams {
                source: Some(self.source(anim, frame)),
                flip_x: facing < 0.,
                ..Default::default()
            },
        );
    }
}

pub struct Assets {
    pub player: SpriteSheet,
    pub skeleton: SpriteSheet,
    pub shadow: Texture2D,
}

pub struct MapAssets {
    pub map: Texture2D,
    pub collision: Image,
    /// map.txt: title, spawn point.
    pub settings: String,
    pub overlay: Texture2D,
    pub overlays: String,
}

pub struct MapInfo {
    pub id: String,
    pub title: String,
}

impl Assets {
    pub fn load() -> Self {
        Assets {
            player: sheet("player", DEFAULT_PLAYER_LAYOUT, DEFAULT_PLAYER),
            skeleton: sheet("skeleton", DEFAULT_SKELETON_LAYOUT, DEFAULT_SKELETON),
            shadow: pixelated(&image("shadow.png", DEFAULT_SHADOW)),
        }
    }
}

pub fn list_maps() -> Vec<MapInfo> {
    let mut ids: Vec<String> = BUILTIN_MAPS.iter().map(|m| m.id.to_string()).collect();
    for dir in asset_dirs() {
        let Ok(entries) = std::fs::read_dir(dir.join("maps")) else { continue };
        let mut found: Vec<String> = entries
            .flatten()
            .filter(|e| e.path().join("map.png").is_file())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|id| !ids.contains(id))
            .collect();
        found.sort();
        ids.extend(found);
    }
    ids.into_iter()
        .map(|id| {
            let settings = map_settings(&id);
            let title = settings
                .lines()
                .find_map(|l| l.trim().strip_prefix("title ").map(|t| t.trim().to_string()))
                .unwrap_or_else(|| id.clone());
            MapInfo { id, title }
        })
        .collect()
}

fn map_settings(id: &str) -> String {
    let builtin = BUILTIN_MAPS.iter().find(|m| m.id == id);
    read_text(&format!("maps/{id}/map.txt"))
        .or_else(|| builtin.map(|m| m.settings.to_string()))
        .unwrap_or_default()
}

pub fn load_map(id: &str) -> MapAssets {
    let builtin = BUILTIN_MAPS.iter().find(|m| m.id == id);
    let file = |name: &str, fallback: Option<&'static [u8]>| {
        find(&format!("maps/{id}/{name}"))
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| Image::from_file_with_format(&bytes, None).ok())
            .or_else(|| fallback.and_then(|b| Image::from_file_with_format(b, None).ok()))
    };
    let map = file("map.png", builtin.map(|m| m.map)).unwrap_or_else(|| Image::gen_image_color(16, 16, BLACK));
    let empty = || Image::gen_image_color(map.width, map.height, BLANK);
    MapAssets {
        collision: file("collision.png", builtin.map(|m| m.collision)).unwrap_or_else(empty),
        overlay: pixelated(&file("overlay.png", builtin.map(|m| m.overlay)).unwrap_or_else(empty)),
        overlays: read_text(&format!("maps/{id}/overlays.txt"))
            .or_else(|| builtin.map(|m| m.overlays.to_string()))
            .unwrap_or_default(),
        settings: map_settings(id),
        map: pixelated(&map),
    }
}

fn asset_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("assets")];
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| Some(p.parent()?.join("assets"))) {
        dirs.insert(0, dir);
    }
    dirs
}

fn find(name: &str) -> Option<PathBuf> {
    asset_dirs().into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

fn read_text(name: &str) -> Option<String> {
    std::fs::read_to_string(find(name)?).ok()
}

fn image(name: &str, default: &[u8]) -> Image {
    find(name)
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| Image::from_file_with_format(&bytes, None).ok())
        .unwrap_or_else(|| Image::from_file_with_format(default, None).expect("built-in PNG is valid"))
}

fn pixelated(image: &Image) -> Texture2D {
    let texture = Texture2D::from_image(image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn sheet(name: &str, default_layout: &str, default_png: &[u8]) -> SpriteSheet {
    let layout = read_text(&format!("{name}.txt")).unwrap_or_else(|| default_layout.to_string());
    let mut sheet = parse_layout(&layout);
    sheet.texture = pixelated(&image(&format!("{name}.png"), default_png));
    sheet
}

fn parse_layout(text: &str) -> SpriteSheet {
    let mut sheet = SpriteSheet {
        texture: Texture2D::empty(),
        frame: vec2(16., 16.),
        feet: 0.,
        step_down: false,
        anims: HashMap::new(),
    };
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("");
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["frame", w, h] => {
                if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                    sheet.frame = vec2(w, h);
                }
            }
            ["feet", f] => sheet.feet = f.parse().unwrap_or(0.),
            ["step", dir] => sheet.step_down = *dir == "down",
            [name, x, y, frames, fps, rest @ ..] => {
                if let (Ok(x), Ok(y), Ok(frames), Ok(fps)) =
                    (x.parse(), y.parse(), frames.parse::<u32>(), fps.parse::<f32>())
                {
                    let looping = rest.first() != Some(&"once");
                    sheet.anims.insert(
                        name.to_string(),
                        Anim { cell: (x, y), frames: frames.max(1), fps: fps.max(0.1), looping },
                    );
                }
            }
            _ => {}
        }
    }
    sheet
}
