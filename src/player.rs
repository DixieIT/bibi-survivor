use crate::assets::{Assets, SpriteSheet};
use crate::level::{Level, Swing};
use macroquad::prelude::*;

const WALK_SPEED: f32 = 70.;
const SPRINT_SPEED: f32 = 115.;
/// Collision box around the feet, so the head can overlap things behind.
const FEET: Vec2 = vec2(8., 4.);
/// Standing still this long plays the sword flourish.
const FLOURISH_AFTER: f32 = 4.;
/// Attacking again within this time after a swing continues the combo.
const COMBO_WINDOW: f32 = 0.4;
/// Animation name, sword reach and hitbox height for each hit of the combo.
const COMBO: [(&str, f32, f32); 3] = [("attack1", 20., 18.), ("attack2", 18., 24.), ("attack3", 26., 10.)];
pub const MAX_HP: i32 = 10;
/// After being hit she can't be hit again for this long (and blinks).
const INVULNERABLE: f32 = 0.8;
const KNOCKBACK: f32 = 150.;
pub const MAX_STAMINA: f32 = 100.;
/// Kills needed to reach slash level 2, 3 and 4.
pub const SLASH_LEVELS: [u32; 3] = [8, 20, 40];
pub const MAX_SLASH_LEVEL: u32 = SLASH_LEVELS.len() as u32 + 1;
/// Level 2: longer, taller swing. Level 3: double damage. Level 4: every swing also throws a wave.
const LONG_SLASH: (f32, f32) = (1.35, 1.25);
/// Every this many kills she gets a shield that takes this many hits instead of her.
pub const SHIELD_EVERY: u32 = 15;
pub const SHIELD_HITS: i32 = 2;
/// Each swing costs this much: about five in a row from full.
const ATTACK_COST: f32 = 18.;
/// Stamina per second, starting shortly after the last swing.
const STAMINA_REGEN: f32 = 45.;
const STAMINA_DELAY: f32 = 0.35;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Walk,
    Flourish,
    Attack(usize),
}

impl State {
    fn anim(self) -> &'static str {
        match self {
            State::Idle => "idle",
            State::Walk => "walk",
            State::Flourish => "flourish",
            State::Attack(i) => COMBO[i].0,
        }
    }
}

struct Input {
    dir: Vec2,
    sprint: bool,
    attack: bool,
}

impl Input {
    fn read() -> Self {
        let down = |keys: &[KeyCode]| keys.iter().any(|k| is_key_down(*k)) as i32 as f32;
        Input {
            dir: vec2(
                down(&[KeyCode::D, KeyCode::Right]) - down(&[KeyCode::A, KeyCode::Left]),
                down(&[KeyCode::S, KeyCode::Down]) - down(&[KeyCode::W, KeyCode::Up]),
            ),
            sprint: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            attack: [KeyCode::J, KeyCode::Space].iter().any(|k| is_key_pressed(*k)),
        }
    }
}

pub struct Player {
    /// Feet position (bottom-center of the sprite).
    pub pos: Vec2,
    /// 1 = facing right, -1 = facing left.
    pub facing: f32,
    state: State,
    state_time: f32,
    anim_time: f32,
    idle_time: f32,
    since_attack: f32,
    /// Index into COMBO of the last attack.
    combo: usize,
    combo_queued: bool,
    swing_id: u32,
    /// Last direction moved or pressed (8 ways): where the next swing goes.
    aim: Vec2,
    /// Direction of the current swing.
    swing_aim: Vec2,
    pub hp: i32,
    invulnerable: f32,
    knockback: Vec2,
    pub stamina: f32,
    stamina_delay: f32,
    /// Counts down after trying to attack without enough stamina (the bar flashes).
    pub out_of_stamina: f32,
    pub slash_level: u32,
    /// A slash wave to launch this frame (its direction), at slash level 4.
    wave: Option<Vec2>,
    /// Hits the shield can still take.
    pub shield: i32,
    /// Counts down after the shield took a hit (for the blue flash).
    pub shield_flash: f32,
}

/// Slash level reached after this many kills.
pub fn slash_level_for(kills: u32) -> u32 {
    1 + SLASH_LEVELS.iter().filter(|&&n| kills >= n).count() as u32
}

impl Player {
    pub fn new(spawn: Vec2) -> Self {
        Player {
            pos: spawn,
            facing: 1.,
            state: State::Idle,
            state_time: 0.,
            anim_time: 0.,
            idle_time: 0.,
            since_attack: f32::MAX,
            combo: 0,
            combo_queued: false,
            swing_id: 0,
            aim: vec2(1., 0.),
            swing_aim: vec2(1., 0.),
            hp: MAX_HP,
            invulnerable: 0.,
            knockback: Vec2::ZERO,
            stamina: MAX_STAMINA,
            stamina_delay: 0.,
            out_of_stamina: 0.,
            slash_level: 1,
            wave: None,
            shield: 0,
            shield_flash: 0.,
        }
    }

