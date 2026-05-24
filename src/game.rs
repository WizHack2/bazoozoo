use macroquad::prelude::*;
use macroquad::audio::{Sound, play_sound_once};

use crate::map_loading::charger_hitboxes;
use crate::player::Player;
use crate::Assets;
use crate::projectile::Projectile;
use crate::particle::ParticleManager;
use crate::boilerplate::network::{PlayerState, NetworkManager, GameMessage, NetworkProjectile, NetworkPlayer, NetworkGameState};
use crate::constants::VIRTUAL_HEIGHT;
use crate::keybindings::{KeyBindings, Layout};
use crate::target::{Target, TrainingDifficulty};

pub struct GameResources {
    pub background: Texture2D,
    pub hollow_background: Texture2D,
    pub platform_tile: Texture2D,
    pub fox_texture: Texture2D,
    pub player_textures: Vec<Texture2D>,
    pub sound_shoot: Sound,
    pub sound_explosion: Sound,
    pub sound_jump: Sound,
    pub sound_land: Sound,
    pub sound_reload: Sound,
}

impl GameResources {
    pub fn from_assets(assets: &Assets) -> Self {
        Self {
            background: assets.background.clone(),
            hollow_background: assets.hollow_background.clone(),
            platform_tile: assets.platform_tile.clone(),
            fox_texture: assets.fox.clone(),
            player_textures: vec![
                assets.player.clone(),
                assets.fox.clone(),
                assets.shadow.clone(),
            ],
            sound_shoot: assets.sound_shoot.clone(),
            sound_explosion: assets.sound_explosion.clone(),
            sound_jump: assets.sound_jump.clone(),
            sound_land: assets.sound_land.clone(),
            sound_reload: assets.sound_reload.clone(),
        }
    }
}

pub struct GameTraining {
    pub is_active: bool,
    pub difficulty: TrainingDifficulty,
    pub score: i32,
    pub targets: Vec<Target>,
}

impl GameTraining {
    pub fn new() -> Self {
        Self {
            is_active: false,
            difficulty: TrainingDifficulty::Normal,
            score: 0,
            targets: Vec::new(),
        }
    }
}

pub struct Game {
    pub res: GameResources,
    pub is_hollow_map: bool,
    pub debug_show_hitboxes: bool,
    pub player: Player,
    pub wallmap: Vec<Rect>,
    pub other_players: Vec<Player>,
    pub is_host: bool,
    pub last_network_send: f64,
    pub pending_shot: bool,
    pub pending_mega: bool,
    pub pending_mouse_x: f32,
    pub pending_mouse_y: f32,
    pub particles: ParticleManager,
    pub training: GameTraining,
    pub camera_center: Vec2,
    pub join_notification_timer: f32,
    pub layout: Layout,
    pub keybindings: KeyBindings,
    pub last_peer_count: usize,
    pub waiting_for_players_timer: f64,
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

    pub fn new(assets: &Assets, is_host1: bool, pseudo: String, character_id: u8, layout: Layout) -> Self {
        set_fullscreen(true);
        let kb = KeyBindings::from_layout(layout);
        let res = GameResources::from_assets(assets);
        
        let chosen_tex = res.player_textures[character_id as usize].clone();
        let mut player = Player::new(chosen_tex);
        player.pseudo = pseudo;
        player.character_id = character_id;
        
        let stats = &crate::constants::CHARACTERS[character_id as usize];
        player.speed = stats.speed;
        player.max_ammo = stats.max_ammo;
        player.current_ammo = stats.max_ammo;

        Self {
            res,
            is_hollow_map: true,
            debug_show_hitboxes: false,
            player,
            wallmap: charger_hitboxes("assets/hollow_map.json".to_string()),
            other_players: Vec::new(),
            is_host: is_host1,

            last_network_send: macroquad::time::get_time(),
            pending_shot: false,
            pending_mega: false,
            pending_mouse_x: 0.0,
            pending_mouse_y: 0.0,
            particles: ParticleManager::new(),
            training: GameTraining::new(),
            camera_center: vec2(20.0, 20.0),
            join_notification_timer: 0.0,
            layout,
            keybindings: kb,
            last_peer_count: 0,
            waiting_for_players_timer: 0.0,
        }
    }

