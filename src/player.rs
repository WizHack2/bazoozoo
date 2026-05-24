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

fn create_bazooka_texture() -> Texture2D {
    // A 12x6 pixel art bazooka
    let mut image = Image::gen_image_color(12, 6, Color::new(0.0, 0.0, 0.0, 0.0));
    
    // Body (dark steel gray)
    for x in 2..10 {
        for y in 2..5 {
            image.set_pixel(x, y, Color::new(0.3, 0.3, 0.35, 1.0));
        }
    }
    // Muzzle (metallic silver/light grey)
    for y in 1..6 {
        image.set_pixel(10, y, Color::new(0.6, 0.6, 0.65, 1.0));
        image.set_pixel(11, y, Color::new(0.7, 0.7, 0.75, 1.0));
    }
    // Back exhaust (darker grey)
    for y in 2..5 {
        image.set_pixel(0, y, Color::new(0.15, 0.15, 0.18, 1.0));
        image.set_pixel(1, y, Color::new(0.2, 0.2, 0.23, 1.0));
    }
    // Scope (top)
    image.set_pixel(5, 1, Color::new(0.1, 0.1, 0.1, 1.0));
    image.set_pixel(6, 1, Color::new(0.1, 0.1, 0.1, 1.0));
    image.set_pixel(7, 1, Color::new(0.1, 0.1, 0.1, 1.0));
    image.set_pixel(6, 0, Color::new(0.0, 0.8, 1.0, 1.0)); // Lens (cyan)
    
    // Handle (bottom)
    image.set_pixel(4, 5, Color::new(0.1, 0.1, 0.1, 1.0));
    image.set_pixel(8, 5, Color::new(0.1, 0.1, 0.1, 1.0));

    let tex = Texture2D::from_image(&image);
    tex.set_filter(FilterMode::Nearest);
    tex
}

pub struct Player {
    pub id: i32,
    pub animation: Animation,
    pub hitbox: Rect,
    pub speed: f32,
    pub projectiles: Vec<Projectile>,
    pub pv: f32,
    physics: Physics,
    pub score: i32,
    pub color: Color,
    pub death_timer: f32,
    jump_available: i32,

    // --- PARTICLE SYSTEM FIELDS ---
    pub particles: ParticleSystem,
    pub is_grounded: bool,
    pub was_grounded: bool,
    pub dust_timer: f32,

    // --- TIR CLAVIER + RECUL ---
    /// Cooldown en secondes avant de pouvoir retirer une rocket (clavier)
    pub rocket_cooldown: f32,

