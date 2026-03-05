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

#[macroquad::main("My Game")]
async fn main() {
    // Création d'une seed pour la génération aléatoire
    macroquad::rand::srand(miniquad::date::now() as u64);
    
    // Chargement des assets
    let assets = Assets::load().await;

    // Init de la partie
    let mut game = Game::new(&assets,true);

    // Init connexion partie
    let mut network = NetworkManager::new("ws://127.0.0.1:3536/salle_privee").await;
    
    loop {
        clear_background(BLACK);

        let my_state = PlayerState { 
            id: game.player.id, 
            x: game.player.hitbox.x, 
            y: game.player.hitbox.y 
        };
        
        network.send_state(&my_state);
        let received_states = network.update_and_receive();
        game.sync_network(received_states, assets.player.clone());

        game.update();
        game.draw();

        next_frame().await;
    }
}