    /// Restores health; false if she's dead or already at full health.
    pub fn heal(&mut self, amount: i32) -> bool {
        if !self.alive() || self.hp >= MAX_HP {
            return false;
        }
        self.hp = (self.hp + amount).min(MAX_HP);
        true
    }

    /// The slash wave thrown by the last swing, if any (taken once).
    pub fn take_wave(&mut self) -> Option<Vec2> {
        self.wave.take()
    }

    pub fn alive(&self) -> bool {
        self.hp > 0
    }

    /// Takes one hit from an enemy standing at `from`. Returns false if she
    /// couldn't be hurt right now (still blinking from the last hit, or dead).
    pub fn hurt(&mut self, from: Vec2) -> bool {
        if !self.alive() || self.invulnerable > 0. {
            return false;
        }
        let away = (self.pos - from).normalize_or_zero();
        let away = if away == Vec2::ZERO { vec2(-self.facing, 0.) } else { away };
        self.invulnerable = INVULNERABLE;
        if self.shield > 0 {
            // The shield takes the hit instead of her, and she's pushed back less.
            self.shield -= 1;
            self.shield_flash = 0.3;
            self.knockback = away * KNOCKBACK * 0.5;
        } else {
            self.hp -= 1;
            self.knockback = away * KNOCKBACK;
        }
        true
    }

    /// Moves one axis at a time so she slides along walls.
    fn slide(&mut self, step: Vec2, level: &Level) {
        for axis in [vec2(step.x, 0.), vec2(0., step.y)] {
            if !level.blocked(self.feet_box(self.pos + axis)) {
                self.pos += axis;
            }
        }
    }

    fn feet_box(&self, pos: Vec2) -> Rect {
        Rect::new(pos.x - FEET.x / 2., pos.y - FEET.y, FEET.x, FEET.y)
    }

    fn set_state(&mut self, state: State) {
        if self.state != state {
            self.state = state;
            self.state_time = 0.;
            self.anim_time = 0.;
        }
    }

    /// Starts a swing if there's stamina for it.
    fn try_attack(&mut self, combo: usize) -> bool {
        if self.stamina < ATTACK_COST {
            self.out_of_stamina = 0.4;
            return false;
        }
        self.stamina -= ATTACK_COST;
        self.stamina_delay = STAMINA_DELAY;
        self.start_attack(combo);
        if self.slash_level >= 4 {
            self.wave = Some(self.swing_aim);
        }
        true
    }

    fn start_attack(&mut self, combo: usize) {
        self.swing_id += 1;
        self.swing_aim = self.aim;
        if self.aim.x.abs() > 0.1 {
            self.facing = self.aim.x.signum();
        }
        self.combo = combo;
        self.combo_queued = false;
        self.set_state(State::Attack(combo));
    }

    pub fn update(&mut self, dt: f32, level: &Level, sheet: &SpriteSheet) {
        if !self.alive() {
            return;
        }
        let input = Input::read();
        // Held direction aims the next swing, also mid-combo.
        if input.dir != Vec2::ZERO {
            self.aim = input.dir.normalize();
        }
        self.invulnerable -= dt;
        if self.knockback.length() > 1. {
            self.slide(self.knockback * dt, level);
            self.knockback *= (1. - 10. * dt).max(0.);
        }
        self.state_time += dt;
        self.anim_time += dt;
        self.since_attack += dt;
        self.out_of_stamina -= dt;
        self.shield_flash -= dt;
        if self.stamina_delay > 0. {
            self.stamina_delay -= dt;
        } else {
            self.stamina = (self.stamina + STAMINA_REGEN * dt).min(MAX_STAMINA);
        }

        if let State::Attack(combo) = self.state {
            if input.attack {
                self.combo_queued = true;
            }
            if self.state_time >= sheet.anim(COMBO[combo].0).duration() {
                if !(self.combo_queued && self.try_attack((combo + 1) % COMBO.len())) {
                    self.combo_queued = false;
                    self.since_attack = 0.;
                    self.set_state(State::Idle);
                }
            }
            return;
        }

        if input.attack {
            let next = if self.since_attack < COMBO_WINDOW { (self.combo + 1) % COMBO.len() } else { 0 };
            if self.try_attack(next) {
                return;
            }
        }

        if input.dir == Vec2::ZERO {
            self.idle_time += dt;
            if self.state == State::Flourish {
                if self.state_time >= sheet.anim("flourish").duration() {
                    self.idle_time = 0.;
                    self.set_state(State::Idle);
                }
            } else if self.idle_time >= FLOURISH_AFTER {
                self.set_state(State::Flourish);
            } else {
                self.set_state(State::Idle);
            }
            return;
        }

        self.idle_time = 0.;
        if input.dir.x != 0. {
            self.facing = input.dir.x.signum();
        }
        self.set_state(State::Walk);
        let speed = if input.sprint { SPRINT_SPEED } else { WALK_SPEED };
        if input.sprint {
            self.anim_time += dt * 0.6;
        }

        self.slide(input.dir.normalize() * speed * dt, level);
    }

