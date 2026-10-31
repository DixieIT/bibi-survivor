use crate::assets::Assets;
use crate::level::{Level, Swing};
use crate::player::Player;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

const SPEED: f32 = 34.;
const MAX_ALIVE: usize = 40;
/// Time between risings: shorter in later waves, down to the minimum.
const SPAWN_EVERY: f32 = 1.3;
const SPAWN_EVERY_MIN: f32 = 0.35;
const SPAWN_SPEEDUP: f32 = 0.1;
const SPAWN_DISTANCE: f32 = 300.;
/// Every wave after the first, skeletons rise with this much more health, move this much
/// faster (up to the cap) and wait this much less between attacks (down to the floor).
const HP_PER_WAVE: f32 = 0.1;
const SPEED_PER_WAVE: f32 = 0.02;
const MAX_SPEEDUP: f32 = 1.3;
const HASTE_PER_WAVE: f32 = 0.03;
const MIN_COOLDOWN: f32 = 0.7;
const RISE_TIME: f32 = 0.6;
const CORPSE_TIME: f32 = 0.8;
const REACH: f32 = 10.;
const FEET: Vec2 = vec2(8., 4.);
const ATTACK_RANGE: f32 = 15.;
const STRIKE_RANGE: f32 = 20.;
const WINDUP: f32 = 0.45;
const STRIKE: f32 = 0.15;
const ATTACK_COOLDOWN: f32 = 0.9;
const ARCHER_RANGE: f32 = 140.;
const ARCHER_KEEP_AWAY: f32 = 64.;
const AIM: f32 = 0.7;
const LOOSE: f32 = 0.25;
const SHOOT_COOLDOWN: f32 = 1.8;
const ARROW_SPEED: f32 = 170.;
const ARROW_LIFE: f32 = 2.5;
const ARROW_HEIGHT: f32 = 10.;
/// From this wave archers loose a fan of this many arrows, this far apart (radians).
const VOLLEY_FROM_WAVE: u32 = 4;
const VOLLEY: usize = 3;
const VOLLEY_SPREAD: f32 = 0.22;
const MAGE_RANGE: f32 = 120.;
const MAGE_KEEP_AWAY: f32 = 72.;
const CAST: f32 = 0.9;
const CAST_RECOVER: f32 = 0.3;
const CAST_COOLDOWN: f32 = 3.2;
/// A mage's fireballs fly out in a ring, slower than arrows.
const FIREBALLS: usize = 8;
const FIREBALL_SPEED: f32 = 85.;
const FIREBALL_LIFE: f32 = 3.;
/// From this wave a mage's ring is bigger and, once out this far, stops and circles
/// the mage (following it) for a while.
const ORBIT_FROM_WAVE: u32 = 8;
const ORBIT_FIREBALLS: usize = 12;
const ORBIT_RADIUS: f32 = 44.;
/// Radians per second.
const ORBIT_SPIN: f32 = 2.2;
const ORBIT_TIME: f32 = 3.;
/// Shaman: calls fire down from the sky, from far away and without seeing her. Each
/// call marks one spot under her and the rest around the shaman, which burn after a warning.
const SHAMAN_RANGE: f32 = 180.;
const SHAMAN_KEEP_AWAY: f32 = 110.;
const CALL: f32 = 1.2;
const CALL_RECOVER: f32 = 0.4;
const CALL_COOLDOWN: f32 = 4.5;
const FLAMES: usize = 6;
const FLAME_REACH: f32 = 170.;
const FLAME_WARN: f32 = 0.9;
const FLAME_BURN: f32 = 0.5;
const FLAME_RADIUS: f32 = 13.;
const HEART_CHANCE: f32 = 0.06;
const HEART_HEAL: i32 = 2;
const HEART_LIFE: f32 = 12.;
const WAVE_SPEED: f32 = 220.;
const WAVE_RANGE: f32 = 110.;

struct Heart {
    pos: Vec2,
    age: f32,
}

