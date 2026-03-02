use macroquad::prelude::*;

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32,
}

impl Projectile {
    pub fn new(start_x: f32, start_y: f32, target_x: f32, target_y: f32) -> Self {
        // 1. Calculer la différence entre la cible et le départ
        let dx = target_x - start_x;
        let dy = target_y - start_y;

        // 2. Calculer la longueur de ce vecteur
        let length = (dx * dx + dy * dy).sqrt();

        // 3. Normaliser la direction (diviser par la longueur) pour éviter que 
        // le projectile aille plus vite si on clique loin !
        let dir_x = if length > 0.0 { dx / length } else { 0.0 };
        let dir_y = if length > 0.0 { dy / length } else { 0.0 };

        Self {
            x: start_x,
            y: start_y,
            dir_x,
            dir_y,
            speed: 150.0, // Vitesse du projectile
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.dir_x * self.speed * dt;
        self.y += self.dir_y * self.speed * dt;
    }

    pub fn draw(&mut self) {
        // Pour commencer, on dessine juste un petit cercle jaune pour le test
        draw_circle(self.x, self.y, 2.0, YELLOW);
    }
}