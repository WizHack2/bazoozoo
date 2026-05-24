use macroquad::prelude::*;

pub struct Assets {
    pub background: Texture2D,
    pub hollow_background: Texture2D,
    pub platform_tile: Texture2D,
    pub player: Texture2D,
    pub fox: Texture2D,
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

        Self {
            background,
            hollow_background,
            platform_tile,
            player,
            fox,
        }
    }
}
