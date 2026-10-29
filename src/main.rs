#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod enemies;
mod level;
mod player;

use assets::{Assets, MapAssets, MapInfo};
use enemies::Horde;
use level::Level;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use player::{
    slash_level_for, Player, MAX_HP, MAX_SLASH_LEVEL, MAX_STAMINA, SHIELD_EVERY, SHIELD_HITS, SLASH_LEVELS,
};

/// Internal resolution; scaled up by whole multiples to the window.
const VIEW_W: f32 = 640.;
const VIEW_H: f32 = 360.;

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    color: Color,
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
        window_title: "Demo Bianca".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn draw_menu(maps: &[MapInfo], selected: usize, preview: &MapAssets) {
    clear_background(Color::from_rgba(18, 16, 22, 255));
    let (sw, sh) = (screen_width(), screen_height());
    // Preview of the highlighted map on the right.
    let size = preview.map.size();
    let pw = sw * 0.5;
    let ph = pw * size.y / size.x;
    let (px, py) = (sw - pw - 40., (sh - ph) / 2.);
    draw_rectangle_lines(px - 3., py - 3., pw + 6., ph + 6., 3., WHITE);
    draw_texture_ex(&preview.map, px, py, WHITE, DrawTextureParams { dest_size: Some(vec2(pw, ph)), ..Default::default() });

    draw_text("DEMO BIANCA", 60., 120., 64., WHITE);
    draw_text("Choose a map", 60., 175., 30., GRAY);
    for (i, map) in maps.iter().enumerate() {
        let (marker, color) = if i == selected { ("> ", Color::from_rgba(255, 214, 90, 255)) } else { ("  ", WHITE) };
        draw_text(&format!("{marker}{}", map.title), 60., 250. + i as f32 * 50., 40., color);
    }
    draw_text("W/S or arrows choose   Enter/Space play   Esc quit", 60., sh - 50., 22., GRAY);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut assets = Assets::load();
    let maps = assets::list_maps();
    // `--map <id>` skips the menu and starts that map straight away.
    let args: Vec<String> = std::env::args().collect();
    let start_map = args.iter().position(|a| a == "--map").and_then(|i| args.get(i + 1));
    let mut selected = start_map.and_then(|id| maps.iter().position(|m| &m.id == id)).unwrap_or(0);
    let mut map = assets::load_map(&maps[selected].id);
    let mut in_menu = start_map.is_none();
    let mut level = Level::new(&map);
    let mut player = Player::new(level.spawn);
    let mut horde = Horde::new();
    let mut particles: Vec<Particle> = Vec::new();
    let mut hitstop = 0.;
    let mut shake = 0.;
    // Screen flash when the player gets hit: red for health, blue for the shield.
    let mut hurt_flash: f32 = 0.;
    let mut flash_color = RED;
    // Text rising over the player's head ("SLASH LV 2!", "SHIELD!").
    let mut popup: Option<(String, Color)> = None;
    let mut popup_time: f32 = 0.;
    // Shields handed out so far, one every SHIELD_EVERY kills.
    let mut shields_given = 0;

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
            draw_menu(&maps, selected, &map);
            if start {
                level = Level::new(&map);
                player = Player::new(level.spawn);
                horde = Horde::new();
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
            player = Player::new(level.spawn);
            horde = Horde::new();
            particles.clear();
            shields_given = 0;
        }

        let dt = get_frame_time().min(1. / 30.);
        if hitstop > 0. {
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
            let earned = slash_level_for(horde.defeated);
            if earned > player.slash_level {
                player.slash_level = earned;
                popup = Some((format!("SLASH LV {}!", earned), Color::from_rgba(255, 214, 90, 255)));
                popup_time = 1.8;
                burst(&mut particles, player.pos - vec2(0., 12.), 24, Vec2::ZERO, Color::from_rgba(255, 214, 90, 255));
            }
            if horde.defeated / SHIELD_EVERY > shields_given {
                shields_given = horde.defeated / SHIELD_EVERY;
                player.shield = SHIELD_HITS;
                popup = Some(("SHIELD!".to_string(), Color::from_rgba(150, 210, 255, 255)));
                popup_time = 1.8;
                burst(&mut particles, player.pos - vec2(0., 11.), 20, Vec2::ZERO, Color::from_rgba(150, 210, 255, 255));
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
        for p in &particles {
            draw_rectangle(p.pos.x.round(), p.pos.y.round(), 2., 2., p.color);
        }
        if let Some((text, color)) = popup.as_ref().filter(|_| popup_time > 0.) {
            let w = measure_text(text, None, 16, 1.).width;
            let rise = (1.8 - popup_time) * 10.;
            let pos = player.pos - vec2(w / 2., 36. + rise);
            draw_text(text, pos.x.round() + 1., pos.y.round() + 1., 16., Color::new(0., 0., 0., 0.7));
            draw_text(text, pos.x.round(), pos.y.round(), 16., *color);
        }

        set_default_camera();
        clear_background(BLACK);
        let scale = (screen_width() / VIEW_W).min(screen_height() / VIEW_H).floor().max(1.);
        let dest = vec2(VIEW_W, VIEW_H) * scale;
        draw_texture_ex(
            &target.texture,
            ((screen_width() - dest.x) / 2.).floor(),
            ((screen_height() - dest.y) / 2.).floor(),
            WHITE,
            // Render targets come out upside down with a y-down camera.
            DrawTextureParams { dest_size: Some(dest), flip_y: true, ..Default::default() },
        );
        draw_text(
            "WASD/arrows move   Shift sprint   J/Space attack   F5 restart   Esc menu",
            16.,
            28.,
            22.,
            WHITE,
        );
        let next = SLASH_LEVELS.get(player.slash_level as usize - 1);
        let slash = match next {
            Some(n) => format!("Slash LV {}  (next at {} kills)", player.slash_level, n),
            None => format!("Slash LV {} (max)", MAX_SLASH_LEVEL),
        };
        draw_text(&format!("Skeletons defeated: {}    {}", horde.defeated, slash), 16., 54., 22., WHITE);

        let (bar_x, bar_y, bar_w) = (16., 66., 200.);
        draw_rectangle(bar_x - 2., bar_y - 2., bar_w + 4., 18., Color::from_rgba(20, 14, 18, 230));
        draw_rectangle(bar_x, bar_y, bar_w * player.hp.max(0) as f32 / MAX_HP as f32, 14., Color::from_rgba(214, 64, 58, 255));
        draw_text(&format!("HP {}/{}", player.hp.max(0), MAX_HP), bar_x + bar_w + 12., bar_y + 13., 22., WHITE);
        // Stamina bar: flashes red when an attack was refused for lack of it.
        let stamina_y = bar_y + 22.;
        let stamina_color = if player.out_of_stamina > 0. && (player.out_of_stamina * 12.) as i32 % 2 == 0 {
            Color::from_rgba(230, 60, 50, 255)
        } else {
            Color::from_rgba(232, 196, 72, 255)
        };
        draw_rectangle(bar_x - 2., stamina_y - 2., bar_w + 4., 12., Color::from_rgba(20, 14, 18, 230));
        draw_rectangle(bar_x, stamina_y, bar_w * player.stamina / MAX_STAMINA, 8., stamina_color);
        draw_text("STAMINA", bar_x + bar_w + 12., stamina_y + 9., 20., WHITE);
        // Shield charges, or how far the next shield is.
        let shield_y = stamina_y + 16.;
        for i in 0..SHIELD_HITS {
            let full = i < player.shield;
            let x = bar_x + i as f32 * 22.;
            draw_rectangle(x - 2., shield_y - 2., 20., 14., Color::from_rgba(20, 14, 18, 230));
            if full {
                draw_rectangle(x, shield_y, 16., 10., Color::from_rgba(120, 190, 255, 255));
            }
        }
        let next_shield = (horde.defeated / SHIELD_EVERY + 1) * SHIELD_EVERY;
        draw_text(&format!("SHIELD  (next at {} kills)", next_shield), bar_x + SHIELD_HITS as f32 * 22. + 8., shield_y + 10., 20., WHITE);

        if hurt_flash > 0. {
            let c = flash_color;
            draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(c.r, c.g, c.b, hurt_flash * 0.8));
        }
        if !player.alive() {
            draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(0., 0., 0., 0.55));
            let center = |text: &str, y: f32, size: f32| {
                let w = measure_text(text, None, size as u16, 1.).width;
                draw_text(text, (screen_width() - w) / 2., y, size, WHITE);
            };
            center("GAME OVER", screen_height() / 2. - 20., 72.);
            center(&format!("Skeletons defeated: {}", horde.defeated), screen_height() / 2. + 30., 30.);
            center("Press F5 to play again", screen_height() / 2. + 70., 26.);
        }

        next_frame().await;
    }
}