    // --- BAZOOKA & RELOAD ---
    pub bazooka_texture: Texture2D,
    pub bazooka_dir: Vec2,
    pub keyboard_dir_timer: f32,
    pub recoil_displacement: f32,
    pub max_ammo: i32,
    pub current_ammo: i32,
    pub is_reloading: bool,
    pub reload_timer: f32,
    pub a_tire_cette_frame: bool,
    pub a_tire_mega_cette_frame: bool,
    pub target_tir_cette_frame: Vec2,
}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            id: macroquad::rand::rand() as i32,
            speed: 40.0,
            hitbox: Rect::new(0.0, 0.0, 5.0, 5.0),
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]),
            projectiles: Vec::new(),
            pv: 100.0,
            physics: Physics::new(200.0, 250.0),
            score: 0,
            color: WHITE,
            death_timer: 0.0,
            jump_available: 2,

            particles: ParticleSystem::new(),
            is_grounded: false,
            was_grounded: false,
            dust_timer: 0.0,

            rocket_cooldown: 0.0,

            bazooka_texture: create_bazooka_texture(),
            bazooka_dir: vec2(1.0, 0.0),
            keyboard_dir_timer: 0.0,
            recoil_displacement: 0.0,
            max_ammo: 3,
            current_ammo: 3,
            is_reloading: false,
            reload_timer: 0.0,
            a_tire_cette_frame: false,
            a_tire_mega_cette_frame: false,
            target_tir_cette_frame: Vec2::ZERO,
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
        let center_x = self.hitbox.x + self.hitbox.w / 2.0;
        let center_y = self.hitbox.y + self.hitbox.h / 2.0;
        
        // On décale le point de spawn dans la direction de visée pour éviter une auto-collision immédiate avec le sol/murs
        let spawn_x = center_x + self.bazooka_dir.x * 6.0;
        let spawn_y = center_y + self.bazooka_dir.y * 6.0;
        
        let nouveau_projectile = Projectile::new(self.id, spawn_x, spawn_y, world_mouse.x, world_mouse.y);
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
        
        // --- TRICHE: MEGA EXPLOSION ---
        if is_key_pressed(KeyCode::O) {
            self.a_tire_cette_frame = true;
            self.a_tire_mega_cette_frame = true;
            self.target_tir_cette_frame = vec2(99999.0, 99999.0);
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

        // --- RELOAD MANUEL ---
        if is_key_pressed(KeyCode::E) && !self.is_reloading && self.current_ammo < self.max_ammo {
            self.is_reloading = true;
            self.reload_timer = 2.0;
        }

        // --- TIR CLAVIER : T/G/F/H/R/Y/V/N = une touche par direction (AZERTY) ---
        if self.rocket_cooldown > 0.0 {
            self.rocket_cooldown -= dt;
        }
        if self.rocket_cooldown <= 0.0 && !self.is_reloading {
            let mut shot_dir = None;
            if is_key_pressed(KeyCode::T) {
                shot_dir = Some(vec2(0.0, -1.0)); // Haut
            } else if is_key_pressed(KeyCode::G) {
                shot_dir = Some(vec2(0.0, 1.0));  // Bas
            } else if is_key_pressed(KeyCode::F) {
                shot_dir = Some(vec2(-1.0, 0.0)); // Gauche
            } else if is_key_pressed(KeyCode::H) {
                shot_dir = Some(vec2(1.0, 0.0));  // Droite
            } else if is_key_pressed(KeyCode::R) {
                shot_dir = Some(vec2(-1.0, -1.0).normalize()); // Haut-Gauche
            } else if is_key_pressed(KeyCode::Y) {
                shot_dir = Some(vec2(1.0, -1.0).normalize());  // Haut-Droite
            } else if is_key_pressed(KeyCode::V) {
                shot_dir = Some(vec2(-1.0, 1.0).normalize());  // Bas-Gauche
            } else if is_key_pressed(KeyCode::N) {
                shot_dir = Some(vec2(1.0, 1.0).normalize());   // Bas-Droite
            }

            if let Some(dir) = shot_dir {
                self.tirer_projectile_clavier(dir);
                let center_x = self.hitbox.x + self.hitbox.w / 2.0;
                let center_y = self.hitbox.y + self.hitbox.h / 2.0;
                self.a_tire_cette_frame = true;
                self.target_tir_cette_frame = vec2(center_x + dir.x * 100.0, center_y + dir.y * 100.0);

                self.current_ammo -= 1;
                self.bazooka_dir = dir;
                self.keyboard_dir_timer = 0.5;
                self.recoil_displacement = 2.0;

                // Muzzle flash particle animation!
                self.spawn_muzzle_flash(dir);

                if self.current_ammo == 0 {
                    self.is_reloading = true;
                    self.reload_timer = 2.0;
                }
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
            if self.death_timer <= 0.0 {
                self.death_timer = 1.0;
            }
        }

        let dt = get_frame_time().clamp(0.001, 0.05);

        if self.death_timer > 0.0 {
            self.death_timer -= dt;
            
            // Spawn ascending glowing death particles in their player color!
            if macroquad::rand::gen_range(0, 4) == 0 {
                use macroquad::rand::gen_range;
                let spawn_pos = vec2(
                    self.hitbox.x + gen_range(0.0, self.hitbox.w),
                    self.hitbox.y + self.hitbox.h,
                );
                let velocity = vec2(gen_range(-3.0, 3.0), gen_range(-25.0, -10.0));
                let p_color = Color::new(self.color.r, self.color.g, self.color.b, 0.7);
                let size = gen_range(0.8, 1.6);
                let lifetime = gen_range(0.5, 1.0);
                self.particles.spawn(spawn_pos, velocity, p_color, size, lifetime);
            }
            
            if self.death_timer <= 0.0 {
                self.pv = 100.0;
                self.hitbox.x = 20.0;
                self.hitbox.y = 20.0;
            }
            
            self.particles.update(dt);
            return;
        }
        self.a_tire_cette_frame = false;

        // --- TIMERS ET DEPLACEMENTS BAZOOKA ---
        if self.keyboard_dir_timer > 0.0 {
            self.keyboard_dir_timer -= dt;
        }

        if self.recoil_displacement > 0.0 {
            self.recoil_displacement -= dt * 15.0;
            if self.recoil_displacement < 0.0 {
                self.recoil_displacement = 0.0;
            }
        }

        // --- VISÉE BAZOOKA SOURIS ---
        if self.keyboard_dir_timer <= 0.0 {
            let mouse_pos = mouse_position();
            let world_mouse = camera.screen_to_world(vec2(mouse_pos.0, mouse_pos.1));
            let center_x = self.hitbox.x + self.hitbox.w / 2.0;
            let center_y = self.hitbox.y + self.hitbox.h / 2.0;
            let to_mouse = vec2(world_mouse.x - center_x, world_mouse.y - center_y);
            let len = to_mouse.length();
            if len > 0.01 {
                self.bazooka_dir = to_mouse / len;
            }
        }

        // --- GESTION DU RECHARGEMENT ---
        if self.is_reloading {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                self.is_reloading = false;
                self.current_ammo = self.max_ammo;
            }

            // Particules de rechargement qui s'élèvent
            if macroquad::rand::gen_range(0, 3) == 0 {
                use macroquad::rand::gen_range;
                let spawn_pos = vec2(
                    self.hitbox.x + gen_range(0.0, self.hitbox.w),
                    self.hitbox.y + self.hitbox.h,
                );
                let velocity = vec2(gen_range(-2.0, 2.0), gen_range(-15.0, -8.0));
                let color = Color::new(0.0, 0.9, 1.0, 0.7); // Cyan translucide
                let size = gen_range(0.8, 1.4);
                let lifetime = gen_range(0.4, 0.8);
                self.particles.spawn(spawn_pos, velocity, color, size, lifetime);
            }
        }

        // 1. Capture preceding grounded state
        self.was_grounded = self.is_grounded;

        // 2. Perform standard inputs
        self.handle_input(dt, wallmap);

        // --- GRAVITE ---
        let old_y = self.hitbox.y;
        self.physics.apply_physics(&mut self.hitbox);
        let dy = self.hitbox.y - old_y; // Velocity direction detection

        // --- LOGIQUE DE TIR ---
        if is_mouse_button_pressed(MouseButton::Left) && !self.is_reloading {
            let mouse_pos = mouse_position();
            let world_mouse = camera.screen_to_world(vec2(mouse_pos.0, mouse_pos.1));
            let center_x = self.hitbox.x + self.hitbox.w / 2.0;
            let center_y = self.hitbox.y + self.hitbox.h / 2.0;
            let dir = vec2(world_mouse.x - center_x, world_mouse.y - center_y);
            let length = dir.length();
            if length > 0.0 {
                let dir = dir / length;

                // Muzzle flash particle animation!
                self.spawn_muzzle_flash(dir);

                const RECOIL_FORCE: f32 = 80.0;
                let recoil = -dir * RECOIL_FORCE;
                self.apply_recoil(recoil);

                self.current_ammo -= 1;
                self.recoil_displacement = 2.0;

                self.a_tire_cette_frame = true;
                self.target_tir_cette_frame = world_mouse;

                if self.current_ammo == 0 {
                    self.is_reloading = true;
                    self.reload_timer = 2.0;
                }
            }

            if is_host {
                self.tirer_projectile(camera);
            }
        }

        // --- HOST SIDE MEGA SHOOT DETECT ---
        if is_host && self.a_tire_mega_cette_frame {
            let center_x = self.hitbox.x + self.hitbox.w / 2.0;
            let center_y = self.hitbox.y + self.hitbox.h / 2.0;
            let nouveau_projectile = Projectile::new_mega(self.id, center_x, center_y);
            self.projectiles.push(nouveau_projectile);
            self.spawn_muzzle_flash(vec2(0.0, -1.0));
            self.a_tire_mega_cette_frame = false;
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

    fn spawn_muzzle_flash(&mut self, dir: Vec2) {
        use macroquad::rand::gen_range;
        let center_x = self.hitbox.x + self.hitbox.w / 2.0;
        let center_y = self.hitbox.y + self.hitbox.h / 2.0;
        let muzzle_pos = vec2(center_x, center_y) + dir * 4.5;

        let count = gen_range(6, 12);
        for _ in 0..count {
            let angle: f32 = gen_range(-0.4, 0.4);
            let rotated_dir = vec2(
                dir.x * angle.cos() - dir.y * angle.sin(),
                dir.x * angle.sin() + dir.y * angle.cos(),
            );
            let speed = gen_range(15.0, 35.0);
            let velocity = rotated_dir * speed;
            let color = if macroquad::rand::gen_range(0, 2) == 0 {
                Color::new(1.0, gen_range(0.4, 0.7), 0.0, 1.0)
            } else {
                Color::new(1.0, 0.9, 0.4, 1.0)
            };
            let size = gen_range(0.6, 1.4);
            let lifetime = gen_range(0.1, 0.25);
            self.particles.spawn(muzzle_pos, velocity, color, size, lifetime);
        }
    }

    pub fn draw_ammobar(&self) {
        if self.is_reloading {
            let bar_w: f32 = 6.0;
            let bar_h: f32 = 0.4;
            let x = self.hitbox.x + self.hitbox.w / 2.0 - bar_w / 2.0;
            let y = self.hitbox.y - 1.2;

            draw_rectangle(x, y, bar_w, bar_h, Color::new(0.2, 0.2, 0.2, 0.7));

            let progress = ((2.0 - self.reload_timer) / 2.0).clamp(0.0, 1.0);
            draw_rectangle(x, y, bar_w * progress, bar_h, Color::new(0.0, 0.9, 1.0, 1.0));
        } else {
            let ammo_w: f32 = 1.0;
            let ammo_h: f32 = 0.4;
            let gap: f32 = 0.3;
            let total_w = (ammo_w * self.max_ammo as f32) + (gap * (self.max_ammo - 1) as f32);
            let start_x = self.hitbox.x + self.hitbox.w / 2.0 - total_w / 2.0;
            let y = self.hitbox.y - 1.2;

            for i in 0..self.max_ammo {
                let color = if i < self.current_ammo {
                    YELLOW
                } else {
                    Color::new(0.3, 0.3, 0.3, 0.5)
                };
                draw_rectangle(start_x + i as f32 * (ammo_w + gap), y, ammo_w, ammo_h, color);
            }
        }
    }

    pub fn draw_healthbar(&self) {
        let width: f32 = 6.;

        draw_rectangle(self.hitbox.x + self.hitbox.w / 2. - width / 2., self.hitbox.y + self.hitbox.h + 0.2, width * self.pv / 100., 0.3, GREEN);
        draw_rectangle(self.hitbox.x + width * self.pv / 100. + self.hitbox.w / 2. - width / 2., self.hitbox.y + self.hitbox.h + 0.2, width * (100. - self.pv) / 100., 0.3, RED);
    }

    pub fn draw(&self) {
        self.particles.draw();
        
        if self.death_timer > 0.0 {
            // Draw elegant Tombstone in their color fading out
            let progress = (self.death_timer / 1.0).clamp(0.0, 1.0);
            let alpha = progress;
            let c = Color::new(self.color.r, self.color.g, self.color.b, alpha * 0.8);
            let border_c = Color::new(self.color.r * 1.2, self.color.g * 1.2, self.color.b * 1.2, alpha);
            
            let tx = self.hitbox.x;
            let ty = self.hitbox.y;
            let tw = self.hitbox.w;
            let th = self.hitbox.h;
            
            // Tombstone shape
            draw_rectangle(tx + 0.5, ty + 1.0, tw - 1.0, th - 1.0, c);
            // Rounded top
            draw_circle(tx + tw/2.0, ty + 1.0, (tw - 1.0)/2.0, c);
            
            // Outline
            draw_rectangle_lines(tx + 0.5, ty + 1.0, tw - 1.0, th - 1.0, 0.2, border_c);
            
            // Cross inside
            let cx = tx + tw/2.0;
            let cy = ty + th/2.0 + 0.3;
            draw_line(cx - 0.8, cy, cx + 0.8, cy, 0.25, border_c);
            draw_line(cx, cy - 1.0, cx, cy + 0.8, 0.25, border_c);
            return;
        }

        let center_x = self.hitbox.x + self.hitbox.w / 2.0;
        let center_y = self.hitbox.y + self.hitbox.h / 2.0;
        let look_right = self.bazooka_dir.x >= 0.0;

        // Render perfect pixel art contour outline around the PNG's transparent contours
        let mut outline_anim = self.animation.clone();
        outline_anim.change_color(self.color);
        let offset = 0.15;
        outline_anim.draw_current_frame(self.hitbox.x - offset, self.hitbox.y, self.hitbox.w, self.hitbox.h, look_right);
        outline_anim.draw_current_frame(self.hitbox.x + offset, self.hitbox.y, self.hitbox.w, self.hitbox.h, look_right);
        outline_anim.draw_current_frame(self.hitbox.x, self.hitbox.y - offset, self.hitbox.w, self.hitbox.h, look_right);
        outline_anim.draw_current_frame(self.hitbox.x, self.hitbox.y + offset, self.hitbox.w, self.hitbox.h, look_right);

        // Draw a small retro pointer chevron above their head
        draw_triangle(
            vec2(center_x, self.hitbox.y - 0.4),
            vec2(center_x - 0.35, self.hitbox.y - 0.75),
            vec2(center_x + 0.35, self.hitbox.y - 0.75),
            self.color
        );
        
        // Draw the player sprite normally with its gorgeous original details on top
        let mut main_anim = self.animation.clone();
        main_anim.change_color(WHITE);
        main_anim.draw_current_frame(self.hitbox.x, self.hitbox.y, self.hitbox.w, self.hitbox.h, look_right);
        
        self.draw_healthbar();
        self.draw_ammobar();

        // Draw rotated Bazooka
        let angle = self.bazooka_dir.y.atan2(self.bazooka_dir.x);
        
        // recoil visual displacement
        let bazooka_pos = vec2(center_x, center_y) - self.bazooka_dir * self.recoil_displacement;
        
        draw_texture_ex(
            &self.bazooka_texture,
            bazooka_pos.x - 0.75,
            bazooka_pos.y - 0.75,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(3.0, 1.5)),
                pivot: Some(vec2(bazooka_pos.x, bazooka_pos.y)),
                rotation: angle,
                ..Default::default()
            }
        );

        for projectile in &self.projectiles {
            projectile.draw();
        }
    }
}