/// The slash wave thrown by every swing at slash level 4.
struct SlashWave {
    pos: Vec2,
    dir: Vec2,
    traveled: f32,
    hit: Vec<u32>,
}

pub struct Events {
    pub hits: Vec<Vec2>,
    pub player_hit: bool,
    pub healed: Option<Vec2>,
    /// Blows that landed on skeletons this frame (for life steal).
    pub landed: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Melee,
    Archer,
    /// Slow, sturdy melee skeleton that hits twice as hard and barely flinches.
    Tank,
    /// Keeps its distance like an archer and casts rings of fireballs.
    Mage,
    /// Keeps further back and calls fire down from the sky in several places.
    Shaman,
}

impl Kind {
    fn hp(self) -> i32 {
        match self {
            Kind::Melee | Kind::Archer => 3,
            Kind::Tank => 9,
            Kind::Mage => 4,
            Kind::Shaman => 5,
        }
    }

    fn speed(self) -> f32 {
        match self {
            Kind::Tank => SPEED * 0.7,
            Kind::Mage => SPEED * 0.9,
            Kind::Shaman => SPEED * 0.85,
            _ => SPEED,
        }
    }

    fn damage(self) -> i32 {
        if self == Kind::Tank { 2 } else { 1 }
    }

    /// How far a hit throws it back.
    fn knockback(self) -> f32 {
        if self == Kind::Tank { 40. } else { 140. }
    }

    /// Shooting range and the distance it backs off to, for ranged kinds.
    fn ranged(self) -> Option<(f32, f32)> {
        match self {
            Kind::Archer => Some((ARCHER_RANGE, ARCHER_KEEP_AWAY)),
            Kind::Mage => Some((MAGE_RANGE, MAGE_KEEP_AWAY)),
            Kind::Shaman => Some((SHAMAN_RANGE, SHAMAN_KEEP_AWAY)),
            _ => None,
        }
    }

    /// They all share the skeleton sprite (tanks with a helmet, mages with a book,
    /// shamans with a hood), tinted rust red, violet and ember.
    fn tint(self) -> Color {
        match self {
            Kind::Tank => Color::new(0.92, 0.66, 0.58, 1.),
            Kind::Mage => Color::new(0.82, 0.7, 1., 1.),
            Kind::Shaman => Color::new(1., 0.86, 0.74, 1.),
            _ => WHITE,
        }
    }
}

/// Who rises in wave `n`, before shuffling.
fn roster(n: u32) -> Vec<Kind> {
    let (melee, archers, tanks, mages) = match n {
        1 => (12, 0, 0, 0),
        2 => (14, 4, 0, 0),
        3 => (14, 8, 0, 0),
        4 => (16, 8, 0, 0),
        // From wave 5 tanks and mages join; every wave brings more of everyone.
        _ => (16 + 2 * (n - 5), 8 + (n - 5), 3 + (n - 5), 2 + (n - 5)),
    };
    // Shamans from wave 7, one more every other wave.
    let shamans = if n >= 7 { 1 + (n - 7) / 2 } else { 0 };
    let mut kinds = Vec::new();
    let groups = [(Kind::Melee, melee), (Kind::Archer, archers), (Kind::Tank, tanks), (Kind::Mage, mages), (Kind::Shaman, shamans)];
    for (kind, count) in groups {
        kinds.extend(std::iter::repeat_n(kind, count as usize));
    }
    crate::shuffle(&mut kinds);
    kinds
}

/// A spot a shaman set on fire: a warning mark on the ground, then flames.
struct Flame {
    pos: Vec2,
    age: f32,
}

struct Arrow {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    /// A mage's fireball rather than an arrow.
    fire: bool,
    /// For an orbiting fireball: the mage it will circle, and its angle around it once there.
    caster: Option<u32>,
    orbit: Option<f32>,
}

struct Skeleton {
    id: u32,
    kind: Kind,
    pos: Vec2,
    moving: bool,
    knockback: Vec2,
    speed: f32,
    facing: f32,
    hp: i32,
    max_hp: i32,
    flash: f32,
    last_swing: u32,
    age: f32,
    anim_time: f32,
    dead_for: Option<f32>,
    attack: Option<f32>,
    struck: bool,
    cooldown: f32,
}

