use macroquad::prelude::*;

mod game;
mod player;
mod boilerplate;
mod assets;
mod projectile;
mod map_loading;
mod menu;

use game::Game;
use assets::Assets;
use boilerplate::network::NetworkManager;

#[macroquad::main("Bazoozoo")]
async fn main() {
    // Création d'une seed pour la génération aléatoire
    macroquad::rand::srand(miniquad::date::now() as u64);
    
    // Chargement des assets
    let assets = Assets::load().await;

    // Récupération des arguments système pour préremplir le menu
    let args: Vec<String> = std::env::args().collect();
    let has_client_arg = args.contains(&"--client".to_string()); 
    let ip_arg = args.iter()
        .find(|arg| arg.starts_with("--ip="))
        .map(|arg| arg.trim_start_matches("--ip=").to_string());

    // --- ENTRÉE DANS LE MENU INTERACTIF DE SÉLECTION ---
    let mut menu_state = menu::MenuState::new();
    if has_client_arg {
        menu_state.role = menu::MenuRole::Client;
    }
    if let Some(ref ip) = ip_arg {
        menu_state.server_ip = ip.clone();
        menu_state.room_name = ip.clone();
    }

    while !menu_state.finished {
        menu_state.update();
        menu_state.draw(&assets);
        next_frame().await;
    }

    // --- LE MENU EST TERMINÉ, INITIALISATION DE LA PARTIE ---
    let is_host = menu_state.role == menu::MenuRole::Host;
    let mut game = Game::new(&assets, is_host, menu_state.pseudo.clone(), menu_state.character_id);

    // Connexion réseau à la salle choisie
    // Pour l'hôte, on se connecte toujours sur le serveur matchbox local (127.0.0.1)
    // Pour le client, on se connecte sur l'IP saisie de l'hôte
    let server_url = if is_host {
        format!("ws://127.0.0.1:3536/{}", menu_state.room_name)
    } else {
        format!("ws://{}:3536/{}", menu_state.server_ip, menu_state.room_name)
    };
    let mut network = NetworkManager::new(&server_url).await;
    
    loop {
        clear_background(BLACK);

        // On passe uniquement le réseau à l'update !
        game.update(&mut network);
        
        game.draw();

        next_frame().await;
    }
}
