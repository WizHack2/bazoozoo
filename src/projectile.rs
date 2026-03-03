use macroquad::prelude::*;
use crate::player::Player;

pub struct Projectile {
    pub owner_id: i32,
    pub hitbox: Circle,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32,
    pub is_exploding: bool,
    pub explosion_duration: f32,
    pub degats: f32,
    pub player_already_damaged: [bool; 4],
}

impl Projectile {
    pub fn new(owner_id: i32, start_x: f32, start_y: f32, target_x: f32, target_y: f32) -> Self {
        let dx = target_x - start_x;
        let dy = target_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();

        Self {
            owner_id,
            hitbox: Circle::new(start_x, start_y, 1.0), // Petit rayon en vol
            dir_x: if length > 0.0 { dx / length } else { 0.0 },
            dir_y: if length > 0.0 { dy / length } else { 0.0 },
            speed: 150.0,
            is_exploding: false,
            explosion_duration: 0.2,
            degats: 5.0,
            player_already_damaged: [false; 4],
        }
    }

    pub fn update(&mut self, dt: f32, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, autres_joueurs: &mut Vec<Player>) {
        self.check_collisions(wallmap, hitboxes_murs, autres_joueurs);
        if self.is_exploding {
            self.explosion_duration -= dt;
            // Le rayon change pendant l'explosion
            self.hitbox.r = self.explosion_duration * 50.0; 
        } else {
            self.hitbox.x += self.dir_x * self.speed * dt;
            self.hitbox.y += self.dir_y * self.speed * dt;
        }
    }

    pub fn draw(&self) {
        if self.is_exploding {
            if self.hitbox.r > 0.0 {
                draw_circle(self.hitbox.x, self.hitbox.y, self.hitbox.r, RED);
            }
        } else {
            draw_circle(self.hitbox.x, self.hitbox.y, self.hitbox.r, YELLOW);
        }
    }

    pub fn check_collisions(&mut self, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, joueurs: &mut Vec<Player>) {
        if self.is_exploding {
            // Phase d'explosion : infliger les dégâts de zone
            for joueur in joueurs {
                if self.hitbox.overlaps_rect(&joueur.hitbox) {
                    let id = joueur.id as usize;
                    if !self.player_already_damaged[id] && joueur.id != self.owner_id { // TODO La maniere dont on a géré ca dans le game.rs nécéssite qu'on vérifie autrement pour le knockback du projectile sur le player emetteur
                        self.player_already_damaged[id] = true;
                        joueur.take_damage(self.degats);
                    }
                }
            }
        } else {
            // Phase de vol : détecter l'impact pour exploser
            let touche_map = wallmap.iter().any(|wall| self.hitbox.overlaps_rect(wall));
            let touche_mur = hitboxes_murs.iter().any(|mur| self.hitbox.overlaps_rect(mur));
            let touche_joueur = joueurs.iter().any(|j| j.id != self.owner_id && self.hitbox.overlaps_rect(&j.hitbox));

            if touche_mur || touche_map || touche_joueur {
                self.explode();
            }
        }
    }

    fn explode(&mut self) {
        if !self.is_exploding {
            self.is_exploding = true;
            self.speed = 0.0; // Le projectile s'arrête
        }
    }

    pub fn is_dead(&self) -> bool {
        self.is_exploding && self.explosion_duration <= 0.0
    }
}