impl Skeleton {
    fn body(&self) -> Rect {
        Rect::new(self.pos.x - 6., self.pos.y - 18., 12., 18.)
    }

    fn feet_box(pos: Vec2) -> Rect {
        Rect::new(pos.x - FEET.x / 2., pos.y - FEET.y, FEET.x, FEET.y)
    }

    fn take_hit(&mut self, damage: i32, push: Vec2) -> bool {
        self.hp -= damage;
        self.flash = 0.12;
        self.knockback = push * self.kind.knockback();
        self.attack = None;
        self.cooldown = self.cooldown.max(0.5);
        if self.hp <= 0 {
            self.dead_for = Some(0.);
            self.anim_time = 0.;
            return true;
        }
        false
    }

    fn walk(&mut self, step: Vec2, level: &Level) {
        for axis in [vec2(step.x, 0.), vec2(0., step.y)] {
            if !level.blocked(Self::feet_box(self.pos + axis)) {
                self.pos += axis;
            }
        }
    }
}

pub struct Horde {
    skeletons: Vec<Skeleton>,
    arrows: Vec<Arrow>,
    flames: Vec<Flame>,
    hearts: Vec<Heart>,
    waves: Vec<SlashWave>,
    next_id: u32,
    spawn_timer: f32,
    /// Skeletons of the current wave still to rise.
    queue: Vec<Kind>,
    /// The current wave, from 1.
    pub wave: u32,
    pub defeated: u32,
}

impl Horde {
    /// A fresh horde, starting at wave `first`.
    pub fn new(first: u32) -> Self {
        let mut horde = Horde {
            skeletons: Vec::new(),
            arrows: Vec::new(),
            flames: Vec::new(),
            hearts: Vec::new(),
            waves: Vec::new(),
            next_id: 0,
            spawn_timer: 0.,
            queue: Vec::new(),
            wave: 0,
            defeated: 0,
        };
        horde.start_wave(first);
        horde
    }

    pub fn start_wave(&mut self, n: u32) {
        self.wave = n;
        self.queue = roster(n);
        self.spawn_timer = 1.;
    }

    /// The whole wave has risen and fallen.
    pub fn wave_cleared(&self) -> bool {
        self.queue.is_empty() && self.alive() == 0
    }

    /// Skeletons of this wave still standing or yet to rise.
    pub fn remaining(&self) -> usize {
        self.queue.len() + self.alive()
    }

    pub fn launch_wave(&mut self, from: Vec2, dir: Vec2) {
        self.waves.push(SlashWave { pos: from + dir * 8., dir, traveled: 0., hit: Vec::new() });
    }

    fn defeat(&mut self, at: Vec2) {
        self.defeated += 1;
        if gen_range(0., 1.) < HEART_CHANCE {
            self.hearts.push(Heart { pos: at, age: 0. });
        }
    }

    /// How much stronger this wave is than the first: 0 in wave 1.
    fn later(&self) -> f32 {
        self.wave.saturating_sub(1) as f32
    }

    pub fn alive(&self) -> usize {
        self.skeletons.iter().filter(|s| s.dead_for.is_none()).count()
    }

