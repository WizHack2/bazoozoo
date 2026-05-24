use macroquad::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrainingDifficulty {
    Fixed,
    Normal,
    Extreme,
}

pub struct Target {
    pub hitbox: Rect,
    pub speed: Vec2,
    pub is_destroyed: bool,
    pub pv: f32,
    pub hits_received: Vec<Vec2>,
}

impl Target {
    pub fn spawn_random(virtual_width: f32, difficulty: TrainingDifficulty, wallmap: &[Rect]) -> Self {
        use macroquad::rand::gen_range;
        let w = 10.0;
        let h = 10.0;

        let mut x = 20.0;
        let mut y = 20.0;

        for _ in 0..200 {
            let px = gen_range(10.0, virtual_width - 15.0);
            let py = gen_range(10.0, 80.0);
            let test_rect = Rect::new(px, py, w, h);

            let overlaps_wall = wallmap.iter().any(|wall| test_rect.overlaps(wall));
            if !overlaps_wall {
                x = px;
                y = py;
                break;
            }
        }

        let speed = match difficulty {
            TrainingDifficulty::Fixed => Vec2::ZERO,
            TrainingDifficulty::Normal => {
                let angle = gen_range(0.0, 2.0 * std::f32::consts::PI);
                let speed_val = gen_range(12.0, 22.0);
                Vec2::new(angle.cos() * speed_val, angle.sin() * speed_val)
            }
            TrainingDifficulty::Extreme => {
                let angle = gen_range(0.0, 2.0 * std::f32::consts::PI);
                let speed_val = gen_range(40.0, 60.0);
                Vec2::new(angle.cos() * speed_val, angle.sin() * speed_val)
            }
        };

        Self {
            hitbox: Rect::new(x, y, w, h),
            speed,
            is_destroyed: false,
            pv: 15.0,
            hits_received: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, virtual_width: f32, wallmap: &[Rect]) {
        if self.speed.length() < 0.01 {
            return;
        }

        self.hitbox.x += self.speed.x * dt;
        self.hitbox.y += self.speed.y * dt;

        if self.hitbox.x < 0.0 {
            self.hitbox.x = 0.0;
            self.speed.x = -self.speed.x;
        } else if self.hitbox.x > virtual_width - self.hitbox.w {
            self.hitbox.x = virtual_width - self.hitbox.w;
            self.speed.x = -self.speed.x;
        }

        if self.hitbox.y < 0.0 {
            self.hitbox.y = 0.0;
            self.speed.y = -self.speed.y;
        } else if self.hitbox.y > 100.0 - self.hitbox.h {
            self.hitbox.y = 100.0 - self.hitbox.h;
            self.speed.y = -self.speed.y;
        }

        for wall in wallmap {
            if self.hitbox.overlaps(wall) {
                self.speed = -self.speed;
                self.hitbox.x += self.speed.x * dt;
                self.hitbox.y += self.speed.y * dt;
                break;
            }
        }
    }

    pub fn draw(&self, texture: &Texture2D) {
        let look_right = self.speed.x >= 0.0;
        draw_texture_ex(
            texture,
            self.hitbox.x,
            self.hitbox.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.hitbox.w, self.hitbox.h)),
                flip_x: !look_right,
                ..Default::default()
            }
        );
        self.draw_healthbar();
    }

    fn draw_healthbar(&self) {
        let width: f32 = 6.0;
        let bar_x = self.hitbox.x + self.hitbox.w / 2.0 - width / 2.0;
        let bar_y = self.hitbox.y - 1.5;
        draw_rectangle(bar_x, bar_y, width * self.pv / 15.0, 0.3, GREEN);
        draw_rectangle(bar_x + width * self.pv / 15.0, bar_y, width * (15.0 - self.pv) / 15.0, 0.3, RED);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_difficulty_copy() {
        let a = TrainingDifficulty::Normal;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_training_difficulty_equality() {
        assert_ne!(TrainingDifficulty::Fixed, TrainingDifficulty::Normal);
        assert_ne!(TrainingDifficulty::Normal, TrainingDifficulty::Extreme);
        assert_eq!(TrainingDifficulty::Fixed, TrainingDifficulty::Fixed);
    }

    #[test]
    fn test_target_default_state() {
        let target = Target {
            hitbox: Rect::new(0.0, 0.0, 10.0, 10.0),
            speed: Vec2::ZERO,
            is_destroyed: false,
            pv: 15.0,
            hits_received: Vec::new(),
        };
        assert!(!target.is_destroyed);
        assert_eq!(target.pv, 15.0);
        assert!(target.hits_received.is_empty());
    }
}
