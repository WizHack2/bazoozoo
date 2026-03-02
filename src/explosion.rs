use macroquad::prelude::*;

pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub timer: f32, // Un chronomètre pour savoir quand l'explosion est finie
}

impl Explosion {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, timer: 0.2 } // L'explosion dure 0.2 secondes
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