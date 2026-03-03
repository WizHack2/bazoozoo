use macroquad::prelude::Rect;
use serde::Deserialize;
use std::{fs, path};

// On reproduit exactement la structure du JSON !
#[derive(Deserialize)]
struct MurJson {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Deserialize)]
struct MapJson {
    murs: Vec<MurJson>,
}

// Fonction qui lit le fichier et renvoie un tableau de hitboxes Macroquad
pub fn charger_hitboxes(path:String) -> Vec<Rect> {
    // 1. Lire le fichier texte
    let fichier_json = fs::read_to_string(path)
        .expect("Impossible de trouver le fichier map.json !");

    // 2. Transformer le texte en structure Rust grâce à serde
    let map_data: MapJson = serde_json::from_str(&fichier_json)
        .expect("Le JSON est mal formaté !");

    // 3. Convertir nos "MurJson" en vrais "Rect" de Macroquad
    let mut hitboxes = Vec::new();
    for mur in map_data.murs {
        hitboxes.push(Rect::new(mur.x, mur.y, mur.w, mur.h));
    }

    hitboxes
}
