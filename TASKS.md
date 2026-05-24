# Refactoring — Grande ménage du code

## Terminé

- [x] Système de keybindings QWERTY/AZERTY (`src/keybindings.rs`)
- [x] Phase 1 — Constantes : création de `src/constants.rs` (CharacterStats, VIRTUAL_HEIGHT, RECOIL_FORCE, PLAYER_COLORS), imports mis à jour dans tout le code
- [x] Phase 2.1 — Types réseau : `NetworkProjectile`, `NetworkPlayer`, `NetworkGameState` migrés dans `boilerplate/network.rs`
- [x] Phase 2.2 — Cibles : `Target`, `TrainingDifficulty` extraits de `game.rs` vers `src/target.rs`
- [x] Phase 3 — Particules : `ParticleSystem`, `ExplosionParticleSystem` → `ParticleManager` unifié dans `src/particle.rs`
- [x] Phase 4 — Sub-structs : `GameResources` + `GameTraining` extraits de `Game`
- [x] Phase 5 — Découpage `update()` (291→10 méthodes) + `draw()` (254→7 méthodes) dans `game.rs`
- [x] Phase 6 — Factorisation : génération host JSON + reconciliation projectiles
- [x] Phase 7 — Gestion d'erreurs : tous les `.unwrap()` → `expect()` / `Result` avec messages descriptifs
- [x] Phase 8 — Perf : `animation.clone()` 5×/frame → 0 clone via `draw_colored()`
- [x] Phase 9 — 0 warnings : toutes les constantes utilisées
- [x] Phase 10 — Mutex : 10 appels `.lock().unwrap()` dans `menu.rs` → `.lock().expect("Mutex poisonné")`
- [x] Tests unitaires : 23 tests (constants, player, projectile, target, physics), tous verts

## Prochaines pistes

- [ ] (à définir)
