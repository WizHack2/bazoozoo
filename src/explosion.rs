use macroquad::prelude::*;

pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub timer: f32, // Un chronomètre pour savoir quand l'explosion est finie
    pub degats: f32,               // Combien de PV elle enlève
    pub a_fait_des_degats: bool,   // La fameuse "mémoire"
    pub rayon_max: f32,
}

impl Explosion {
    pub fn new(x: f32, y: f32) -> Self {
        Self { 
            x, 
            y, 
            timer: 0.2, 
            degats: 5.0, // L'explosion enlève 5 PV
            a_fait_des_degats: false, // Au début, elle n'a touché personne
            rayon_max: 15.0, // La taille de l'explosion
        } 
    }

    pub fn get_hitbox(&self) -> Rect {
        Rect::new(
            self.x - self.rayon_max,
            self.y - self.rayon_max,
            self.rayon_max * 2.0,
            self.rayon_max * 2.0
        )
    }

    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
    }

    pub fn draw(&mut self) {
        // En attendant ton vrai sprite d'explosion, on dessine un cercle rouge qui rétrécit !
        let rayon = self.timer * 50.0;
        if rayon > 0.0 {
            draw_circle(self.x, self.y, rayon, RED);
        }
    }
}