use macroquad::prelude::*;

mod game;
mod player;
mod boilerplate;
mod assets;
mod projectile;
mod map_loading;

use game::Game;
use assets::Assets;

#[macroquad::main("My Game")]
async fn main() {
    // Création d'une seed pour la génération aléatoire
    macroquad::rand::srand(miniquad::date::now() as u64);
    
    // Chargement des assets
    let assets = Assets::load().await;

    // Init de la partie
    let mut game = Game::new(&assets);
    
    loop {
        clear_background(BLACK);

        game.update();
        game.draw();

        next_frame().await;
    }
}
