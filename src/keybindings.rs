use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    Azerty,
    Qwerty,
}

#[allow(dead_code)]
pub struct KeyBindings {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub jump: [KeyCode; 3],
    pub reload: KeyCode,
    pub rocket_up: KeyCode,
    pub rocket_down: KeyCode,
    pub rocket_left: KeyCode,
    pub rocket_right: KeyCode,
    pub rocket_up_left: KeyCode,
    pub rocket_up_right: KeyCode,
    pub rocket_down_left: KeyCode,
    pub rocket_down_right: KeyCode,
}

impl KeyBindings {
    pub fn azerty() -> Self {
        Self {
            move_up: KeyCode::Z,
            move_down: KeyCode::S,
            move_left: KeyCode::Q,
            move_right: KeyCode::D,
            jump: [KeyCode::Z, KeyCode::Space, KeyCode::Up],
            reload: KeyCode::E,
            rocket_up: KeyCode::T,
            rocket_down: KeyCode::G,
            rocket_left: KeyCode::F,
            rocket_right: KeyCode::H,
            rocket_up_left: KeyCode::R,
            rocket_up_right: KeyCode::Y,
            rocket_down_left: KeyCode::V,
            rocket_down_right: KeyCode::N,
        }
    }

    pub fn qwerty() -> Self {
        Self {
            move_up: KeyCode::W,
            move_down: KeyCode::S,
            move_left: KeyCode::A,
            move_right: KeyCode::D,
            jump: [KeyCode::W, KeyCode::Space, KeyCode::Up],
            reload: KeyCode::E,
            rocket_up: KeyCode::T,
            rocket_down: KeyCode::G,
            rocket_left: KeyCode::F,
            rocket_right: KeyCode::H,
            rocket_up_left: KeyCode::R,
            rocket_up_right: KeyCode::Y,
            rocket_down_left: KeyCode::V,
            rocket_down_right: KeyCode::N,
        }
    }

    pub fn from_layout(layout: Layout) -> Self {
        match layout {
            Layout::Azerty => Self::azerty(),
            Layout::Qwerty => Self::qwerty(),
        }
    }
}
