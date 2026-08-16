# Roadmap

Phase 1 is the playable spine. Later phases add arsenal, theaters, and command-post comfort. Each phase should ship something you would actually launch after dinner — not a folder of stubs. Phases 1–11 are the 1.0 plan; they are all shipped.

## Phase 1 — Outpost

Endless match on **Kilo Outpost**. Maze rules, flow-field pathing, ground and air, four buildables, upgrades, sell, wave caller, pause/speed, canvas battlefield, command HUD.

**Exit test:** a stranger can fortify, leak, rebuild, cover air, and die on a wave they almost held — without reading the source.

Shipped.

## Phase 2 — Arsenal

The Phase 1 guns are a loadout, not a roster.

- Inferno, Arc Lance, Pulse Array, Helios, Swarm Rack, Siege Rail
- Four upgrade tiers with named breakpoints (Mark I–III, Apex)
- Targeting modes: first / last / strong / weak / flying
- Consumable strikes: satchel, overload, orbital (`Q` `W` `E`)
- Helios ground→air convert
- Pause overlay and best-wave persistence

**Exit test:** a skilled player argues about killbox composition, not about missing tools.

Shipped.

## Phase 3 — Theater

Maps and modifiers are the long-term game.

- Six theaters: Kilo Outpost, Redoubt, Dust Cut, Split Relay, Enclave, Twin Cores
- Map data as declarative layouts in `otd-core` (not in the HUD)
- Modifiers: ground only, accelerated, rich bounties, fixed scrap, ten/twenty gun cap
- Briefing picks a theater **and** a modifier, with hazards called out
- Seeded daily assignment from the UTC date

**Exit test:** you pick a map *and* a modifier on purpose, not because it is the only button.

Shipped.

## Phase 4 — Command

Make it feel like a product.

- Sound: placement, tracers, splashes, leak alarm, wave sting (mute + volume)
- Screen shake / hit punch kept small
- Settings: reduced FX, color-safe and high-contrast palettes, UI scale
- Key rebind persisted locally
- Mobile: scrollable tap tray, pinch zoom, drag pan, safe-area padding
- Replay copy from pause / defeat

**Exit test:** playable on a laptop speakers-off, a phone, and a projector without shame.

Shipped.

## Phase 5 — Campaign & workshop

- JSON map + catalog-adjacent `MapDoc` with a validator in `otd-core`
- In-browser map probe (`/workshop`): paint rocks, spawns, cores; export; deploy
- Headless `otd-bench`: validate JSON, run until wave/tick, replay an order log
- Replay file: seed + order log
- Short campaign of six scripted theaters (`/campaign`). Hold the listed wave; the match stays endless underneath

**Exit test:** a contributor ships a map without touching the renderer.

Shipped.

## Phase 6 — After the outpost

- Challenge seeds on the ops board
- Shared replay hash + claimed outcome; `otd-bench --verify` and `WasmGame.verifyReplay`
- Catalog packs: JSON loadout overlay (`/pack`), presets, replay-hashed

Catalog packs and hash verification are in. Multiplayer is not a Phase 6 promise.

Shipped.

## Phase 7 — Director

- Named wave scripts: mixed, swarm, air corridor, armor column, split, colossus
- Wave intel on the HUD (next / now composition)
- **Mite**: cheap swarm hull that splits once on death
- After-action on defeat: spend, kills/leaks by kind, gun scores
- Replay desk (`/replay`): paste JSON, verify hash, watch on the canvas

The director is the next thing you play. Replay watch closes the order-log loop.

Shipped.

## Phase 8 — Holdings

- Interest on leftover scrap when the field clears (4%, cap 48). Fixed scrap pays none
- **Medic** hulls that heal nearby ground. Armor columns and late mixed waves bring them
- Seventh theater: **Tri-Gate** — north, west, and east doors
- Challenge seed **Three Doors**
- Minimap: click (or drag) to look

Holdings is the economy you feel. Three doors is the map you misplay once.

Shipped.

## Phase 9 — Field craft

- **Paint**: drag while a structure is selected to place a line of cells
- **Repair** (`V`): 35 scrap restores 1 relay integrity, not above max, not after a fall
- **Shade**: ground camo. Guns without Det walk right past it. Pulse / Arc / Helios (and strikes) can see it
- Seventh campaign mission: **Three Gates** on Tri-Gate

Field craft is the maze you draw with a held mouse, and the hull that laughs at your starter guns.

Shipped.

## Phase 10 — Survey

- **Walk** meter: longest spawn-to-relay path in tiles. Hover a placement to see the new length
- **Move** (`G`): relocate a selected wall or gun for 6 scrap. Same path rules as placing
- **Flicker**: hops three tiles along the walk. A long maze is not a win by itself
- Targeting cycle includes **Camo**
- Challenge seed **Blind Spot**

Survey is reading the maze you built, then moving a gun one tile because the Walk number lied.

Shipped.

## Phase 11 — Holdfast (1.0)

- Dashed **walk overlay** on the canvas from each spawn cluster
- **Colossus roar**: periodic stun of nearby turrets
- **Overcharge** (`B`): 40 scrap, selected turret fires faster for a few seconds
- Eighth theater: **Oxbow** — a rock U; seal the banks or they go around
- Eighth campaign mission: **The Bend**. Challenge seed **The Fold**

Holdfast is the 1.0 closer. The planned phases are complete.

Shipped.

## Non-goals (until someone makes a case)

- Lootboxes, live-service currencies, account walls
- 3D for its own sake
- Simulation in TypeScript “just for now”
- Sealing the maze and hoping air does not exist
- Multiplayer of any kind; this is a single-player game
