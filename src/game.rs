use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::map_loading::charger_hitboxes;
use crate::player::Player;
use crate::Assets;
use crate::projectile::{Projectile, ExplosionParticleSystem};
use crate::boilerplate::network::{PlayerState, NetworkManager, GameMessage};

pub const VIRTUAL_HEIGHT: f32 = 100.0;

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkProjectile {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub is_exploding: bool,
}

// On modifie NetworkPlayer pour qu'il ait des poches pleines de missiles
#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkPlayer {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub pv: f32,
    pub aim_x: f32,
    pub aim_y: f32,
    pub projectiles: Vec<NetworkProjectile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkGameState {
    pub players: Vec<NetworkPlayer>,
}

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
    pub fn spawn_random(virtual_width: f32, difficulty: TrainingDifficulty, wallmap: &Vec<Rect>) -> Self {
        use macroquad::rand::gen_range;
        let w = 10.0;
        let h = 10.0;
        
        let mut x = 20.0;
        let mut y = 20.0;
        
        // Try up to 200 times to find a position that does not overlap with any wall
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

    pub fn update(&mut self, dt: f32, virtual_width: f32, wallmap: &Vec<Rect>) {
        if self.speed.length() < 0.01 {
            return;
        }

        self.hitbox.x += self.speed.x * dt;
        self.hitbox.y += self.speed.y * dt;

        // Bounce off screen boundaries
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

        // Bounce off platforms
        for wall in wallmap {
            if self.hitbox.overlaps(wall) {
                self.speed = -self.speed;
                self.hitbox.x += self.speed.x * dt;
                self.hitbox.y += self.speed.y * dt;
                break;
            }
        }
    }

    pub fn draw_healthbar(&self) {
        let width: f32 = 6.0;
        let bar_x = self.hitbox.x + self.hitbox.w / 2.0 - width / 2.0;
        let bar_y = self.hitbox.y - 1.5;
        draw_rectangle(bar_x, bar_y, width * self.pv / 15.0, 0.3, GREEN);
        draw_rectangle(bar_x + width * self.pv / 15.0, bar_y, width * (15.0 - self.pv) / 15.0, 0.3, RED);
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
}

pub struct Game {
    pub background: Texture2D,
    pub hollow_background: Texture2D,
    pub platform_tile: Texture2D,
    pub is_hollow_map: bool,
    pub debug_show_hitboxes: bool,
    pub player: Player,
    pub wallmap: Vec<Rect>,
    pub other_players: Vec<Player>,
    pub is_host: bool,
    //TEST TICK RATE
    pub last_network_send: f64,
    pub pending_shot: bool,
    pub pending_mouse_x: f32,
    pub pending_mouse_y: f32,
    pub explosion_particles: ExplosionParticleSystem,
    pub fox_texture: Texture2D,

    // --- TRAINING MODE ---
    pub is_training_mode: bool,
    pub training_difficulty: TrainingDifficulty,
    pub training_score: i32,
    pub targets: Vec<Target>,
    pub camera_center: Vec2,
}

impl Game {
    pub fn get_game_camera(&self) -> Camera2D {
        let aspect_ratio = screen_width() / screen_height();
        
        let cam_h = 75.0;
        let cam_w = cam_h * aspect_ratio;
        
        Camera2D::from_display_rect(Rect::new(
            self.camera_center.x - cam_w / 2.0,
            self.camera_center.y + cam_h / 2.0,
            cam_w,
            -cam_h,
        ))
    }

    pub fn new(assets: &Assets, is_host1: bool) -> Self {
        set_fullscreen(true);
        Self {
            background: assets.background.clone(),
            hollow_background: assets.hollow_background.clone(),
            platform_tile: assets.platform_tile.clone(),
            is_hollow_map: true,
            debug_show_hitboxes: false,
            player: Player::new(assets.player.clone()),
            wallmap: charger_hitboxes("assets/hollow_map.json".to_string()),
            other_players: Vec::new(),
            is_host: is_host1,

            last_network_send: macroquad::time::get_time(),
            pending_shot: false,
            pending_mouse_x: 0.0,
            pending_mouse_y: 0.0,
            explosion_particles: ExplosionParticleSystem::new(),
            fox_texture: assets.fox.clone(),

            is_training_mode: false,
            training_difficulty: TrainingDifficulty::Normal,
            training_score: 0,
            targets: Vec::new(),
            camera_center: vec2(20.0, 20.0),
        }
    }

    pub fn sync_network(&mut self, states: Vec<PlayerState>, player_tex: Texture2D) {
        for state in states {
            if let Some(p) = self.other_players.iter_mut().find(|p| p.id == state.id) {
                p.hitbox.x = state.x;
                p.hitbox.y = state.y;
                
                // Update aiming direction from souris coordinates
                let center_x = p.hitbox.x + p.hitbox.w / 2.0;
                let center_y = p.hitbox.y + p.hitbox.h / 2.0;
                let to_target = vec2(state.souris_x - center_x, state.souris_y - center_y);
                if to_target.length() > 0.01 {
                    p.bazooka_dir = to_target.normalize();
                }
                
                // --- NOUVEAU : L'HÔTE CRÉE LE TIR DU CLIENT ---
                if state.a_tire {
                    let nouveau_proj = Projectile::new(
                        state.id, 
                        p.hitbox.x + p.hitbox.w / 2.0, 
                        p.hitbox.y + p.hitbox.h / 2.0, 
                        state.souris_x, 
                        state.souris_y
                    );
                    p.projectiles.push(nouveau_proj);
                }

            } else {
                let mut new_p = Player::new(player_tex.clone());
                new_p.id = state.id;
                new_p.hitbox.x = state.x;
                new_p.hitbox.y = state.y;
                self.other_players.push(new_p);
            }
        }
    }

    pub fn update(&mut self, network: &mut NetworkManager, player_tex: Texture2D) {
        if is_key_pressed(KeyCode::F3) {
            self.debug_show_hitboxes = !self.debug_show_hitboxes;
        }

        if is_key_pressed(KeyCode::M) {
            self.is_hollow_map = !self.is_hollow_map;
            if self.is_hollow_map {
                self.wallmap = charger_hitboxes("assets/hollow_map.json".to_string());
            } else {
                self.wallmap = charger_hitboxes("assets/map2.json".to_string());
            }
        }

        // --- TOGGLE TRAINING MODE ---
        if is_key_pressed(KeyCode::F4) {
            self.is_training_mode = !self.is_training_mode;
            if self.is_training_mode {
                self.training_score = 0;
                self.targets.clear();
                let aspect_ratio = screen_width() / screen_height();
                let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
                for _ in 0..3 {
                    self.targets.push(Target::spawn_random(virtual_width, self.training_difficulty, &self.wallmap));
                }
            } else {
                self.targets.clear();
            }
        }

        if self.is_training_mode {
            let mut diff_changed = false;
            if is_key_pressed(KeyCode::Key1) {
                self.training_difficulty = TrainingDifficulty::Fixed;
                diff_changed = true;
            } else if is_key_pressed(KeyCode::Key2) {
                self.training_difficulty = TrainingDifficulty::Normal;
                diff_changed = true;
            } else if is_key_pressed(KeyCode::Key3) {
                self.training_difficulty = TrainingDifficulty::Extreme;
                diff_changed = true;
            }

            if diff_changed {
                self.training_score = 0;
                self.targets.clear();
                let aspect_ratio = screen_width() / screen_height();
                let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
                for _ in 0..3 {
                    self.targets.push(Target::spawn_random(virtual_width, self.training_difficulty, &self.wallmap));
                }
            }
        }

        let dt = get_frame_time().clamp(0.001, 0.05);
        let player_center = vec2(
            self.player.hitbox.x + self.player.hitbox.w / 2.0,
            self.player.hitbox.y + self.player.hitbox.h / 2.0,
        );
        self.camera_center = self.camera_center.lerp(player_center, 5.0 * dt);

        let camera = self.get_game_camera();

        if self.player.a_tire_cette_frame {
            self.pending_shot = true;
            self.pending_mouse_x = self.player.target_tir_cette_frame.x;
            self.pending_mouse_y = self.player.target_tir_cette_frame.y;
        }

        let messages = network.receive_messages();
        let mut client_states = Vec::new();

        for msg in messages {
            match msg {
                GameMessage::ClientUpdate(state) => {
                    if self.is_host { client_states.push(state); }
                }
                GameMessage::HostSync(json_str) => {
                    if !self.is_host { self.apply_network_json(&json_str,player_tex.clone()); }
                }
            }
        }

        if self.is_host && !client_states.is_empty() {
            self.sync_network(client_states,player_tex.clone());
        }

        ////////PARDON FAUT METTRE HITBOXES MUR LA DCP /////////
        //--- DEFINITION DES HITBOXES ----
        let aspect_ratio = screen_width() / screen_height();
        let virtual_width = VIRTUAL_HEIGHT * aspect_ratio; 
        let virtual_height = VIRTUAL_HEIGHT;

        let epaisseur = 50.0;
        let mur_gauche = Rect::new(-epaisseur, 0.0, epaisseur, virtual_height);
        let mur_droit  = Rect::new(virtual_width, 0.0, epaisseur, virtual_height);
        let mur_haut   = Rect::new(-epaisseur, -epaisseur, virtual_width + epaisseur * 2.0, epaisseur);
        let mur_bas    = Rect::new(-epaisseur, virtual_height, virtual_width + epaisseur * 2.0, epaisseur);

        // On les met dans un tableau pour les tester facilement
        let hitboxes_murs = vec![mur_gauche, mur_droit, mur_haut, mur_bas];
        ////////////////////// FIN PARDON ////////////////////////////////////
        


        if self.is_host{
            // 1. TOI tu tires sur les autres
            for proj in &mut self.player.projectiles {
                let was_exploding = proj.is_exploding;
                proj.update(dt, &self.wallmap, &hitboxes_murs, &mut self.other_players, None);
                if !was_exploding && proj.is_exploding {
                    self.explosion_particles.spawn_burst(vec2(proj.hitbox.x, proj.hitbox.y));
                }
            }
            self.player.projectiles.retain(|p| !p.is_dead());
            
            // 2. On CONFISQUE temporairement les projectiles de TOUS les autres joueurs
            // std::mem::take remplace leurs projectiles par une liste vide le temps du calcul
            let mut projectiles_des_autres: Vec<Vec<Projectile>> = self.other_players
                .iter_mut()
                .map(|p| std::mem::take(&mut p.projectiles))
                .collect();
            // 3. Maintenant que les joueurs n'ont plus leurs balles dans les poches, 
            // la liste `other_players` est totalement LIBRE ! On peut faire les calculs.
            for liste_projs in &mut projectiles_des_autres {
                for proj in liste_projs.iter_mut() {
                    // Les balles des autres peuvent toucher les autres, et l'hôte !
                    let was_exploding = proj.is_exploding;
                    proj.update(dt, &self.wallmap, &hitboxes_murs, &mut self.other_players, Some(&mut self.player));
                    if !was_exploding && proj.is_exploding {
                        self.explosion_particles.spawn_burst(vec2(proj.hitbox.x, proj.hitbox.y));
                    }
                }
                liste_projs.retain(|p| !p.is_dead());
            }
            // 4. On REND les balles à leurs propriétaires
            for (i, joueur) in self.other_players.iter_mut().enumerate() {
                joueur.projectiles = std::mem::take(&mut projectiles_des_autres[i]);
            }

            for other in &mut self.other_players {
                if other.pv <= 0.0 {
                    other.pv = 100.0;
                    other.hitbox.x = 20.0;
                    other.hitbox.y = 20.0;
                }
            }
        }
        self.player.update(&camera,&self.wallmap, &mut self.other_players,self.is_host);
        self.explosion_particles.update(dt, &self.wallmap);

        // --- LOGIQUE MODE ENTRAÎNEMENT ---
        if self.is_training_mode {
            let aspect_ratio = screen_width() / screen_height();
            let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;

            // 1. Update targets positions
            for target in &mut self.targets {
                target.update(dt, virtual_width, &self.wallmap);
            }

            // 2. Collision with projectiles
            for target in &mut self.targets {
                if target.is_destroyed {
                    continue;
                }

                // Check direct impacts (flying projectiles hit target)
                for proj in &mut self.player.projectiles {
                    if !proj.is_exploding && proj.hitbox.overlaps_rect(&target.hitbox) {
                        proj.explode();
                    }
                }

                for other in &mut self.other_players {
                    for proj in &mut other.projectiles {
                        if !proj.is_exploding && proj.hitbox.overlaps_rect(&target.hitbox) {
                            proj.explode();
                        }
                    }
                }

                // Check explosion hits
                for proj in &self.player.projectiles {
                    if proj.is_exploding && proj.hitbox.overlaps_rect(&target.hitbox) {
                        let proj_center = vec2(proj.hitbox.x, proj.hitbox.y);
                        if !target.hits_received.contains(&proj_center) {
                            target.hits_received.push(proj_center);
                            target.pv -= proj.degats;
                            if target.pv <= 0.0 {
                                target.is_destroyed = true;
                                self.training_score += 1;
                                self.explosion_particles.spawn_purple_burst(vec2(target.hitbox.x + target.hitbox.w / 2.0, target.hitbox.y + target.hitbox.h / 2.0));
                            }
                        }
                    }
                }

                for other in &self.other_players {
                    for proj in &other.projectiles {
                        if proj.is_exploding && proj.hitbox.overlaps_rect(&target.hitbox) {
                            let proj_center = vec2(proj.hitbox.x, proj.hitbox.y);
                            if !target.hits_received.contains(&proj_center) {
                                target.hits_received.push(proj_center);
                                target.pv -= proj.degats;
                                if target.pv <= 0.0 {
                                    target.is_destroyed = true;
                                    self.training_score += 1;
                                    self.explosion_particles.spawn_purple_burst(vec2(target.hitbox.x + target.hitbox.w / 2.0, target.hitbox.y + target.hitbox.h / 2.0));
                                }
                            }
                        }
                    }
                }
            }

            // 3. Respawn destroyed targets
            let current_diff = self.training_difficulty;
            self.targets.retain(|t| !t.is_destroyed);
            while self.targets.len() < 3 {
                self.targets.push(Target::spawn_random(virtual_width, current_diff, &self.wallmap));
            }
        }

        let time_now = macroquad::time::get_time();
        let network_tick_rate = 1.0 / 120.0;

        if time_now - self.last_network_send > network_tick_rate {
            self.last_network_send = time_now;

            if self.is_host {
                let json_state = self.generate_host_json();
                network.send_json(&json_state);
                self.pending_shot = false; 
            } else {
                let mut my_state = self.get_local_player_state(&camera);
                if self.pending_shot {
                    my_state.a_tire = true;
                    my_state.souris_x = self.pending_mouse_x;
                    my_state.souris_y = self.pending_mouse_y;
                    self.pending_shot = false;
                }
                network.send_state(&my_state);
            }
        }

    }




    pub fn draw(&mut self) {
        // --- CONFIGURATION CAMERA ---
        let aspect_ratio = screen_width() / screen_height();
        let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
        let camera = self.get_game_camera();

        // --- DESSIN DU MONDE (Avec la caméra) ---
        set_camera(&camera);
        clear_background(BLACK);

        // --- DESSIN DU BACKGROUND ---
        let bg_tex = if self.is_hollow_map { &self.hollow_background } else { &self.background };
        let dev_x = self.camera_center.x - virtual_width / 2.0;
        let dev_y = self.camera_center.y - VIRTUAL_HEIGHT / 2.0;
        let bg_x = dev_x * 0.4;
        let bg_y = dev_y * 0.4;
        draw_texture_ex(bg_tex, bg_x, bg_y, WHITE, DrawTextureParams { dest_size: Some(vec2(virtual_width, VIRTUAL_HEIGHT)), ..Default::default() });
        
        // --- DESSIN DES MURS (PLATES-FORMES TEXTURÉES AVEC 3-SLICING ET TILING PARFAIT) ---
        for wall in &self.wallmap {
            let cap_size = wall.h; // Embout carré (4.0 x 4.0)
            let tex_w = self.platform_tile.width();
            let tex_h = self.platform_tile.height();
            
            if wall.w <= cap_size * 2.0 {
                // Si la plateforme est trop petite, on dessine le bloc entier
                draw_texture_ex(
                    &self.platform_tile,
                    wall.x,
                    wall.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(wall.w, wall.h)),
                        ..Default::default()
                    }
                );
            } else {
                // 1. Embout Gauche (25% en haut à gauche de la texture, aspect 1:1)
                let left_src = Rect::new(0.0, 0.0, tex_w * 0.25, tex_h * 0.25);
                draw_texture_ex(
                    &self.platform_tile,
                    wall.x,
                    wall.y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(left_src),
                        dest_size: Some(vec2(cap_size, wall.h)),
                        ..Default::default()
                    }
                );

                // 2. Embout Droit (25% en haut à droite de la texture, aspect 1:1)
                let right_src = Rect::new(tex_w * 0.75, 0.0, tex_w * 0.25, tex_h * 0.25);
                draw_texture_ex(
                    &self.platform_tile,
                    wall.x + wall.w - cap_size,
                    wall.y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(right_src),
                        dest_size: Some(vec2(cap_size, wall.h)),
                        ..Default::default()
                    }
                );

                // 3. Remplissage du Milieu (Tiling de la section 25%-75% de la texture, aspect 2:1)
                let mut current_x = wall.x + cap_size;
                let end_x = wall.x + wall.w - cap_size;
                let tile_width = wall.h * 2.0; // Aspect ratio 2:1 pour la tuile du milieu (8.0 x 4.0)
                
                while current_x < end_x {
                    let draw_width = if current_x + tile_width > end_x {
                        end_x - current_x
                    } else {
                        tile_width
                    };
                    
                    let ratio = draw_width / tile_width;
                    let middle_src = Rect::new(
                        tex_w * 0.25,
                        0.0,
                        tex_w * 0.50 * ratio,
                        tex_h * 0.25
                    );
                    
                    draw_texture_ex(
                        &self.platform_tile,
                        current_x,
                        wall.y,
                        WHITE,
                        DrawTextureParams {
                            source: Some(middle_src),
                            dest_size: Some(vec2(draw_width, wall.h)),
                            ..Default::default()
                        }
                    );
                    
                    current_x += tile_width;
                }
            }
        }

        // --- DESSIN DES COLLIDERS DE DEBUG ---
        if self.debug_show_hitboxes {
            for wall in &self.wallmap {
                draw_rectangle(wall.x, wall.y, wall.w, wall.h, Color::new(0.0, 1.0, 0.0, 0.35)); // Vert translucide pour un debug propre !
            }
        }
        if self.is_training_mode {
            for target in &self.targets {
                target.draw(&self.fox_texture);
            }
        }

        // --- DESSIN DES JOUEURS ---
        self.player.draw();
        for player in &self.other_players {
            player.draw()
        }

        self.explosion_particles.draw();

        // --- DESSIN DE L'UI (Sans la caméra) ---
        set_default_camera();

        // Draw HUD help text
        let font_size = 20.0;
        let text_color = LIGHTGRAY;
        draw_text("Pressez [M] pour changer de carte | [F3] pour hitboxes | [F4] pour Mode Entraînement", 10.0, 30.0, font_size, text_color);
        
        if self.is_training_mode {
            let diff_str = match self.training_difficulty {
                TrainingDifficulty::Fixed => "Fixe",
                TrainingDifficulty::Normal => "Normal",
                TrainingDifficulty::Extreme => "Extrême",
            };
            draw_text(&format!("MODE ENTRAÎNEMENT ACTIF | Difficulté: {} | Score: {}", diff_str, self.training_score), 10.0, 55.0, font_size, Color::new(0.8, 0.4, 1.0, 1.0));
            draw_text("Changer Difficulté: [1] Fixe | [2] Normal | [3] Extrême", 10.0, 80.0, font_size, Color::new(0.7, 0.6, 0.9, 1.0));
        } else {
            draw_text("[Clic gauche] Tirer a la souris | Rockets : [T] haut [G] bas [F] gauche [H] droite | Le recul propulse !", 10.0, 55.0, font_size, Color::new(1.0, 0.8, 0.2, 1.0));
            if self.is_hollow_map {
                draw_text("Carte active : Hollow Knight (Pixel Art)", 10.0, 80.0, font_size, SKYBLUE);
            } else {
                draw_text("Carte active : Origine", 10.0, 80.0, font_size, ORANGE);
            }
        }
    }

    pub fn generate_host_json(&self) -> String {
        let mut net_players = Vec::new();

        let my_net_projs: Vec<NetworkProjectile> = self.player.projectiles.iter().map(|p| {
            NetworkProjectile { x: p.hitbox.x, y: p.hitbox.y, r: p.hitbox.r, is_exploding: p.is_exploding }
        }).collect();

        net_players.push(NetworkPlayer {
            id: self.player.id,
            x: self.player.hitbox.x,
            y: self.player.hitbox.y,
            pv: self.player.pv,
            aim_x: self.player.bazooka_dir.x,
            aim_y: self.player.bazooka_dir.y,
            projectiles: my_net_projs,
        });

        for other in &self.other_players {
            let other_net_projs: Vec<NetworkProjectile> = other.projectiles.iter().map(|p| {
                NetworkProjectile { x: p.hitbox.x, y: p.hitbox.y, r: p.hitbox.r, is_exploding: p.is_exploding }
            }).collect();
            
            net_players.push(NetworkPlayer {
                id: other.id,
                x: other.hitbox.x,
                y: other.hitbox.y,
                pv: other.pv,
                aim_x: other.bazooka_dir.x,
                aim_y: other.bazooka_dir.y,
                projectiles: other_net_projs,
            });
        }

        let state = NetworkGameState { players: net_players };
        serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn apply_network_json(&mut self, json_str: &str, player_tex: Texture2D) {
        if let Ok(state) = serde_json::from_str::<NetworkGameState>(json_str) {
            for net_p in state.players {
                if let Some(other) = self.other_players.iter_mut().find(|p| p.id == net_p.id) {
                    other.hitbox.x = net_p.x;
                    other.hitbox.y = net_p.y;
                    other.pv = net_p.pv;
                    other.bazooka_dir = vec2(net_p.aim_x, net_p.aim_y);
                    
                    let old_projectiles = std::mem::take(&mut other.projectiles);
                    for net_proj in net_p.projectiles {
                        let mut projectile_marionnette = Projectile::new(other.id, net_proj.x, net_proj.y, net_proj.x, net_proj.y);
                        projectile_marionnette.hitbox.x = net_proj.x;
                        projectile_marionnette.hitbox.y = net_proj.y;
                        
                        projectile_marionnette.hitbox.r = net_proj.r; 
                        projectile_marionnette.is_exploding = net_proj.is_exploding; 
                        
                        let was_exploding = old_projectiles.iter()
                            .find(|p| (p.hitbox.x - net_proj.x).abs() < 5.0 && (p.hitbox.y - net_proj.y).abs() < 5.0)
                            .map(|p| p.is_exploding)
                            .unwrap_or(false);

                        if !was_exploding && net_proj.is_exploding {
                            self.explosion_particles.spawn_burst(vec2(net_proj.x, net_proj.y));
                        }
                        
                        other.projectiles.push(projectile_marionnette);
                    }
                } else if net_p.id == self.player.id {
                    if self.player.pv < 100.0 && net_p.pv == 100.0 && net_p.x == 20.0 && net_p.y == 20.0 {
                        self.player.hitbox.x = 20.0;
                        self.player.hitbox.y = 20.0;
                    }
                    self.player.pv = net_p.pv;
                    
                    let old_projectiles = std::mem::take(&mut self.player.projectiles);
                    for net_proj in net_p.projectiles {
                        let mut projectile_marionnette = Projectile::new(self.player.id, net_proj.x, net_proj.y, net_proj.x, net_proj.y);
                        projectile_marionnette.hitbox.x = net_proj.x;
                        projectile_marionnette.hitbox.y = net_proj.y;

                        projectile_marionnette.hitbox.r = net_proj.r; 
                        projectile_marionnette.is_exploding = net_proj.is_exploding;
                        
                        let was_exploding = old_projectiles.iter()
                            .find(|p| (p.hitbox.x - net_proj.x).abs() < 5.0 && (p.hitbox.y - net_proj.y).abs() < 5.0)
                            .map(|p| p.is_exploding)
                            .unwrap_or(false);

                        if !was_exploding && net_proj.is_exploding {
                            self.explosion_particles.spawn_burst(vec2(net_proj.x, net_proj.y));
                        }
                        
                        self.player.projectiles.push(projectile_marionnette);
                    }
                }
                else {
                    // --- CHANGEMENT ICI : Le client crée l'Hôte s'il ne le connaît pas ---
                    let mut new_p = Player::new(player_tex.clone());
                    new_p.id = net_p.id;
                    new_p.hitbox.x = net_p.x;
                    new_p.hitbox.y = net_p.y;
                    new_p.pv = net_p.pv;
                    new_p.bazooka_dir = vec2(net_p.aim_x, net_p.aim_y);
                    self.other_players.push(new_p);
                }
            }
        }
    }

    pub fn get_local_player_state(&self, _camera: &Camera2D) -> PlayerState {
        let center_x = self.player.hitbox.x + self.player.hitbox.w / 2.0;
        let center_y = self.player.hitbox.y + self.player.hitbox.h / 2.0;
        let aim_target = vec2(center_x, center_y) + self.player.bazooka_dir * 100.0;
        PlayerState {
            id: self.player.id,
            x: self.player.hitbox.x,
            y: self.player.hitbox.y,
            a_tire: false, 
            souris_x: aim_target.x,
            souris_y: aim_target.y,
        }
    }


}
