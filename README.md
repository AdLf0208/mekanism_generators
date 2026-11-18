# Mekanism Generators — a Horizon port

**Mekanism Generators** is a Minecraft mod. A mod for Minecraft. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [mekanism/Mekanism](https://github.com/mekanism/Mekanism) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Mekanism Generators ([source](https://github.com/mekanism/Mekanism)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `mekanismgenerators.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- a **fuel slot** (container, slot 0) that takes any item with an engine burn
- a **24,000 mB lava tank** (this port's saved int `lava`; upstream's fluid
- a **heat capacitor** (10 heat units) fed by 200 heat/t while lava burns, 30
- a **160,000 FE buffer** (`native_block_entity_energy`, the int `energy` in the
- `/mekanismgenerators info|energy`, the harness controls, since there is no GUI.
- **A content port's package root is the old `content/`.** `assets/` and `data/` move up
- **Attach Parts, not hooks.** A block's behaviour is a `BlockEntityPart` attached to
- **The block id and the block entity type id are the same string** here

## Not yet ported

- **No FE output.** The generator fills its own buffer and nothing pushes it
- **No fluid capability.** The tank is a saved int; no pipe can fill it and
- **No GUI.** Fuel goes in through the container (automation's path, and what
- **No `active` block state**, so the `heat_generator_active` model, the
- **No custom shape**: the barrel template's full cube collision is used.
- **No side configuration**: fuel enters and FE leaves on every face, where
- **Ambient temperature is a fixed 300 K** (`HeatAPI.AMBIENT_TEMP`), not the
- **No comparator** on the tank; `has_analog_output` is not declared.

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `mekanismgenerators.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Mekanism Generators was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/mekanism/Mekanism

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run mekanismgenerators` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/mekanismgenerators/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
