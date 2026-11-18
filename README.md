# Mekanism: Generators for Horizon — slice 1: the Heat Generator

A **native Horizon mod** in the flat SDK package (mod API 0.2, `.hmod`) ported from
Mekanism: Generators 10.7.19.85 (MIT): one package carries the block, its item,
models, textures, lang, loot table, recipe and creative tab (`assets/`, `data/`)
**and** the Rust that runs it (`src/`, compiled by the launcher against the
pinned game build).

- `mekanismgenerators:heat_generator` — turns burnable items into lava (80 mB a
  coal), burns 10 mB/t into a 10-heat-unit capacitor, converts the heat loss
  through the Carnot efficiency into FE in its own **160,000 FE** buffer, and
  settles at **112 FE/t** on fuel, 96 FE/t on six lava sides, 39 FE/t in the
  Nether. The buffer is a real block energy capability, so another mod's cable
  can drain it.

Read [notes.md](notes.md) for the slice boundary, the deviations, the gap list
and the migration record, and [FINDINGS.md](FINDINGS.md) for the upstream
analysis (the files read and the numbers from tag `v1.21.1-10.7.19.85`).

The rules live in `src/runtime.rs` (byte-identical to the retired `.hnmod`
bundle's `runtime.rs`, and testable with `cargo test` alone); `src/lib.rs` is the
one `impl Hooks` and `src/heat_generator.rs` is the block's Part, its container
and energy declarations and the command bodies. The harness commands
`/mekanismgenerators info <x> <y> <z>` and `/mekanismgenerators energy <amount>
<x> <y> <z>` exist because this slice has no GUI.

```sh
cargo test                                    # the pure runtime + the glue's declarations
cargo clippy --all-targets -- -D warnings
cargo fmt --check
rm -f Cargo.lock                              # the package must not carry a lock (notes.md, trap 5)
cargo horizon-mod build                       # -> target/horizon-mod/mekanismgenerators.hmod
cp target/horizon-mod/mekanismgenerators.hmod ./mekanismgenerators.hmod
cargo horizon-mod check mekanismgenerators.hmod
cargo horizon-mod fits mekanismgenerators.hmod --build nightly
```

Run every build through the campaign's slot registry (`ports/tools/build-slot.sh`
with `--port .` from this directory), never a bare build in a port. The
in-game scenario is `tools/e2e.sh`; it needs a **launcher-built** game carrying
this package (`horizon-launcher native-build . --instance <i> --trust-native-code`)
and `GAME=` pointing at it, plus a matching `horizon-ctl`.

**Local-only** until the maintainer approves publishing (MIT, licence rank 0;
ARCHITECTURE.md §8).
