#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod atmosphere;
mod enemies;
mod hud;
mod level;
mod player;
mod upgrades;

use assets::Assets;
use atmosphere::Atmosphere;
use enemies::Horde;
use hud::{Status, Ui};
use level::Level;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use player::Player;
use upgrades::Offer;

/// Internal resolution; scaled up by whole multiples to the window.
const VIEW_W: f32 = 640.;
const VIEW_H: f32 = 360.;
/// Breather between picking a power-up and the next wave.
const WAVE_PAUSE: f32 = 4.;

enum Phase {
    Fighting,
    /// Wave cleared: the fight is frozen until she picks one of these.
    Choosing(Vec<Offer>),
    /// Time left before the next wave rises.
    Pause(f32),
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    color: Color,
}

/// Fisher-Yates shuffle. macroquad's `ChooseRandom::shuffle` never leaves an item
/// where it was, so the first of a list almost never came up first.
pub fn shuffle<T>(items: &mut [T]) {
    for i in (1..items.len()).rev() {
        items.swap(i, gen_range(0, i + 1));
    }
}

fn burst(particles: &mut Vec<Particle>, at: Vec2, count: usize, push: Vec2, color: Color) {
    for _ in 0..count {
        let spread = vec2(gen_range(-60., 60.), gen_range(-90., 0.));
        let vel = push * gen_range(30., 110.) + spread;
        particles.push(Particle { pos: at, vel, life: gen_range(0.2, 0.45), color });
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Bibi-Survivor".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::rand::srand(macroquad::miniquad::date::now().to_bits());
    let mut assets = Assets::load();
    let maps = assets::list_maps();
    // `--map <id>` skips the menu and starts that map straight away.
    let args: Vec<String> = std::env::args().collect();
    let start_map = args.iter().position(|a| a == "--map").and_then(|i| args.get(i + 1));
    let mut selected = start_map.and_then(|id| maps.iter().position(|m| &m.id == id)).unwrap_or(0);
    let mut map = assets::load_map(&maps[selected].id);
    let mut in_menu = start_map.is_none();
    // `--wave <n>` starts every run from that wave (for testing later waves).
    let first_wave = args.iter().position(|a| a == "--wave").and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(1).max(1);
    let mut level = Level::new(&map);
    let mut atmosphere = Atmosphere::new(&map.settings, vec2(VIEW_W, VIEW_H));
    let mut player = Player::new(level.spawn);
    let mut horde = Horde::new(first_wave);
    let mut particles: Vec<Particle> = Vec::new();
    let mut hitstop = 0.;
    let mut shake = 0.;
    // Screen flash when the player gets hit: red for health, blue for the shield.
    let mut hurt_flash: f32 = 0.;
    let mut flash_color = RED;
    // Text rising over the player's head ("SLASH!", "WAVE 2").
    let mut popup: Option<(String, Color)> = None;
    let mut popup_time: f32 = 0.;
    let mut phase = Phase::Fighting;
    let menu_art = hud::MenuArt::new(assets.menu.clone());
    let alpha_fix = hud::AlphaFix::new();

    let target = render_target(VIEW_W as u32, VIEW_H as u32);
    target.texture.set_filter(FilterMode::Nearest);
    let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., VIEW_W, VIEW_H));
    camera.render_target = Some(target.clone());
    let mut cam_pos = player.pos;

    loop {
        if in_menu {
            if is_key_pressed(KeyCode::Escape) {
                break;
            }
            let pressed = |keys: &[KeyCode]| keys.iter().any(|k| is_key_pressed(*k));
            let step = pressed(&[KeyCode::S, KeyCode::Down]) as usize + (maps.len() - 1) * pressed(&[KeyCode::W, KeyCode::Up]) as usize;
            if step > 0 {
                selected = (selected + step) % maps.len();
                map = assets::load_map(&maps[selected].id);
            }
            let start = pressed(&[KeyCode::Enter, KeyCode::KpEnter, KeyCode::Space, KeyCode::J]);
            hud::draw_menu(&menu_art, &maps, selected, &map);
            alpha_fix.apply();
            if start {
                level = Level::new(&map);
                atmosphere = Atmosphere::new(&map.settings, vec2(VIEW_W, VIEW_H));
                player = Player::new(level.spawn);
                horde = Horde::new(first_wave);
                phase = Phase::Fighting;
                particles.clear();
                cam_pos = player.pos;
                in_menu = false;
            }
            next_frame().await;
            continue;
        }
        if is_key_pressed(KeyCode::Escape) {
            in_menu = true;
            next_frame().await;
            continue;
        }
        if is_key_pressed(KeyCode::F5) {
            assets = Assets::load();
            map = assets::load_map(&maps[selected].id);
            level = Level::new(&map);
            atmosphere = Atmosphere::new(&map.settings, vec2(VIEW_W, VIEW_H));
            player = Player::new(level.spawn);
            horde = Horde::new(first_wave);
            phase = Phase::Fighting;
            particles.clear();
        }

        let dt = get_frame_time().min(1. / 30.);
        if let Phase::Choosing(offers) = &phase {
            let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3];
            let picked = keys.iter().zip(offers).find(|(k, _)| is_key_pressed(**k)).map(|(_, &o)| o);
            if let Some(offer) = picked {
                player.upgrades.take(offer);
                if offer == Offer::Shield {
                    player.restore_shield();
                }
                popup = Some((format!("{}!", offer.title().to_uppercase()), Color::from_rgba(255, 214, 90, 255)));
                popup_time = 1.8;
                burst(&mut particles, player.pos - vec2(0., 12.), 24, Vec2::ZERO, Color::from_rgba(255, 214, 90, 255));
                phase = Phase::Pause(WAVE_PAUSE);
            }
        } else if hitstop > 0. {
            hitstop -= dt;
        } else {
            player.update(dt, &level, &assets.player);
            if let Some(dir) = player.take_wave() {
                horde.launch_wave(player.pos, dir);
            }
            level.update_flow(player.pos);
            let swing = player.swing(&assets.player);
            let events = horde.update(dt, &level, &mut player, swing, &assets);
            if events.player_hit {
                hitstop = 0.08;
                shake = 0.2;
                hurt_flash = 0.25;
                if player.shield_flash > 0. {
                    flash_color = Color::new(0.3, 0.6, 1., 1.);
                    burst(&mut particles, player.pos - vec2(0., 11.), 14, Vec2::ZERO, Color::from_rgba(150, 210, 255, 255));
                } else {
                    flash_color = Color::new(0.8, 0., 0., 1.);
                }
            }
            for hit in events.hits {
                hitstop = 0.05;
                shake = 0.12;
                burst(&mut particles, hit, 8, vec2(player.facing, 0.), Color::from_rgba(239, 233, 220, 255));
            }
            if let Some(at) = events.healed {
                burst(&mut particles, at - vec2(0., 6.), 10, Vec2::ZERO, Color::from_rgba(240, 110, 120, 255));
            }
            if player.leech(events.landed) {
                burst(&mut particles, player.pos - vec2(0., 11.), 8, Vec2::ZERO, Color::from_rgba(240, 110, 120, 255));
            }
            match &mut phase {
                // No reward if she died before the wave was over.
                Phase::Fighting if player.alive() && horde.wave_cleared() => {
                    let offers = player.upgrades.offers();
                    phase = if offers.is_empty() { Phase::Pause(WAVE_PAUSE) } else { Phase::Choosing(offers) };
                }
                Phase::Pause(t) => {
                    *t -= dt;
                    if *t <= 0. {
                        horde.start_wave(horde.wave + 1);
                        player.restore_shield();
                        popup = Some((format!("WAVE {}", horde.wave), Color::from_rgba(240, 110, 120, 255)));
                        popup_time = 1.8;
                        phase = Phase::Fighting;
                    }
                }
                _ => {}
            }
            for p in &mut particles {
                p.vel *= 1. - 4. * dt;
                p.pos += p.vel * dt;
                p.life -= dt;
            }
            particles.retain(|p| p.life > 0.);
        }
        shake -= dt;
        hurt_flash -= dt;
        popup_time -= dt;

        // Camera: follow with a little look-ahead, stay inside the map.
        let half = vec2(VIEW_W, VIEW_H) / 2.;
        let size = level.size();
        let look = player.pos - vec2(-player.facing * 24., 10.);
        cam_pos = cam_pos.lerp(look, 1. - (-5. * dt).exp());
        let mut center = vec2(
            cam_pos.x.clamp(half.x, (size.x - half.x).max(half.x)),
            cam_pos.y.clamp(half.y, (size.y - half.y).max(half.y)),
        );
        if shake > 0. {
            center += vec2(gen_range(-2., 2.), gen_range(-2., 2.));
        }
        camera.target = center.round();

        set_camera(&camera);
        clear_background(BLACK);
        level.draw(&map);
        horde.draw_flame_marks();
        horde.draw_hearts(&assets);
        // Whoever stands lower on screen is in front.
        let mut order: Vec<(Vec2, Option<usize>)> = horde.positions().map(|(p, i)| (p, Some(i))).collect();
        order.push((player.pos, None));
        order.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
        let draw = |who: Option<usize>| match who {
            Some(i) => horde.draw_one(i, &assets),
            None => player.draw(&assets),
        };
        for &(_, who) in &order {
            draw(who);
        }
        // Cut-outs like tree crowns cover whoever is behind them; whoever is in
        // front was painted over too, so draw them again on top.
        for overlay in &level.overlays {
            overlay.draw(&map);
            for &(feet, who) in &order {
                if overlay.near(feet) && !overlay.hides(feet) {
                    draw(who);
                }
            }
        }
        horde.draw_arrows(&assets);
        // Night, fog and firelight; she carries a faint light of her own.
        let view = Rect::new(camera.target.x - half.x, camera.target.y - half.y, VIEW_W, VIEW_H);
        atmosphere.update(dt, view);
        let glows: Vec<(Vec2, f32)> = std::iter::once((player.pos - vec2(0., 10.), 110.)).chain(horde.glows()).collect();
        atmosphere.draw(view, &glows);
        for p in &particles {
            draw_rectangle(p.pos.x.round(), p.pos.y.round(), 2., 2., p.color);
        }
        if let Some((text, color)) = popup.as_ref().filter(|_| popup_time > 0.) {
            let rise = (1.8 - popup_time) * 10.;
            let pos = player.pos - vec2(hud::text_width(text) / 2., 38. + rise);
            hud::text_shadowed(text, pos.x.round(), pos.y.round(), 1., *color);
        }

        set_default_camera();
        clear_background(BLACK);
        let scale = (screen_width() / VIEW_W).min(screen_height() / VIEW_H).floor().max(1.);
        let dest = vec2(VIEW_W, VIEW_H) * scale;
        let (view_x, view_y) = (((screen_width() - dest.x) / 2.).floor(), ((screen_height() - dest.y) / 2.).floor());
        draw_texture_ex(
            &target.texture,
            view_x,
            view_y,
            WHITE,
            // Render targets come out upside down with a y-down camera.
            DrawTextureParams { dest_size: Some(dest), flip_y: true, ..Default::default() },
        );
        let ui = Ui::new(Rect::new(view_x, view_y, dest.x, dest.y), VIEW_W);
        let status = Status {
            player: &player,
            wave: horde.wave,
            remaining: horde.remaining(),
            defeated: horde.defeated,
            next_wave: if let Phase::Pause(t) = phase { Some(t) } else { None },
            time: get_time() as f32,
        };
        hud::draw_hud(&ui, &status);
        if let Phase::Choosing(offers) = &phase {
            hud::draw_choice(&ui, horde.wave, offers, &player, get_time() as f32);
        }

        if hurt_flash > 0. {
            let c = flash_color;
            draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(c.r, c.g, c.b, hurt_flash * 0.8));
        }
        if !player.alive() {
            hud::draw_game_over(&ui, horde.wave, horde.defeated, get_time() as f32);
        }
        alpha_fix.apply();

        next_frame().await;
    }
}
