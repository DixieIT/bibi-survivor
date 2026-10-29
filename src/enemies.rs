use crate::assets::Assets;
use crate::level::{Level, Swing};
use crate::player::Player;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

const HP: i32 = 3;
const SPEED: f32 = 34.;
const MAX_ALIVE: usize = 40;
const SPAWN_EVERY: f32 = 2.5;
const SPAWN_EVERY_MIN: f32 = 0.6;
const SPAWN_SPEEDUP: f32 = 0.01;
const SPAWN_DISTANCE: f32 = 300.;
const RISE_TIME: f32 = 0.6;
const CORPSE_TIME: f32 = 0.8;
const REACH: f32 = 10.;
const FEET: Vec2 = vec2(8., 4.);
const ATTACK_RANGE: f32 = 15.;
const STRIKE_RANGE: f32 = 20.;
const WINDUP: f32 = 0.45;
const STRIKE: f32 = 0.15;
const ATTACK_COOLDOWN: f32 = 0.9;
const ARCHER_CHANCE: f32 = 0.25;
const ARCHER_RANGE: f32 = 140.;
const ARCHER_KEEP_AWAY: f32 = 64.;
const AIM: f32 = 0.7;
const LOOSE: f32 = 0.25;
const SHOOT_COOLDOWN: f32 = 1.8;
const ARROW_SPEED: f32 = 170.;
const ARROW_LIFE: f32 = 2.5;
const ARROW_HEIGHT: f32 = 10.;
const HEART_CHANCE: f32 = 0.2;
const HEART_HEAL: i32 = 2;
const HEART_LIFE: f32 = 12.;
const WAVE_SPEED: f32 = 220.;
const WAVE_RANGE: f32 = 110.;

struct Heart {
    pos: Vec2,
    age: f32,
}

struct Wave {
    pos: Vec2,
    dir: Vec2,
    traveled: f32,
    hit: Vec<u32>,
}

pub struct Events {
    pub hits: Vec<Vec2>,
    pub player_hit: bool,
    pub healed: Option<Vec2>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Melee,
    Archer,
}

struct Arrow {
    pos: Vec2,
    vel: Vec2,
    life: f32,
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
        self.knockback = push * 140.;
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
    hearts: Vec<Heart>,
    waves: Vec<Wave>,
    next_id: u32,
    spawn_timer: f32,
    elapsed: f32,
    pub defeated: u32,
}

impl Horde {
    pub fn new() -> Self {
        Horde {
            skeletons: Vec::new(),
            arrows: Vec::new(),
            hearts: Vec::new(),
            waves: Vec::new(),
            next_id: 0,
            spawn_timer: 1.,
            elapsed: 0.,
            defeated: 0,
        }
    }

    pub fn launch_wave(&mut self, from: Vec2, dir: Vec2) {
        self.waves.push(Wave { pos: from + dir * 8., dir, traveled: 0., hit: Vec::new() });
    }