    pub fn update(
        &mut self,
        dt: f32,
        level: &Level,
        player: &mut Player,
        swing: Option<Swing>,
        assets: &Assets,
    ) -> Events {
        let mut player_hit = false;
        let mut landed = 0;
        let mut kills = Vec::new();
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0. && !self.queue.is_empty() {
            self.spawn_timer = (SPAWN_EVERY - self.wave as f32 * SPAWN_SPEEDUP).max(SPAWN_EVERY_MIN);
            if self.alive() < MAX_ALIVE {
                self.spawn(level, player.pos);
            }
        }

        let die_time = assets.skeleton.anim("die").duration();
        let volley = if self.wave >= VOLLEY_FROM_WAVE { VOLLEY } else { 1 };
        let haste = (1. - HASTE_PER_WAVE * self.later()).max(MIN_COOLDOWN);
        let orbiting = self.wave >= ORBIT_FROM_WAVE;
        let mut hits = Vec::new();
        for s in &mut self.skeletons {
            s.age += dt;
            s.anim_time += dt;
            s.flash = (s.flash - dt).max(0.);
            s.cooldown -= dt;
            if let Some(t) = &mut s.dead_for {
                *t += dt;
                continue;
            }

            if let Some(sw) = &swing {
                if s.last_swing != sw.id && sw.hitbox.overlaps(&s.body()) {
                    s.last_swing = sw.id;
                    hits.push(s.body().center());
                    landed += 1;
                    let push = if sw.radial { (s.pos - sw.push).normalize_or(vec2(s.facing, 0.)) } else { sw.push };
                    if s.take_hit(sw.damage, push) {
                        kills.push(s.pos);
                        continue;
                    }
                }
            }

            if s.knockback.length() > 1. {
                let k = s.knockback * dt;
                s.walk(k, level);
                s.knockback *= (1. - 10. * dt).max(0.);
                continue;
            }
            if s.age < RISE_TIME {
                continue;
            }

            let to_player = player.pos - s.pos;
            let dist = to_player.length();
            let face_player = |s: &mut Skeleton| {
                if to_player.x.abs() > 0.5 {
                    s.facing = to_player.x.signum();
                }
            };
            let toward = if dist > 0.1 { to_player / dist } else { vec2(s.facing, 0.) };
            s.moving = false;
            if let Some(t) = &mut s.attack {
                *t += dt;
                let t = *t;
                match s.kind {
                    Kind::Melee | Kind::Tank => {
                        if t >= WINDUP && !s.struck {
                            s.struck = true;
                            if dist <= STRIKE_RANGE && player.hurt(s.pos, s.kind.damage()) {
                                player_hit = true;
                            }
                        }
                        if t >= WINDUP + STRIKE {
                            s.attack = None;
                            s.cooldown = ATTACK_COOLDOWN * haste;
                        }
                    }
                    Kind::Archer => {
                        if t >= AIM && !s.struck {
                            s.struck = true;
                            for i in 0..volley {
                                let turn = (i as f32 - (volley - 1) as f32 / 2.) * VOLLEY_SPREAD;
                                let dir = Vec2::from_angle(turn).rotate(toward);
                                self.arrows.push(Arrow { pos: s.pos + dir * 6., vel: dir * ARROW_SPEED, life: ARROW_LIFE, fire: false, caster: None, orbit: None });
                            }
                        }
                        if t >= AIM + LOOSE {
                            s.attack = None;
                            s.cooldown = SHOOT_COOLDOWN * haste * gen_range(0.8, 1.2);
                        }
                    }
                    Kind::Mage => {
                        if t >= CAST && !s.struck {
                            s.struck = true;
                            // The ring starts aimed at her, so one fireball always comes straight on.
                            let (count, caster) = if orbiting { (ORBIT_FIREBALLS, Some(s.id)) } else { (FIREBALLS, None) };
                            for i in 0..count {
                                let dir = Vec2::from_angle(i as f32 * std::f32::consts::TAU / count as f32).rotate(toward);
                                self.arrows.push(Arrow {
                                    pos: s.pos + dir * 6.,
                                    vel: dir * FIREBALL_SPEED,
                                    life: FIREBALL_LIFE,
                                    fire: true,
                                    caster,
                                    orbit: None,
                                });
                            }
                        }
                        if t >= CAST + CAST_RECOVER {
                            s.attack = None;
                            s.cooldown = CAST_COOLDOWN * haste * gen_range(0.8, 1.2);
                        }
                    }
                    Kind::Shaman => {
                        if t >= CALL && !s.struck {
                            s.struck = true;
                            self.flames.push(Flame { pos: player.pos + vec2(gen_range(-8., 8.), gen_range(-6., 6.)), age: 0. });
                            // The others spread around the shaman, one per slice of the circle.
                            let others = FLAMES - 1;
                            for i in 0..others {
                                let angle = (i as f32 + gen_range(0.2, 0.8)) * std::f32::consts::TAU / others as f32;
                                let pos = s.pos + Vec2::from_angle(angle) * gen_range(50., FLAME_REACH);
                                self.flames.push(Flame { pos, age: 0. });
                            }
                        }
                        if t >= CALL + CALL_RECOVER {
                            s.attack = None;
                            s.cooldown = CALL_COOLDOWN * haste * gen_range(0.8, 1.2);
                        }
                    }
                }
                continue;
            }

            match s.kind.ranged() {
                None => {
                    if dist <= ATTACK_RANGE && s.cooldown <= 0. && player.alive() {
                        s.attack = Some(0.);
                        s.struck = false;
                        face_player(s);
                        continue;
                    }
                    if dist <= REACH {
                        continue;
                    }
                }
                Some((range, keep_away)) => {
                    // Fireballs fly through walls, so mages don't need to see her.
                    let sees = matches!(s.kind, Kind::Mage | Kind::Shaman) || level.line_clear(s.pos - vec2(0., 2.), player.pos - vec2(0., 2.));
                    if sees && dist <= range {
                        face_player(s);
                        if s.cooldown <= 0. && player.alive() {
                            s.attack = Some(0.);
                            s.struck = false;
                        } else if dist < keep_away {
                            s.walk(-toward * s.speed * 0.8 * dt, level);
                            s.moving = true;
                        }
                        continue;
                    }
                }
            }
            let close = dist < 24. && level.line_clear(s.pos - vec2(0., 2.), player.pos - vec2(0., 2.));
            let target = if close { player.pos } else { level.next_step(s.pos).unwrap_or(player.pos) };
            let dir = (target - s.pos).normalize_or_zero();
            if dir.x.abs() > 0.1 {
                s.facing = dir.x.signum();
            }
            s.walk(dir * s.speed * dt, level);
            s.moving = true;
        }

        let standing: Vec<(u32, Vec2)> = self.skeletons.iter().filter(|s| s.dead_for.is_none()).map(|s| (s.id, s.pos)).collect();
        for a in &mut self.arrows {
            a.life -= dt;
            let center = a.caster.and_then(|id| standing.iter().find(|m| m.0 == id)).map(|m| m.1);
            match (center, a.orbit) {
                // Its mage fell: fly off along the circle's tangent.
                (None, _) => a.caster = None,
                (Some(c), None) if a.pos.distance(c) >= ORBIT_RADIUS => {
                    a.orbit = Some((a.pos - c).to_angle());
                    a.life = ORBIT_TIME;
                }
                (Some(c), Some(angle)) if dt > 0. => {
                    let angle = angle + ORBIT_SPIN * dt;
                    a.orbit = Some(angle);
                    a.vel = (c + Vec2::from_angle(angle) * ORBIT_RADIUS - a.pos) / dt;
                }
                _ => {}
            }
            a.pos += a.vel * dt;
            let flying = a.pos - vec2(0., ARROW_HEIGHT);
            let parried = swing.as_ref().is_some_and(|sw| sw.hitbox.overlaps(&Rect::new(flying.x - 3., flying.y - 3., 6., 6.)));
            if !a.fire && level.blocked(Rect::new(a.pos.x - 1., a.pos.y - 1., 2., 2.)) {
                a.life = 0.;
            } else if parried {
                a.life = 0.;
                hits.push(flying);
            } else if a.pos.distance(player.pos) < 7. && player.hurt(a.pos - a.vel, 1) {
                a.life = 0.;
                player_hit = true;
            }
        }
        self.arrows.retain(|a| a.life > 0.);

        for w in &mut self.waves {
            let step = WAVE_SPEED * dt;
            w.pos += w.dir * step;
            w.traveled += step;
            let cut = Rect::new(w.pos.x - 10., w.pos.y - 22., 20., 24.);
            if level.blocked(Rect::new(w.pos.x - 1., w.pos.y - 2., 2., 2.)) {
                w.traveled = WAVE_RANGE;
            }
            for s in self.skeletons.iter_mut().filter(|s| s.dead_for.is_none() && s.age >= RISE_TIME) {
                if !w.hit.contains(&s.id) && cut.overlaps(&s.body()) {
                    w.hit.push(s.id);
                    hits.push(s.body().center());
                    landed += 1;
                    if s.take_hit(1, w.dir) {
                        kills.push(s.pos);
                    }
                }
            }
            for a in &mut self.arrows {
                if cut.contains(a.pos - vec2(0., ARROW_HEIGHT)) {
                    a.life = 0.;
                    hits.push(a.pos - vec2(0., ARROW_HEIGHT));
                }
            }
        }
        self.waves.retain(|w| w.traveled < WAVE_RANGE);
        self.arrows.retain(|a| a.life > 0.);

        for f in &mut self.flames {
            f.age += dt;
            let burning = f.age >= FLAME_WARN;
            if burning && f.pos.distance(player.pos) < FLAME_RADIUS && player.hurt(f.pos, 1) {
                player_hit = true;
            }
        }
        self.flames.retain(|f| f.age < FLAME_WARN + FLAME_BURN);

        for at in kills {
            self.defeat(at);
        }
        let mut healed = None;
        for h in &mut self.hearts {
            h.age += dt;
            if h.pos.distance(player.pos) < 12. && player.heal(HEART_HEAL) {
                healed = Some(h.pos);
                h.age = HEART_LIFE;
            }
        }
        self.hearts.retain(|h| h.age < HEART_LIFE);

        self.separate(level);
        self.skeletons.retain(|s| s.dead_for.map_or(true, |t| t < die_time + CORPSE_TIME));
        Events { hits, player_hit, healed, landed }
    }