    pub fn sync_network(&mut self, states: Vec<PlayerState>) {
        for state in states {
            if let Some(p) = self.other_players.iter_mut().find(|p| p.id == state.id) {
                p.last_seen = macroquad::time::get_time();
                p.hitbox.x = state.x;
                p.hitbox.y = state.y;
                p.pseudo = state.pseudo.clone();
                p.character_id = state.character_id;

                // Si la texture est différente de celle actuelle, on la recharge
                let chosen_tex = self.res.player_textures[p.character_id as usize].clone();
                p.animation = crate::boilerplate::animation::Animation::new(Some(chosen_tex), 2, 1, vec![0]);
                
                // Update aiming direction from souris coordinates if it is not a mega sentinel
                if state.souris_x < 90000.0 {
                    let center_x = p.hitbox.x + p.hitbox.w / 2.0;
                    let center_y = p.hitbox.y + p.hitbox.h / 2.0;
                    let to_target = vec2(state.souris_x - center_x, state.souris_y - center_y);
                    if to_target.length() > 0.01 {
                        p.bazooka_dir = to_target.normalize();
                    }
                }
                
                // --- NOUVEAU : L'HÔTE CRÉE LE TIR DU CLIENT ---
                if state.a_tire {
                    let center_x = p.hitbox.x + p.hitbox.w / 2.0;
                    let center_y = p.hitbox.y + p.hitbox.h / 2.0;
                    
                    let nouveau_proj = if state.souris_x > 90000.0 && state.souris_y > 90000.0 {
                        Projectile::new_mega(
                            state.id,
                            center_x,
                            center_y,
                        )
                    } else {
                        // On décale le point de spawn dans la direction de visée pour éviter une auto-collision immédiate avec le sol/murs
                        let spawn_x = center_x + p.bazooka_dir.x * 6.0;
                        let spawn_y = center_y + p.bazooka_dir.y * 6.0;
                        
                        Projectile::new(
                            state.id, 
                            spawn_x, 
                            spawn_y, 
                            state.souris_x, 
                            state.souris_y
                        )
                    };
                    p.projectiles.push(nouveau_proj);
                }

            } else {
                let chosen_tex = self.res.player_textures[state.character_id as usize].clone();
                let mut new_p = Player::new(chosen_tex);
                new_p.id = state.id;
                new_p.pseudo = state.pseudo.clone();
                new_p.character_id = state.character_id;
                new_p.hitbox.x = state.x;
                new_p.hitbox.y = state.y;

                let stats = &crate::constants::CHARACTERS[state.character_id as usize];
                new_p.speed = stats.speed;
                new_p.max_ammo = stats.max_ammo;

                let now = macroquad::time::get_time();
                new_p.last_seen = now;
                self.other_players.push(new_p);
                self.join_notification_timer = 3.0;
            }
        }
    }

    pub fn update_player_colors(&mut self) {
        // Collect all players (local player + other players)
        let mut players: Vec<&mut Player> = Vec::new();
        players.push(&mut self.player);
        for p in &mut self.other_players {
            players.push(p);
        }
        
        // Sort deterministically by ID
        players.sort_by_key(|p| p.id);

        let colors = crate::constants::PLAYER_COLORS;

        for (i, p) in players.into_iter().enumerate() {
            let color = colors[i % colors.len()];
            p.color = color;
            p.animation.change_color(color);
        }
    }

    fn handle_debug_toggles(&mut self) {
        if is_key_pressed(KeyCode::F3) {
            self.debug_show_hitboxes = !self.debug_show_hitboxes;
        }
        if is_key_pressed(KeyCode::M) {
            self.is_hollow_map = !self.is_hollow_map;
            self.wallmap = if self.is_hollow_map {
                charger_hitboxes("assets/hollow_map.json".to_string())
            } else {
                charger_hitboxes("assets/map2.json".to_string())
            };
        }
    }

