use macroquad::prelude::*;

pub struct Assets {
    pub background: Texture2D,
    pub hollow_background: Texture2D,
    pub platform_tile: Texture2D,
    pub player: Texture2D,
    pub fox: Texture2D,
    pub shadow: Texture2D,
    // Ajoute des futurs boss ici
}

impl Assets {
    pub async fn load() -> Self {
        let background = load_texture("assets/background.png").await.unwrap();
        background.set_filter(FilterMode::Nearest);
        
        let hollow_background = load_texture("assets/hollow_map.png").await.unwrap();
        hollow_background.set_filter(FilterMode::Nearest);

        let platform_tile = load_texture("assets/platform_tile.png").await.unwrap();
        platform_tile.set_filter(FilterMode::Nearest);

        let player = load_texture("assets/Asterion.png").await.unwrap();
        player.set_filter(FilterMode::Nearest);

        let fox = load_texture("assets/fox.png").await.unwrap();
        fox.set_filter(FilterMode::Nearest);

        // Création du personnage "Shadow" par décalage de couleur
        let player_img = load_image("assets/Asterion.png").await.unwrap();
        let shadow = create_shadow_texture(&player_img);

        Self {
            background,
            hollow_background,
            platform_tile,
            player,
            fox,
            shadow,
        }
    }
}

fn create_shadow_texture(img: &Image) -> Texture2D {
    let mut shadow_image = img.clone();
    for y in 0..shadow_image.height() as u32 {
        for x in 0..shadow_image.width() as u32 {
            let color = shadow_image.get_pixel(x, y);
            if color.a > 0.0 {
                // Si la couleur est dans les tons rouges/orangés, on la passe en violet/indigo
                if color.r > 0.4 && color.g < 0.8 && color.b < 0.8 {
                    let new_color = Color::new(color.r * 0.5, color.g * 0.25, color.r * 0.9, color.a);
                    shadow_image.set_pixel(x, y, new_color);
                } else if color.r > 0.2 && color.g > 0.2 && color.b > 0.2 && (color.r - color.g).abs() < 0.25 && (color.g - color.b).abs() < 0.25 {
                    // Les tons grisâtres (armure/habits) passent en gris-violet sombre
                    let new_color = Color::new(color.r * 0.45, color.g * 0.4, color.b * 0.75, color.a);
                    shadow_image.set_pixel(x, y, new_color);
                }
            }
        }
    }
    let texture = Texture2D::from_image(&shadow_image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

