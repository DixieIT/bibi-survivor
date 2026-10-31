/// Dash cooldown at dash level 1, 2 and 3.
pub const DASH_COOLDOWN: [f32; 3] = [1.6, 1.2, 0.8];
/// Chance per life steal level that a blow that lands heals her 1 HP.
pub const LIFE_STEAL: f32 = 0.06;
/// Vortex stamina cost, lowered by each cost upgrade down to the minimum.
const VORTEX_COST: f32 = 50.;
const VORTEX_COST_CUT: f32 = 8.;
/// Extra max stamina per stamina level.
pub const STAMINA_BONUS: f32 = 20.;

/// A power-up offered at the end of a wave.
#[derive(Clone, Copy, PartialEq)]
pub enum Offer {
    /// Slash level 2: longer, taller swings. 3: double damage. 4: every swing throws a wave.
    Slash,
    /// A shield that takes hits instead of her, recharged every wave.
    Shield,
    Dash,
    LifeSteal,
    /// Unlocks the vortex; once she has it, it's upgraded through the next two.
    Vortex,
    VortexCost,
    VortexDamage,
    Stamina,
}

const ALL: [Offer; 8] = [Offer::Slash, Offer::Shield, Offer::Dash, Offer::LifeSteal, Offer::Vortex, Offer::VortexCost, Offer::VortexDamage, Offer::Stamina];

impl Offer {
    pub fn title(self) -> &'static str {
        match self {
            Offer::Slash => "Slash",
            Offer::Shield => "Shield",
            Offer::Dash => "Dash",
            Offer::LifeSteal => "Life steal",
            Offer::Vortex => "Sword vortex",
            Offer::VortexCost => "Vortex: cheaper",
            Offer::VortexDamage => "Vortex: stronger",
            Offer::Stamina => "Stamina",
        }
    }

    fn max(self) -> u32 {
        match self {
            Offer::Vortex => 1,
            Offer::VortexDamage => 2,
            Offer::Stamina => 5,
            _ => 3,
        }
    }
}

/// Power-up levels taken this run (0 = not taken).
#[derive(Default)]
pub struct Upgrades {
    pub slash: u32,
    pub shield: u32,
    pub dash: u32,
    pub life_steal: u32,
    pub vortex: u32,
    pub vortex_cost: u32,
    pub vortex_damage: u32,
    pub stamina: u32,
}

impl Upgrades {
    fn level(&self, offer: Offer) -> u32 {
        match offer {
            Offer::Slash => self.slash,
            Offer::Shield => self.shield,
            Offer::Dash => self.dash,
            Offer::LifeSteal => self.life_steal,
            Offer::Vortex => self.vortex,
            Offer::VortexCost => self.vortex_cost,
            Offer::VortexDamage => self.vortex_damage,
            Offer::Stamina => self.stamina,
        }
    }

    fn level_mut(&mut self, offer: Offer) -> &mut u32 {
        match offer {
            Offer::Slash => &mut self.slash,
            Offer::Shield => &mut self.shield,
            Offer::Dash => &mut self.dash,
            Offer::LifeSteal => &mut self.life_steal,
            Offer::Vortex => &mut self.vortex,
            Offer::VortexCost => &mut self.vortex_cost,
            Offer::VortexDamage => &mut self.vortex_damage,
            Offer::Stamina => &mut self.stamina,
        }
    }

    /// Up to three different power-ups she can still take, picked at random.
    pub fn offers(&self) -> Vec<Offer> {
        let has_vortex = self.vortex > 0;
        let mut open: Vec<Offer> = ALL
            .into_iter()
            .filter(|&o| self.level(o) < o.max())
            .filter(|&o| !matches!(o, Offer::VortexCost | Offer::VortexDamage) || has_vortex)
            .collect();
        crate::shuffle(&mut open);
        open.truncate(3);
        open
    }

    pub fn take(&mut self, offer: Offer) {
        *self.level_mut(offer) += 1;
    }

    /// The level `offer` would bring her to.
    pub fn next_level(&self, offer: Offer) -> u32 {
        self.level(offer) + 1
    }

    /// What taking `offer` would do, given what she has now.
    pub fn describe(&self, offer: Offer) -> String {
        let next = self.level(offer) + 1;
        match offer {
            Offer::Slash => match next {
                1 => "Longer, taller swings".to_string(),
                2 => "Swings deal double damage".to_string(),
                _ => "Every swing also throws a slash wave".to_string(),
            },
            Offer::Shield if next == 1 => format!("A shield takes {} hits for you, recharged every wave", Self::shield_hits_at(1)),
            Offer::Shield => format!("Shield takes {} hits (now {})", Self::shield_hits_at(next), self.shield_hits()),
            Offer::Dash => format!("E: dash, untouchable while dashing. Cooldown {}s", DASH_COOLDOWN[next as usize - 1]),
            Offer::LifeSteal => format!("Each hit has a {}% chance to heal 1 HP", (next as f32 * LIFE_STEAL * 100.).round()),
            Offer::Vortex => format!("Q: spin, hitting all around for {} damage", Self::vortex_damage_at(0)),
            Offer::VortexCost => format!("Vortex costs {} stamina (now {})", Self::vortex_cost_at(next), self.vortex_cost()),
            Offer::VortexDamage => format!("Vortex deals {} damage (now {})", Self::vortex_damage_at(next), self.vortex_damage()),
            Offer::Stamina => format!("Max stamina +{}", STAMINA_BONUS),
        }
    }

    /// Two hits at level 1, one more per level.
    fn shield_hits_at(level: u32) -> i32 {
        if level == 0 { 0 } else { 1 + level as i32 }
    }

    pub fn shield_hits(&self) -> i32 {
        Self::shield_hits_at(self.shield)
    }

    pub fn dash_cooldown(&self) -> f32 {
        DASH_COOLDOWN[(self.dash.max(1) - 1) as usize]
    }

    fn vortex_cost_at(level: u32) -> f32 {
        VORTEX_COST - VORTEX_COST_CUT * level as f32
    }

    pub fn vortex_cost(&self) -> f32 {
        Self::vortex_cost_at(self.vortex_cost)
    }

    /// Twice the sword's base damage, plus one per damage upgrade.
    fn vortex_damage_at(level: u32) -> i32 {
        2 + level as i32
    }

    pub fn vortex_damage(&self) -> i32 {
        Self::vortex_damage_at(self.vortex_damage)
    }
}