    fn handle_training_mode(&mut self, dt: f32) {
        let virtual_width = VIRTUAL_HEIGHT * screen_width() / screen_height();

        if is_key_pressed(KeyCode::F4) {
            self.training.is_active = !self.training.is_active;
            if self.training.is_active {
                self.training.score = 0;
                self.training.targets.clear();
                for _ in 0..3 {
                    self.training.targets.push(Target::spawn_random(virtual_width, self.training.difficulty, &self.wallmap));
                }
            } else {
                self.training.targets.clear();
            }
        }

        if self.training.is_active {
            let mut diff_changed = false;
            if is_key_pressed(KeyCode::Key1) {
                self.training.difficulty = TrainingDifficulty::Fixed;
                diff_changed = true;
            } else if is_key_pressed(KeyCode::Key2) {
                self.training.difficulty = TrainingDifficulty::Normal;
                diff_changed = true;
            } else if is_key_pressed(KeyCode::Key3) {
                self.training.difficulty = TrainingDifficulty::Extreme;
                diff_changed = true;
            }
            if diff_changed {
                self.training.score = 0;
                self.training.targets.clear();
                for _ in 0..3 {
                    self.training.targets.push(Target::spawn_random(virtual_width, self.training.difficulty, &self.wallmap));
                }
            }

            self.update_training_targets(dt, virtual_width);
        }
    }

