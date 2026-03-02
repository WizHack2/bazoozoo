use macroquad::prelude::*;


pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32,
}

impl Projectile {
    // Calcul de la direction vers la souris
    pub fn new(start_x: f32, start_y: f32, target_x: f32, target_y: f32) -> Self {
        let dx = target_x - start_x;
        let dy = target_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();

        // Normalisation (pour que la balle aille toujours à la même vitesse)
        let dir_x = if length > 0.0 { dx / length } else { 0.0 };
        let dir_y = if length > 0.0 { dy / length } else { 0.0 };

        Self {
            x: start_x,
            y: start_y,
            dir_x,
            dir_y,
            speed: 150.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.dir_x * self.speed * dt;
        self.y += self.dir_y * self.speed * dt;
    }

    pub fn draw(&mut self) {
        // On dessine un petit trait jaune pour faire style "laser" ou un cercle
        draw_circle(self.x, self.y, 1.0, YELLOW);
    }

    pub fn est_hors_ecran(&self, largeur_ecran: f32, hauteur_ecran: f32) -> bool {
        // La balle est hors écran si elle dépasse les bords (gauche, droite, bas, haut)
        self.x < 0.0 || self.x > largeur_ecran || self.y < 0.0 || self.y > hauteur_ecran
    }

    pub fn get_hitbox(&self) -> Rect {
        let taille = 2.0; // La largeur/hauteur de la hitbox de ta balle
        Rect::new(
            self.x - (taille / 2.0), // On centre la hitbox sur x
            self.y - (taille / 2.0), // On centre la hitbox sur y
            taille,
            taille
        )
    }
}