    /// The sword hurts only during the middle frames of each attack.
    pub fn swing(&self, sheet: &SpriteSheet) -> Option<Swing> {
        let State::Attack(combo) = self.state else { return None };
        if !self.alive() {
            return None;
        }
        let (name, mut reach, mut height) = COMBO[combo];
        if self.slash_level >= 2 {
            reach *= LONG_SLASH.0;
            height *= LONG_SLASH.1;
        }
        let anim = sheet.anim(name);
        let frame = anim.frame_at(self.state_time);
        if anim.frames >= 3 && (frame == 0 || frame + 1 >= anim.frames) {
            return None;
        }
        // The hitbox sits in front of her body along the aim: long sideways,
        // tall up/down, square on diagonals.
        let aim = self.swing_aim;
        let (w, h) = if aim.y.abs() < 0.3 {
            (reach, height)
        } else if aim.x.abs() < 0.3 {
            (height + 6., reach)
        } else {
            (reach * 0.85, reach * 0.85)
        };
        let center = self.pos - vec2(0., 9.) + aim * reach * 0.55;
        let damage = if self.slash_level >= 3 { 2 } else { 1 };
        Some(Swing { hitbox: Rect::new(center.x - w / 2., center.y - h / 2., w, h), id: self.swing_id, push: aim, damage })
    }

    pub fn draw(&self, assets: &Assets) {
        let sheet = &assets.player;
        let shadow = vec2(12., 4.);
        draw_texture_ex(
            &assets.shadow,
            (self.pos.x - shadow.x / 2.).round(),
            (self.pos.y - shadow.y + 1.).round(),
            Color::new(1., 1., 1., 0.3),
            DrawTextureParams { dest_size: Some(shadow), ..Default::default() },
        );

        let anim = sheet.anim(self.state.anim());
        let frame = anim.frame_at(self.anim_time);
        // The source art has no walk cycle: hop on alternate frames instead.
        let hop = if self.state == State::Walk && frame % 2 == 1 { 1. } else { 0. };
        let tint = if !self.alive() {
            Color::new(0.45, 0.45, 0.5, 1.)
        } else if self.shield_flash > 0.15 {
            Color::new(0.6, 0.85, 1., 1.)
        } else if self.invulnerable > INVULNERABLE - 0.15 {
            Color::new(1., 0.4, 0.4, 1.)
        } else {
            WHITE
        };
        // Blink while invulnerable.
        if self.invulnerable > 0. && (self.invulnerable * 14.) as i32 % 2 == 1 {
            return;
        }
        sheet.draw(anim, frame, self.pos - vec2(0., hop), self.facing, tint);

        // Shield bubble: fainter and thinner with one hit left, pulsing gently.
        if self.shield > 0 && self.alive() {
            let strength = self.shield as f32 / SHIELD_HITS as f32;
            let pulse = 0.85 + 0.15 * (get_time() as f32 * 5.).sin();
            let center = self.pos - vec2(0., 11.);
            draw_ellipse(center.x, center.y, 14., 16., 0., Color::new(0.5, 0.8, 1., 0.12 * strength * pulse));
            draw_ellipse_lines(center.x, center.y, 14., 16., 0., 1. + strength, Color::new(0.6, 0.88, 1., 0.75 * strength * pulse));
        }

        // A glowing arc shows where the swing hits (the sprite only swings sideways):
        // white, blue from slash level 2, golden from level 3.
        if let Some(swing) = self.swing(sheet) {
            let color = match self.slash_level {
                1 => Color::new(1., 1., 1., 0.45),
                2 => Color::new(0.75, 0.9, 1., 0.5),
                _ => Color::new(1., 0.85, 0.45, 0.55),
            };
            let start = self.swing_aim.y.atan2(self.swing_aim.x).to_degrees() - 60.;
            let center = self.pos - vec2(0., 9.);
            let radius = swing.hitbox.w.max(swing.hitbox.h) * 0.9;
            draw_arc(center.x, center.y, 12, radius, start, 3., 120., color);
        }
    }
}
