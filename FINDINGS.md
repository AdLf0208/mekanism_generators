# Mekanism: Generators — upstream findings (slice 1, the Heat Generator)

Read from `https://github.com/mekanism/Mekanism`, tag `v1.21.1-10.7.19.85`,
commit `a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`, cloned read-only at
`ports/_shared/upstream/mekanism` (the addon is the `src/generators/` module of
the monorepo). Analysis only: the port is a clean-room Rust rewrite, no Java was
copied or translated.

## Files read

| File | What it gave |
|---|---|
| `src/generators/java/mekanism/generators/common/tile/TileEntityHeatGenerator.java` | the thermal constants, the tank, the fuel slot, the tick order, the passive boost, the Carnot conversion and its 510 cap |
| `src/generators/java/mekanism/generators/common/tile/TileEntityGenerator.java` | the output buffer and `getMaxOutput` (twice the production cap), the front-side emit |
| `src/generators/java/mekanism/generators/common/config/GeneratorsConfig.java` | `heatGeneration` 200, `heatGenerationLava` 30, `heatGenerationNether` 100, `heatTankCapacity` 24,000, `heatGenerationFluidRate` 10 |
| `src/generators/java/mekanism/generators/common/config/GeneratorsStorageConfig.java` | `storage.heatGenerator` 160,000 FE |
| `src/generators/java/mekanism/generators/common/slot/FluidFuelInventorySlot.java` | one item a tick becomes `burnTime / 20` mB, only when the whole amount fits the tank; container remainders |
| `src/main/java/mekanism/common/capabilities/heat/BasicHeatCapacitor.java` | temperature = stored heat / heat capacity; starts at ambient x capacity |
| `src/api/java/mekanism/api/heat/HeatAPI.java` | `AMBIENT_TEMP` 300, `AIR_INVERSE_COEFFICIENT` 10,000, `DEFAULT_INVERSE_INSULATION` 0 |
| `src/main/java/mekanism/common/capabilities/heat/ITileHeatHandler.java` | the environment leak per face: `(T - ambient) / (air + insulation + conduction) x heatCapacity` |
| `src/generators/java/mekanism/generators/common/registries/GeneratorsBlockTypes.java`, `GeneratorsBlocks.java` | the block type, the GUI, the custom shape/sound, `ACTIVE_MELT_LIGHT`, the item's fluid/heat/item containers |
| `src/main/java/mekanism/common/block/prefab/BlockTile.java` | the default block properties `strength(3.5F, 16)`, `requiresCorrectToolForDrops` |
| the jar's `assets/mekanismgenerators/**`, `data/mekanismgenerators/**` | the blockstate, models, textures, lang, loot and recipe carried |

## Behaviour extracted

- **Fuel → lava.** The fuel slot takes any item with a burn time; one item a
  tick becomes `burnTime / 20` mB of lava into a 24,000 mB tank, and only when
  the whole amount fits (`fuel > 0 && fuel <= tank.getNeeded()`). A coal (1600
  ticks) is **80 mB**; a lava bucket (20,000 ticks) is **1,000 mB**. A
  container remainder (the empty bucket) replaces the stack.
- **Burning.** While the FE buffer has room, a tick extracts **10 mB** and adds
  **200 heat** to the capacitor; otherwise nothing is extracted (the lava is
  kept).
- **Passive heat.** Every tick adds `30 x lavaSides` heat (a lava-logged block
  counts as a seventh side) and, in an ultrawarm dimension, another **100**.
- **Conversion.** Every tick: `carnot = 1 - min(T, ambient) / max(T, ambient)`;
  `heatLost = 0.5 x (T - ambient)`; the capacitor loses that heat and the
  buffer gains `min(heatLost x carnot, 510)` FE, clamped by its space. Then each
  of the six faces leaks `(T - ambient) / (10,000 + insulation + 5) x 10` heat
  (the bottom face has insulation 0, the others 100).
- **Settled numbers (defaults, ambient 300 K).** Fuelled only: the capacitor
  settles 395.3 K above ambient (695.3 K), loses 197.6 heat/t, Carnot 0.5685 →
  **112 FE/t**. Six lava sides and no fuel: 355.8 K above ambient → **96 FE/t**.
  Nether only: 197.7 K above ambient → **39 FE/t**. Fuel plus six sides plus the
  Nether: **360 FE/t**. The 510 cap is an upper bound the defaults never reach
  (the heat input itself is at most 480).
- **Buffer and output.** 160,000 FE; `getMaxOutput` is `MAX_PRODUCTION x 2` =
  1,020 FE/t, emitted from the front side only. A full buffer stops the lava
  extraction but the fuel slot still fills the tank while it has room.
- **Block.** `BlockTileModel` with `strength(3.5, 16)`, `requiresCorrectToolForDrops`,
  horizontal `facing` plus an `active` state (light 13), a custom sound, a custom
  shape, a GUI (fuel slot, energy slot, lava gauge), a comparator on the tank and
  computer methods (`getProductionRate`, losses, temperature, tank, slots).
- **Carried data.** `blockstates/heat_generator.json` (8 variants: 4 facings x
  active), the idle/active models (a drum over plate textures, `render_type`
  cutout, `neoforge_data` light 15 on the LED), the item model, the block item's
  `copy_components` loot (energy/items/fluids/heat), the shaped recipe
  `III / WOW / CFC` over `c:ingots/*`, planks and a furnace, and the "Your First
  Generator" advancement.

## Where the rewrite differs

Recorded in full in `../mekanismgenerators/notes.md`; the source-level ones are:
no GUI (container + `/mekanismgenerators info|energy` instead), no fluid
capability (the tank is a saved int `lava` that only the fuel slot fills), no
energy output push (the brief's slice boundary: the buffer stores), no `active`
state/light/particles/sound, the FE extractable from all six faces instead of
the front, a fixed 300 K ambient instead of the biome temperature, the template's
full-cube shape instead of the custom one, and the recipe carried but not
craftable (it needs `c:ingots/osmium` and other `c:` tags this port does not
register).
