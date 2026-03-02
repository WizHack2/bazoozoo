use macroquad::prelude::*;

use crate::{assets, boilerplate::animation::Animation};

pub struct Player {
    animation: Animation,

}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]) }

    }

    pub fn update(&mut self) {

    }

    pub fn draw(&mut self) {
        self.animation.draw_current_frame(0., 0., 10., 10., true);

    }
}
