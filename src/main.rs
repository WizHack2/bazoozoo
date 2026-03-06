use macroquad::prelude::*;

mod game;
mod player;
mod boilerplate;
mod assets;
mod projectile;
mod map_loading;

use game::Game;
use assets::Assets;
use boilerplate::network::NetworkManager;
use boilerplate::network::PlayerState;
use boilerplate::network::GameMessage;

#[macroquad::main("My Game")]
async fn main() {
    // Création d'une seed pour la génération aléatoire
    macroquad::rand::srand(miniquad::date::now() as u64);
    
    // Chargement des assets
    let assets = Assets::load().await;

    // Init de la partie
    //TODO un menu pour choisir si on est host ou client et entrer l'url de la salle
    let is_host = false; 
    let mut game = Game::new(&assets, is_host);

    // Init connexion partie
    let mut network = NetworkManager::new("ws://127.0.0.1:3536/salle_privee").await;
    
    loop {
        clear_background(BLACK);

        // On passe le réseau et la texture du joueur à l'update !
        game.update(&mut network, assets.player.clone());
        
        game.draw();

        next_frame().await;
    }
}