    /// Raises the next skeleton of the wave somewhere out of sight (or retries next time).
    fn spawn(&mut self, level: &Level, player: Vec2) {
        let size = level.size();
        for _ in 0..40 {
            let p = vec2(gen_range(0., size.x), gen_range(0., size.y));
            if p.distance(player) > SPAWN_DISTANCE && level.can_spawn_at(p) {
                let Some(kind) = self.queue.pop() else { return };
                let hp = (kind.hp() as f32 * (1. + HP_PER_WAVE * self.later())).round() as i32;
                let speedup = (1. + SPEED_PER_WAVE * self.later()).min(MAX_SPEEDUP);
                self.next_id += 1;
                self.skeletons.push(Skeleton {
                    id: self.next_id,
                    kind,
                    pos: p,
                    moving: false,
                    knockback: Vec2::ZERO,
                    speed: kind.speed() * speedup * gen_range(0.85, 1.15),
                    facing: if player.x < p.x { -1. } else { 1. },
                    hp,
                    max_hp: hp,
                    flash: 0.,
                    last_swing: 0,
                    age: 0.,
                    anim_time: gen_range(0., 1.),
                    dead_for: None,
                    attack: None,
                    struck: false,
                    cooldown: 0.,
                });
                return;
            }
        }
    }

    fn separate(&mut self, level: &Level) {
        const MIN_GAP: f32 = 10.;
        for i in 0..self.skeletons.len() {
            for j in i + 1..self.skeletons.len() {
                if self.skeletons[i].dead_for.is_some() || self.skeletons[j].dead_for.is_some() {
                    continue;
                }
                let delta = self.skeletons[j].pos - self.skeletons[i].pos;
                let d = delta.length();
                if d < MIN_GAP {
                    let push = if d > 0.01 { delta / d } else { vec2(1., 0.) } * (MIN_GAP - d) * 0.5;
                    self.skeletons[i].walk(-push, level);
                    self.skeletons[j].walk(push, level);
                }
            }
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (Vec2, usize)> + '_ {
        self.skeletons.iter().enumerate().map(|(i, s)| (s.pos, i))
    }

    pub fn draw_one(&self, i: usize, assets: &Assets) {
        let s = &self.skeletons[i];
        let sheet = &assets.skeleton;
        let (anim, alpha) = match s.dead_for {
            Some(t) => {
                let anim = sheet.anim("die");
                (anim, 1. - ((t - anim.duration()) / CORPSE_TIME).clamp(0., 1.))
            }
            None => {
                let name = match (s.kind, s.attack) {
                    (Kind::Archer, None) => "archer_walk",
                    (Kind::Archer, Some(t)) if t < AIM => "archer_aim",
                    (Kind::Archer, Some(_)) => "archer_shoot",
                    (Kind::Tank, _) => "tank_walk",
                    (Kind::Mage, None) => "mage_walk",
                    (Kind::Mage, Some(_)) => "mage_cast",
                    (Kind::Shaman, None) => "shaman_walk",
                    (Kind::Shaman, Some(_)) => "shaman_call",
                    (Kind::Melee, _) => "walk",
                };
                (sheet.anim(name), (s.age / RISE_TIME).min(1.))
            }
        };
        if s.dead_for.is_none() {
            draw_texture_ex(
                &assets.shadow,
                (s.pos.x - 5.).round(),
                (s.pos.y - 2.).round(),
                Color::new(1., 1., 1., 0.3 * alpha),
                DrawTextureParams { dest_size: Some(vec2(10., 3.)), ..Default::default() },
            );
        }
        let standing = s.dead_for.is_none() && (s.age < RISE_TIME || !s.moving);
        let frame = if standing && s.attack.is_none() { 1.min(anim.frames - 1) } else { anim.frame_at(s.anim_time) };
        let (lean, mut tint) = match (s.kind, s.attack) {
            (Kind::Melee | Kind::Tank, Some(t)) if t < WINDUP => (-2. * t / WINDUP, Color::new(1., 0.85, 0.55, 1.)),
            (Kind::Melee | Kind::Tank, Some(_)) => (4., s.kind.tint()),
            // A mage glows brighter as the spell gathers.
            (Kind::Mage, Some(t)) if t < CAST => (0., Color::new(1., 0.55 + 0.4 * t / CAST, 0.3, 1.)),
            // A shaman flickers like embers while calling.
            (Kind::Shaman, Some(t)) if t < CALL => (0., Color::new(1., 0.65 + 0.2 * (t * 18.).sin(), 0.4, 1.)),
            _ => (0., s.kind.tint()),
        };
        if s.flash > 0. {
            tint = Color::new(1., 0.4, 0.4, 1.);
        }
        tint.a = alpha;
        let sink = if s.dead_for.is_none() { (1. - s.age / RISE_TIME).max(0.) * 8. } else { 0. };
        let feet = s.pos + vec2(lean * s.facing, sink);
        sheet.draw(anim, frame, feet, s.facing, tint);

        if s.dead_for.is_none() && s.age >= RISE_TIME {
            let top = (s.pos.y - sheet.frame.y - 3.).round();
            let left = (s.pos.x - 6.).round();
            draw_rectangle(left - 1., top - 1., 14., 4., Color::from_rgba(30, 20, 26, 220));
            draw_rectangle(left, top, 12. * s.hp as f32 / s.max_hp as f32, 2., Color::from_rgba(214, 64, 58, 255));
        }
    }

    /// Warning marks where fire is about to fall, under everyone.
    pub fn draw_flame_marks(&self) {
        for f in self.flames.iter().filter(|f| f.age < FLAME_WARN) {
            let t = f.age / FLAME_WARN;
            let blink = if (f.age * 10.) as i32 % 2 == 0 { 1. } else { 0.7 };
            draw_ellipse(f.pos.x, f.pos.y, FLAME_RADIUS * t, FLAME_RADIUS * t * 0.5, 0., Color::new(0.9, 0.3, 0.1, 0.35 * blink));
            draw_ellipse_lines(f.pos.x, f.pos.y, FLAME_RADIUS, FLAME_RADIUS * 0.5, 0., 1., Color::new(1., 0.45, 0.15, 0.8 * blink));
        }
    }

    /// Fireballs and burning spots light up the dark: (position, radius).
    pub fn glows(&self) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        let fireballs = self.arrows.iter().filter(|a| a.fire).map(|a| (a.pos - vec2(0., ARROW_HEIGHT), 34.));
        let flames = self.flames.iter().filter(|f| f.age >= FLAME_WARN).map(|f| (f.pos - vec2(0., 8.), 52.));
        fireballs.chain(flames)
    }

