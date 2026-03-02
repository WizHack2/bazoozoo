use macroquad::prelude::*;

use crate::map_loading::charger_hitboxes;
use crate::player::*;
use crate::Assets;

pub const VIRTUAL_HEIGHT: f32 = 100.0;

pub fn get_camera() -> Camera2D {
    let aspect_ratio = screen_width() / screen_height();
    let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;
    Camera2D::from_display_rect(Rect::new(0.0, VIRTUAL_HEIGHT, virtual_width, -VIRTUAL_HEIGHT))
}

pub struct Game {
    pub background: Texture2D,
    pub player: Player,
    pub wallmap: Vec<Rect>
}

impl Game {
    pub fn new(assets: &Assets) -> Self {
        set_fullscreen(true);
        Self {
            background: assets.background.clone(),
            player: Player::new(assets.player.clone()),
            wallmap: charger_hitboxes("assets/map1.json".to_string())
        }
    }

    pub fn update(&mut self) {
        let camera = get_camera();
        self.player.update(&camera,&self.wallmap);
    }

    pub fn draw(&mut self) {
        // --- CONFIGURATION CAMERA ---
        let aspect_ratio = screen_width() / screen_height();
        let virtual_width = VIRTUAL_HEIGHT * aspect_ratio;

        let camera = Camera2D::from_display_rect(Rect::new(0.0, VIRTUAL_HEIGHT, virtual_width, -VIRTUAL_HEIGHT)); // Le 0 de la caméra est placé en bas a droite de l'écran pour qu'on garde une logiqe de y diminue quand on monte.
        // --- DESSIN DU MONDE (Avec la caméra) ---
        set_camera(&camera);
        
        clear_background(BLACK);
        draw_texture_ex(
            &self.background, 0., 0., WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(virtual_width, VIRTUAL_HEIGHT)),
                ..Default::default()
            }
        );
        for wall in self.wallmap.clone() {
            draw_rectangle(wall.x,wall.y, wall.w,wall.h, GRAY);
        }
        
        
        self.player.draw();

        // --- DESSIN DE L'UI (Sans la caméra) ---
        set_default_camera();
    }
}