    fn defeat(&mut self, at: Vec2) {
        self.defeated += 1;
        if gen_range(0., 1.) < HEART_CHANCE {
            self.hearts.push(Heart { pos: at, age: 0. });
        }
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
        let mut kills = Vec::new();
        self.elapsed += dt;
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0. {
            self.spawn_timer = (SPAWN_EVERY - self.elapsed * SPAWN_SPEEDUP).max(SPAWN_EVERY_MIN);
            if self.alive() < MAX_ALIVE {
                self.spawn(level, player.pos);
            }
        }

        let die_time = assets.skeleton.anim("die").duration();
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
                    if s.take_hit(sw.damage, sw.push) {
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
            s.moving = false;
            if let Some(t) = &mut s.attack {
                *t += dt;
                let t = *t;
                match s.kind {
                    Kind::Melee => {
                        if t >= WINDUP && !s.struck {
                            s.struck = true;
                            if dist <= STRIKE_RANGE && player.hurt(s.pos) {
                                player_hit = true;
                            }
                        }
                        if t >= WINDUP + STRIKE {
                            s.attack = None;
                            s.cooldown = ATTACK_COOLDOWN;
                        }
                    }
                    Kind::Archer => {
                        if t >= AIM && !s.struck {
                            s.struck = true;
                            let dir = if dist > 0.1 { to_player / dist } else { vec2(s.facing, 0.) };
                            self.arrows.push(Arrow { pos: s.pos + dir * 6., vel: dir * ARROW_SPEED, life: ARROW_LIFE });
                        }
                        if t >= AIM + LOOSE {
                            s.attack = None;
                            s.cooldown = SHOOT_COOLDOWN * gen_range(0.8, 1.2);
                        }
                    }
                }
                continue;
            }

            match s.kind {
                Kind::Melee => {
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
                Kind::Archer => {
                    let sees = level.line_clear(s.pos - vec2(0., 2.), player.pos - vec2(0., 2.));
                    if sees && dist <= ARCHER_RANGE {
                        face_player(s);
                        if s.cooldown <= 0. && player.alive() {
                            s.attack = Some(0.);
                            s.struck = false;
                        } else if dist < ARCHER_KEEP_AWAY {
                            s.walk(-to_player / dist.max(0.1) * s.speed * 0.8 * dt, level);
                            s.moving = true;
                        }
                        continue;
                    }
                }
            }
            let target = if dist < 24. { player.pos } else { level.next_step(s.pos).unwrap_or(player.pos) };
            let dir = (target - s.pos).normalize_or_zero();
            if dir.x.abs() > 0.1 {
                s.facing = dir.x.signum();
            }
            s.walk(dir * s.speed * dt, level);
            s.moving = true;
        }

        for a in &mut self.arrows {
            a.life -= dt;
            a.pos += a.vel * dt;
            let flying = a.pos - vec2(0., ARROW_HEIGHT);
            let parried = swing.as_ref().is_some_and(|sw| sw.hitbox.overlaps(&Rect::new(flying.x - 3., flying.y - 3., 6., 6.)));
            if level.blocked(Rect::new(a.pos.x - 1., a.pos.y - 1., 2., 2.)) {
                a.life = 0.;
            } else if parried {
                a.life = 0.;
                hits.push(flying);
            } else if a.pos.distance(player.pos) < 7. && player.hurt(a.pos - a.vel) {
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
        Events { hits, player_hit, healed }
    }

    fn spawn(&mut self, level: &Level, player: Vec2) {
        let size = level.size();
        for _ in 0..40 {
            let p = vec2(gen_range(0., size.x), gen_range(0., size.y));
            if p.distance(player) > SPAWN_DISTANCE && level.can_spawn_at(p) {
                self.next_id += 1;
                self.skeletons.push(Skeleton {
                    id: self.next_id,
                    kind: if gen_range(0., 1.) < ARCHER_CHANCE { Kind::Archer } else { Kind::Melee },
                    pos: p,
                    moving: false,
                    knockback: Vec2::ZERO,
                    speed: SPEED * gen_range(0.85, 1.15),
                    facing: if player.x < p.x { -1. } else { 1. },
                    hp: HP,
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
                    (Kind::Melee, _) => "walk",
                    (Kind::Archer, None) => "archer_walk",
                    (Kind::Archer, Some(t)) if t < AIM => "archer_aim",
                    (Kind::Archer, Some(_)) => "archer_shoot",
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
        let (lean, tint) = match (s.kind, s.attack) {
            (Kind::Melee, Some(t)) if t < WINDUP => (-2. * t / WINDUP, Color::new(1., 0.85, 0.55, alpha)),
            (Kind::Melee, Some(_)) => (4., Color::new(1., 1., 1., alpha)),
            _ => (0., Color::new(1., 1., 1., alpha)),
        };
        let tint = if s.flash > 0. { Color::new(1., 0.4, 0.4, alpha) } else { tint };
        let sink = if s.dead_for.is_none() { (1. - s.age / RISE_TIME).max(0.) * 8. } else { 0. };
        let feet = s.pos + vec2(lean * s.facing, sink);
        sheet.draw(anim, frame, feet, s.facing, tint);

        if s.dead_for.is_none() && s.age >= RISE_TIME {
            let top = (s.pos.y - sheet.frame.y - 3.).round();
            let left = (s.pos.x - 6.).round();
            draw_rectangle(left - 1., top - 1., 14., 4., Color::from_rgba(30, 20, 26, 220));
            draw_rectangle(left, top, 12. * s.hp as f32 / HP as f32, 2., Color::from_rgba(214, 64, 58, 255));
        }
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
            let angle = a.vel.y.atan2(a.vel.x);
            sheet.draw_rotated(anim, 0, a.pos - vec2(0., ARROW_HEIGHT), angle, Color::new(1., 1., 1., alpha));
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
