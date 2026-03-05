use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::map_loading::charger_hitboxes;
use crate::player;
use crate::player::*;
use crate::Assets;
use crate::boilerplate::network::PlayerState;
use crate::projectile::Projectile;

pub const VIRTUAL_HEIGHT: f32 = 100.0;

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkProjectile {
    pub x: f32,
    pub y: f32,
}

// On modifie NetworkPlayer pour qu'il ait des poches pleines de missiles
#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkPlayer {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub pv: f32,
    pub projectiles: Vec<NetworkProjectile>, // <-- NOUVEAU : La liste des missiles !
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkGameState {
    pub players: Vec<NetworkPlayer>,
}

pub fn get_camera() -> Camera2D {
    let aspect_ratio = screen_width() / screen_height();
    let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
    Camera2D::from_display_rect(Rect::new(0.0, VIRTUAL_HEIGHT, virtual_width, -VIRTUAL_HEIGHT))
}

pub struct Game {
    pub background: Texture2D,
    pub player: Player,
    pub wallmap: Vec<Rect>,
    pub other_players: Vec<Player>,
    pub is_host :bool
}

impl Game {
    pub fn new(assets: &Assets,is_host1:bool) -> Self {
        set_fullscreen(true);
        Self {
            background: assets.background.clone(),
            player: Player::new(assets.player.clone()),
            wallmap: charger_hitboxes("assets/map2.json".to_string()),
            other_players: Vec::new(),
            is_host : is_host1
        }
    }

    pub fn add_player(&mut self,Player_a_ajouter:Player){
        if self.other_players.len()>3{
            println!("erreur nombre max de joueur atteint");
        }
        else{
        self.other_players.push(Player_a_ajouter);
        }
    }

    pub fn sync_network(&mut self, states: Vec<PlayerState>, player_tex: Texture2D) {
        for state in states {
            if let Some(p) = self.other_players.iter_mut().find(|p| p.id == state.id) {
                p.hitbox.x = state.x;
                p.hitbox.y = state.y;
            } else {
                let mut new_p = Player::new(player_tex.clone());
                new_p.id = state.id;
                new_p.hitbox.x = state.x;
                new_p.hitbox.y = state.y;
                self.other_players.push(new_p);
            }
        }
    }

    pub fn update(&mut self) {
        let camera = get_camera();

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
        


        let dt = get_frame_time().clamp(0.001, 0.05);
        if self.is_host{
            //TODO ajouter condition sur provenance du projectile pour eviter le tir alié
            // 1. TOI tu tires sur les autres (Ça, ça ne pose aucun problème)
            self.player.update_projectile(&self.wallmap, &hitboxes_murs, &mut self.other_players, dt);
            // 2. On CONFISQUE temporairement les projectiles de TOUS les autres joueurs
            // std::mem::take remplace leurs projectiles par une liste vide le temps du calcul
            let mut projectiles_des_autres: Vec<Vec<Projectile>> = self.other_players
                .iter_mut()
                .map(|p| std::mem::take(&mut p.projectiles))
                .collect();
            // 3. Maintenant que les joueurs n'ont plus leurs balles dans les poches, 
            // la liste `other_players` est totalement LIBRE ! On peut faire les calculs.
            for liste_projs in &mut projectiles_des_autres {
                for proj in liste_projs {
                    // Les balles des autres peuvent toucher les autres
                    proj.update(dt, &self.wallmap, &hitboxes_murs, &mut self.other_players);
                }
            }
            // 4. On REND les balles à leurs propriétaires
            for (i, joueur) in self.other_players.iter_mut().enumerate() {
                joueur.projectiles = std::mem::take(&mut projectiles_des_autres[i]);
            }
        }
        self.player.update(&camera,&self.wallmap, &mut self.other_players);


        //TODO Gerer l'import et l'export des json avec les fonctions generate json et apply json

    }




    pub fn draw(&mut self) {
        // --- CONFIGURATION CAMERA ---
        let aspect_ratio = screen_width() / screen_height();
        let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
        let camera = Camera2D::from_display_rect(Rect::new(0.0, VIRTUAL_HEIGHT, virtual_width, -VIRTUAL_HEIGHT)); // Le 0 de la caméra est placé en bas a droite de l'écran pour qu'on garde une logiqe de y diminue quand on monte.

        // --- DESSIN DU MONDE (Avec la caméra) ---
        set_camera(&camera);
        clear_background(BLACK);

        // --- DESSIN DU BACKGROUND ---
        draw_texture_ex(&self.background, 0., 0., WHITE,DrawTextureParams {dest_size: Some(vec2(virtual_width, VIRTUAL_HEIGHT)),..Default::default()});
        // --- DESSIN DES MURS ---
        for wall in &self.wallmap { draw_rectangle(wall.x,wall.y, wall.w,wall.h, GRAY); }
        // --- DESSIN DES JOUEURS ---
        self.player.draw();
        for player in &self.other_players{
            player.draw()
        }

        // --- DESSIN DE L'UI (Sans la caméra) ---
        set_default_camera();
    }

    pub fn generate_host_json(&self) -> String {
        let mut net_players = Vec::new();

        let my_net_projs: Vec<NetworkProjectile> = self.player.projectiles.iter().map(|p| {
            NetworkProjectile { x: p.hitbox.x, y: p.hitbox.y }
        }).collect();

        net_players.push(NetworkPlayer {
            id: self.player.id,
            x: self.player.hitbox.x,
            y: self.player.hitbox.y,
            pv: self.player.PV,
            projectiles: my_net_projs,
        });

        for other in &self.other_players {
            let other_net_projs: Vec<NetworkProjectile> = other.projectiles.iter().map(|p| {
                NetworkProjectile { x: p.hitbox.x, y: p.hitbox.y }
            }).collect();

            net_players.push(NetworkPlayer {
                id: other.id,
                x: other.hitbox.x,
                y: other.hitbox.y,
                pv: other.PV,
                projectiles: other_net_projs,
            });
        }

        let state = NetworkGameState { players: net_players };
        serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn apply_network_json(&mut self, json_str: &str) {
        if let Ok(state) = serde_json::from_str::<NetworkGameState>(json_str) {
            for net_p in state.players {
                if let Some(other) = self.other_players.iter_mut().find(|p| p.id == net_p.id) {
                    other.hitbox.x = net_p.x;
                    other.hitbox.y = net_p.y;
                    other.PV = net_p.pv;
                    
                    other.projectiles.clear();
                    
                    for net_proj in net_p.projectiles {
                        let mut projectile_marionnette = Projectile::new(other.id, net_proj.x, net_proj.y, net_proj.x, net_proj.y);
                        projectile_marionnette.hitbox.x = net_proj.x;
                        projectile_marionnette.hitbox.y = net_proj.y;
                        other.projectiles.push(projectile_marionnette);
                    }
                } else if net_p.id == self.player.id {
                    self.player.PV = net_p.pv;
                }
            }
        }
    }

}
