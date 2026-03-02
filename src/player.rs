use macroquad::prelude::*;
use crate::projectile::Projectile;
use crate::{assets, boilerplate::animation::Animation};

pub struct Player {
    animation: Animation,
    hitbox:Rect,
    pub speed: f32,
    pub liste_projectiles: Vec<Projectile>

}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            speed: 10.0,
            hitbox: Rect::new(0.0,0.0,10.0,10.0),
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]),
            liste_projectiles: vec![]
         }

    }

    fn tirer_projectile(&mut self, camera: &Camera2D) {
        let mouse_pos = mouse_position();
        let world_mouse = camera.screen_to_world(vec2(mouse_pos.0, mouse_pos.1));
        
        // On centre le départ du tir (ajuste les + 5.0 selon la taille de ton sprite)
        let nouveau_tir = Projectile::new(self.hitbox.x + 5.0, self.hitbox.y + 5.0, world_mouse.x, world_mouse.y);
        
        self.liste_projectiles.push(nouveau_tir);
    }

    pub fn update(&mut self, camera: &Camera2D) {
        let dt = get_frame_time();

        // --- MOUVEMENTS ZQSD ---
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { self.hitbox.x += self.speed * dt; }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::Q) { self.hitbox.x -= self.speed * dt; }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::Z) { self.hitbox.y -= self.speed * dt; }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { self.hitbox.y += self.speed * dt; }

        // --- LOGIQUE DE TIR ---
        if is_mouse_button_pressed(MouseButton::Left) {
            self.tirer_projectile(camera);
        }

        // --- MISE À JOUR DES PROJECTILES ---
        for proj in &mut self.liste_projectiles {
            proj.update(dt);
        }
    }



    pub fn draw(&mut self) {
        self.animation.draw_current_frame(self.hitbox.x, self.hitbox.y, 10., 10., true);
        
        for proj in &mut self.liste_projectiles {
            proj.draw();
        }
    }
}
