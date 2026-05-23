use macroquad::prelude::*;
use crate::projectile::Projectile;
use crate::boilerplate::animation::Animation;
use crate::boilerplate::physics::Physics;
use crate::game::VIRTUAL_HEIGHT;

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    pub fn spawn(&mut self, position: Vec2, velocity: Vec2, color: Color, size: f32, lifetime: f32) {
        self.particles.push(Particle {
            position,
            velocity,
            color,
            size,
            lifetime,
            max_lifetime: lifetime,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.velocity.y += 15.0 * dt;
            p.velocity.x *= 1.0 - (2.0 * dt);
            p.position += p.velocity * dt;
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let progress = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut color = p.color;
            color.a = progress * p.color.a;
            let current_size = p.size * progress;
            draw_circle(p.position.x, p.position.y, current_size, color);
        }
    }
}

pub struct Player {
    pub id: i32,
    animation: Animation,
    pub hitbox: Rect,
    pub speed: f32,
    pub projectiles: Vec<Projectile>,
    pub pv: f32,
    physics: Physics,
    jump_available: i32,

    // --- PARTICLE SYSTEM FIELDS ---
    pub particles: ParticleSystem,
    pub is_grounded: bool,
    pub was_grounded: bool,
    pub dust_timer: f32,
}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            id: macroquad::rand::rand() as i32,
            speed: 50.0,
            hitbox: Rect::new(0.0, 0.0, 10.0, 10.0),
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]),
            projectiles: Vec::new(),
            pv: 100.0,
            physics: Physics::new(200.0, 0.5),
            jump_available: 2,

            particles: ParticleSystem::new(),
            is_grounded: false,
            was_grounded: false,
            dust_timer: 0.0,
        }
    }

    pub fn take_damage(&mut self,val:f32){
        if self.pv - val < 0. {
            self.pv = 0.;
        }
        else{
            self.pv -= val;
        }
    }

    pub fn heal(&mut self,val:f32){
        if self.pv + val > 100. {
            self.pv = 100.;
        }
        else{
            self.pv += val;
        }
    }


    fn tirer_projectile(&mut self, camera: &Camera2D) {
        let mouse_pos = mouse_position();
        let world_mouse = camera.screen_to_world(vec2(mouse_pos.0, mouse_pos.1));
        let nouveau_projectile = Projectile::new(self.id ,self.hitbox.x + self.hitbox.w/2. , self.hitbox.y + self.hitbox.h/2. , world_mouse.x, world_mouse.y);
        self.projectiles.push(nouveau_projectile);
    }

    pub fn handle_input(&mut self, dt: f32, wallmap: &Vec<Rect>) {
        //////////////////////////////////////////////////////////////////////////////////////////////// TODO A SUPPRIMER V FINALE
        if is_key_pressed(KeyCode::P){
            self.heal(5.);
        }
        if is_key_pressed(KeyCode::M){
            self.take_damage(5.);
        }
        //////////////////////////////////////////////////////////////////////////////////////////////////

        // --- MOUVEMENTS ZQSD ---
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { self.hitbox.x += self.speed * dt; }
        if wallmap.iter().any(|wall| self.hitbox.overlaps(wall)){
             self.hitbox.x -= self.speed * dt; 
        }

        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::Q) { self.hitbox.x -= self.speed * dt; }
        if wallmap.iter().any(|wall| self.hitbox.overlaps(wall)){
             self.hitbox.x += self.speed * dt; 
        }
        //if is_key_down(KeyCode::Up) || is_key_down(KeyCode::Z) { self.hitbox.y -= self.speed * dt; }
        if self.jump_available>0{
            if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Space){
                //println!("🕹️ SAUT DÉCLENCHÉ ! (Touches détectées)");
                self.physics.jump(100.);
                self.jump_available -= 1;
                }
        }

        // --- SCREEN BOUNDS CLAMP ---
        let aspect_ratio = screen_width() / screen_height();
        let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
        if self.hitbox.x < 0.0 {
            self.hitbox.x = 0.0;
        } else if self.hitbox.x > virtual_width - self.hitbox.w {
            self.hitbox.x = virtual_width - self.hitbox.w;
        }
    }


    pub fn update(&mut self, camera: &Camera2D, wallmap: &Vec<Rect>, _joueurs: &mut Vec<Player>, is_host: bool) {
        if self.pv <= 0.0 {
            self.pv = 100.0;
            self.hitbox.x = 20.0;
            self.hitbox.y = 20.0;
        }

        let dt = get_frame_time().clamp(0.001, 0.05);

        // 1. Capture preceding grounded state
        self.was_grounded = self.is_grounded;

        // 2. Perform standard inputs
        self.handle_input(dt, wallmap);

        // --- GRAVITE ---
        let old_y = self.hitbox.y;
        self.physics.apply_physics(&mut self.hitbox);
        let dy = self.hitbox.y - old_y; // Velocity direction detection

        // --- LOGIQUE DE TIR ---
        if is_host && is_mouse_button_pressed(MouseButton::Left) {
            self.tirer_projectile(camera);
        }

        // 3. Collision check and grounding resolution
        let mut grounded_this_frame = false;

        if self.hitbox.y > VIRTUAL_HEIGHT - self.hitbox.h {
            self.hitbox.y = VIRTUAL_HEIGHT - self.hitbox.h;
            self.physics.set_velocity_y(0.);
            self.jump_available = 2;
            grounded_this_frame = true;
        }

        for wall in wallmap {
            if self.hitbox.overlaps(wall) {
                if dy > 0.0 {
                    // On tombe. On se pose PILE sur le mur.
                    self.hitbox.y = wall.y - self.hitbox.h;
                    self.physics.set_velocity_y(0.0);
                    self.jump_available = 2; // BINGO ! On récupère nos sauts ici !
                    grounded_this_frame = true;
                } else if dy < 0.0 {
                    // On monte (on se cogne la tête). On se colle PILE sous le mur.
                    self.hitbox.y = wall.y + wall.h;
                    self.physics.set_velocity_y(0.0);
                }
            }
        }

        self.is_grounded = grounded_this_frame;

        // 4. Trigger Landing Burst
        if !self.was_grounded && self.is_grounded {
            self.spawn_landing_burst();
        }

        // 5. Trigger Running Dust Trails
        if self.is_grounded {
            let right_active = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
            let left_active = is_key_down(KeyCode::Left) || is_key_down(KeyCode::Q);
            let is_moving = right_active ^ left_active; // XOR ensures opposite inputs cancel out

            if is_moving {
                self.dust_timer += dt;
                if self.dust_timer >= 0.08 { // Emit particle every 80ms
                    self.dust_timer = 0.0;
                    self.spawn_running_dust(right_active);
                }
            } else {
                self.dust_timer = 0.0;
            }
        } else {
            self.dust_timer = 0.0;
        }

        // 6. Update Particle System lifetimes
        self.particles.update(dt);
    }

    fn spawn_running_dust(&mut self, moving_right: bool) {
        use macroquad::rand::gen_range;

        let feet_x = self.hitbox.x + self.hitbox.w / 2.0;
        let feet_y = self.hitbox.y + self.hitbox.h;

        // Position slightly randomized around feet
        let spawn_pos = vec2(
            feet_x + gen_range(-1.5, 1.5),
            feet_y - gen_range(0.0, 0.8),
        );

        // Blow opposite to horizontal moving direction
        let vx = if moving_right {
            gen_range(-18.0, -8.0)
        } else {
            gen_range(8.0, 18.0)
        };
        // Float upwards
        let vy = gen_range(-8.0, -2.0);

        let color = Color::new(0.83, 0.80, 0.77, 0.55); // Warm grey dust
        let size = gen_range(1.0, 2.2);
        let lifetime = gen_range(0.25, 0.45);

        self.particles.spawn(spawn_pos, vec2(vx, vy), color, size, lifetime);
    }

    fn spawn_landing_burst(&mut self) {
        use macroquad::rand::gen_range;

        let feet_y = self.hitbox.y + self.hitbox.h;
        let left_edge = self.hitbox.x;
        let right_edge = self.hitbox.x + self.hitbox.w;
        let center_x = self.hitbox.x + self.hitbox.w / 2.0;

        let particle_count = gen_range(7, 11);
        for _ in 0..particle_count {
            let spawn_x = gen_range(left_edge, right_edge);
            let spawn_pos = vec2(spawn_x, feet_y);

            // Compute direction away from player center
            let is_left = spawn_x < center_x;
            let vx = if is_left {
                gen_range(-28.0, -12.0)
            } else {
                gen_range(12.0, 28.0)
            };
            // Strong upward bounce velocity
            let vy = gen_range(-24.0, -6.0);

            let color = Color::new(0.79, 0.76, 0.73, 0.70); // Slightly darker/thicker dust for impact
            let size = gen_range(1.3, 2.8);
            let lifetime = gen_range(0.35, 0.55);

            self.particles.spawn(spawn_pos, vec2(vx, vy), color, size, lifetime);
        }
    }

    pub fn update_projectile(&mut self, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, joueurs: &mut Vec<Player>, dt: f32) {
        for proj in &mut self.projectiles {
            proj.update(dt, wallmap, hitboxes_murs, joueurs, None);
        }
    }

    pub fn draw_healthbar(&self) {
        let width: f32 = 6.;

        draw_rectangle(self.hitbox.x + self.hitbox.w / 2. - width / 2., self.hitbox.y + self.hitbox.h + 0.2, width * self.pv / 100., 0.3, GREEN);
        draw_rectangle(self.hitbox.x + width * self.pv / 100. + self.hitbox.w / 2. - width / 2., self.hitbox.y + self.hitbox.h + 0.2, width * (100. - self.pv) / 100., 0.3, RED);
    }

    pub fn draw(&self) {
        self.particles.draw();
        self.animation.draw_current_frame(self.hitbox.x, self.hitbox.y, 10., 10., true);
        self.draw_healthbar();

        for projectile in &self.projectiles {
            projectile.draw();
        }
    }
}
