use macroquad::prelude::*;

pub const VIRTUAL_HEIGHT: f32 = 100.0;
pub const RECOIL_FORCE: f32 = 80.0;
pub const NETWORK_TICK_RATE: f64 = 1.0 / 120.0;
pub const WALL_THICKNESS: f32 = 50.0;
pub const PARALLAX_FACTOR: f32 = 0.4;
pub const PLAYER_RESPAWN_X: f32 = 20.0;
pub const PLAYER_RESPAWN_Y: f32 = 20.0;
pub const PLAYER_MAX_PV: f32 = 100.0;
pub const JUMP_FORCE: f32 = 100.0;
pub const PROJECTILE_SPEED: f32 = 150.0;
pub const DT_CLAMP_MIN: f32 = 0.001;
pub const DT_CLAMP_MAX: f32 = 0.05;

pub const PLAYER_COLORS: [Color; 4] = [
    Color::new(0.95, 0.25, 0.25, 1.0),
    Color::new(0.25, 0.60, 0.95, 1.0),
    Color::new(0.95, 0.85, 0.15, 1.0),
    Color::new(0.25, 0.85, 0.25, 1.0),
];

pub struct CharacterStats {
    pub speed: f32,
    pub max_ammo: i32,
}

pub const CHARACTERS: [CharacterStats; 3] = [
    CharacterStats { speed: 40.0, max_ammo: 3 },
    CharacterStats { speed: 48.0, max_ammo: 2 },
    CharacterStats { speed: 34.0, max_ammo: 4 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_height() {
        assert_eq!(VIRTUAL_HEIGHT, 100.0);
    }

    #[test]
    fn test_recoil_force() {
        assert_eq!(RECOIL_FORCE, 80.0);
    }

    #[test]
    fn test_character_stats_asterion() {
        let stats = &CHARACTERS[0];
        assert_eq!(stats.speed, 40.0);
        assert_eq!(stats.max_ammo, 3);
    }

    #[test]
    fn test_character_stats_fox() {
        let stats = &CHARACTERS[1];
        assert_eq!(stats.speed, 48.0);
        assert_eq!(stats.max_ammo, 2);
    }

    #[test]
    fn test_character_stats_shadow() {
        let stats = &CHARACTERS[2];
        assert_eq!(stats.speed, 34.0);
        assert_eq!(stats.max_ammo, 4);
    }

    #[test]
    fn test_player_colors_count() {
        assert_eq!(PLAYER_COLORS.len(), 4);
    }
}
