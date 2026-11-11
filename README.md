# mekanismgenerators (Horizon port)

A Horizon port of [mekanismgenerators](mekanism/Mekanism) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- `mekanismgenerators:heat_generator`, one block:
- a **fuel slot** (container, slot 0) that takes any item with an engine burn
- time and turns one item a tick into `burnTime / 20` mB of lava (coal 80 mB,
- a lava bucket 1,000 mB; a crafting remainder replaces the stack);
- a **24,000 mB lava tank** (this port's saved int `lava`; upstream's fluid
- capability is not declared in this slice, so only the fuel slot fills it);
- a **heat capacitor** (10 heat units) fed by 200 heat/t while lava burns, 30
- per lava side and 100 in the Nether, cooled by the 0.5 thermal-efficiency

## Not yet ported

- No FE output.** The generator fills its own buffer and nothing pushes it
- into a neighbour (the brief's slice boundary; the transfer helpers exist at
- this pin, so a cable slice can add it).
- No fluid capability.** The tank is a saved int; no pipe can fill it and
- nothing can read it as lava (upstream's `lavaTank` is a fluid handler).
- No GUI.** Fuel goes in through the container (automation's path, and what
- the check uses) and state is read with the command.
- No `active` block state**, so the `heat_generator_active` model, the

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: mekanism/Mekanism. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
