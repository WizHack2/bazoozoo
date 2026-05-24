# Bazoozoo

Arena multiplayer 2D rocket-jumping en Rust/Macroquad.

Deux joueurs s'affrontent dans une arène en vue de dessus avec des bazookas à propulsion. Le recul des roquettes permet de se déplacer et de faire des rocket jumps.

## Personnages

| Personnage | Classe | Vitesse | Munitions | Description |
|-----------|--------|---------|-----------|-------------|
| Asterion | Guerrier | 40 | 3 | Équilibré |
| Fox | Agile | 48 | 2 | Très rapide |
| Shadow | Lourd | 34 | 4 | Robuste |

## Contrôles

| Touche | Action |
|--------|--------|
| ZQSD / Flèches | Déplacement |
| Espace / Z / ↑ | Saut (double saut) |
| Clic gauche | Tirer (vise la souris) |
| T / G / F / H | Rocket : Haut/Bas/Gauche/Droite |
| R / Y / V / N | Rocket : Diagonales |
| E | Rechargement manuel |
| M | Changer de carte |
| F3 | Afficher les hitboxes |
| F4 | Mode entraînement |
| 1 / 2 / 3 | Difficulté entraînement |

## Architecture

```
src/
├── main.rs              # Point d'entrée, boucle menu + jeu
├── assets.rs            # Chargement textures + sons procéduraux
├── game.rs              # Boucle de jeu, réseau, scores, mode entraînement
├── player.rs            # Joueur, physique, particules, animations
├── projectile.rs        # Projectiles, explosions, particules d'explosion
├── menu.rs              # Menu principal, scan réseau LAN
├── map_loading.rs       # Chargement des cartes JSON
└── boilerplate/
    ├── mod.rs
    ├── network.rs        # Client réseau matchbox (WebRTC)
    ├── physics.rs        # Gravité, friction
    └── animation.rs      # Animation sprite sheet
```

## Lancer le jeu

```bash
# Lancer le serveur matchbox (dans un terminal séparé)
cargo install matchbox_server
matchbox_server

# Lancer l'hôte (dans un autre terminal)
cargo run

# Lancer un client (optionnel, pour le multijoueur)
cargo run -- --client --ip=127.0.0.1
```

### Scan LAN

En mode Client, le menu scanne automatiquement le réseau local pour trouver des serveurs matchbox (port 3536). Cliquez sur une IP détectée pour vous connecter.

## Réseau

Le jeu utilise [matchbox](https://github.com/nicbarker/matchbox) pour le WebRTC. L'hôte est autoritaire : il gère les collisions, les projectiles et synchronise les états vers les clients.
