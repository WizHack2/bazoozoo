use macroquad::prelude::*;
use crate::player::Player;

pub struct Projectile {
    pub owner_id: i32,
    pub hitbox: Circle,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32,
    pub is_exploding: bool,
    pub explosion_duration: f32,
    pub degats: f32,
    pub players_already_damaged: Vec<i32>,
    pub is_mega: bool,
}

impl Projectile {
    pub fn new(owner_id: i32, start_x: f32, start_y: f32, target_x: f32, target_y: f32) -> Self {
        let dx = target_x - start_x;
        let dy = target_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();

        Self {
            owner_id,
            hitbox: Circle::new(start_x, start_y, 1.0), // Petit rayon en vol
            dir_x: if length > 0.0 { dx / length } else { 0.0 },
            dir_y: if length > 0.0 { dy / length } else { 0.0 },
            speed: crate::constants::PROJECTILE_SPEED,
            is_exploding: false,
            explosion_duration: 0.2,
            degats: 10.0,
            players_already_damaged: Vec::new(),
            is_mega: false,
        }
    }

    pub fn new_mega(owner_id: i32, start_x: f32, start_y: f32) -> Self {
        let mut p = Self::new(owner_id, start_x, start_y, start_x, start_y);
        p.is_mega = true;
        p.degats = 100.0; // Mort instantanée !
        p.is_exploding = false; // Initialisé à false pour déclencher la condition !was_exploding && is_exploding
        p.speed = 0.0;
        p.explosion_duration = 1.0; // Dure 1 seconde
        p
    }

    pub fn update(&mut self, dt: f32, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, autres_joueurs: &mut Vec<Player>, mut local_player: Option<&mut Player>) {
        if self.is_mega && !self.is_exploding {
            self.explode();
        }
        let kills = self.check_collisions(
            wallmap,
            hitboxes_murs,
            autres_joueurs,
            match local_player {
                Some(ref mut p) => Some(*p),
                None => None,
            }
        );
        if kills > 0 {
            let o_id = self.owner_id;
            if let Some(ref mut lp) = local_player {
                if lp.id == o_id {
                    lp.score += kills;
                }
            }
            for p in autres_joueurs.iter_mut() {
                if p.id == o_id {
                    p.score += kills;
                    break;
                }
            }
        }
        if self.is_exploding {
            self.explosion_duration -= dt;
            // Le rayon change pendant l'explosion
            if self.is_mega {
                self.hitbox.r = (1.0 - self.explosion_duration) * 800.0; // Couvre toute la map
            } else {
                self.hitbox.r = self.explosion_duration * 50.0; 
            }
        } else {
            self.hitbox.x += self.dir_x * self.speed * dt;
            self.hitbox.y += self.dir_y * self.speed * dt;
        }
    }

    pub fn draw(&self) {
        if !self.is_exploding {
            draw_circle(self.hitbox.x, self.hitbox.y, self.hitbox.r, YELLOW);
        }
    }

    pub fn check_collisions(&mut self, wallmap: &Vec<Rect>, hitboxes_murs: &Vec<Rect>, joueurs: &mut Vec<Player>, mut local_player: Option<&mut Player>) -> i32 {
        let mut kills = 0;
        if self.is_exploding {
            // Phase d'explosion : infliger les dégâts de zone
            for joueur in joueurs.iter_mut() {
                if self.hitbox.overlaps_rect(&joueur.hitbox) {
                    if !self.players_already_damaged.contains(&joueur.id) && joueur.id != self.owner_id { 
                        self.players_already_damaged.push(joueur.id);
                        let was_alive = joueur.pv > 0.0;
                        joueur.take_damage(self.degats);
                        if was_alive && joueur.pv <= 0.0 {
                            kills += 1;
                        }
                    }
                }
            }
            if let Some(ref mut lp) = local_player {
                if self.hitbox.overlaps_rect(&lp.hitbox) {
                    if !self.players_already_damaged.contains(&lp.id) && lp.id != self.owner_id {
                        self.players_already_damaged.push(lp.id);
                        let was_alive = lp.pv > 0.0;
                        lp.take_damage(self.degats);
                        if was_alive && lp.pv <= 0.0 {
                            kills += 1;
                        }
                    }
                }
            }
        } else {
            // Phase de vol : détecter l'impact pour exploser
            let touche_map = wallmap.iter().any(|wall| self.hitbox.overlaps_rect(wall));
            let touche_mur = hitboxes_murs.iter().any(|mur| self.hitbox.overlaps_rect(mur));
            let touche_joueur = joueurs.iter().any(|j| j.id != self.owner_id && self.hitbox.overlaps_rect(&j.hitbox));
            let touche_local = local_player.as_ref().map_or(false, |lp| lp.id != self.owner_id && self.hitbox.overlaps_rect(&lp.hitbox));

            if touche_mur || touche_map || touche_joueur || touche_local {
                self.explode();
            }
        }
        kills
    }

    pub fn explode(&mut self) {
        if !self.is_exploding {
            self.is_exploding = true;
            self.speed = 0.0; // Le projectile s'arrête
        }
    }

    pub fn is_dead(&self) -> bool {
        self.is_exploding && self.explosion_duration <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projectile_new() {
        let p = Projectile::new(1, 10.0, 20.0, 30.0, 40.0);
        assert_eq!(p.owner_id, 1);
        assert!(!p.is_exploding);
        assert!(!p.is_mega);
        assert_eq!(p.speed, crate::constants::PROJECTILE_SPEED);
        assert_eq!(p.degats, 10.0);
    }

    #[test]
    fn test_projectile_new_mega() {
        let p = Projectile::new_mega(2, 50.0, 60.0);
        assert_eq!(p.owner_id, 2);
        assert!(!p.is_exploding);
        assert!(p.is_mega);
        assert_eq!(p.degats, 100.0);
        assert_eq!(p.speed, 0.0);
    }

    #[test]
    fn test_projectile_is_dead_not_exploding() {
        let p = Projectile::new(1, 0.0, 0.0, 10.0, 10.0);
        assert!(!p.is_dead());
    }

    #[test]
    fn test_projectile_is_dead_exploding_expired() {
        let mut p = Projectile::new(1, 0.0, 0.0, 10.0, 10.0);
        p.is_exploding = true;
        p.explosion_duration = 0.0;
        assert!(p.is_dead());
    }

    #[test]
    fn test_projectile_is_dead_exploding_active() {
        let mut p = Projectile::new(1, 0.0, 0.0, 10.0, 10.0);
        p.is_exploding = true;
        p.explosion_duration = 0.1;
        assert!(!p.is_dead());
    }
}