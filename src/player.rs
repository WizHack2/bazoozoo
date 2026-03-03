use macroquad::prelude::*;
use crate::projectile::Projectile;
use crate::{assets, boilerplate::animation::Animation};
use crate::boilerplate::physics::Physics;
use crate::game::VIRTUAL_HEIGHT;

pub struct Player {
    pub id: i32,
    animation: Animation,
    pub hitbox:Rect,
    pub speed: f32,
    pub projectiles: Vec<Projectile>,
    PV : f32,
    physics : Physics,
    jump_available: i32,
}

impl Player {
    pub fn new(spritesheet: Texture2D) -> Self {
        Self {
            id: 0,
            speed: 50.0,
            hitbox: Rect::new(0.0,0.0,10.0,10.0),
            animation: Animation::new(Some(spritesheet), 2, 1, vec![0]),
            projectiles: Vec::new(),
            PV : 25.,
            physics : Physics::new(200., 0.5),
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
    }


    pub fn update(&mut self, camera: &Camera2D, wallmap:&Vec<Rect>, joueurs: &mut Vec<Player>) {
        let dt = get_frame_time().clamp(0.001, 0.05);

        self.handle_input(dt, wallmap);
        
        







        //// NE PAS REGARDER EN DESSOUS. C"EST DEGEULASSE TODO A EXPLOSER SA GRAND MERE

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

        for proj in &mut self.projectiles {
            proj.update(dt, wallmap, &hitboxes_murs, joueurs);
        }
    }

    pub fn draw_healthbar(& self){
        let width:f32 = 6.;
        
        draw_rectangle(self.hitbox.x + self.hitbox.w/2. - width/2., self.hitbox.y + self.hitbox.h + 0.2, width*self.PV/100., 0.3, GREEN);
        draw_rectangle(self.hitbox.x + width*self.PV/100. + self.hitbox.w/2. - width/2., self.hitbox.y + self.hitbox.h + 0.2, width*(100.-self.PV)/100., 0.3, RED);
    }

    pub fn draw(&self) {
        self.animation.draw_current_frame(self.hitbox.x, self.hitbox.y, 10., 10., true);
        self.draw_healthbar();
        
        for projectile in &self.projectiles {
            projectile.draw();
        }
    }
}
