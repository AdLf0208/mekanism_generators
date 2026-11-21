# Mekanism: Generators → Horizon, slice 1 (the Heat Generator)

Ported from **Mekanism: Generators 10.7.19.85** (`mekanism/Mekanism`, **MIT**),
the exact ATM10 jar `MekanismGenerators-1.21.1-10.7.19.85.jar` (SHA-1
`44c4d8006597580ff7e4101b5a22f7b426cdcaf6`, equal to the pack's
`modrinthHash`; SHA-256 `0e5783b1…a4ac`; kept at
`ports/_shared/carry-work/mekanismgenerators/`, provenance in `SOURCE.md`
there). Upstream behaviour was read from tag `v1.21.1-10.7.19.85` (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`), analysis only: the port is a
clean-room Rust rewrite. What was read and the numbers extracted:
[`FINDINGS.md`](FINDINGS.md).

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT is licence rank 0 (permissive); the approval is still required.

Status: **version 0.2.0 — migrated to the flat SDK package** (mod API 0.2, one
`.hmod`, 2026-11-18; `## Migration to the SDK format` below). The behaviour is
slice 1 as it was verified in game at 0.1.0: one block,
`mekanismgenerators:heat_generator`, a fuel slot that turns burnable items into
lava, a lava tank, a heat capacitor, and a 160,000 FE buffer the block fills
itself. Everything else in the mod is a planned gap (below).

History: 0.1.0 was the retired `.hnmod` container (submitted as **#840**, closed
in the format purge) with the content mod embedded inside it (the AE2 pattern);
its sources, package and tooling are kept at
`ports/_shared/evidence/mekanismgenerators/retired-hnmod-0.1.0/`. `runtime.rs`
came through the migration byte-identically.

## Class + tier, approach, licence

- **Class `content`, tier `deep`** (`ports/ledger.csv`, from
  `ports/_shared/source-map/atm10-sources.csv`); it is an **add-on** of
  `mekanism` (which is already `ported-partial`).
- The **slice shape is machines**, not the mod's whole deep tier: a block
  entity, its own FE capability (API 142), a container, a tick and harness
  commands (the pin's `docs/modpacks.md`: a machines feature needs a block
  entity, its state in NBT, a container/menu and the energy balance in its own
  tick). Reactors, multiblocks, the suit modules and renderers are deferred.
- Queue position: licence rank 0, mechanics, core; it is the mod's smallest
  self-contained generator and the one the pack's own recipe chain starts from.
- **Licence: MIT → rank 0**. The brief's exception-free "why now" is the
  licence-first queue; the parent `mekanism` is already ported-partial so the
  add-on is workable.

## What the slice is

`mekanismgenerators:heat_generator`, one block:

- a **fuel slot** (container, slot 0) that takes any item with an engine burn
  time and turns one item a tick into `burnTime / 20` mB of lava (coal 80 mB,
  a lava bucket 1,000 mB; a crafting remainder replaces the stack);
- a **24,000 mB lava tank** (this port's saved int `lava`; upstream's fluid
  capability is not declared in this slice, so only the fuel slot fills it);
- a **heat capacitor** (10 heat units) fed by 200 heat/t while lava burns, 30
  per lava side and 100 in the Nether, cooled by the 0.5 thermal-efficiency
  loss and the six-face environment leak;
- a **160,000 FE buffer** (`native_block_entity_energy`, the int `energy` in the
  block entity's saved compound) that receives
  `min(heatLost x carnot, 510, space)` FE a tick;
- `/mekanismgenerators info|energy`, the harness controls, since there is no GUI.

## Behaviour (upstream → this port)

| | Upstream | This port |
|---|---|---|
| buffer | `storage.heatGenerator` **160,000 FE** | same |
| output | 1,020 FE/t from the front side (`MAX_PRODUCTION x 2`) | the buffer exposes 1,020 FE/t on every face; **nothing pushes** (slice boundary) |
| fuel | `FluidFuelInventorySlot`: `burnTime / 20` mB, one item a tick, only if the whole amount fits | same (`accept_fuel`), with the crafting remainder |
| tank | 24,000 mB, lava only, input sides | 24,000 mB, our own saved int, fed only by the fuel slot |
| burning | while the buffer has room: 10 mB → 200 heat; else kept | same |
| passive | 30 heat x lava sides (a lava-logged block counts as the 7th), +100 ultrawarm | same; a lava-logged block is not detected (no fluid-logged state), `ultrawarm` = the Nether dimension id |
| conversion | `carnot = 1 - min/max(T, ambient)`, `heatLost = 0.5 (T - ambient)`, energy `min(heatLost x carnot, 510)` | same, unit-tested |
| leak | per face `(T - ambient) / (10,000 + insulation + 5) x 10`, bottom insulation 0 | same |
| settled rate | 112 FE/t on fuel, 96 on six lava sides, 39 in the Nether, 360 with all three | same, pinned by unit test and measured in game |
| block | `strength(3.5, 16)`, requires a correct tool, horizontal facing + `active`, light 13, custom shape/sound, GUI, comparator | barrel template 3.5/16, requires tool, stone sounds, facing x `open` (never set), no active state/light/particles/sound, no GUI, no comparator |
| fuel slot sides | front/left/back/top/bottom take fuel | every face (no side config) |

## Hooks

*The 0.1.0 wiring, kept as history. The 0.2.0 mapping is the helper→hook table in
`## Migration to the SDK format`; these injections no longer exist.*

| Hook (API) | Helper | What it does |
|---|---|---|
| `content_root` (target, not a hook) | `embedded_content.rs` | Writes the bundle's embedded `.hmod` to `.horizon-cache/` and appends it to the dev-mod list, so one `.hnmod` installs the content and the code. |
| `block_entity` (15) | `block_entity.rs` | The block gets a block entity. |
| `block_entity_ticks` (15) | `ticks.rs` | It ticks on the server. |
| `block_entity_container` (16) | `container.rs` | The one fuel slot. |
| `block_entity_energy` (142) | `energy.rs` | The 160,000 FE buffer (`energy` in the saved compound). |
| `block_entity_tick` (17) | `tick.rs` | Fuel, lava, heat and FE, one tick. |
| `register_commands` (8) | `register.rs` | `/mekanismgenerators info\|energy`. |
| `perform_command` (8) | `perform.rs` | The command bodies. |

Seven stamped hook injections plus one `content_root` target injection (the
embedded content), none unstamped, no guessed API. The buffer is a real
`Capabilities.Energy.BLOCK` store: another mod's cable can drain it with
`native_energy_extract`/`native_move_energy` (API 142, as the `mekanism` and
`powah` ports do at this pin) even though this slice does not push.

## Command (harness addition)

```
/mekanismgenerators info <x> <y> <z>
mekanismgenerators:heat_generator at 0,100,0: 49152/160000 FE, lava 10730/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, fuel empty, t=768
/mekanismgenerators energy <amount> <x> <y> <z>     (operator level 2)
Energy set to 160000 FE.
```

`t=` is the world game time, so the in-game check can tell a fresh reply from a
stale one (`ctl chat` round trips are ~1–2 s). `energy` is a harness control,
not upstream gameplay, and is recorded as an addition (precedent:
`/mekanism energy`, `/powah set`).

## Content (`.hmod`)

`content/` is checked in assembled (`content/README.md` lists every file and
where it came from); `cargo horizon-mod build` rebuilds the package. Carried:
the blockstate (with `active` folded onto the barrel template's `open`), the
idle model (its five `mekanism:` texture ids moved into this namespace), the
item model + 26.3 `items/` definition, the two own textures and the three plate
textures it shares with Mekanism, the lang keys, the 26.3 self-drop loot table,
the shaped recipe in the 26.3 ingredient shape, the pickaxe tag and a creative
tab. **The recipe loads but does not craft** (it needs `c:ingots/osmium`).

## One delivered file (embedded content, 2026-11-11)

*Historical: the `.hnmod` format was retired on 2026-11-18 and one `.hmod` now carries the
content and the code natively — see `## Migration to the SDK format`. What follows describes
0.1.0 and is kept because its in-game evidence is still the behaviour's proof.*

`horizon-rust/horizon-mods` stores **one file per project**, and at this pin the
registry indexes a native mod by the `.hnmod` extension only: a split
content+code port could not be listed (`upstream_ready.py` said so until this
change, and reviewers closed `refinedstorage_quartz_arsenal` #498 for the same
reason). The port now follows the AE2 pattern
(`docs/native-mods.md` "Data files", and the `ports/mekanism` embedded section):

- `mixins/mekanismgenerators/data/mekanismgenerators-content.hmod` is a
  byte-identical copy of `content/mekanismgenerators.hmod` (9,380 B, SHA-256
  `bd3063b0…0924`); `native_data!` compiles it into the game;
- one `content_root` HEAD injection (`embedded-content`, cancellable,
  `embedded_content.rs`) writes it to
  `.horizon-cache/mekanismgenerators-embedded-content.hmod` when that file
  differs, appends it to the dev-mod list and hands the list back through
  `content_root(root, enabled, &with)`. An installed copy of the mod is found
  first, so it still wins;
- `tools/package.py` carries `data/` into the archive and **refuses** an
  embedded copy that is not the freshly built `.hmod` (the packer guard);
- `tools/check-embedded.py` proves the three places agree (content input,
  embedded copy, archive entry) and that the plan's injection is the right one.

Delivered: **`mekanismgenerators.hnmod` 20,581 B**, SHA-256
`bd079515…1323`, **13 entries** (the 11 sources + `embedded_content.rs` +
`data/mekanismgenerators-content.hmod`); that data entry is byte-identical to
`content/mekanismgenerators.hmod`. Two consecutive `python3 tools/package.py`
runs after the final sources are **byte-identical** (`cmp`, same SHA-256).

No behaviour changed: the same hooks, the same block, the same numbers. What
changed is where the block, item and data come from when the mod is installed —
the bundle itself, not a second file beside it.

## Deviations and gaps (slice 1)

1. **No FE output.** The generator fills its own buffer and nothing pushes it
   into a neighbour (the brief's slice boundary; the transfer helpers exist at
   this pin, so a cable slice can add it).
2. **No fluid capability.** The tank is a saved int; no pipe can fill it and
   nothing can read it as lava (upstream's `lavaTank` is a fluid handler).
3. **No GUI.** Fuel goes in through the container (automation's path, and what
   the check uses) and state is read with the command.
4. **No `active` block state**, so the `heat_generator_active` model, the
   light 13, the smoke/flame particles and the hum are absent; the idle model
   always shows.
5. **No custom shape**: the barrel template's full cube collision is used.
6. **No side configuration**: fuel enters and FE leaves on every face, where
   upstream feeds five sides and emits on the front only.
7. **Ambient temperature is a fixed 300 K** (`HeatAPI.AMBIENT_TEMP`), not the
   biome temperature at the block (`HeatAPI.getAmbientTemp`); the pin has no
   biome-temperature helper.
8. **No comparator** on the tank; `has_analog_output` is not declared.
9. **Breaking the block loses its FE, lava and heat** (upstream's loot copies
   `mekanism:energy/fluids/heat_data` components into the item).
10. **The recipe is not craftable** (needs `c:ingots/osmium`), and the
    advancement is not carried.
11. **The rest of the mod**: solar/wind/bio/gas/advanced solar generators, the
    fission reactor and fusion reactor multiblocks, the industrial turbine,
    hohlraum, the geothermal/solar suit modules, the coloured resources,
    upgrades, muffling and the computer methods are **not ported**.
12. **One item converts to lava a tick**, as upstream's `fillOrBurn` does, so a
    stack of 64 coal takes 64 ticks to become 5,120 mB; a full buffer still
    converts fuel into the tank while the tank has room (upstream too), so the
    full-buffer control asserts "no lava burned", not "no item taken".
13. **`heat` is our own saved double**; upstream stores the capacitor in an NBT
    `heat_data` attachment with the same meaning (stored heat, temperature =
    heat/10).

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Verification

*The 0.1.0 ladder and its in-game runs. The 0.2.0 migration's ladder is in
`## Migration to the SDK format`; the in-game scenario was not re-run (gap 14 there).*

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6); checkouts
`ports/_shared/pins/horizon-1d63e0a` and `.../horizon-mixins-5a64af9`. Every
build, test and e2e ran through `ports/tools/build-slot.sh` (never a bare
`cargo`, `CARGO_INCREMENTAL=0`, no `--target-dir`), with
`--horizon ports/_shared/pins/horizon-1d63e0a`; the e2e used the pin-matched
shared `horizon-ctl` (`ports/_shared/bin/horizon-1d63e0a5/horizon-ctl`) and the
port's own fast game.

### Ladder

| Step | Command (through the slot runner) | Result |
|---|---|---|
| runtime tests (RED) | `cargo test --offline` before the core existed | **cargo_exit 101**, 56 compile errors, the tests' API missing (`unit-red.log`) |
| runtime tests | `cargo test --offline` in the port | **19 passed**, 0 failed |
| format | `cargo fmt --check` | clean |
| runtime lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** |
| stage | `tools/horizon.py stage` | exit 0, 7 injections, 2 modules |
| check | `tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** (`horizon-check.log`) |
| embedded test | `tools/horizon.py test` | `test result: ok. 19 passed; 0 failed; 1240 filtered out`, `cargo_exit: 0` (`horizon-test.log`) |
| fast build | `tools/horizon.py build` | `cargo_exit: 0` (`horizon-build.log`) |
| e2e | `build-slot.sh e2e -- … -- tools/e2e.sh` | **PASS, `EXIT:0`** (`e2e-default.log`) |
| fail-capable run | `SELFTEST=1 tools/e2e.sh` | **FAIL as required, `EXIT:1`** (`e2e-selftest.log`) |
| content `.hmod` | `cargo-horizon-mod build` + `check --json` | **9,380 B**, SHA-256 `bd3063b0…0924`, 16 files, API 0.2, `["registries"]`, no code |
| native `.hnmod` | `python3 tools/package.py` | **12,106 B**, SHA-256 `7f0204a2…56d9`, 11 entries (**superseded 2026-11-11 by the embedded bundle: 20,581 B, 13 entries — below**) |
| load-check | `tools/hmod-load.sh` (shared game) | **PASS with `--dev-mod`**, unknown without it (`hmod-load.log`) |
| e2e log checker | `python3 ports/tools/check-e2e-logs.py mekanismgenerators --strict` | `logs=2 ok=2 missing-exit=0 bad-exit=0`, exit 0 |

Evidence: `ports/_shared/evidence/mekanismgenerators/` (`unit-red.log`,
`horizon-{check,test,build}.log`, `e2e-default.log`, `e2e-selftest.log`,
`hmod-load.log`; the 2026-11-11 re-run adds
`horizon-{check,test,build}-embedded.log`, `e2e-embedded-default.log`,
`e2e-embedded-selftest.log`).

### Embedded re-verification, 2026-11-11 (one delivered file, no `--dev-mod`)

Same pin, same sources' behaviour; only the packaging changed, so the whole
ladder was re-run against the embedded bundle. `stage`/`check`/`build` also ran
once on the second machine (exit 0) and again locally for the logs below.

| Step | Command (through the slot runner) | Result |
|---|---|---|
| helper format | `rustfmt --check --edition 2024 mixins/mekanismgenerators/embedded_content.rs` | clean (1.95.0) |
| runtime tests | `cargo test --locked --offline` | **19 passed**, 0 failed, exit 0 |
| runtime lints | `cargo clippy --all-targets --locked --offline -- -D warnings` | exit 0, **0 warnings** |
| stage | `tools/horizon.py stage` | exit 0; the plan holds **8 `[[injection]]` blocks** (7 stamped hooks + the `content_root` target) and 2 modules (`horizon-check-embedded.log`) |
| check | `tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** in `target/horizon-check.log` (`horizon-check-embedded.log`) |
| embedded test | `tools/horizon.py test` | `19 passed; 0 failed; 1240 filtered out`, `cargo_exit: 0` (`horizon-test-embedded.log`) |
| fast build | `tools/horizon.py build` | `cargo_exit: 0`, 0 warnings (`horizon-build-embedded.log`) |
| e2e, delivered bundle | `build-slot.sh e2e -- … -- tools/e2e.sh` (**no `--dev-mod`**) | **PASS, `EXIT:0`** (`e2e-embedded-default.log`) |
| embedded-content control | same run, `CONTROL=1` (default) | **PASS**: ids unknown, native command answers (quoted below) |
| embedded-file check | same run | `.horizon-cache/mekanismgenerators-embedded-content.hmod` 9,380 B, **identical** to `content/mekanismgenerators.hmod` |
| fail-capable run | `SELFTEST=1 EMBEDDED=1 tools/e2e.sh` | **FAIL as required, `EXIT:1`** at the flipped no-fuel assertion (`e2e-embedded-selftest.log`) |
| native `.hnmod` | `python3 tools/package.py` (guard: embedded copy must be the built `.hmod`) | **20,581 B**, SHA-256 `bd07951573c8…1323`, **13 entries**, the shipped one is this file (`mekanismgenerators.hnmod`) |
| determinism | `python3 tools/package.py` twice, `cmp` | **byte-identical**, same SHA-256 |
| package check | `python3 tools/check-embedded.py` | exit 0 (content = embedded = archive entry) |
| e2e log checker | `python3 ports/tools/check-e2e-logs.py mekanismgenerators --strict` | `logs=4 ok=4 missing-exit=0 bad-exit=0`, exit 0 |
| submit gate | `python3 ports/tools/upstream_ready.py mekanismgenerators` | split-port problem **gone**; one blocker left: the parent `mekanism` is not in the registry |

No `--dev-mod` reaches the game in the delivered-bundle run, so the block, the
item and the data can only come from the `.hnmod`'s embedded copy; the control
leg (`.horizon-cache` a plain file, the helper cannot write) shows the same
binary with **no** heat generator while `/mekanismgenerators` still answers.

### The in-game check (quoted from `e2e-default.log`)

Overworld arena at y=100: the generator at `0,100,0`, an unfuelled control at
`0,100,4`. Quoted verbatim:

```
  generator: mekanismgenerators:heat_generator at 0,100,0: 0/160000 FE, lava 0/24000 mB, temp 300.0 K, producing 0 FE/t, burning false, fuel empty, t=161
  after 120 ticks: mekanismgenerators:heat_generator at 0,100,0: 0/160000 FE, lava 0/24000 mB, temp 300.0 K, producing 0 FE/t, burning false, fuel empty, t=290
  info(0,100,4): mekanismgenerators:heat_generator at 0,100,4: 0/160000 FE, lava 0/24000 mB, temp 300.0 K, producing 0 FE/t, burning false, fuel empty, t=298
  mekanismgenerators:heat_generator at 0,100,0: 14544/160000 FE, lava 9420/24000 mB, temp 675.4 K, producing 112 FE/t, burning true, fuel minecraft:coal x55, t=459
  generator: mekanismgenerators:heat_generator at 0,100,0: 49152/160000 FE, lava 10730/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, fuel empty, t=768
  energy 49152 FE, lava 10730 mB, producing 112 FE/t
  50944 -> 63264 FE over ~100 ticks (delta 12320)
  full: mekanismgenerators:heat_generator at 0,100,0: 160000/160000 FE, lava 9400/24000 mB, temp 397.4 K, producing 0 FE/t, burning false, fuel empty, t=927
  after 100 ticks full: mekanismgenerators:heat_generator at 0,100,0: 160000/160000 FE, lava 9400/24000 mB, temp 300.3 K, producing 0 FE/t, burning false, fuel empty, t=1036
  drained: mekanismgenerators:heat_generator at 0,100,0: 4890/160000 FE, lava 8730/24000 mB, temp 663.9 K, producing 107 FE/t, burning true, fuel empty, t=1110
  before save: mekanismgenerators:heat_generator at 0,100,0: 5754/160000 FE, lava 8650/24000 mB, temp 667.9 K, producing 109 FE/t, burning true, fuel empty, t=1118
  after relaunch: mekanismgenerators:heat_generator at 0,100,0: 18689/160000 FE, lava 7490/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, fuel empty, t=1235
  (the saved compound survives: FE 5754 -> 18689, lava 8650 -> 7490)
PASS: a fuelled heat generator converts coal into lava, burns 10 mB/t and makes 112 FE/t into its own 160,000 FE buffer, an unfuelled one makes nothing, a full buffer produces nothing and burns no lava while a drained one resumes, and the FE, lava and heat survive a save and relaunch
EXIT:0
```

What the numbers prove:

- an unfuelled generator at ambient temperature produces **nothing** for 120
  ticks (`producing 0`, `temp 300.0 K`), and neither does the control;
- three coal top-ups became lava (9,420 mB left of 15,360 after ~390 ticks),
  the block settles at **675.5 K** and **112 FE/t** — the Carnot arithmetic the
  unit tests pin (200 heat/t, 0.5 efficiency, 0.5685 Carnot);
- the buffer grew **12,320 FE in ~110 ticks**, i.e. 112/t (± the chat latency),
  not 0 and not the 510 cap;
- at **160,000 FE** it produced **0** and the lava stayed at 9,400 for 100
  ticks (`burning false`) while the capacitor cooled to 300.3 K — a full buffer
  burns no lava;
- drained to 0, it burned lava again (9,400 → 8,730) and made 4,890 FE in 60
  ticks, so the buffer's space is what gates the fuel;
- after `save`, quit and relaunch the compound read back: FE 5,754 → 18,689 and
  lava 8,650 → 7,490 (both changed only by the ticks in between; nothing was
  lost), with the capacitor still at 675.5 K.

### Fail-capable check (evidence `e2e-selftest.log`)

`SELFTEST=1` flips the "no fuel" assertion; the run then fails as required:

```
FAIL: SELFTEST flipped: the unfuelled generator made no FE, as it should
EXIT:1
```

### The in-game check, delivered bundle (quoted from `e2e-embedded-default.log`)

The same scenario with **no `--dev-mod`**: the bundle's embedded content is what
registers the block. The control leg runs first, then the main run:

```
== control: the same game with no embedded content file ==
  launch: no --dev-mod; the bundle carries its own content
  arena: y=100.0 floor(0,99,3)=minecraft:stone feet(0,100,3)=minecraft:air
  placed block: minecraft:air
  native command: No block entity there.
  without the embedded file: no heat generator block or item; the native command still answers (it is in the binary)
== launch (Overworld arena at y=100, the delivered bundle) ==
  launch: no --dev-mod; the bundle carries its own content
  embedded content: 9380 B at /tmp/opencode/mekanismgenerators-home.93fWrr/.horizon-cache/mekanismgenerators-embedded-content.hmod, identical to content/mekanismgenerators.hmod
  arena: y=100.0 floor(0,99,3)=minecraft:stone feet(0,100,3)=minecraft:air
  block(0,100,0) = mekanismgenerators:heat_generator
  block(0,100,4) = mekanismgenerators:heat_generator
  generator: mekanismgenerators:heat_generator at 0,100,0: 0/160000 FE, lava 0/24000 mB, temp 300.0 K, producing 0 FE/t, burning false, fuel empty, t=126
  after 120 ticks: mekanismgenerators:heat_generator at 0,100,0: 0/160000 FE, lava 0/24000 mB, temp 300.0 K, producing 0 FE/t, burning false, fuel empty, t=256
  info(0,100,4): mekanismgenerators:heat_generator at 0,100,4: 0/160000 FE, lava 0/24000 mB, temp 300.0 K, producing 0 FE/t, burning false, fuel empty, t=264
  generator: mekanismgenerators:heat_generator at 0,100,0: 49152/160000 FE, lava 10730/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, fuel empty, t=734
  energy 49152 FE, lava 10730 mB, producing 112 FE/t
  50944 -> 63152 FE over ~100 ticks (delta 12208)
  full: mekanismgenerators:heat_generator at 0,100,0: 160000/160000 FE, lava 9410/24000 mB, temp 392.5 K, producing 0 FE/t, burning false, fuel empty, t=893
  after 100 ticks full: mekanismgenerators:heat_generator at 0,100,0: 160000/160000 FE, lava 9410/24000 mB, temp 300.3 K, producing 0 FE/t, burning false, fuel empty, t=1002
  drained: mekanismgenerators:heat_generator at 0,100,0: 4890/160000 FE, lava 8740/24000 mB, temp 663.9 K, producing 107 FE/t, burning true, fuel empty, t=1076
  before save: mekanismgenerators:heat_generator at 0,100,0: 5754/160000 FE, lava 8660/24000 mB, temp 667.9 K, producing 109 FE/t, burning true, fuel empty, t=1084
  after relaunch: mekanismgenerators:heat_generator at 0,100,0: 17457/160000 FE, lava 7610/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, fuel empty, t=1190
  (the saved compound survives: FE 5754 -> 17457, lava 8660 -> 7610)
PASS: a fuelled heat generator converts coal into lava, burns 10 mB/t and makes 112 FE/t into its own 160,000 FE buffer, an unfuelled one makes nothing, a full buffer produces nothing and burns no lava while a drained one resumes, and the FE, lava and heat survive a save and relaunch
EXIT:0
```

The numbers match the earlier `--dev-mod` run within chat latency (112 FE/t,
12,208 FE in ~100 ticks, no burn at full, resume when drained, the compound
intact); what the run adds is that the content came from the bundle.

### Fail-capable check, delivered bundle (evidence `e2e-embedded-selftest.log`)

`SELFTEST=1 EMBEDDED=1 tools/e2e.sh` flips the "no fuel" assertion; the run then
fails as required (its control leg still passes first):

```
  without the embedded file: no heat generator block or item; the native command still answers (it is in the binary)
  embedded content: 9380 B at /tmp/opencode/mekanismgenerators-home.OSK4i5/.horizon-cache/mekanismgenerators-embedded-content.hmod, identical to content/mekanismgenerators.hmod
  SELFTEST: expecting the unfuelled generator to have made FE (this must fail)
FAIL: SELFTEST flipped: the unfuelled generator made no FE, as it should
EXIT:1
```

### Content load check (evidence `hmod-load.log`)

The shared unmodded `1d63e0a5` game with `--dev-mod mekanismgenerators.hmod`
registers the block (`block 0,64,0 = mekanismgenerators:heat_generator`) and
gives its item; the **same world without `--dev-mod`** answers
`Unknown block type 'mekanismgenerators:heat_generator'` and the block stays air,
so the check cannot pass vacuously. `PASS`, `EXIT:0`.

## Pin findings for the next slice

- `block_entity_energy` (API 142) gives a real FE buffer whose amount is the int
  `energy` in the saved compound, and it **does** have cross-block helpers
  (`native_energy`, `native_energy_insert/extract`, `native_move_energy`) — the
  mekanism and powah ports use them at this pin. ARCHITECTURE.md §3's gap row is
  stale on that point; this slice does not need it (the brief stops at storing).
- `level.block_state/block` through `LevelRef` (API 381) works inside a block
  entity tick; the lava-side count reads the six neighbours that way.
- A block entity's writes inside its own tick persist (the same result the
  `mekanism` port recorded): the saved `energy`, `lava` and `heat` all survived
  save + relaunch here, and `mark_modified` is still called for the chunk.
- `ctl container X Y Z item n` inserts on the next tick and the generator
  converts one item that tick, so a stack reads back as `n - ticks`.
- A fresh chat command reply takes ~1–2 s; the `t=` field in `info` is what the
  check uses to prove a reply is fresh, and `/mekanismgenerators energy` is
  confirmed by its own reply rather than a read-back (a producing generator
  gains FE again the tick after a set).

## Next slices (in dependency order)

1. **Energy output**: push the buffer into adjacent acceptors at
   `level_tick_post` with `native_move_energy` (the `powah` cable shape), so the
   generator feeds a cable/cell.
2. **The other generators** in the mod's own order: solar, wind, bio, gas,
   advanced solar — each is the same `TileEntityGenerator` shape with its own
   fuel rule.
3. **The fluid tank as a real capability** (`native_block_entity_fluids`), so
   pipes can feed lava and the passive bonus can read the block's own
   lava-logged state.
4. **GUI/menu + `active` state** and the comparator.
5. **The reactors/turbine multiblocks** (deep; separate slices).

## Review repairs (PR #674, 2026-11-11)

Review: `ports/_shared/horizon-mods/reviews/pr-674-mekanismgenerators.md` (CHANGES_REQUESTED).

| Reviewer's item | This job |
|---|---|
| artifact not in the registry release/staging path | **orchestrator** (no write access; horizon-mods#686) |
| relative link `(mekanism/Mekanism)`, missing original credit | link was already absolute; **`LICENSE` fixed** (see below) and the description rewritten |
| description ends bullets mid-sentence (thermal efficiency, active model) | **rewritten** from this file: the fuel/lava/heat/FE chain and the four gaps |
| **[P1] `support.rs:179-186` unconditionally replaces `*energy`** | **fixed** (see below) |
| ship the upstream MIT notice, not only a claim | **fixed**: the upstream copyright line (2017-2025 Aidan C. Brady) is in `LICENSE`, and the packager writes it into the archive |

### [P1] the energy hook clobbered every other block's registration

`on_block_entity_energy` wrote `(block == GENERATOR).then_some(...)`, so for every
block this mod does not own it wrote `None` into the hook's shared output — erasing a
buffer another bundle's helper had registered for that block, depending on hook order.
It now returns early for a foreign block, as the container hook in the same file already
did.

Tests (in `support.rs`, run as `game::mekanismgenerators_state::tests`):
`a_foreign_blocks_energy_registration_is_left_alone` (fails on the old code: the buffer
became `None`) and `the_generator_declares_its_output_only_buffer`.

### Packaging

`tools/package.py` writes `mixins/mekanismgenerators/LICENSE` into the bundle (the
bundle reader accepts crate text files there — `horizon_mods::index::native::text`), and
`tools/notice.py` writes `LICENSE` into the embedded content `.hmod`. Two runs give the
same bytes; `cargo horizon-mod check` passes.

Verified: `python3 tools/horizon.py test` in a build slot — staged compile `cargo_exit: 0`,
**21/21** embedded tests pass, including
`game::mekanismgenerators_state::tests::a_foreign_blocks_energy_registration_is_left_alone`
and `the_generator_declares_its_output_only_buffer`
(`ports/_shared/evidence/mekanismgenerators/review-repairs-test.log`). The test harness's
filter was widened from `game::mekanismgenerators::tests` to `game::mekanismgenerators` so a
port's own support module is covered from now on.

Artifact: `mekanismgenerators.hnmod` **23,076 B** `7e87140c455d4853cb7abe6e1e89bfcee413231b5f596a2068bd10ce0d6a7905`
(content `mekanismgenerators.hmod` **10,332 B** `d01cc858a2c9804229eaddb36291231b28b0efaf290fd495e783fa12fd09ee37`).

## Migration to the SDK format (version 0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`, and the
registry closed every `.hnmod` submission with one reason. The unit is now **one flat
`.hmod`**: content, native code, or both. Pins for this migration: nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78, `cargo-horizon-mod`
from `horizon-rust/horizon@main`. The recipe this port follows was written from `imfast`
(the first migration); this port is the first **content+code** port to go through it, so
§"What this port adds to the recipe" below is what the large ports need.

### The line that makes this a migration and not a rewrite

`src/runtime.rs` is `mixins/mekanismgenerators/runtime.rs` **byte-identical** (SHA-256
`31349d4d6a6a85106a6a6c9b04db7dc2d4f03490171294a0a9b39e4dfe3d12bc`; `cmp` exit 0 against
the retired file, and the packaged entry is again the same bytes —
`ports/_shared/evidence/mekanismgenerators/runtime-carryover.txt`). Its **19 tests were
green before any glue was written** (`sdk-runtime-first.log`: `running 19 tests` … `19
passed; 0 failed`, with `src/lib.rs` holding only `pub mod runtime;`), and they still pass
with the glue (24 tests).

### File map

| 0.2.0 path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity: `id`, `version = "0.2.0"`, `api = "0.2"`, `side`, `capabilities`, licence/homepage/issues, `[native]` game/`horizon`/`mixins` |
| `Cargo.toml` | one crate, edition 2024; its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: `register`, `register_commands`, `run_mod_command` |
| `src/runtime.rs` | the pure logic, byte-identical (fuel → lava, tank, capacitor, Carnot, FE) |
| `src/heat_generator.rs` | the glue: `HeatGeneratorPart` (a `BlockEntityPart`), `container()`, `battery()`, `read`/`write` of the saved compound, the environment probe, `info_line`, `set_energy`, and the tests of the declarations |
| `assets/mekanismgenerators/**`, `data/**` | the carried content, moved up from the retired `content/` **unchanged** (blockstate, models, two own textures + three plate textures, lang, 26.3 `items/` definition, self-drop loot, shaped recipe, pickaxe tag, creative tab) |
| `LICENSE`, `LICENSE-NOTICE.md` | inside the package, as the registry requires |
| `README.md` | the package's own description; the packer includes it, so editing it changes the sha256 |
| `tools/e2e.sh` | the in-game scenario, rewritten for a launcher-built game (see the gap) |

Moved to `ports/_shared/evidence/mekanismgenerators/retired-hnmod-0.1.0/`: `mixins/`
(`horizon-mixin.toml`, `support.rs`, the eight helpers), `mekanismgenerators.hnmod`,
`compatibility.json`, `rust-toolchain.toml`, the 0.1.0 `Cargo.toml`/`Cargo.lock`,
`content/` (with its README, which lists every carried file and its source) and the six
old tools (`horizon.py`, `package.py`, `check-embedded.py`, `hmod-load.sh`, `notice.py`,
`test_stage.py`/`test_template.py`). Nothing was deleted: the old tree is the evidence.

### Manifest mapping

| 0.1.0 | 0.2.0 |
|---|---|
| `horizon-mixin.toml` `[mod] id = "mekanismgenerators"` | `id = "mekanismgenerators"` (also the namespace) |
| `[mod] version = "0.1.0"` | `version = "0.2.0"` — a resubmission is a **new version** |
| `[mod] priority = 100` | gone; ordering is `#[listener(priority = …)]` on a hook |
| `[compatibility] horizon_revision = "1d63e0a5…"` | `[native] horizon = "0b14f8ac…"` (40 hex, exactly) |
| mixins `5a64af93…` (bundle) | `[native] mixins = "e59da9094f0c…"` — **it moved** |
| game 26.3 (template `horizon-mod.toml`) | `[native] game = "26.3"` |
| `[[module]] runtime` + `state` | **no plan section**: `src/runtime.rs` is a module, `src/heat_generator.rs` its glue |
| `[[injection]] hook = … + helper + bindings` | one `Hooks` method / one Part method / one declaration (table below) |
| `horizon-native.json` | gone |
| — | `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]` (the mod declares a block), `license`, `homepage`, `issues` |

### Helper → hook table

| 0.1.0 injection (API) | helper | 0.2.0 | kept / gap |
|---|---|---|---|
| `block_entity` (15) | `block_entity.rs` | `BlockEntityPart::block_entity`, attached to the type declared by `registrar.block_entity_type("heat_generator").blocks(&["heat_generator"]).build()` | **kept** — returns `Some(Compound::new())` |
| `block_entity_ticks` (15) | `ticks.rs` | `BlockEntityPart::block_entity_ticks` sets `event.ticks` | **kept** |
| `block_entity_container` (16) | `container.rs` | `Registrar::block_container(&block, ModContainer)` | **kept** — the same `ModContainer` struct |
| `block_entity_energy` (142) | `energy.rs` | `Registrar::block_energy(&block, ModEnergyBuffer)` | **kept** — the same `ModEnergyBuffer`, and the amount is still the int `energy` in the saved compound |
| `block_entity_tick` (17) | `tick.rs` | `BlockEntityPart::block_entity_tick` | **kept**; `mark_modified` became the `Changed`/`Unchanged` reply |
| `register_commands` (8) | `register.rs` | `Hooks::register_commands` pushing `Command::new("mekanismgenerators info <pos:block_pos>")` and `Command::op("… energy <amount:integer> <pos:block_pos>", 2)` | **kept** |
| `perform_command` (8) | `perform.rs` | `Hooks::run_mod_command`, reading the typed `CommandInvocation` | **kept, simpler**: no `split_whitespace` parsing, no string positions; feedback through `host.commands().feedback`, which keeps the same silence/track-output routing `native_feedback` had |
| `content_root` (target, not a hook) | `embedded_content.rs` + `data/mekanismgenerators-content.hmod` | — | **gone by design, not a gap**: one `.hmod` carries `assets/`, `data/` and `src/`, so the helper that wrote an embedded content file out and appended it to the dev list has nothing left to work around |

Host calls, 0.1.0 → 0.2.0:

| 0.1.0 (`Game`) | 0.2.0 |
|---|---|
| `game.res.providers().burn_time(items, stack, false)` | `host.item_burn_time(stack.item())` |
| `items().get(&stack.item).remainder` | `host.item_crafting_remainder(&stack)` (then `ItemStack::shrink(1)` for the rest) |
| `game.native_level(dimension)` + `native_level_block(level, pos)` | `host.level(host.dimension())` + `LevelRef::block(host, pos)` |
| `game.native_dimension()` | `host.dimension()` |
| `game.world.game_time` | `LevelRef::game_time(host)` |
| `game.world.modified.insert(chunk)` (`mark_modified`) | nothing: the tick reply dirties the chunk, `Changed` when the payload moved |
| `game.native_with_mod_block_entity(pos, closure)` | `host.block_entity(level, pos)` + `BlockEntityRef::{data, item_slot, merge_data}` |
| `game.native_feedback(Text)` | `host.commands().feedback(Text)` |

The seven stamped hooks and the one target injection are therefore **7 hooks mapped, 0
recorded as gaps**; the eighth (the embedded-content target) has no successor because the
format removed the reason it existed.

### Build and check (the SDK ladder, measured)

Run through the campaign runner, **from inside the port** (trap 1), with
`CARGO_INCREMENTAL=0`:

```sh
cd ports/mekanismgenerators
../tools/build-slot.sh run --job sdk-mg --port . --step test  -- cargo test
../tools/build-slot.sh run --job sdk-mg --port . --step lint  -- cargo clippy --all-targets -- -D warnings
../tools/build-slot.sh run --job sdk-mg --port . --step fmt   -- cargo fmt --check
rm -f Cargo.lock
../tools/build-slot.sh run --job sdk-mg --port . --step build -- cargo horizon-mod build
cp target/horizon-mod/mekanismgenerators.hmod ./mekanismgenerators.hmod
../tools/build-slot.sh run --job sdk-mg --port . --step check -- cargo horizon-mod check mekanismgenerators.hmod
../tools/build-slot.sh run --job sdk-mg --port . --step check -- cargo horizon-mod fits mekanismgenerators.hmod --build nightly
```

| Check | Result | Evidence |
|---|---|---|
| runtime carried over | `cmp` exit 0, SHA-256 `31349d4d…12bc`, packaged entry identical | `runtime-carryover.txt` |
| runtime tests before glue | **19 passed**, 0 failed (`src/lib.rs` = `pub mod runtime;` only) | `sdk-runtime-first.log` |
| all tests | **24 passed**, 0 failed | `sdk-test.log` |
| the glue's test can fail | `max_insert: 1` → `the_generator_declares_its_output_only_buffer` **FAILED** (`left: 1, right: 0`), restored → green | `sdk-glue-red.log` |
| lints | `cargo clippy --all-targets -- -D warnings` exit 0, **0 warnings** | `sdk-clippy.log` |
| formatting | `cargo fmt --check` clean (no `--all`: it would walk the fetched SDK) | `sdk-fmt.log` |
| package | `cargo horizon-mod build` → `target/horizon-mod/mekanismgenerators.hmod` | `sdk-build.log` |
| determinism | two consecutive builds **byte-identical** (`cmp`), same SHA-256 | this file, §package |
| registry check | `mekanismgenerators 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, **31,106 B**, unsigned, **EXIT 0** | `sdk-check.log`, `sdk-check-json.log` |
| fits the pinned build | `0b14f8a made for`, **EXIT 0** | `sdk-fits.log` |
| carried content | `diff -r assets/ data/` against the retired `content/` — **identical**, every file | this file, §File map |
| package size | 31,106 B, 24 entries — far inside the registry's per-file and per-package limits (`check` accepts it) | `sdk-check.log` |
| surface | `uses.json` in the package: **232** entries (112 type, 49 handle, 23 method, 20 hook, 20 trait, 5 part, 2 macro, 1 capability) | `sdk-uses-summary.log` |
| skills line | `python3 ports/tools/check-skills.py --notes ports/mekanismgenerators` | §Verification below |

**Package:** `mekanismgenerators.hmod` **31,106 B**, SHA-256
`500bf80d5d3f9927808c6b62ab68833f238afab22ac602c80920cc4982ee4da2`, **24 entries**
(`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, 10 asset
files, 5 data files, `src/lib.rs`, `src/heat_generator.rs`, `src/runtime.rs`, `uses.json`).

### What this port adds to the recipe (the first content+code migration)

1. **A content port's package root is the old `content/`.** `assets/` and `data/` move up
   unchanged; `content/horizon-mod.toml` is replaced by the root manifest (same `api = "0.2"`
   and `capabilities = ["registries"]`, plus `[native]`); the built `.hmod` is no longer a
   *second* package to embed. No carry tooling runs in this migration: the carried files are
   already the 26.3 shapes.
2. **Attach Parts, not hooks.** A block's behaviour is a `BlockEntityPart` attached to
   `block_entity_type(<name>).blocks(&[<name>]).build()`, and per-block capabilities are
   **declarations** (`block_container`, `block_energy`, `block_fluids`) on the block key.
   This is the shape every Mekanism machine reuses: one BE type per machine, one Part, and
   the container/tank/buffer declared beside it.
3. **The block id and the block entity type id are the same string** here
   (`mekanismgenerators:heat_generator`). The type is declared in this mod's namespace
   (`block_entity_type("heat_generator")`), never `minecraft:` — a `minecraft:` type is
   refused (`BlockEntityPartError::VanillaType`).
4. **The declaration replaces the shared output.** The old `block_entity_energy` hook wrote
   into a shared `Option<ModEnergyBuffer>` for *every* block in the game, which is what PR
   #674's [P1] was about; a declaration is keyed by the block, so the defect cannot recur.
   `the_generator_declares_its_output_only_buffer` pins the same numbers on `battery()`, and
   the old `a_foreign_blocks_energy_registration_is_left_alone` is retired as structurally
   impossible.

### Traps this migration hit (beyond `imfast`'s)

1. **The runner runs the command in the caller's working directory.** `build-slot.sh run
   --port ports/mekanismgenerators -- cargo test` from the campaign root answers
   `could not find Cargo.toml in …/all_the_mods_10_rust_rewrite`. An SDK package has no
   staged tree, so run from inside the port with `--port .`.
2. **The SDK is fetched per clone.** `cargo test` before any `cargo horizon-mod` command in
   the port fails with `failed to read …/.horizon/sdk/api/crates/horizon-mod-api/Cargo.toml`.
   One `cargo horizon-mod` command (e.g. `hooks block_entity`) fetches `.horizon/sdk/`.
3. **Reads of a level are on the `LevelRef` handle, not on `host`.** `host.level_block` and
   `host.level_game_time` do not exist: it is `level.block(host, pos)` and
   `level.game_time(host)` (the host's `LevelsHost` carries only `is_loaded` and `chunk`;
   the rest is the hidden `LevelsBackend`, and the generated `impl LevelRef` is the mod-side
   surface).
4. **A `Compound` is keyed by `JavaString`, which is not exported.** `nbt.get("energy")`
   does not compile, and `Compound::get` takes the UTF-16 key type. `Tag::get(&str)` is the
   public reader, so the tick does one `std::mem::take` of the map, lends it to a `Tag`, and
   takes it back — no clone and no per-tick allocation. That is why `read`/`write` take
   `&Tag`/`&mut Tag`.
5. **`block_entity_part` returns a `Result` and records nothing itself.** A refused part
   would vanish silently; `register` hands the error to `RegistrationContext::refuse`.
6. **A tick Part is `Changed` by default**, which marks the chunk every tick. Keep the old
   `mark_modified` condition by setting `*event.reply = BlockEntityTickReply::Unchanged` when
   the fuel, the state and the slot count did not move. (Overworld item scanners read a
   permanently dirty chunk as activity.)
7. **`#[horizon_part]` fills only the trait it is on.** `BlockEntityPart` needs a second,
   empty `#[horizon_part] impl BlockPart for …`, or `block_subscribed` is missing.
8. **The packer includes root text files.** `README.md` and `LICENSE*` are inside the
   `.hmod`, so a README edit changes the SHA-256; `notes.md` and `FINDINGS.md` are not
   packed. This migration's first build carried the stale README and was rebuilt.
9. **`check --json` reports `"side": null` for a native package** even with `side = "both"`
   in the manifest (native mods are not in the side list yet). That is the checker's
   report shape, not a manifest error.
10. **`cargo test`/`clippy` write `Cargo.lock`** — remove it before packaging, or the artifact
    changes and the lock resolves crates newer than the SDK's own (`imfast` trap 5).
11. **The packaged manifest must be read back, not assumed.** After this port's first
    `cargo horizon-mod build`, `horizon-mod.toml` no longer carried `license`, `homepage` or
    `issues`; re-adding them and rebuilding kept them, and `unzip -p mekanismgenerators.hmod
    horizon-mod.toml` now shows all three. Whatever the cause, the check to make is the packed
    file (`unzip -p`), which is what the registry reads.

### Gaps after the migration

Behaviour gaps 1–13 above are unchanged: the slice boundary is the same. The migration adds
three record-level ones:

14. **No in-game run at 0.2.0.** `horizon-launcher` is not installed here, so a game carrying
    this package cannot be built (an SDK package is compiled **into** the client by the
    launcher; there is no staging path for it). `tools/e2e.sh` was rewritten to take
    `GAME=` (the launcher build carrying the package) and `PLAIN_GAME=` (an unmodified build
    for the control leg) and was **not run**; the 0.1.0 runs below remain the in-game
    evidence for the behaviour, which the byte-identical `runtime.rs`, the same saved keys,
    the same numbers and the same command line make the same behaviour.
    **The first thing the in-game run must confirm** is the one claim this migration cannot
    test offline: that the content block gets its block entity from the *declaration*
    (`block_entity_type("heat_generator").blocks(&["heat_generator"])`), where 0.1.0 asked
    the `block_entity` hook. Place the block and read `/mekanismgenerators info`: a
    registered entity answers `0/160000 FE`, and a missing one answers `No block entity
    there.` — that is the assertion the scenario already makes first. The second is the
    foreign-block guard: `info` at a chest must answer `No block entity there.`
15. **The old `content_root` control leg is gone.** It proved the block came from the
    package's embedded file; in the SDK format the package *is* the block, so the control is
    the unmodified game (`PLAIN_GAME`): the block id is unknown and `/mekanismgenerators` is
    not a command there.
16. **The two `support.rs` tests are retired** (see §"What this port adds to the recipe", 4).
    Their replacements are `the_generator_declares_its_output_only_buffer` and
    `the_fuel_slot_is_the_only_slot_and_every_face_reaches_it` in
    `heat_generator::tests`, plus `the_tick_s_state_reads_back_from_the_compound_it_wrote`
    and `the_info_line_is_the_one_the_e2e_asserts`.

### What the rest of the Mekanism family reuses (`mekanismtools`, `mekmm`)

- **The Part + declaration machine shape**: one `block_entity_type(...).blocks(&[…])` per
  machine, one `BlockEntityPart`, and `block_container` / `block_energy` / (new for `mekmm`)
  `block_fluids` on the block key. This is the entire "Mekanism machine" model — a BE, a
  slot, a tank, a buffer — and it is the single pattern the family shares.
- **The saved-state convention**: `energy` (int, the capability's amount), `lava`/`mB`
  ints, `heat` as `Tag::Double` bits, plus the report fields, read and written through the
  `Tag::get(&str)` / `mem::take` pair. `mekmm`'s machines can add keys beside these without
  a new mechanism.
- **The typed command pair** as the GUI-less harness control (`register_commands` +
  `run_mod_command` with `block_pos`), and `host.commands().feedback` for the reply.
- **The reply discipline**: `Unchanged` when an idle machine changed nothing.
- **The package shell and ladder**: manifest fields, pin triples, `rm -f Cargo.lock`,
  `cargo fmt --check`, the runner invocation from inside the port, and the byte-identical
  `runtime.rs` rule.
- **One-offs not to copy**: the Carnot/heat maths and the fuel burn-time rule (this port's
  runtime), the lava-side environment probe (each generator probes its own surroundings),
  and the `minecraft:the_nether` id test standing in for `DimensionType.ultraWarm`.

For `mekanismtools` the same rule holds but the parts differ: its armour and tools are
`ItemPart`s (tooltips, hurt, attributes), so its glue is a Part on item keys rather than a
`BlockEntityPart`, and the deep-tier behaviours (attribute modifiers, renderers) are the
gaps to record. For `mekmm` the machine shape above applies directly, with `block_fluids`
for its tanks.

### Changelog for the resubmission (verbatim)

> Supersedes #840, closed because Horizon retired `.hnmod` (#17992). Changes: flat SDK
> package with `api = "0.2"` and `[native]` pinned to `0b14f8a`; hooks rewritten as
> `impl mod_api::Hooks` methods against typed hosts/events (7 hooks mapped, 0 recorded as
> gaps; the `content_root` embedded-content workaround is no longer needed, because one
> `.hmod` carries the content and the code); `runtime.rs` and its tests unchanged.

## Review fixes (2026-11-18)

`var/review-all/1189-mekanismgenerators.md`, the one finding fixed; version **0.2.1** (32,286 B,
sha256 `93074380504c2feb932c6a1ee9ace3898bdb225bb47aa18c7147f23a13181add`).

- **Info disclosure (src/heat_generator.rs:165, src/lib.rs:52).** `/mekanismgenerators info`
  read the fuel item and count from any loaded generator at caller-supplied coordinates. It now
  requires an operator (permission ≥ 2) or a player in the same level within 8 blocks of the
  block; any other caller gets a generic refusal and no fuel/energy line. Pure rule:
  `runtime::may_inspect`, pinned by `a_readout_needs_an_operator_reach_or_the_same_level`.

### In-game scenario updated (harness limitation, not a weakening)

The e2e inserted coal with `ctl container 0 100 0 …`; at this pin that command reports
`container is full` for a native mod block entity (the agent's world mirror has no slots for it,
while the block entity itself works). The scenario now loads a vanilla hopper above the generator
and lets it push through the block's declared container capability — the automation path a player
uses — and asserts the same chain: coal becomes lava, burning is true, 112 FE/t into the buffer, a
full buffer produces no FE, a drained one resumes, and FE/lava/heat survive a save and relaunch.
The old `lava > 2000` check became `0 < lava < 1000` because a one-at-a-time feed keeps the tank
nearly empty by construction; fuel consumption is also pinned by the hopper's falling stock. PASS
line and the full run are in `ports/_shared/evidence/mekanismgenerators-e2e/`.

Ladder: `cargo test` passed; `cargo clippy --all-targets -- -D warnings` exit 0; `cargo fmt --check`
clean; `cargo horizon-mod build` twice byte-identical; `check` and `fits` exit 0. In-game:
`launcher-game.sh run mekanismgenerators` → `PASS` + `EXIT:0`; `GAME=$CONTROL_GAME` → `FAIL`
(`Unknown block type 'mekanismgenerators:heat_generator'`) + `EXIT:1`.

## Slice 2 — the energy output (version 0.2.2, 2026-11-21)

Pin **Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (nightly 2026-11-20) + mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`; the campaign repin from `0b14f8a`. The SDK in
`.horizon/sdk` was refreshed from the `0a875da` release (**trap**: bumping `[native] horizon` alone
leaves the old SDK in place, and the next build silently compiles against `0b14f8a`).

**The gap line this slice closes** is gap 1 of §"Deviations and gaps (slice 1)" and the registry
entry's own "No FE *output* into a neighbour":

> **No FE output.** The generator fills its own buffer and nothing pushes it into a neighbour (the
> brief's slice boundary; the transfer helpers exist at this pin, so a cable slice can add it).

It fits the tier as a **machines** feature (the block entity, its FE capability and its tick are
already here; this adds the energy balance's other half), and it is the family the parent mod makes
provable: `mekanism:basic_energy_cube` (our own port, merged) is storage — it takes FE on every face
and pulls from nothing — so a cube that gains FE beside the generator's `facing` face gained it
*from this generator*, and a cube on the opposite face must stay empty.

### Upstream rule → the code

| Upstream (`TileEntityGenerator`, tag `v1.21.1-10.7.19.85`) | This port |
|---|---|
| `ENERGY_SIDES = {RelativeSide.FRONT}` — one output side, the block's `facing` | `emitting_face` reads the block state's `facing` (`level.block_property`) and emits into that neighbour |
| `getMaxOutput()` = `MAX_PRODUCTION x 2` = 1,020 FE/t | `MAX_OUTPUT` (unchanged constant) caps the offer |
| `CableUtils.emit(outputs, container, getMaxOutput())`: extract `min(stored, maxOutput)`, let the acceptor take what fits, subtract what it took | `runtime::HeatGenerator::output_offer(room, max_output)` = `min(stored, max_output, room)`; `HeatGenerator::emit(accepted)` subtracts the acceptor's reply |
| emit runs after the generator's own update (`onUpdateServer`) | `emit_front` is called after `state.tick`, before the state is written back |
| the acceptor answers through its own handler (`IStrictEnergyHandler.insert`) | `level.capability(host, &ENERGY, neighbour, Some(face.opposite()))` → `insert(host, tx, want)` in one `host.transaction`, the `powah`/`mekanism` cable shape |

The buffer is this port's saved int `energy`, not a capability handle read back: during the tick the
compound has been taken out of the block entity, so the move is measured by the **acceptor's reply**
(`insert` returns what it took) and the buffer loses exactly that. An acceptor that over-answers
aborts the transaction and nothing moves (`output_offer`/`emit` clamp as well).

### Red → green

1. **Unit red** (`ports/_shared/evidence/mekanismgenerators/sdk-output-red.log`, `cargo_exit 101`):
   the two new runtime tests — `the_front_block_is_offered_the_buffer_up_to_the_output_rate` and
   `emitting_takes_only_what_the_front_block_accepted` — did not compile:

   ```
   error[E0599]: no method named `output_offer` found for struct `runtime::HeatGenerator` in the current scope
   error[E0599]: no method named `emit` found for struct `runtime::HeatGenerator` in the current scope
   ```

2. **Glue red** (mutation, restored): `facing_of("north")` forced to `East` →
   `heat_generator::tests::the_emitting_face_is_the_one_the_block_state_names` **FAILED**,
   `left: Some(East) right: Some(North)`; restored, green.
3. **In-game red** (`ports/_shared/evidence/mekanismgenerators-e2e/e2e-launcher-emission-red.log`):
   the same two-mod game built from sources with the `emit_front` call removed — the front cube stays
   at 0 while the generator's buffer grows:

   ```
     front cube: 0 FE, back cube: 0 FE
     generator held 56432 FE, now: mekanismgenerators:heat_generator at 0,100,0: 73568/160000 FE, …
     front cube: 0 -> 0 FE (+0), back cube: 0 FE
   FAIL: the buffered FE did not reach the front cube (+0 of 56432): mekanismgenerators:heat_generator at 0,100,0: 73568/160000 FE, lava 70/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, facing north, fuel empty, t=964
   EXIT:1
   ```

   Restored and rebuilt: the same scenario passes (below). The launcher keys a native build on the
   sources (the red and green builds are different versions, `0fcd04c07fd0` vs `0eb8c98ee99b`), so
   the red run really ran different code.

### Ladder (all through `ports/tools/build-slot.sh`, from inside the port)

| Step | Command | Result | Evidence |
|---|---|---|---|
| unit tests | `cargo test` | **28 passed**, 0 failed, exit 0 | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| format | `cargo fmt --check` | clean | `sdk-fmt.log` |
| build ×2 | `rm -f Cargo.lock` + `cargo horizon-mod build` | **byte-identical** (`cmp` exit 0) | `sdk-build.log`, `sdk-build-2.log` |
| registry check | `cargo horizon-mod check mekanismgenerators.hmod` | `0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **35,034 B**, exit 0 | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits … --build 0a875da` after `sha256sum` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| fits, another build | `… --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-other.log` |
| package gate | `verify-sdk-package.py mekanismgenerators` | 1/1 | report |
| in-game (two mods) | `tools/build-game.sh build` then the leg | `PASS` + `EXIT:0` | `mekanismgenerators-e2e/e2e-launcher.log` |
| control | same scenario, `GAME=$PLAIN_GAME` | `FAIL` (`Unknown block type …`) + `EXIT:1` | `…-control.log` |
| fail-capable | `SELFTEST=1` | `FAIL` + `EXIT:1` | `…-selftest.log` |
| log checker | `check-e2e-logs.py mekanismgenerators-e2e --strict` | `logs=4 ok=4 missing-exit=0 bad-exit=0` | report |

**Package:** `mekanismgenerators.hmod` **35,034 B**, SHA-256
`c68cf4f1cd3318083da0169be1d8ee192bce74cbdeca918124f0e28647192ad9`, **24 entries**, no
`Cargo.lock`. A copy of the delivered file is at
`ports/_shared/evidence/mekanismgenerators/mekanismgenerators.hmod` with its hash in
`sdk-sha256.txt`.

### The in-game check (quoted from `e2e-launcher.log`)

The game carries **two** packages — this one and the parent `mekanism` — because the acceptor is the
parent's own block. `tools/build-game.sh` stages both and calls `native-build` on the folder (the
launcher takes "a mod folder … or a folder of them"); the export lands where
`launcher-game.sh env mekanismgenerators` expects it.

```
== the front face emits into an acceptor ==
  info(0,100,0): mekanismgenerators:heat_generator at 0,100,0: 55536/160000 FE, lava 0/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, facing north, fuel minecraft:coal x1, t=802
  block(0,100,-1) = mekanism:basic_energy_cube
  block(0,100,1) = mekanism:basic_energy_cube
  front cube: 15300 FE, back cube: 0 FE
  generator held 56432 FE, now: mekanismgenerators:heat_generator at 0,100,0: 0/160000 FE, lava 0/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, facing north, fuel minecraft:coal x1, t=962
  front cube: 15300 -> 71888 FE (+56588), back cube: 0 FE
== the emit rate is the generator's own production ==
  set(0,100,0) = 0 FE
  front cube: 76144 -> 88240 FE over ~100 ticks (gain 12096)
== counterfactual: with the front cube gone the generator stores its own FE ==
  set(0,100,0) = 0 FE
  mekanismgenerators:heat_generator at 0,100,0: 11984/160000 FE, lava 20/24000 mB, temp 675.5 K, producing 112 FE/t, burning true, facing north, fuel empty, t=1208
```

What the numbers prove:

- the generator held **56,432 FE** and the cube in front of its `facing` (north, -z) went from 15,300
  to **71,888 FE** (+56,588): the whole buffer left at up to 1,020 FE/t, and the generator then holds
  **0** — it is emitting, not hoarding;
- a cube on the **opposite** face stayed at **0 FE** through both windows, so the emission follows the
  block state's `facing` and is not a radial push;
- with the buffer drained, the front cube gains **12,096 FE in ~100 ticks**, the generator's own
  112 FE/t within chat latency (the same rate the no-acceptor control measures in its buffer);
- with the front cube removed the generator stores **11,984 FE in ~100 ticks** itself, so the
  emission is what moves the energy, not a loss.

### Coverage after this slice

| Family (jar) | State | Where |
|---|---|---|
| fuel slot → lava, the tank, the capacitor, the Carnot conversion, the FE buffer | `tested` | slice 1's ladder + this leg (`burning true`, `112 FE/t`, full-buffer/drain controls) |
| **energy output to a neighbour** | `tested` | this leg, §"the front face emits": buffer dump + steady rate + back-face differential + counterfactual |
| `/mekanismgenerators info` | `tested` | every step asserts the parsed line, now including `facing` |
| `/mekanismgenerators energy` | `tested` | `set(…) = N FE` replies in this leg |
| block, item, pickaxe tag, creative tab | `tested` | placement + `Unknown block type` control |
| the carried recipe (`heat_generator.json`) | `blocked(no c:ingots/osmium provider)` | loads, and the game logs `recipes: invalid ingredient … "#c:ingots/osmium"`; nothing in this campaign's mods registers that tag, so it cannot be crafted here and is **not** claimed |
| fluid capability, GUI/menu, `active` model + light, comparator, custom shape, side configuration | `not ported` | §"Deviations and gaps" 2–8 |
| the other generators, reactors, turbine, suit modules | `not ported` | §"Deviations and gaps" 11 |

No upstream defect was found in this family: the emit side, the rate and the "only what the acceptor
takes" arithmetic all match `TileEntityGenerator`/`CableUtils` at the tag read.

### Gaps after slice 2

1. ~~No FE output~~ — **closed**: the generator emits up to 1,020 FE/t into the block on its
   `facing` side.
2. **One acceptor, one side**, as upstream (a second front neighbour is not a thing in a block
   world), but the buffer is *also* still declared extractable on all six faces, so a cable may pull
   in parallel with the push (upstream exposes the container on the front only). Unchanged from
   slice 1 and recorded as deviation 6.
3. **The emit side follows the carried `facing` property**, read every tick. Upstream caches the
   output neighbour and invalidates the cache when the block is rotated; reading the state each tick
   gives the same result (a rotated block emits from its new face) without the cache.
4. **`producing N FE/t` still reports the production rate**, not the net change of the buffer: with an
   acceptor attached the buffer stays at 0 while the readout says 112 FE/t, which is what the FE/t
   means (upstream's `getProductionRate`).
5. Everything else from §"Deviations and gaps (slice 1)" is unchanged (fluid capability, GUI,
   `active`, comparator, geometry, side configuration, the other generators, reactors, turbine).

## Next slice

The next family in the port's own gap list is **gap 2, the lava tank as a real fluid capability**
(`Registrar::block_fluids` on the block key, a `ModFluidTanks` handler holding
`mekanismgenerators:lava`): upstream's `lavaTank` is an `IFluidHandler`, so today a pipe can neither
fill it nor read it, and the passive lava-side heat can only be reached from the fuel slot. It is a
`machines` feature (`block_fluids`, one tank, no new block or renderer), and it is provable in game
with the same two-mod build plus `rangedpumps` (also ported): `rangedpumps:pump` pushes the fluid it
holds into every adjacent handler, so with the pump drawing lava and the generator beside it,
`/mekanismgenerators info 0 100 0` must show a **rising `lava`** (`lava N/24000 mB`, N > 0 with an
empty fuel slot) and — with no lava beside the block and no fuel — a `temp` above **300.0 K** after
the tank fills, while the same run with the pump absent keeps `lava 0` and `temp 300.0 K`. The
recipe family stays `blocked(no c:ingots/osmium provider)`.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices
