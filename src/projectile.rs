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
    pub is_mega: bool,
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
            degats: 10.0,
            players_already_damaged: Vec::new(),
            is_mega: false,
        }
    }

    pub fn new_mega(owner_id: i32, start_x: f32, start_y: f32) -> Self {
        let mut p = Self::new(owner_id, start_x, start_y, start_x, start_y);
        p.is_mega = true;
        p.degats = 100.0; // Mort instantanée !
        p.is_exploding = true;
        p.speed = 0.0;
        p.explosion_duration = 1.0; // Dure 1 seconde
        p
    }

    pub fn update(&mut self, dt: f32, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, autres_joueurs: &mut Vec<Player>, mut local_player: Option<&mut Player>) {
        let kills = self.check_collisions(
            wallmap,
            hitboxes_murs,
            autres_joueurs,
            match local_player {
                Some(ref mut p) => Some(*p),
                None => None,
            }
        );
        if kills > 0 {
            let o_id = self.owner_id;
            if let Some(ref mut lp) = local_player {
                if lp.id == o_id {
                    lp.score += kills;
                }
            }
            for p in autres_joueurs.iter_mut() {
                if p.id == o_id {
                    p.score += kills;
                    break;
                }
            }
        }
        if self.is_exploding {
            self.explosion_duration -= dt;
            // Le rayon change pendant l'explosion
            if self.is_mega {
                self.hitbox.r = (1.0 - self.explosion_duration) * 800.0; // Couvre toute la map
            } else {
                self.hitbox.r = self.explosion_duration * 50.0; 
            }
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

    pub fn check_collisions(&mut self, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, joueurs: &mut Vec<Player>, mut local_player: Option<&mut Player>) -> i32 {
        let mut kills = 0;
        if self.is_exploding {
            // Phase d'explosion : infliger les dégâts de zone
            for joueur in joueurs.iter_mut() {
                if self.hitbox.overlaps_rect(&joueur.hitbox) {
                    if !self.players_already_damaged.contains(&joueur.id) && joueur.id != self.owner_id { 
                        self.players_already_damaged.push(joueur.id);
                        let was_alive = joueur.pv > 0.0;
                        joueur.take_damage(self.degats);
                        if was_alive && joueur.pv <= 0.0 {
                            kills += 1;
                        }
                    }
                }
            }
            if let Some(ref mut lp) = local_player {
                if self.hitbox.overlaps_rect(&lp.hitbox) {
                    if !self.players_already_damaged.contains(&lp.id) && lp.id != self.owner_id {
                        self.players_already_damaged.push(lp.id);
                        let was_alive = lp.pv > 0.0;
                        lp.take_damage(self.degats);
                        if was_alive && lp.pv <= 0.0 {
                            kills += 1;
                        }
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
        kills
    }

    pub fn explode(&mut self) {
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
    pub trail: Vec<Vec2>, // Motion history to render trailing sparks
    pub is_smoke: bool,   // Distinguishes sparks from expanding, floating smoke
}

pub struct ExplosionParticleSystem {
    pub particles: Vec<ExplosionParticle>,
}

impl ExplosionParticleSystem {
    pub fn new() -> Self {
        Self { particles: Vec::new() }
    }

    /// Spawns a radial burst of active sparks and floating smoke puffs
    pub fn spawn_burst(&mut self, pos: Vec2) {
        use macroquad::rand::gen_range;
        use std::f32::consts::PI;

        // 1. Spawning sparks (110 to 160 tiny glowing embers with trails)
        let count = gen_range(110, 160);
        for _ in 0..count {
            let angle = gen_range(0.0, 2.0 * PI);
            let speed = if gen_range(0.0, 1.0) < 0.75 {
                gen_range(15.0, 45.0)
            } else {
                gen_range(45.0, 75.0)
            };
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            // Vibrant, high-end retro pixel color palette (Red -> Orange -> Yellow -> White core)
            let color = if gen_range(0.0, 1.0) < 0.75 {
                match gen_range(0, 4) {
                    0 => Color::new(1.0, gen_range(0.0, 0.2), 0.0, 1.0),       // Incandescent Red
                    1 => Color::new(1.0, gen_range(0.3, 0.6), 0.0, 1.0),       // Hot Orange
                    2 => Color::new(1.0, gen_range(0.7, 0.95), gen_range(0.0, 0.3), 1.0), // Bright Yellow
                    _ => Color::new(1.0, 0.95, 0.6, 1.0),                      // White-hot core
                }
            } else {
                // Sparks of ash gray
                let gray = gen_range(0.4, 0.6);
                Color::new(gray, gray, gray, gen_range(0.6, 0.9))
            };

            let initial_size = gen_range(0.12, 0.38);
            let lifetime = gen_range(1.2, 2.6);

            self.particles.push(ExplosionParticle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                trail: Vec::new(),
                is_smoke: false,
            });
        }

        // 2. Spawning smoke puffs (20 to 35 larger expanding clouds)
        let smoke_count = gen_range(20, 35);
        for _ in 0..smoke_count {
            let angle = gen_range(0.0, 2.0 * PI);
            // Spawn smoke in a full 360-degree radial expansion
            let speed = gen_range(12.0, 26.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            // Soft ash-gray and translucent white-smoke colors
            let gray = gen_range(0.55, 0.8);
            let color = Color::new(gray, gray, gray, gen_range(0.15, 0.35));

            let initial_size = gen_range(0.5, 1.2);
            let lifetime = gen_range(0.6, 1.3);

            self.particles.push(ExplosionParticle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                trail: Vec::new(),
                is_smoke: true,
            });
        }
    }

    pub fn spawn_purple_burst(&mut self, pos: Vec2) {
        use macroquad::rand::gen_range;
        use std::f32::consts::PI;

        let count = gen_range(30, 50);
        for _ in 0..count {
            let angle = gen_range(0.0, 2.0 * PI);
            let speed = gen_range(15.0, 45.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let color = match gen_range(0, 3) {
                0 => Color::new(0.6, 0.0, 0.9, 1.0), // Deep Purple
                1 => Color::new(0.8, 0.2, 1.0, 1.0), // Bright Violet
                _ => Color::new(0.9, 0.6, 1.0, 1.0), // Light Lilac
            };
            let initial_size = gen_range(0.15, 0.4);
            let lifetime = gen_range(0.8, 1.8);

            self.particles.push(ExplosionParticle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                trail: Vec::new(),
                is_smoke: false,
            });
        }
    }

    pub fn spawn_mega_burst(&mut self, pos: Vec2, color: Color) {
        use macroquad::rand::gen_range;
        use std::f32::consts::PI;

        let count = gen_range(600, 800);
        for _ in 0..count {
            let angle = gen_range(0.0, 2.0 * PI);
            let speed = gen_range(40.0, 250.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            let alpha = gen_range(0.8, 1.0);
            let p_color = if gen_range(0.0, 1.0) < 0.8 {
                Color::new(color.r, color.g, color.b, alpha)
            } else {
                Color::new(1.0, 1.0, 1.0, alpha)
            };

            let initial_size = gen_range(0.2, 0.9);
            let lifetime = gen_range(1.5, 3.5);

            self.particles.push(ExplosionParticle {
                position: pos,
                velocity,
                color: p_color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                trail: Vec::new(),
                is_smoke: false,
            });
        }
    }

    pub fn update(&mut self, dt: f32, wallmap: &[Rect]) {
        let gravity = 65.0;
        let friction = 0.5;
        let elasticity = 0.35;
        let floor_y = 100.0;

        for p in &mut self.particles {
            if p.is_smoke {
                // Smoke physics: upward buoyancy and high air resistance
                p.velocity.y -= 12.0 * dt; // upward drift
                p.velocity *= 1.0 - (2.5 * dt); // fast deceleration
                p.position += p.velocity * dt;
            } else {
                // Sparks physics: gravity, friction, trailing and platform bouncing
                p.trail.push(p.position);
                if p.trail.len() > 9 {
                    p.trail.remove(0);
                }

                p.velocity.y += gravity * dt;
                p.velocity *= 1.0 - (friction * dt);

                let next_pos = p.position + p.velocity * dt;
                let mut bounced = false;

                // 1. Collision with bottom floor (Y = 100.0)
                if p.velocity.y > 0.0 && next_pos.y >= floor_y {
                    p.position.y = floor_y;
                    p.position.x = next_pos.x;
                    p.velocity.y = -elasticity * p.velocity.y;
                    bounced = true;
                }

                // 2. Collision with platform top surfaces
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

                if !bounced {
                    p.position = next_pos;
                }
            }

            p.lifetime -= dt;
        }

        // Retain only active particles
        self.particles.retain(|p| p.lifetime > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let progress = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut render_color = p.color;
            render_color.a = progress * p.color.a; // Smooth fade out

            if p.is_smoke {
                // Smoke expands over time (grows to 1.8x original size)
                let size = p.initial_size * (1.8 - 0.8 * progress);
                draw_rectangle(
                    p.position.x - size / 2.0,
                    p.position.y - size / 2.0,
                    size,
                    size,
                    render_color,
                );
            } else {
                // Sparks rendering with trails
                let size = p.initial_size * progress; // Shrink

                // 1. Draw trail segments with fading alpha and size
                let trail_len = p.trail.len();
                for (i, trail_pos) in p.trail.iter().enumerate() {
                    let trail_progress = (i + 1) as f32 / (trail_len + 1) as f32;
                    let mut trail_color = render_color;
                    trail_color.a = render_color.a * 0.7 * trail_progress; 
                    let trail_size = size * (0.45 + 0.55 * trail_progress);

                    draw_rectangle(
                        trail_pos.x - trail_size / 2.0,
                        trail_pos.y - trail_size / 2.0,
                        trail_size,
                        trail_size,
                        trail_color,
                    );
                }

                // 2. Render main particle as retro square pixels
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
}
