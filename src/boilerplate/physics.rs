use macroquad::prelude::*;

pub struct Physics {
    velocity: Vec2,
    gravity_force: f32,
    friction_force: f32,
}

impl Physics {
    pub fn new(gravity_force: f32, friction_force: f32) -> Self {
        Self {
            velocity: Vec2::ZERO,
            gravity_force,
            friction_force,
        }
    }

    pub fn apply_physics(&mut self, hurtbox: &mut Rect) {
        let dt = get_frame_time().clamp(0.001, 0.05);
        hurtbox.x += self.velocity.x * dt;
        hurtbox.y += self.velocity.y * dt;
        self.apply_gravity();
        self.apply_friction();
    }

    #[allow(dead_code)]
    pub fn get_velocity(& self)-> f32{
        self.velocity.y
    }

    pub fn jump(&mut self, jump_force: f32) {
        self.velocity.y = -jump_force;
    }

    pub fn apply_gravity(&mut self) {
        let dt = get_frame_time().clamp(0.001, 0.05);
        self.velocity.y += self.gravity_force * dt;
    }

    pub fn apply_friction(&mut self) {
        let dt = get_frame_time().clamp(0.001, 0.05);
        let friction = self.friction_force * dt;

        if self.velocity.x == 0. {
            return;
        }
        
        // Slowing down
        if self.velocity.x > 0. {
            self.velocity.x -= friction;
            if self.velocity.x < 0. { self.velocity.x = 0.; }
        } else {
            self.velocity.x += friction;
            if self.velocity.x > 0. { self.velocity.x = 0.; }
        }
    }
    
    #[allow(dead_code)]
    pub fn set_velocity_x(&mut self, new_velocity: f32) {
        self.velocity.x = new_velocity;
    }

    #[allow(dead_code)]
    pub fn add_velocity_x(&mut self, added_velocity: f32) {
        self.velocity.x += added_velocity;
    }

    pub fn set_velocity_y(&mut self, new_velocity: f32) {
        self.velocity.y = new_velocity;
    }

    /// Ajoute une impulsion externe à la vélocité actuelle (utilisé par le rocket jump)
    pub fn add_velocity(&mut self, impulse: Vec2) {
        self.velocity += impulse;
    }

    /// Ajoute uniquement un boost vertical (pratique pour garantir la hauteur du rocket jump)
    #[allow(dead_code)]
    pub fn add_velocity_y(&mut self, added_velocity: f32) {
        self.velocity.y += added_velocity;
    }

    /// Retourne le vecteur de vélocité actuel (debug / réseau)
    #[allow(dead_code)]
    pub fn get_velocity_vec(&self) -> Vec2 {
        self.velocity
    }

    #[allow(dead_code)]
    pub fn set_gravity(&mut self, new_gravity: f32) {
        self.gravity_force = new_gravity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump() {
        let mut phys = Physics::new(200.0, 250.0);
        phys.jump(100.0);
        assert_eq!(phys.get_velocity_vec().y, -100.0);
    }

    #[test]
    fn test_set_velocity_y() {
        let mut phys = Physics::new(200.0, 250.0);
        phys.set_velocity_y(50.0);
        assert_eq!(phys.get_velocity_vec().y, 50.0);
    }

    #[test]
    fn test_add_velocity() {
        let mut phys = Physics::new(200.0, 250.0);
        phys.add_velocity(vec2(10.0, 20.0));
        assert_eq!(phys.get_velocity_vec().x, 10.0);
        assert_eq!(phys.get_velocity_vec().y, 20.0);
    }

    #[test]
    fn test_add_velocity_x() {
        let mut phys = Physics::new(200.0, 250.0);
        phys.set_velocity_y(0.0);
        phys.add_velocity_x(15.0);
        assert_eq!(phys.get_velocity_vec().x, 15.0);
    }

    #[test]
    fn test_set_gravity() {
        let mut phys = Physics::new(200.0, 250.0);
        assert_eq!(phys.gravity_force, 200.0);
        phys.set_gravity(500.0);
        assert_eq!(phys.gravity_force, 500.0);
    }
}

