# Bazoozoo — Agent guidance

## Build & run

```sh
cargo run                          # host (needs matchbox_server on :3536)
cargo run -- --client --ip=127.0.0.1  # connect as client
```

`matchbox_server` must be installed (`cargo install matchbox_server`). The game auto-launches it when you select **Host**; no separate terminal needed.

## Rust edition

`edition = "2024"` (Cargo.toml:4). Make sure `rust-analyzer` or any linting uses a toolchain that supports it (rustc ≥ 1.85).

## Project structure

| Path | Purpose |
|------|---------|
| `src/main.rs` | Entrypoint, menu loop → game loop |
| `src/game.rs` | Core game loop, decoupled into 10 `update_*` + 7 `draw_*` methods |
| `src/player.rs` | Player movement, rocket-jump physics, particles, sprite animation |
| `src/projectile.rs` | Projectiles, explosion particles |
| `src/menu.rs` | Main menu, LAN scan for matchbox servers |
| `src/map_loading.rs` | Map JSON loader (hitbox rectangles) |
| `src/keybindings.rs` | QWERTY/AZERTY keyboard layout config |
| `src/constants.rs` | Game-wide constants (CharacterStats, physics, colors) |
| `src/target.rs` | Training-mode targets (`Target`, `TrainingDifficulty`) |
| `src/particle.rs` | Unified `ParticleManager` (dust, explosions, trails) |
| `src/assets.rs` | Loads textures + procedural sounds |
| `src/boilerplate/network.rs` | WebRTC via matchbox_socket, authoritative host model |
| `src/boilerplate/physics.rs` | Gravity, friction helpers |
| `src/boilerplate/animation.rs` | Sprite-sheet animation |

## Maps

JSON + PNG pairs in `assets/`. Map files define wall hitboxes. Existing: `map1.json`, `map2.json`, `hollow_map.json`.

## Networking

- **Host-authoritative**: host runs physics/collisions, serialises `NetworkGameState` via bincode to clients.
- `NetworkManager::new(room_url)` spawns a background WebRTC loop.
- Client sends `PlayerState` (position, aim, fire flag) each frame.
- Matchbox server port: 3536 (WebSocket).

## Controls

Key bindings in `README.md`. AZERTY/QWERTY layout selectable in menu (`src/keybindings.rs`).
Notable debug keys: `F3` toggles hitbox overlay, `F4` toggles training mode.

## Task tracking

See `TASKS.md` for backlog and todo.

## Tests

23 unit tests across `constants.rs`, `player.rs`, `projectile.rs`, `target.rs`, `physics.rs`. Run with `cargo test`.

**Note**: tests that depend on macroquad runtime context (texture creation, `get_frame_time()`) are isolated to a `TestPlayer` helper struct so they stay pure and fast.
