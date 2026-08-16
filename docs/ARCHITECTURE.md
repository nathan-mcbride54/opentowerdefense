# Architecture

The client is a thin command post. The match lives in Rust.

```
SvelteKit HUD  ──orders──►  TypeScript session  ──cell clicks──►  Wasm (otd-wasm)
        ▲                          │                                    │
        │                     Canvas rAF                                 │
        │                          │                                    ▼
        └────── snapshot JSON ─────┴────────────────────────────  otd-core
```

## Crates

**`otd-core`** is pure Rust. No `wasm-bindgen`, no browser types. Native `cargo test` covers pathing, placement rules, combat, and the wave director. This is the crate you extend when you add a turret, a map, or a hostile.

**`otd-wasm`** is a `cdylib` façade: construct a match, `step()` one tick, `snapshot()` JSON, forward clicks and hotkeys. Panic messages are piped to the browser console.

## Tick

Simulation uses a fixed `1/60s` step. The browser accumulates frame time, multiplies by playback speed (`1`, `2`, `4`), and calls `step` up to a cap so a tab hitch cannot explode the sim. Pause means the session stops calling `step`; rendering continues.

Orders (place, sell, upgrade, call wave, repair, lift/move, overcharge) apply immediately on the current tick. They are not queued behind physics. Drag-painting issues one `Click` per cell. Lift then Click relocates a structure.

## Pathing

Ground movement is a **flow field** recomputed whenever occupancy changes:

1. BFS from every relay (core) cell over 4-connected walkable tiles.
2. Each tile stores distance-to-core and the neighbour that leads inward.
3. Units steer toward the centre of that next tile.

Walkable: empty terrain, spawn cells, core cells. Blocked: rock, barricades, turrets.

Air units steer at the **nearest relay** (core cluster) with a light weave. They ignore occupancy. On Twin Cores that means you cannot starve one relay of guns and expect the sky to follow the ground.

Placement is committed only after a trial recompute shows:

- every spawn still reaches the core, and
- every living ground unit still has a finite distance.

That is the “do not seal the relay / do not pocket a creep” rule.

## Snapshot

Once per frame the engine serialises a `Snapshot`: HUD numbers, hover validity, selected turret, occupancy, units, projectiles, short-lived FX, **wave intel**, **walk length**, and an **after-action** block. The renderer is a pure function of that data plus local canvas caches (static terrain).

The HUD prefers the same snapshot so the bars cannot disagree with the battlefield.

Catalog data (names, costs, ranges, blurbs) is also owned by `otd-core` and exported as JSON. The build tray does not hardcode prices.

## Rendering

TypeScript owns pixels. Turrets, hulls, and tracers are drawn procedurally on a 2D canvas so Phase 1 has no asset pipeline. Barrel angles and projectile positions come from the sim; the renderer does not invent hits.

Procedural art lives in `web/src/lib/game/sprites.ts` (hulls, guns, terrain stamps, HUD icons). `renderer.ts` composites that onto the canvas from the snapshot. If a sprite sheet arrives later, bind it to the same snapshot kinds (`autocannon`, `runner`, …) without moving combat math into the client.

## UI

SvelteKit is static (`adapter-static`, `ssr = false`). Routes:

- `/` briefing: theater + modifier select, daily assignment, settings
- `/campaign` ops board: missions (unlock in order) and challenge seeds
- `/play?map=&mod=` canvas + overlay HUD
- `/play?mission=` / `/play?challenge=` campaign and challenge starts
- `/play?day=` seeded daily (UTC day number)
- `/play?map=&mod=&seed=` explicit seed (hex or decimal)
- `/play?workshop=1` custom JSON map from the probe (sessionStorage)
- `/play?pack=1` custom catalog pack from the loadout probe
- `/play?replay=1` watch a pasted replay (sessionStorage)
- `/workshop` map probe: paint rocks/spawns/cores, validate, export JSON
- `/pack` loadout probe: retune catalog numbers, presets, export JSON
- `/replay` paste / verify / watch an order log

Hotkeys live in the session layer and are **rebindable** (persisted in `localStorage`). The canvas supports drag-paint (when a structure is selected), middle-drag pan, wheel zoom, and pinch zoom.

Match rules (ground only, bounty, turret cap, starting scrap) live in `otd-core` as a `Modifier`. The HUD reads cap and names from the snapshot. Do not hardcode modifier math in TypeScript.

## Maps as JSON

A theater is a `MapDoc`: size, seed, core cells, spawn cells, rocks. `otd-core` validates bounds, overlap, and spawn reachability before a match starts. Built-in maps still live as Rust painters; they can be exported to the same JSON. Contributors can ship a map without touching the renderer.

`otd-bench` runs that validator and can step a match (empty base or a replay order log) without a browser:

```
cargo run -p otd-bench -- --map kilo --until-wave 8
cargo run -p otd-bench -- --validate theater.json
cargo run -p otd-bench -- --verify replay.json
cargo run -p otd-bench -- --mission 0 --until-wave 8
cargo run -p otd-bench -- --validate-pack pack.json
cargo run -p otd-bench -- --map kilo --pack pack.json --until-wave 8
```

## Determinism

`otd-core` uses SplitMix64 seeded per match. Do not call `rand` or `Math.random` inside the sim, and do not force seeds odd — that was an xorshift guard and made every even seed identical to seed+1. Every order (build, click, upgrade, call wave, …) is logged with a tick. Pause → Copy replay writes `{ mapId, seed, modifierId, orders, pack, outcome, hash }`. The hash ignores `hash` and `outcome` and fingerprints the resolved loadout. Replays feed `Game::from_replay` / `otd-bench --orders`. `otd-bench --verify` replays to the claimed tick count and checks hash + outcome.

## Adding content

| You want to… | Change |
| --- | --- |
| Tune damage / cost / HP | `crates/otd-core/src/defs.rs` |
| Add a turret or hostile | `defs.rs` + a renderer branch |
| Change wave mix | `crates/otd-core/src/director.rs` |
| Add a map | `crates/otd-core/src/maps.rs` **or** a JSON `MapDoc` (workshop / `--map-json`) |
| Add a campaign mission | `crates/otd-core/src/campaign.rs` |
| Retune guns without a rebuild | JSON `PackDoc` (`/pack` / `--pack`) |
| Add a modifier | `crates/otd-core/src/modifiers.rs` + HUD chip if needed |
| Add an order (targeting mode, strike) | `sim.rs` command + Wasm method + HUD control |
| Change how the field looks | `web/src/lib/game/sprites.ts` (art) and `renderer.ts` (the canvas pass) |

Keep combat math out of Svelte components.
