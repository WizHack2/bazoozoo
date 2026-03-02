use macroquad::prelude::*;
use crate::explosion::{self, Explosion};
use crate::projectile::Projectile;
use crate::{assets, boilerplate::animation::Animation};
use crate::boilerplate::physics::Physics;
use crate::game::VIRTUAL_HEIGHT;

pub struct Player {
    animation: Animation,
    hitbox:Rect,
    pub speed: f32,
    pub liste_projectiles: Vec<Projectile>,
    pub explosions: Vec<Explosion>,
    PV : f32,
    physics : Physics,
    jump_available: i32
    
}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            speed: 50.0,
            hitbox: Rect::new(0.0,0.0,10.0,10.0),
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]),
            liste_projectiles: Vec::new(),
            explosions: Vec::new(),
            PV : 25.,
            physics : Physics::new(50., 0.5),
            jump_available: 2
         }

    }

    pub fn take_damage(&mut self,val:f32){
        if self.PV - val < 0. {
            self.PV = 0.;
        }
        else{
            self.PV -= val;
        }
    }

    pub fn heal(&mut self,val:f32){
        if self.PV + val > 100. {
            self.PV = 100.;
        }
        else{
            self.PV += val;
        }
    }


    fn tirer_projectile(&mut self, camera: &Camera2D) {
        
        let mouse_pos = mouse_position();
        let world_mouse = camera.screen_to_world(vec2(mouse_pos.0, mouse_pos.1));
        
        // On centre le départ du tir (ajuste les + 5.0 selon la taille de ton sprite)
        let nouveau_tir = Projectile::new(self.hitbox.x + 5.0, self.hitbox.y + 5.0, world_mouse.x, world_mouse.y);
        
        self.liste_projectiles.push(nouveau_tir);
    }

    pub fn verifier_degats_explosions(&mut self, explosions_actives: &mut Vec<Explosion>) {
        for explosion in explosions_actives {
            // Si l'explosion n'a pas encore touché CE joueur
            if !explosion.a_fait_des_degats {
                if self.hitbox.overlaps(&explosion.get_hitbox()) {
                    
                    self.take_damage(explosion.degats);
                    
                    // On marque l'explosion pour dire qu'elle a déjà frappé
                    explosion.a_fait_des_degats = true;
                }
            }
        }
    }

    pub fn update(&mut self, camera: &Camera2D, wallmap:&Vec<Rect>) {

        ////////////////////////////////////////////////////////////////////////////////////////////////A SUPPRIMER V FINALE
        if is_key_pressed(KeyCode::P){
            self.heal(5.);
        }
        if is_key_pressed(KeyCode::M){
            self.take_damage(5.);
        }
        //////////////////////////////////////////////////////////////////////////////////////////////////




        //println!("{:?}",self.physics.get_velocity());
        let dt = get_frame_time().clamp(0.001, 0.05);

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
                self.physics.jump(50.);
                self.jump_available -= 1;
                }
            
        }
        
        //if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { self.hitbox.y += self.speed * dt; }

        //--- GRAVITE ---
        let old_y = self.hitbox.y;
        self.physics.apply_physics(&mut self.hitbox);
        let dy = self.hitbox.y - old_y; //Pour calculer la direction de la collision (si on tombe ou si on monte)


        // --- LOGIQUE DE TIR ---
        if is_mouse_button_pressed(MouseButton::Left) {
            self.tirer_projectile(camera);
        }

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

        //--- COLISION
        if self.hitbox.y > VIRTUAL_HEIGHT - self.hitbox.h {
            self.hitbox.y = VIRTUAL_HEIGHT - self.hitbox.h;
            self.physics.set_velocity_y(0.);
            self.jump_available = 2;
        }

        for wall in wallmap {
            if self.hitbox.overlaps(wall) {
                ///let vy = self.physics.get_velocity();
                if dy > 0.0 {
                    // On tombe. On se pose PILE sur le mur.
                    self.hitbox.y = wall.y - self.hitbox.h;
                    self.physics.set_velocity_y(0.0);
                    self.jump_available = 2; // BINGO ! On récupère nos sauts ici !
                } else if dy < 0.0 {
                    // On monte (on se cogne la tête). On se colle PILE sous le mur.
                    self.hitbox.y = wall.y + wall.h;
                    self.physics.set_velocity_y(0.0);
                }
            }
        }

        
        // --- MISE À JOUR DES PROJECTILES ---
        for proj in &mut self.liste_projectiles {
            proj.update(dt);
        }

        

        // 2. ÉTAPE A : Détecter les collisions avec Macroquad (overlaps)
        for proj in &self.liste_projectiles {
            let hitbox_proj = proj.get_hitbox();
            
            // On vérifie si la hitbox du projectile touche (overlaps) UNE des hitboxes des murs ou de la map
            let a_touche_la_map : bool = wallmap.iter().any(|wall| hitbox_proj.overlaps(wall));
            let a_touche_un_mur = hitboxes_murs.iter().any(|mur| hitbox_proj.overlaps(mur));

            if a_touche_un_mur || a_touche_la_map {
                self.explosions.push(Explosion::new(proj.x, proj.y));
            }
        }

        // 3. ÉTAPE B : La méthode magique RETAIN avec les hitboxes
        // On garde uniquement les projectiles dont la hitbox NE TOUCHE PAS les murs
        self.liste_projectiles.retain(|proj| {
            let hitbox_proj = proj.get_hitbox();
            !hitboxes_murs.iter().any(|mur| hitbox_proj.overlaps(mur))
        });
        self.liste_projectiles.retain(|proj| {
            let hitbox_proj = proj.get_hitbox();
            !wallmap.iter().any(|mur| hitbox_proj.overlaps(mur))
        });

        for explosion in &mut self.explosions{
            explosion.update(dt);
        }
        
        let mut mes_explosions = std::mem::take(&mut self.explosions);
        self.verifier_degats_explosions(&mut mes_explosions);
        self.explosions = mes_explosions;

        self.explosions.retain(|expl| expl.timer > 0.0);

    }

    pub fn draw_healthbar(& self){
        let width:f32 = 6.;
        
        draw_rectangle(self.hitbox.x + self.hitbox.w/2. - width/2., self.hitbox.y + self.hitbox.h + 0.2, width*self.PV/100., 0.3, GREEN);
        draw_rectangle(self.hitbox.x + width*self.PV/100. + self.hitbox.w/2. - width/2., self.hitbox.y + self.hitbox.h + 0.2, width*(100.-self.PV)/100., 0.3, RED);
    }



    pub fn draw(&mut self) {
        self.animation.draw_current_frame(self.hitbox.x, self.hitbox.y, 10., 10., true);
        self.draw_healthbar();
        
        for proj in &mut self.liste_projectiles {
            proj.draw();
        }

        for explosion in &mut self.explosions{
            explosion.draw()
        }
    }
}
