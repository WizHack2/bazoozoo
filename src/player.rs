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
            // Dessine de vrais pixels carrés rétro !
            draw_rectangle(
                p.position.x - current_size / 2.0,
                p.position.y - current_size / 2.0,
                current_size,
                current_size,
                color,
            );
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

    // --- TIR CLAVIER + RECUL ---
    /// Cooldown en secondes avant de pouvoir retirer une rocket (clavier)
    pub rocket_cooldown: f32,
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
            physics: Physics::new(200.0, 250.0),
            jump_available: 2,

            particles: ParticleSystem::new(),
            is_grounded: false,
            was_grounded: false,
            dust_timer: 0.0,

            rocket_cooldown: 0.0,
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

    /// Tire une rocket dans une direction et applique un RECUL immédiat au joueur.
    /// C'est le recul qui crée le rocket jump, pas l'explosion.
    fn tirer_projectile_clavier(&mut self, dir: Vec2) {
        let center_x = self.hitbox.x + self.hitbox.w / 2.0;
        let center_y = self.hitbox.y + self.hitbox.h / 2.0;
        // On décale le point de spawn pour éviter une auto-collision immédiate
        let spawn_x = center_x + dir.x * 6.0;
        let spawn_y = center_y + dir.y * 6.0;
        let target_x = center_x + dir.x * 100.0;
        let target_y = center_y + dir.y * 100.0;
        let projectile = Projectile::new(self.id, spawn_x, spawn_y, target_x, target_y);
        self.projectiles.push(projectile);

        // --- RECUL DE L'ARME (rocket jump) ---
        // Le joueur reçoit une impulsion dans la direction OPPOSÉE au tir
        const RECOIL_FORCE: f32 = 80.0;
        let recoil = -dir * RECOIL_FORCE;
        self.apply_recoil(recoil);

        self.rocket_cooldown = 0.4;
    }

    /// Applique un recul d'arme au joueur (impulsion immédiate)
    fn apply_recoil(&mut self, impulse: Vec2) {
        // Si le recul pousse vers le haut et qu'on est au sol,
        // on SET la velocity Y pour garantir la hauteur du saut
        if impulse.y < 0.0 {
            self.physics.set_velocity_y(impulse.y);
            if impulse.x.abs() > 0.01 {
                self.physics.add_velocity_x(impulse.x);
            }
        } else {
            // Recul latéral ou vers le bas : on cumule
            self.physics.add_velocity(impulse);
        }
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

        // --- TIR CLAVIER : T/G/F/H/R/Y/V/N = une touche par direction (AZERTY) ---
        if self.rocket_cooldown > 0.0 {
            self.rocket_cooldown -= dt;
        }
        if self.rocket_cooldown <= 0.0 {
            if is_key_pressed(KeyCode::T) {
                self.tirer_projectile_clavier(vec2(0.0, -1.0)); // Haut
            } else if is_key_pressed(KeyCode::G) {
                self.tirer_projectile_clavier(vec2(0.0, 1.0));  // Bas
            } else if is_key_pressed(KeyCode::F) {
                self.tirer_projectile_clavier(vec2(-1.0, 0.0)); // Gauche
            } else if is_key_pressed(KeyCode::H) {
                self.tirer_projectile_clavier(vec2(1.0, 0.0));  // Droite
            } else if is_key_pressed(KeyCode::R) {
                self.tirer_projectile_clavier(vec2(-1.0, -1.0).normalize()); // Haut-Gauche
            } else if is_key_pressed(KeyCode::Y) {
                self.tirer_projectile_clavier(vec2(1.0, -1.0).normalize());  // Haut-Droite
            } else if is_key_pressed(KeyCode::V) {
                self.tirer_projectile_clavier(vec2(-1.0, 1.0).normalize());  // Bas-Gauche
            } else if is_key_pressed(KeyCode::N) {
                self.tirer_projectile_clavier(vec2(1.0, 1.0).normalize());   // Bas-Droite
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
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let world_mouse = camera.screen_to_world(vec2(mouse_pos.0, mouse_pos.1));
            let center_x = self.hitbox.x + self.hitbox.w / 2.0;
            let center_y = self.hitbox.y + self.hitbox.h / 2.0;
            let dir = vec2(world_mouse.x - center_x, world_mouse.y - center_y);
            let length = dir.length();
            if length > 0.0 {
                let dir = dir / length;
                const RECOIL_FORCE: f32 = 80.0;
                let recoil = -dir * RECOIL_FORCE;
                self.apply_recoil(recoil);
            }

            if is_host {
                self.tirer_projectile(camera);
            }
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
            // Détecte si le joueur est déjà posé à plat sur le haut de ce mur (évite le jitter d'overlaps)
            let is_on_top = self.hitbox.x + self.hitbox.w > wall.x
                && self.hitbox.x < wall.x + wall.w
                && (self.hitbox.y + self.hitbox.h - wall.y).abs() < 0.1;

            if self.hitbox.overlaps(wall) || is_on_top {
                if dy > 0.0 || is_on_top {
                    // On tombe ou on est déjà posé. On se pose PILE sur le mur.
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

        // Position légèrement aléatoire sous les pieds
        let spawn_pos = vec2(
            feet_x + gen_range(-1.5, 1.5),
            feet_y - gen_range(0.0, 0.8),
        );

        // Propulsion dans le sens inverse de la course
        let vx = if moving_right {
            gen_range(-18.0, -8.0)
        } else {
            gen_range(8.0, 18.0)
        };
        let vy = gen_range(-8.0, -2.0);

        // Mélange de pixels de fumée grise et de poussière de mousse verte
        let color = if macroquad::rand::gen_range(0, 3) == 0 {
            Color::new(0.38, 0.53, 0.35, 0.65) // Vert mousse
        } else {
            Color::new(0.85, 0.85, 0.85, 0.60) // Gris fumée
        };
        
        let size = gen_range(0.8, 1.8);
        let lifetime = gen_range(0.25, 0.45);

        self.particles.spawn(spawn_pos, vec2(vx, vy), color, size, lifetime);
    }

    fn spawn_landing_burst(&mut self) {
        use macroquad::rand::gen_range;

        let feet_y = self.hitbox.y + self.hitbox.h;
        let left_edge = self.hitbox.x;
        let right_edge = self.hitbox.x + self.hitbox.w;
        let center_x = self.hitbox.x + self.hitbox.w / 2.0;

        let particle_count = gen_range(8, 14);
        for _ in 0..particle_count {
            let spawn_x = gen_range(left_edge, right_edge);
            let spawn_pos = vec2(spawn_x, feet_y);

            let is_left = spawn_x < center_x;
            let vx = if is_left {
                gen_range(-28.0, -12.0)
            } else {
                gen_range(12.0, 28.0)
            };
            let vy = gen_range(-24.0, -6.0);

            // Mélange de pixels gris, d'ardoise et de mousse verte sur l'impact
            let color = match macroquad::rand::gen_range(0, 3) {
                0 => Color::new(0.38, 0.53, 0.35, 0.75), // Vert mousse
                1 => Color::new(0.50, 0.55, 0.60, 0.70), // Ardoise foncée
                _ => Color::new(0.90, 0.90, 0.90, 0.80), // Blanc de fumée
            };
            
            let size = gen_range(1.0, 2.3);
            let lifetime = gen_range(0.35, 0.55);

            self.particles.spawn(spawn_pos, vec2(vx, vy), color, size, lifetime);
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