    pub fn draw_hearts(&self, assets: &Assets) {
        let sheet = &assets.skeleton;
        let anim = sheet.anim("heart");
        for h in &self.hearts {
            if h.age > HEART_LIFE - 3. && (h.age * 8.) as i32 % 2 == 0 {
                continue;
            }
            draw_texture_ex(
                &assets.shadow,
                (h.pos.x - 4.).round(),
                (h.pos.y - 2.).round(),
                Color::new(1., 1., 1., 0.3),
                DrawTextureParams { dest_size: Some(vec2(8., 3.)), ..Default::default() },
            );
            let bob = ((h.age * 4.).sin() * 1.5).round();
            sheet.draw(anim, 0, h.pos - vec2(0., 2. + bob), 1., WHITE);
        }
    }

    pub fn draw_arrows(&self, assets: &Assets) {
        let sheet = &assets.skeleton;
        let anim = sheet.anim("arrow");
        for a in &self.arrows {
            let alpha = (a.life / 0.2).min(1.);
            draw_texture_ex(
                &assets.shadow,
                (a.pos.x - 3.).round(),
                (a.pos.y - 1.).round(),
                Color::new(1., 1., 1., 0.2 * alpha),
                DrawTextureParams { dest_size: Some(vec2(6., 2.)), ..Default::default() },
            );
            let flying = a.pos - vec2(0., ARROW_HEIGHT);
            if a.fire {
                let flicker = 0.5 + 0.5 * (a.life * 30.).sin();
                draw_circle(flying.x.round(), flying.y.round(), 3.5 + flicker * 0.5, Color::new(0.95, 0.35, 0.1, 0.85 * alpha));
                draw_circle(flying.x.round(), flying.y.round(), 1.8, Color::new(1., 0.9, 0.5, alpha));
                continue;
            }
            let angle = a.vel.y.atan2(a.vel.x);
            sheet.draw_rotated(anim, 0, flying, angle, Color::new(1., 1., 1., alpha));
        }
        // Burning spots: a scorched patch and a flickering column of fire.
        for f in self.flames.iter().filter(|f| f.age >= FLAME_WARN) {
            let t = (f.age - FLAME_WARN) / FLAME_BURN;
            let fade = 1. - t * t;
            draw_ellipse(f.pos.x, f.pos.y, FLAME_RADIUS, FLAME_RADIUS * 0.5, 0., Color::new(0.35, 0.08, 0.04, 0.5 * fade));
            let height = 26. * (t * 6.).min(1.);
            for k in 0..6 {
                let up = k as f32 / 5.;
                let sway = ((f.age * 25. + k as f32 * 1.7).sin() * 2.).round();
                let r = (FLAME_RADIUS * 0.8 * (1. - up * 0.75)).max(1.5);
                let color = Color::new(1., 0.25 + 0.65 * up, 0.08 + 0.3 * up, 0.85 * fade);
                draw_circle((f.pos.x + sway).round(), (f.pos.y - height * up).round(), r, color);
            }
        }
        for w in &self.waves {
            let alpha = 0.8 * (1. - w.traveled / WAVE_RANGE);
            let start = w.dir.y.atan2(w.dir.x).to_degrees() - 70.;
            let center = w.pos - w.dir * 6. - vec2(0., 10.);
            draw_arc(center.x, center.y, 12, 11., start, 3., 140., Color::new(1., 0.9, 0.55, alpha));
            let inner = center - w.dir * 3.;
            draw_arc(inner.x, inner.y, 12, 9., start, 2., 140., Color::new(1., 1., 1., alpha * 0.6));
        }
    }
}
