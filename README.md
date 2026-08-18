# Open Tower Defense

An open-source **maze tower defense** game. You are not painting turrets along a fixed track. You are a commander with scrap, guns, and a grid: wall the field into a kill corridor, cover the air, and keep the frontier relay alive for as many waves as you can.

**Shape the path. Keep the lamp lit.**

MIT licensed. Built to be played, forked, and extended.

## Play (1.0)

Endless matches on ten theaters, an **eight-mission** campaign, known-seed challenges, and **catalog packs**. Barricades and towers **block ground pathing**. Air ignores the maze and steers at the nearest relay. Ten buildables, four upgrade tiers, targeting modes, three strikes, settings (mute, palettes, UI scale, **key rebind**), pan/zoom, a **map probe**, a **loadout probe**, replay verify hashes, a **wave director** with intel, **interest** on leftover scrap, **paint-drag** placement, **relay repair**, **structure move**, **overcharge**, a **Walk** overlay, and a **replay desk** (`/replay`).

### Controls

| Input | Action |
| --- | --- |
| `1`–`9` | Barricade through Swarm Rack (rebindable) |
| `0` | Siege Rail |
| `Q` `W` `E` | Satchel / Overload / Orbital |
| `T` | Cycle targeting (first / last / strong / weak / flying / camo) |
| `C` | Convert Helios to air (1 scrap, no undo except sell) |
| `V` | Repair the relay (35 scrap, +1 integrity) |
| `G` | Move the selected structure (6 scrap) |
| `B` | Overcharge the selected turret (40 scrap) |
| `Esc` / right-click | Cancel, then pause |
| `U` / `X` | Upgrade / sell |
| `N` | Call next wave |
| `Space` / `F` | Pause / speed |
| `M` | Mute |
| `Home` | Reset view |
| Drag / pinch / wheel | Paint while a structure is selected; middle-drag pans; pinch/wheel zoom |

## Stack

| Layer | Role |
| --- | --- |
| **Rust → Wasm** | Authoritative simulation: grid, flow-field pathing, combat, economy, waves |
| **TypeScript** | Canvas battlefield, game loop, input |
| **SvelteKit** | Shell, briefing, command HUD |

The browser never decides hits, pathing, or prices. It sends orders; the engine returns a snapshot.

## Requirements

- Rust 1.80+ with `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- Node.js 22+

## Develop

```bash
npm install --prefix web
npm run dev
```

Then open the URL Vite prints (usually `http://localhost:5173`). After changing Rust, run `npm run wasm` again (or restart `npm run dev`) so the browser picks up a new module.

`npm run dev` builds **debug** Wasm — fast to compile, but several times slower to run. Use `npm run dev:fast` (release Wasm, same dev server) before drawing any conclusion about simulation performance.

```bash
npm test              # Rust workspace tests (core, bench, wasm façade)
npm run check         # Wasm + Svelte/TS
npm run build         # Release Wasm + static site in web/build
cargo run -p otd-bench -- --map kilo --until-wave 8
cargo run -p otd-bench -- --validate path/to/map.json
cargo run -p otd-bench -- --verify path/to/replay.json
cargo run -p otd-bench -- --validate-pack path/to/pack.json
```

## Deploy

The site is fully static (SvelteKit `adapter-static` + Wasm), hosted on Cloudflare Workers static assets and served at **https://opentd.fileark.ca**. Config lives in [`wrangler.jsonc`](wrangler.jsonc); `web/static/_headers` sets long-lived caching for hashed assets.

```bash
npm install                     # once: pulls in wrangler
npx wrangler login              # once: authorise the Cloudflare account that owns fileark.ca
npm run deploy                  # release Wasm + web build + upload
```

`npm run deploy:dry` runs the same build and validates the upload without publishing. If the link-preview card changes, regenerate it with `npm --prefix web run og` (edit `web/scripts/og.html`, needs a local Chrome/Edge).

## Repository layout

```
crates/otd-core   Pure Rust simulation (native tests)
crates/otd-wasm   wasm-bindgen façade
crates/otd-bench  Headless validator / replay runner
web/              SvelteKit client
docs/             Architecture, roadmap, gameplay
```

## Docs

- [Gameplay](docs/GAMEPLAY.md) — how a match works, roster, economy
- [Architecture](docs/ARCHITECTURE.md) — engine, snapshot, UI boundaries
- [Roadmap](docs/ROADMAP.md) — phases 1–11 (1.0) and Lantern Dusk

## Known issues

- After the relay falls, the hover still talks like a live match: placement says “Can't build there,” and a strike ring can look valid even though the click does nothing. The defeat overlay is the source of truth.
- The command dock is tightly packed on mid-width windows (~1100–1545px). Settings / Ops / Briefing can vanish if that layout regresses, because the play view is `overflow: hidden`. Long wave-script names can clip there too.
- Replays recorded before Lantern Dusk will not `--verify`. The RNG, the hash, and the old co-op `player` field all changed. The JSON still parses.
- `npm run favicon` is manual. Changing the SVG without regenerating `favicon.ico` (and bumping `?v=` in `web/src/app.html`) leaves the old icon in the tab.

## Contributing

Play it. Break it. File issues with wave number and what you built. Balance patches and new maps should land as data in `otd-core` whenever possible, not as one-off UI hacks.

Please keep the simulation deterministic: same orders, same seed, same match.