    fn update_training_targets(&mut self, dt: f32, virtual_width: f32) {
        for target in &mut self.training.targets {
            target.update(dt, virtual_width, &self.wallmap);
        }
        for target in &mut self.training.targets {
            if target.is_destroyed { continue; }
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
            for proj in &self.player.projectiles {
                if proj.is_exploding && proj.hitbox.overlaps_rect(&target.hitbox) {
                    let proj_center = vec2(proj.hitbox.x, proj.hitbox.y);
                    if !target.hits_received.contains(&proj_center) {
                        target.hits_received.push(proj_center);
                        target.pv -= proj.degats;
                        if target.pv <= 0.0 {
                            target.is_destroyed = true;
                            self.training.score += 1;
                            self.particles.spawn_purple_burst(vec2(target.hitbox.x + target.hitbox.w / 2.0, target.hitbox.y + target.hitbox.h / 2.0));
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
                                self.training.score += 1;
                                self.particles.spawn_purple_burst(vec2(target.hitbox.x + target.hitbox.w / 2.0, target.hitbox.y + target.hitbox.h / 2.0));
                            }
                        }
                    }
                }
            }
        }
        let current_diff = self.training.difficulty;
        self.training.targets.retain(|t| !t.is_destroyed);
        while self.training.targets.len() < 3 {
            self.training.targets.push(Target::spawn_random(virtual_width, current_diff, &self.wallmap));
        }
    }

    fn update_camera_and_timers(&mut self, dt: f32) {
        if self.join_notification_timer > 0.0 {
            self.join_notification_timer -= dt;
        }
        let player_center = vec2(
            self.player.hitbox.x + self.player.hitbox.w / 2.0,
            self.player.hitbox.y + self.player.hitbox.h / 2.0,
        );
        self.camera_center = self.camera_center.lerp(player_center, 5.0 * dt);
    }

    fn handle_pending_shots(&mut self) {
        if self.player.a_tire_cette_frame {
            self.pending_shot = true;
            self.pending_mouse_x = self.player.target_tir_cette_frame.x;
            self.pending_mouse_y = self.player.target_tir_cette_frame.y;
            if self.player.a_tire_mega_cette_frame {
                self.pending_mega = true;
                self.player.a_tire_mega_cette_frame = false;
            }
        }
    }

    fn cleanup_stale_players(&mut self) {
        let now = macroquad::time::get_time();
        self.other_players.retain(|p| {
            p.last_seen == 0.0 || now - p.last_seen < 5.0
        });
    }

    fn handle_network_messages(&mut self, network: &mut NetworkManager) {
        let messages = network.receive_messages();
        let mut client_states = Vec::new();
        for msg in messages {
            match msg {
                GameMessage::ClientUpdate(state) => {
                    if self.is_host { client_states.push(state); }
                }
                GameMessage::HostSync(json_str) => {
                    if !self.is_host { self.apply_network_json(&json_str); }
                }
            }
        }
        if self.is_host && !client_states.is_empty() {
            self.sync_network(client_states);
        }

        self.last_peer_count = network.peer_count();
        self.waiting_for_players_timer += get_frame_time() as f64;
    }

    fn update_host_projectiles(&mut self, dt: f32, hitboxes_murs: &Vec<Rect>) {
        if !self.is_host { return; }

        for proj in &mut self.player.projectiles {
            let was_exploding = proj.is_exploding;
            proj.update(dt, &self.wallmap, hitboxes_murs, &mut self.other_players, None);
            if !was_exploding && proj.is_exploding {
                if proj.is_mega {
                    self.particles.spawn_mega_burst(vec2(proj.hitbox.x, proj.hitbox.y), self.player.color);
                } else {
                    self.particles.spawn_burst(vec2(proj.hitbox.x, proj.hitbox.y));
                }
                play_sound_once(&self.res.sound_explosion);
            }
        }
        self.player.projectiles.retain(|p| !p.is_dead());

        let mut projectiles_des_autres: Vec<Vec<Projectile>> = self.other_players
            .iter_mut()
            .map(|p| std::mem::take(&mut p.projectiles))
            .collect();
        for (i, liste_projs) in projectiles_des_autres.iter_mut().enumerate() {
            let color = self.other_players[i].color;
            for proj in liste_projs.iter_mut() {
                let was_exploding = proj.is_exploding;
                proj.update(dt, &self.wallmap, hitboxes_murs, &mut self.other_players, Some(&mut self.player));
                if !was_exploding && proj.is_exploding {
                    if proj.is_mega {
                        self.particles.spawn_mega_burst(vec2(proj.hitbox.x, proj.hitbox.y), color);
                    } else {
                        self.particles.spawn_burst(vec2(proj.hitbox.x, proj.hitbox.y));
                    }
                    play_sound_once(&self.res.sound_explosion);
                }
            }
            liste_projs.retain(|p| !p.is_dead());
        }
        for (i, joueur) in self.other_players.iter_mut().enumerate() {
            joueur.projectiles = std::mem::take(&mut projectiles_des_autres[i]);
        }
    }

    fn update_other_players_death(&mut self, dt: f32) {
        for other in &mut self.other_players {
            if other.pv <= 0.0 && other.death_timer <= 0.0 {
                other.death_timer = 1.0;
            }
            if other.death_timer > 0.0 {
                other.death_timer -= dt;
                if macroquad::rand::gen_range(0, 4) == 0 {
                    use macroquad::rand::gen_range;
                    let spawn_pos = vec2(
                        other.hitbox.x + gen_range(0.0, other.hitbox.w),
                        other.hitbox.y + other.hitbox.h,
                    );
                    let velocity = vec2(gen_range(-3.0, 3.0), gen_range(-25.0, -10.0));
                    let p_color = Color::new(other.color.r, other.color.g, other.color.b, 0.7);
                    other.particles.spawn(spawn_pos, velocity, p_color, gen_range(0.8, 1.6), gen_range(0.5, 1.0));
                }
                if other.death_timer <= 0.0 {
                    other.pv = crate::constants::PLAYER_MAX_PV;
                    other.hitbox.x = crate::constants::PLAYER_RESPAWN_X;
                    other.hitbox.y = crate::constants::PLAYER_RESPAWN_Y;
                }
            }
            other.particles.update(dt, &self.wallmap);
        }
    }

    fn send_network_state(&mut self, network: &mut NetworkManager, camera: &Camera2D) {
        let time_now = macroquad::time::get_time();
        if time_now - self.last_network_send > crate::constants::NETWORK_TICK_RATE {
            self.last_network_send = time_now;
            if self.is_host {
                let json_state = self.generate_host_json();
                network.send_json(&json_state);
                self.pending_shot = false;
            } else {
                let mut my_state = self.get_local_player_state(camera);
                if self.pending_shot {
                    my_state.a_tire = true;
                    my_state.souris_x = self.pending_mouse_x;
                    my_state.souris_y = self.pending_mouse_y;
                    self.pending_shot = false;
                }
                self.pending_mega = false;
                network.send_state(&my_state);
            }
        }
    }

    pub fn update(&mut self, network: &mut NetworkManager) {
        self.update_player_colors();
        self.handle_debug_toggles();

        let dt = get_frame_time().clamp(crate::constants::DT_CLAMP_MIN, crate::constants::DT_CLAMP_MAX);

        self.handle_training_mode(dt);
        self.update_camera_and_timers(dt);
        self.handle_network_messages(network);

        let aspect_ratio = screen_width() / screen_height();
        let virtual_height = VIRTUAL_HEIGHT;
        let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;

        let hitboxes_murs = vec![
            Rect::new(-crate::constants::WALL_THICKNESS, 0.0, crate::constants::WALL_THICKNESS, virtual_height),
            Rect::new(virtual_width, 0.0, crate::constants::WALL_THICKNESS, virtual_height),
            Rect::new(-crate::constants::WALL_THICKNESS, -crate::constants::WALL_THICKNESS, virtual_width + crate::constants::WALL_THICKNESS * 2.0, crate::constants::WALL_THICKNESS),
            Rect::new(-crate::constants::WALL_THICKNESS, virtual_height, virtual_width + crate::constants::WALL_THICKNESS * 2.0, crate::constants::WALL_THICKNESS),
        ];

        self.update_host_projectiles(dt, &hitboxes_murs);
        self.update_other_players_death(dt);

        let camera = self.get_game_camera();
        self.player.update(&camera, &self.wallmap, &mut self.other_players, self.is_host,
            &self.res.sound_shoot, &self.res.sound_jump, &self.res.sound_land, &self.res.sound_reload, &self.keybindings);
        self.handle_pending_shots();
        self.cleanup_stale_players();
        self.particles.update(dt, &self.wallmap);

        self.send_network_state(network, &camera);
    }




    fn draw_world(&self, camera: &Camera2D, virtual_width: f32) {
        set_camera(camera);
        clear_background(BLACK);

        let bg_tex = if self.is_hollow_map { &self.res.hollow_background } else { &self.res.background };
        let dev_x = self.camera_center.x - virtual_width / 2.0;
        let dev_y = self.camera_center.y - VIRTUAL_HEIGHT / 2.0;
        let bg_x = dev_x * crate::constants::PARALLAX_FACTOR;
        let bg_y = dev_y * 0.4;
        draw_texture_ex(bg_tex, bg_x, bg_y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(virtual_width, VIRTUAL_HEIGHT)),
            ..Default::default()
        });
    }

    fn draw_walls(&self) {
        for wall in &self.wallmap {
            let cap_size = wall.h;
            let tex_w = self.res.platform_tile.width();
            let tex_h = self.res.platform_tile.height();

            if wall.w <= cap_size * 2.0 {
                draw_texture_ex(&self.res.platform_tile, wall.x, wall.y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(wall.w, wall.h)),
                    ..Default::default()
                });
            } else {
                let left_src = Rect::new(0.0, 0.0, tex_w * 0.25, tex_h * 0.25);
                draw_texture_ex(&self.res.platform_tile, wall.x, wall.y, WHITE, DrawTextureParams {
                    source: Some(left_src),
                    dest_size: Some(vec2(cap_size, wall.h)),
                    ..Default::default()
                });

                let right_src = Rect::new(tex_w * 0.75, 0.0, tex_w * 0.25, tex_h * 0.25);
                draw_texture_ex(&self.res.platform_tile, wall.x + wall.w - cap_size, wall.y, WHITE, DrawTextureParams {
                    source: Some(right_src),
                    dest_size: Some(vec2(cap_size, wall.h)),
                    ..Default::default()
                });

                let mut current_x = wall.x + cap_size;
                let end_x = wall.x + wall.w - cap_size;
                let tile_width = wall.h * 2.0;

                while current_x < end_x {
                    let draw_width = if current_x + tile_width > end_x { end_x - current_x } else { tile_width };
                    let ratio = draw_width / tile_width;
                    let middle_src = Rect::new(tex_w * 0.25, 0.0, tex_w * 0.50 * ratio, tex_h * 0.25);
                    draw_texture_ex(&self.res.platform_tile, current_x, wall.y, WHITE, DrawTextureParams {
                        source: Some(middle_src),
                        dest_size: Some(vec2(draw_width, wall.h)),
                        ..Default::default()
                    });
                    current_x += tile_width;
                }
            }
        }
    }

    fn draw_debug_overlay(&self) {
        if self.debug_show_hitboxes {
            for wall in &self.wallmap {
                draw_rectangle(wall.x, wall.y, wall.w, wall.h, Color::new(0.0, 1.0, 0.0, 0.35));
            }
        }
        if self.training.is_active {
            for target in &self.training.targets {
                target.draw(&self.res.fox_texture);
            }
        }
    }

    fn draw_players(&self) {
        self.player.draw();
        for player in &self.other_players {
            player.draw();
        }
        self.particles.draw();
    }

    fn draw_ui(&self) {
        set_default_camera();
        self.draw_pseudo_labels();
        self.draw_join_notification();
        self.draw_connection_status();
        self.draw_scoreboard();
        self.draw_hud();
    }

    fn draw_connection_status(&self) {
        let status_y = 110.0;
        if self.last_peer_count > 0 {
            let text = if self.is_host {
                format!("✓ {} joueur(s) connecté(s)", self.last_peer_count + 1)
            } else {
                "✓ Connecté au serveur".to_string()
            };
            draw_text(&text, 10.0, status_y, 16.0, GREEN);
        } else if self.waiting_for_players_timer > 5.0 {
            let msg = if self.is_host {
                "⚠ Aucun joueur connecté — partagez votre IP à vos coéquipiers"
            } else {
                "⚠ Impossible de joindre le serveur — vérifiez l'IP saisie"
            };
            draw_text(msg, 10.0, status_y, 16.0, RED);
        } else if self.waiting_for_players_timer > 1.0 {
            draw_text("◌ Connexion en cours...", 10.0, status_y, 16.0, ORANGE);
        }
    }

    fn draw_pseudo_labels(&self) {
        let camera = self.get_game_camera();
        let draw_one = |p: &Player| {
            if p.death_timer > 0.0 { return; }
            let world_pos = vec2(p.hitbox.x + p.hitbox.w / 2.0, p.hitbox.y - 1.8);
            let screen_pos = camera.world_to_screen(world_pos);
            let text_w = measure_text(&p.pseudo, None, 16, 1.0).width;

            draw_rectangle(screen_pos.x - text_w / 2.0 - 5.0, screen_pos.y - 12.0, text_w + 10.0, 16.0, Color::new(0.02, 0.02, 0.03, 0.7));
            draw_rectangle_lines(screen_pos.x - text_w / 2.0 - 5.0, screen_pos.y - 12.0, text_w + 10.0, 16.0, 1.0, p.color);
            draw_text(&p.pseudo, screen_pos.x - text_w / 2.0, screen_pos.y, 16.0, WHITE);
        };
        draw_one(&self.player);
        for p in &self.other_players {
            draw_one(p);
        }
    }

    fn draw_join_notification(&self) {
        if self.join_notification_timer <= 0.0 { return; }
        let progress = (self.join_notification_timer / 3.0).clamp(0.0, 1.0);
        let alpha = if progress > 0.8 {
            (1.0 - progress) / 0.2
        } else if progress < 0.2 {
            progress / 0.2
        } else {
            1.0
        };
        let screen_w = screen_width();
        let banner_w = 400.0;
        let banner_h = 45.0;
        let banner_x = (screen_w - banner_w) / 2.0;
        let banner_y = 20.0;

        draw_rectangle(banner_x, banner_y, banner_w, banner_h, Color::new(0.04, 0.04, 0.06, alpha * 0.90));
        draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h, 2.0, Color::new(0.25, 0.60, 0.95, alpha));
        draw_text("UN JOUEUR A REJOINT LA PARTIE !", banner_x + 45.0, banner_y + 28.0, 20.0, Color::new(0.3, 0.8, 1.0, alpha));
        draw_circle(banner_x + 25.0, banner_y + 22.0, 5.0, Color::new(0.2, 0.9, 1.0, alpha));
        draw_circle(banner_x + banner_w - 25.0, banner_y + 22.0, 5.0, Color::new(0.2, 0.9, 1.0, alpha));
    }

    fn draw_scoreboard(&self) {
        let mut all_players: Vec<&Player> = Vec::new();
        all_players.push(&self.player);
        for p in &self.other_players {
            all_players.push(p);
        }
        all_players.sort_by(|a, b| b.score.cmp(&a.score));

        let sb_w = 280.0;
        let sb_x = screen_width() - sb_w - 30.0;
        let sb_y = 30.0;
        let sb_h = 60.0 + (all_players.len() as f32 * 35.0);

        draw_rectangle(sb_x, sb_y, sb_w, sb_h, Color::new(0.04, 0.04, 0.06, 0.92));
        draw_rectangle_lines(sb_x, sb_y, sb_w, sb_h, 3.0, Color::new(0.25, 0.25, 0.30, 1.0));
        draw_rectangle_lines(sb_x + 3.0, sb_y + 3.0, sb_w - 6.0, sb_h - 6.0, 1.0, Color::new(0.12, 0.12, 0.15, 1.0));
        draw_text("TABLEAU DES SCORES", sb_x + 32.0, sb_y + 31.0, 22.0, WHITE);
        draw_line(sb_x + 15.0, sb_y + 40.0, sb_x + sb_w - 15.0, sb_y + 40.0, 2.0, Color::new(0.35, 0.35, 0.40, 1.0));

        for (i, p) in all_players.iter().enumerate() {
            let text = format!("{}: {}", p.pseudo, p.score);
            let item_y = sb_y + 70.0 + (i as f32 * 35.0);
            draw_circle(sb_x + 30.0, item_y - 7.0, 7.0, p.color);
            draw_circle(sb_x + 30.0, item_y - 7.0, 3.0, WHITE);
            draw_text(&text, sb_x + 50.0, item_y, 22.0, p.color);
        }
    }

    fn draw_hud(&self) {
        let font_size = 20.0;
        draw_text("Pressez [M] pour changer de carte | [F3] pour hitboxes | [F4] pour Mode Entraînement", 10.0, 30.0, font_size, LIGHTGRAY);

        if self.training.is_active {
            let diff_str = match self.training.difficulty {
                TrainingDifficulty::Fixed => "Fixe",
                TrainingDifficulty::Normal => "Normal",
                TrainingDifficulty::Extreme => "Extrême",
            };
            draw_text(&format!("MODE ENTRAÎNEMENT ACTIF | Difficulté: {} | Score: {}", diff_str, self.training.score), 10.0, 55.0, font_size, Color::new(0.8, 0.4, 1.0, 1.0));
            draw_text("Changer Difficulté: [1] Fixe | [2] Normal | [3] Extrême", 10.0, 80.0, font_size, Color::new(0.7, 0.6, 0.9, 1.0));
        } else {
            let move_keys = match self.layout {
                Layout::Azerty => "ZQSD",
                Layout::Qwerty => "WASD",
            };
            draw_text(&format!("[Clic gauche] Tirer | {}/Fleches : Deplacement | Rockets : [T]^ [G]v [F]< [H]> | Le recul propulse !", move_keys), 10.0, 55.0, font_size, Color::new(1.0, 0.8, 0.2, 1.0));
            draw_text(if self.is_hollow_map { "Carte active : Hollow Knight (Pixel Art)" } else { "Carte active : Origine" }, 10.0, 80.0, font_size, if self.is_hollow_map { SKYBLUE } else { ORANGE });
        }
    }

    pub fn draw(&mut self) {
        self.update_player_colors();
        let virtual_width = VIRTUAL_HEIGHT * screen_width() / screen_height();
        let camera = self.get_game_camera();

        self.draw_world(&camera, virtual_width);
        self.draw_walls();
        self.draw_debug_overlay();
        self.draw_players();
        self.draw_ui();
    }

    fn player_to_network_player(player: &Player) -> NetworkPlayer {
        let projectiles: Vec<NetworkProjectile> = player.projectiles.iter().map(|p| {
            NetworkProjectile { x: p.hitbox.x, y: p.hitbox.y, r: p.hitbox.r, is_exploding: p.is_exploding, is_mega: p.is_mega }
        }).collect();
        NetworkPlayer {
            id: player.id,
            x: player.hitbox.x,
            y: player.hitbox.y,
            pv: player.pv,
            aim_x: player.bazooka_dir.x,
            aim_y: player.bazooka_dir.y,
            score: player.score,
            projectiles,
            pseudo: player.pseudo.clone(),
            character_id: player.character_id,
        }
    }

    pub fn generate_host_json(&self) -> String {
        let mut net_players = Vec::new();
        net_players.push(Self::player_to_network_player(&self.player));
        for other in &self.other_players {
            net_players.push(Self::player_to_network_player(other));
        }
        let state = NetworkGameState { players: net_players };
        serde_json::to_string(&state).unwrap_or_else(|e| {
            eprintln!("Warning: failed to serialize NetworkGameState: {}", e);
            "{}".to_string()
        })
    }

    fn reconcile_projectiles(particles: &mut ParticleManager, player: &mut Player, net_projs: Vec<NetworkProjectile>, color: Color) {
        let old_projectiles = std::mem::take(&mut player.projectiles);
        for net_proj in net_projs {
            let mut marionnette = Projectile::new(player.id, net_proj.x, net_proj.y, net_proj.x, net_proj.y);
            marionnette.hitbox.x = net_proj.x;
            marionnette.hitbox.y = net_proj.y;
            marionnette.hitbox.r = net_proj.r;
            marionnette.is_exploding = net_proj.is_exploding;
            marionnette.is_mega = net_proj.is_mega;

            let was_exploding = old_projectiles.iter()
                .find(|p| (p.hitbox.x - net_proj.x).abs() < 5.0 && (p.hitbox.y - net_proj.y).abs() < 5.0)
                .map(|p| p.is_exploding)
                .unwrap_or(false);

            if !was_exploding && net_proj.is_exploding {
                if net_proj.is_mega {
                    particles.spawn_mega_burst(vec2(net_proj.x, net_proj.y), color);
                } else {
                    particles.spawn_burst(vec2(net_proj.x, net_proj.y));
                }
            }
            player.projectiles.push(marionnette);
        }
    }

    pub fn apply_network_json(&mut self, json_str: &str) {
        if let Ok(state) = serde_json::from_str::<NetworkGameState>(json_str) {
            for net_p in state.players {
                if let Some(other) = self.other_players.iter_mut().find(|p| p.id == net_p.id) {
                    other.last_seen = macroquad::time::get_time();
                    other.hitbox.x = net_p.x;
                    other.hitbox.y = net_p.y;
                    other.pv = net_p.pv;
                    other.score = net_p.score;
                    other.bazooka_dir = vec2(net_p.aim_x, net_p.aim_y);
                    other.pseudo = net_p.pseudo.clone();
                    other.character_id = net_p.character_id;

                    let chosen_tex = self.res.player_textures[other.character_id as usize].clone();
                    other.animation = crate::boilerplate::animation::Animation::new(Some(chosen_tex), 2, 1, vec![0]);

                    Self::reconcile_projectiles(&mut self.particles, other, net_p.projectiles, other.color);
                } else if net_p.id == self.player.id {
                    if self.player.pv < 100.0 && net_p.pv == 100.0 && net_p.x == 20.0 && net_p.y == 20.0 {
                        self.player.hitbox.x = 20.0;
                        self.player.hitbox.y = 20.0;
                    }
                    self.player.pv = net_p.pv;
                    self.player.score = net_p.score;

                    let color = self.player.color;
                    Self::reconcile_projectiles(&mut self.particles, &mut self.player, net_p.projectiles, color);
                } else {
                    let chosen_tex = self.res.player_textures[net_p.character_id as usize].clone();
                    let mut new_p = Player::new(chosen_tex);
                    new_p.id = net_p.id;
                    new_p.pseudo = net_p.pseudo.clone();
                    new_p.character_id = net_p.character_id;
                    new_p.hitbox.x = net_p.x;
                    new_p.hitbox.y = net_p.y;
                    new_p.pv = net_p.pv;
                    new_p.score = net_p.score;
                    new_p.bazooka_dir = vec2(net_p.aim_x, net_p.aim_y);

                    let stats = &crate::constants::CHARACTERS[net_p.character_id as usize];
                    new_p.speed = stats.speed;
                    new_p.max_ammo = stats.max_ammo;

                    new_p.last_seen = macroquad::time::get_time();
                    self.other_players.push(new_p);
                    self.join_notification_timer = 3.0;
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
            pseudo: self.player.pseudo.clone(),
            character_id: self.player.character_id,
        }
    }


}


