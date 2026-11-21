# Validation — `mekanismgenerators` 0.2.2 (slice 2, the energy output)

Pin **Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (nightly 2026-11-20) + horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`. Every command below ran through
`ports/tools/build-slot.sh` (`--port .` from this directory, or `--port ports/mekanismgenerators`
from the campaign root), never a bare `cargo`, with `CARGO_INCREMENTAL=0`. `notes.md`
§"Slice 2 — the energy output" is the narrative; this file is the ladder and where each artefact is.

## The package

| | |
|---|---|
| file | `mekanismgenerators.hmod` (a copy at `ports/_shared/evidence/mekanismgenerators/mekanismgenerators.hmod`) |
| size | **35,034 B** |
| sha256 | **`c68cf4f1cd3318083da0169be1d8ee192bce74cbdeca918124f0e28647192ad9`** |
| declared version | `0.2.2` (`horizon-mod.toml`; `Cargo.toml` is not the identity) |
| format | `native-1/0a875da91dbb0fdcb7ba8c42c11341aa229af51a/e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, for game 26.3 |
| entries | **24**, no `Cargo.lock` |

## Ladder

| # | Step | Command | Result | Evidence |
|---|---|---|---|---|
| 1 | unit red | `cargo test` before the rules existed | **exit 101**, `no method named output_offer` / `no method named emit` | `evidence/mekanismgenerators/sdk-output-red.log` |
| 2 | unit | `cargo test` | **28 passed**, 0 failed, exit 0 | `sdk-test.log` |
| 3 | lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** | `sdk-clippy.log` |
| 4 | format | `cargo fmt --check` | clean | `sdk-fmt.log` |
| 5 | build 1 | `rm -f Cargo.lock` → `cargo horizon-mod build` | exit 0 | `sdk-build.log` |
| 6 | build 2 | `rm -f Cargo.lock` → `cargo horizon-mod build` | **`cmp` byte-identical** to build 1, same sha | `sdk-build-2.log` |
| 7 | registry check | `cargo horizon-mod check mekanismgenerators.hmod` | `0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 35,034 B, **exit 0** | `sdk-check.log` |
| 8 | fits, the pin | `sha256sum` + `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 9 | fits, another build | `sha256sum` + `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-other.log` |
| 10 | package gate | `python3 ports/tools/verify-sdk-package.py mekanismgenerators` | **1/1** | `sdk-verify.log` |
| 11 | game | `bash tools/build-game.sh build` (this package + `mekanism`) | built `native-0a875da-0eb8c98ee99b`, both mods compiled | `e2e/mekanismgenerators-e2e/e2e-launcher.log` |
| 12 | in game | the leg on that game | **`PASS` + `EXIT:0`** | `e2e/mekanismgenerators-e2e/e2e-launcher.log` |
| 13 | in-game red | same leg, game built with the emission disabled | **`FAIL` + `EXIT:1`** (`+0 of 56432`) | `e2e/mekanismgenerators-e2e/e2e-launcher-emission-red.log` |
| 14 | control | same leg on the unmodified Nightly | **`FAIL`** (`Unknown block type …`) + `EXIT:1` | `e2e/mekanismgenerators-e2e/e2e-launcher-control.log` |
| 15 | fail-capable | `SELFTEST=1` | **`FAIL` + `EXIT:1`** | `e2e/mekanismgenerators-e2e/e2e-launcher-selftest.log` |
| 16 | log checker | `check-e2e-logs.py mekanismgenerators-e2e --strict` | `logs=4 ok=4 missing-exit=0 bad-exit=0` | report |
| 17 | skills gate | `check-skills.py --notes ports/mekanismgenerators --strict` | PASS | report |

## The two-mod game

The energy output is proven against a real acceptor, `mekanism:basic_energy_cube` from the parent
port (storage: it takes FE and pulls from nothing). `tools/build-game.sh` stages this port and
`../mekanism` in one folder and calls `horizon-launcher native-build <folder>` — the launcher accepts
"a mod folder … or a folder of them" — then exports it exactly where
`ports/tools/launcher-game.sh env mekanismgenerators` looks. The wrapper itself builds one port and
refuses a folder, which is why that script exists.

```
bash tools/build-game.sh build
eval "$(bash tools/build-game.sh env)"
ports/tools/build-slot.sh run --job W1-mekanismgenerators --port ports/mekanismgenerators \
    --step e2e -- bash -c 'eval "$(bash ports/mekanismgenerators/tools/build-game.sh env)"; bash ports/mekanismgenerators/tools/e2e.sh'
```

Run the leg as `--step e2e`: `--step build` is forwarded to the second machine, which does not carry
the launcher, and the launcher leg compiles no cargo workspace anyway.
