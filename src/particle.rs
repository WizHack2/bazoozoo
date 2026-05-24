use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub enum ParticleKind {
    Simple,
    ExplosionSpark,
    ExplosionSmoke,
    MegaSpark,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub initial_size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub kind: ParticleKind,
    pub trail: Vec<Vec2>,
}

impl Particle {
    fn progress(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }
}

pub struct ParticleManager {
    pub particles: Vec<Particle>,
}

impl ParticleManager {
    pub fn new() -> Self {
        Self { particles: Vec::new() }
    }

    pub fn spawn(&mut self, position: Vec2, velocity: Vec2, color: Color, size: f32, lifetime: f32) {
        self.particles.push(Particle {
            position,
            velocity,
            color,
            initial_size: size,
            lifetime,
            max_lifetime: lifetime,
            kind: ParticleKind::Simple,
            trail: Vec::new(),
        });
    }

    pub fn spawn_burst(&mut self, pos: Vec2) {
        use macroquad::rand::gen_range;
        use std::f32::consts::PI;

        let count = gen_range(110, 160);
        for _ in 0..count {
            let angle = gen_range(0.0, 2.0 * PI);
            let speed = if gen_range(0.0, 1.0) < 0.75 {
                gen_range(15.0, 45.0)
            } else {
                gen_range(45.0, 75.0)
            };
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            let color = if gen_range(0.0, 1.0) < 0.75 {
                match gen_range(0, 4) {
                    0 => Color::new(1.0, gen_range(0.0, 0.2), 0.0, 1.0),
                    1 => Color::new(1.0, gen_range(0.3, 0.6), 0.0, 1.0),
                    2 => Color::new(1.0, gen_range(0.7, 0.95), gen_range(0.0, 0.3), 1.0),
                    _ => Color::new(1.0, 0.95, 0.6, 1.0),
                }
            } else {
                let gray = gen_range(0.4, 0.6);
                Color::new(gray, gray, gray, gen_range(0.6, 0.9))
            };

            let initial_size = gen_range(0.12, 0.38);
            let lifetime = gen_range(1.2, 2.6);

            self.particles.push(Particle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                kind: ParticleKind::ExplosionSpark,
                trail: Vec::new(),
            });
        }

        let smoke_count = gen_range(20, 35);
        for _ in 0..smoke_count {
            let angle = gen_range(0.0, 2.0 * PI);
            let speed = gen_range(12.0, 26.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            let gray = gen_range(0.55, 0.8);
            let color = Color::new(gray, gray, gray, gen_range(0.15, 0.35));

            let initial_size = gen_range(0.5, 1.2);
            let lifetime = gen_range(0.6, 1.3);

            self.particles.push(Particle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                kind: ParticleKind::ExplosionSmoke,
                trail: Vec::new(),
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
                0 => Color::new(0.6, 0.0, 0.9, 1.0),
                1 => Color::new(0.8, 0.2, 1.0, 1.0),
                _ => Color::new(0.9, 0.6, 1.0, 1.0),
            };
            let initial_size = gen_range(0.15, 0.4);
            let lifetime = gen_range(0.8, 1.8);

            self.particles.push(Particle {
                position: pos,
                velocity,
                color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                kind: ParticleKind::ExplosionSpark,
                trail: Vec::new(),
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

            self.particles.push(Particle {
                position: pos,
                velocity,
                color: p_color,
                initial_size,
                lifetime,
                max_lifetime: lifetime,
                kind: ParticleKind::MegaSpark,
                trail: Vec::new(),
            });
        }
    }

    pub fn update(&mut self, dt: f32, wallmap: &[Rect]) {
        let gravity = 65.0;
        let floor_y = 100.0;

        for p in &mut self.particles {
            match p.kind {
                ParticleKind::Simple => {
                    p.velocity.y += 15.0 * dt;
                    p.velocity.x *= 1.0 - (2.0 * dt);
                    p.position += p.velocity * dt;
                }
                ParticleKind::ExplosionSmoke => {
                    p.velocity.y -= 12.0 * dt;
                    p.velocity *= 1.0 - (2.5 * dt);
                    p.position += p.velocity * dt;
                }
                ParticleKind::ExplosionSpark | ParticleKind::MegaSpark => {
                    p.trail.push(p.position);
                    if p.trail.len() > 9 {
                        p.trail.remove(0);
                    }

                    p.velocity.y += gravity * dt;
                    p.velocity *= 1.0 - (0.5 * dt);

                    let next_pos = p.position + p.velocity * dt;
                    let mut bounced = false;

                    if p.velocity.y > 0.0 && next_pos.y >= floor_y {
                        p.position.y = floor_y;
                        p.position.x = next_pos.x;
                        p.velocity.y = -0.35 * p.velocity.y;
                        bounced = true;
                    }

                    if !bounced && p.velocity.y > 0.0 {
                        for wall in wallmap {
                            if next_pos.x >= wall.x && next_pos.x <= wall.x + wall.w {
                                if p.position.y <= wall.y && next_pos.y >= wall.y {
                                    p.position.y = wall.y;
                                    p.position.x = next_pos.x;
                                    p.velocity.y = -0.35 * p.velocity.y;
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
            }

            p.lifetime -= dt;
        }

        self.particles.retain(|p| p.lifetime > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let progress = p.progress();
            let mut render_color = p.color;
            render_color.a = progress * p.color.a;

            match p.kind {
                ParticleKind::Simple => {
                    let current_size = p.initial_size * progress;
                    draw_rectangle(
                        p.position.x - current_size / 2.0,
                        p.position.y - current_size / 2.0,
                        current_size,
                        current_size,
                        render_color,
                    );
                }
                ParticleKind::ExplosionSmoke => {
                    let size = p.initial_size * (1.8 - 0.8 * progress);
                    draw_rectangle(
                        p.position.x - size / 2.0,
                        p.position.y - size / 2.0,
                        size,
                        size,
                        render_color,
                    );
                }
                ParticleKind::ExplosionSpark | ParticleKind::MegaSpark => {
                    let size = p.initial_size * progress;

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
}
