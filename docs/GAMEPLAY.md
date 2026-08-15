# Gameplay

Open Tower Defense is an **open-field maze** defender. The map is a buildable grid with a **relay** you must protect and **ingress points** where hostiles appear. Nothing forces a lane until you do.

## Match flow

1. A short **fortify** window. Spend credits on barricades and guns. The first wave will not wait forever.
2. **Call wave** (`N`) to start early and pocket a credit bonus, or let the timer expire.
3. Hostiles path toward the relay. Ground units walk the current shortest maze. Air units ignore walls and towers.
4. Kills pay. Leaks chip **relay integrity**. At zero, the outpost falls.
5. When the field is clear, the next fortify window begins. Waves never end.

The score is the wave you reached. Best wave is stored locally per theater **and** modifier. The HUD names the next wave script during fortify so you can spend on purpose.

## The maze rule

Barricades **and** towers occupy a cell and block ground traffic.

- Stretch the walk so guns have time on target.
- Fold both ingresses into one killbox if you can afford the walls.
- Never cut the last ground path — the engine rejects a placement that would isolate the relay, or trap living ground units in a pocket.

Air does not care about your maze.

## Targeting

Default is **first** — closest to the relay. Cycle with `T`: last, strong, weak, flying, **camo**. There is no line-of-sight check. Guns shoot over their own maze. **Shades** are skipped unless the gun **detects** (Pulse, Arc, Helios, and the air pieces). Strikes ignore camo. Camo targeting prefers Shades when the gun can see them.

While a structure is selected, **drag** across cells to paint. Each cell is its own recorded click. Middle-drag still pans. **Move** (`G`) picks up the selected structure; the next click sets it down for 6 scrap. **Overcharge** (`B`) spends 40 scrap so the selected turret fires faster for a few seconds. The HUD **Walk** meter is the longest spawn-to-relay path in tiles. A dashed overlay traces the current walk from each spawn cluster. Hovering a placement shows what the walk would become.

## Economy

- Start with scrap, not a finished base.
- Bounties scale with wave index.
- Unspent scrap pays **interest** when the field clears (4%, capped). Fixed scrap pays none.
- Four tiers: Mark I → Mark II → Mark III → Apex. Each breakpoint is a real jump, not a flat percent.
- Sell-back is a loss on purpose.
- **Repair** (`V`) spends 35 scrap to restore 1 integrity, never above the starting pool. The relay cannot be patched after it falls.
- **Move** (`G`) spends 6 scrap to relocate a selected wall or gun. Same path rules as placing.
- **Overcharge** (`B`) spends 40 scrap to heat a selected turret for about six seconds.
- Strikes cost scrap and recharge. They are panic tools, not a farm.

## Roster

| Hotkey | Name | Role |
| --- | --- | --- |
| `1` | Barricade | Cheap path block |
| `2` | Autocannon | Dual-purpose tracers |
| `3` | Howitzer | Ground splash |
| `4` | Skystinger | Air bursts |
| `5` | Inferno | Ground cone — wants a fold |
| `6` | Arc Lance | Line damage down a corridor |
| `7` | Pulse Array | Disc chip + slow; stacks along a lane |
| `8` | Helios | Dwell beam. Sees Shades. `C` converts to air for 1 scrap |
| `9` | Swarm Rack | Homing volley, retargets in-range |
| `0` | Siege Rail | Slow, armour-blind, single target. Blind to Shades |

Guns marked **Det** in the tray can see camo. Autocannon, Howitzer, Inferno, and Siege cannot. Air guns that detect still only shoot air.

### Strikes

| Hotkey | Name | Job |
| --- | --- | --- |
| `Q` | Satchel | Small boom |
| `W` | Overload | Slow field |
| `E` | Orbital | Wide radial, hardest at centre |

### Hostiles

| Name | Profile |
| --- | --- |
| Runner | Fast, thin, ground |
| Lorry | Medium hull, ground |
| Bulwark | Slow, armored. Autocannons tickle it; siege does not |
| Wasp | Air. Ignores the maze |
| Colossus | Boss, every tenth wave. Roars and stuns nearby turrets |
| Mite | Fast scrap. Splits once when killed, not when it leaks |
| Medic | Slow ground. Heals nearby hulls. Kill it first |
| Shade | Ground camo. Guns without Det skip it. Strikes always hit |
| Flicker | Ground. Hops three tiles along the walk every couple of seconds |

## Theaters

| Map | Lesson |
| --- | --- |
| Kilo Outpost | Open field, north + east ingress |
| Redoubt | Courtyard core, two opposite doors |
| Dust Cut | Rock pass. Every tile is a fight |
| Split Relay | Stone spine. Two corridors, one late merge |
| Enclave | Tight yard. Ingress is already close |
| Twin Cores | Two relays, one integrity pool. Air hunts the nearest |
| Tri-Gate | Three doors. North, west, east |
| Oxbow | Rock U opening north. Seal the banks or they go around |

## Modifiers

Pick one per match. They do not stack.

| Modifier | Rule |
| --- | --- |
| Standard | Default economy, air, no turret ceiling. Interest on leftover scrap |
| Ground only | No Wasps. Extra ground in their place |
| Accelerated | Thin hulls, long gait |
| Rich bounties | Kills pay extra |
| Fixed scrap | 10,000 scrap at start. Kills pay nothing. No interest |
| Ten guns | At most ten turrets. Barricades ignore the cap |
| Twenty guns | Twenty turret ceiling |

A seeded **daily assignment** picks a theater and a modifier from the UTC date. Same day, same fight.

## Director

Waves are named. The HUD shows the next script during fortify.

| Script | What it is |
| --- | --- |
| Mixed | The usual soup |
| Swarm | Runners and mites. Splash earns its keep |
| Air corridor | Wasps. Ground-only turns this into extra hull |
| Armor column | Lorries, bulwarks, and a medic |
| Split | Mite weather. They split once when killed. Shades and Flickers in the mix |
| Colossus | Every tenth wave. Medics in the train. Roar stuns guns that stand too close |

## Campaign

`/campaign` is eight missions in order. Each one is a theater + modifier + seed with a **hold through wave N** objective. Clearing it unlocks the next. The sim does not freeze — after the hold you can advance or keep the endless walk.

Challenges on the same board are public seeds (Dry Season, Night Watch, Iron Budget, Three Doors, Blind Spot, The Fold). Same orders should produce the same hash.

## Workshop

`/workshop` paints core, spawn, and rock cells and exports a JSON `MapDoc`. The engine validates it (size, overlap, every spawn can walk to a relay) before a match starts.

`/pack` retunes the catalog — cost, range, damage, fire mode — without changing the tick. Presets: Stock, Glass Cannons, Fortress, Skywatch, Bargain Bin. Disabled guns leave the tray.

Pause or defeat → **Copy replay** writes seed, orders, pack, a claimed outcome, and a hash for `otd-bench --verify`. `/replay` pastes that JSON, verifies it, and watches the same ticks on the canvas.

Defeat also prints an after-action: scrap spent, kills and leaks by hull, and which guns scored.

## Local co-op

`/play?coop=1` (briefing and ops board have a checkbox). Two commanders, **one simulation**, **shared scrap and integrity**. Player 1 keeps mouse and the rebindable keymap. Player 2 moves an amber cursor with the arrows, places with Enter, and uses a second keymap (`A S D Z Y` / `H J K L P` for guns, `[ ] \` for strikes, `-` to sell). Pause, speed, mute, and view reset stay on the P1 map. P1 binds win if a key is on both maps. Replays store a `player` field on each order.
