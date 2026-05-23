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
    pub players_already_damaged: Vec<i32>,
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
            players_already_damaged: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, autres_joueurs: &mut Vec<Player>, local_player: Option<&mut Player>) {
        self.check_collisions(wallmap, hitboxes_murs, autres_joueurs, local_player);
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
        if !self.is_exploding {
            draw_circle(self.hitbox.x, self.hitbox.y, self.hitbox.r, YELLOW);
        }
    }

    pub fn check_collisions(&mut self, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, joueurs: &mut Vec<Player>, mut local_player: Option<&mut Player>) {
        if self.is_exploding {
            // Phase d'explosion : infliger les dégâts de zone
            for joueur in joueurs {
                if self.hitbox.overlaps_rect(&joueur.hitbox) {
                    if !self.players_already_damaged.contains(&joueur.id) && joueur.id != self.owner_id { 
                        self.players_already_damaged.push(joueur.id); // On l'ajoute
                        joueur.take_damage(self.degats);
                    }
                }
            }
            if let Some(ref mut lp) = local_player {
                if self.hitbox.overlaps_rect(&lp.hitbox) {
                    if !self.players_already_damaged.contains(&lp.id) && lp.id != self.owner_id {
                        self.players_already_damaged.push(lp.id);
                        lp.take_damage(self.degats);
                    }
                }
            }
        } else {
            // Phase de vol : détecter l'impact pour exploser
            let touche_map = wallmap.iter().any(|wall| self.hitbox.overlaps_rect(wall));
            let touche_mur = hitboxes_murs.iter().any(|mur| self.hitbox.overlaps_rect(mur));
            let touche_joueur = joueurs.iter().any(|j| j.id != self.owner_id && self.hitbox.overlaps_rect(&j.hitbox));
            let touche_local = local_player.as_ref().map_or(false, |lp| lp.id != self.owner_id && self.hitbox.overlaps_rect(&lp.hitbox));

            if touche_mur || touche_map || touche_joueur || touche_local {
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

#[derive(Clone, Debug)]
pub struct ExplosionParticle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub initial_size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct ExplosionParticleSystem {
    pub particles: Vec<ExplosionParticle>,
}

impl ExplosionParticleSystem {
    pub fn new() -> Self {
        Self { particles: Vec::new() }
    }

    /// Spawns a radial burst of active particles at the given location
    pub fn spawn_burst(&mut self, pos: Vec2) {
        use macroquad::rand::gen_range;
        use std::f32::consts::PI;

        let count = gen_range(16, 28);
        for _ in 0..count {
            let angle = gen_range(0.0, 2.0 * PI);
            let speed = gen_range(40.0, 90.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            // Retro color palette selector (60% fire, 40% ash/smoke)
            let color = if gen_range(0.0, 1.0) < 0.6 {
                // Fire colors: Red to Yellow transition
                match gen_range(0, 3) {
                    0 => RED,
                    1 => ORANGE,
                    _ => YELLOW,
                }
            } else {
                // Smoke/debris colors: Slate gray to light ash
                match gen_range(0, 2) {
                    0 => Color::new(0.3, 0.3, 0.3, 1.0),
                    _ => Color::new(0.6, 0.6, 0.6, 1.0),
                }
            };

            let initial_size = gen_range(1.0, 2.5);
            let lifetime = gen_range(0.3, 0.7);

            self.particles.push(ExplosionParticle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
            });
        }
    }

    pub fn update(&mut self, dt: f32, wallmap: &[Rect]) {
        let gravity = 80.0;
        let friction = 1.5;
        let elasticity = 0.4;
        let floor_y = 100.0;

        for p in &mut self.particles {
            // Apply gravity
            p.velocity.y += gravity * dt;
            // Apply air resistance/friction
            p.velocity *= 1.0 - (friction * dt);

            // Compute next tentative position
            let next_pos = p.position + p.velocity * dt;
            let mut bounced = false;

            // 1. Collision with the screen bottom floor (Y = 100.0)
            if p.velocity.y > 0.0 && next_pos.y >= floor_y {
                p.position.y = floor_y;
                p.position.x = next_pos.x;
                p.velocity.y = -elasticity * p.velocity.y;
                bounced = true;
            }

            // 2. Collision with top surfaces of wallmap platforms
            if !bounced && p.velocity.y > 0.0 {
                for wall in wallmap {
                    if next_pos.x >= wall.x && next_pos.x <= wall.x + wall.w {
                        if p.position.y <= wall.y && next_pos.y >= wall.y {
                            p.position.y = wall.y;
                            p.position.x = next_pos.x;
                            p.velocity.y = -elasticity * p.velocity.y;
                            bounced = true;
                            break;
                        }
                    }
                }
            }

            // If no bounce occurred, perform normal movement
            if !bounced {
                p.position = next_pos;
            }

            // Age particle
            p.lifetime -= dt;
        }

        // Retain only active particles
        self.particles.retain(|p| p.lifetime > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let progress = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut render_color = p.color;
            render_color.a = progress; // Fade out
            let size = p.initial_size * progress; // Shrink

            // Render as retro square pixels
            draw_rectangle(
                p.position.x - size / 2.0,
                p.position.y - size / 2.0,
                size,
                size,
                render_color,
            );
        }
    }
}
