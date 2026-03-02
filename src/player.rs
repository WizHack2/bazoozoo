use macroquad::prelude::*;

use crate::{assets, boilerplate::animation::Animation};

pub struct Player {
    animation: Animation,
    hitbox:Rect,
    pub speed: f32

}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            speed: 10.0,
            hitbox: Rect::new(0.0,0.0,10.0,10.0),
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]) }

    }

    pub fn update(&mut self) {
        // Temps écoulé depuis la dernière image (en secondes)
        let dt = get_frame_time();

        // Mouvement horizontal
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.hitbox.x += self.speed * dt;
        }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::Q) {
            self.hitbox.x -= self.speed * dt;
        }

        // Mouvement vertical
        // ATTENTION : D'après ton commentaire sur la caméra, ton Y diminue quand tu montes.
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::Z) {
            self.hitbox.y -= self.speed * dt;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.hitbox.y += self.speed * dt;
        }
    }



    pub fn draw(&mut self) {
        self.animation.draw_current_frame(self.hitbox.x, self.hitbox.y, 10., 10., true);

    }
}
