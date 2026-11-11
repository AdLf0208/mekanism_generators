# Content: `mekanismgenerators.hmod`

Assembled from the jar `MekanismGenerators-1.21.1-10.7.19.85.jar` (MIT, SHA-1
`44c4d800…caf6`, the pack's own file; provenance in
`../../_shared/carry-work/mekanismgenerators/SOURCE.md`) and the shared plate
textures from `Mekanism-1.21.1-10.7.19.85.jar` (same author and licence).
Rerun `cargo horizon-mod build` in this folder to rebuild the package.

## Carried

| Files | Source |
|---|---|
| `assets/mekanismgenerators/blockstates/heat_generator.json` | the jar's 8 variants (4 facings x `active`), with `active` renamed to the barrel template's `open` and the template's `up`/`down` facings added; both `open` values show the idle model |
| `assets/mekanismgenerators/models/block/heat_generator.json` | byte-identical to the jar except the five `mekanism:` texture ids, which are now `mekanismgenerators:` (a content mod cannot reference another mod's assets) |
| `assets/mekanismgenerators/models/item/heat_generator.json`, `assets/mekanismgenerators/items/heat_generator.json` | the jar's item model, parented to the block model, plus the 26.3 `items/` definition (which names `mekanismgenerators:block/heat_generator` directly: a block model is a valid item model, as Powah's carried items do) |
| `assets/mekanismgenerators/textures/block/model/heat_generator{,_led}.png` | the jar's own two textures |
| `assets/mekanismgenerators/textures/block/models/{ports,ports_led}.png`, `textures/block/bottom.png` | the three plate textures the model shares with Mekanism, carried into this namespace |
| `assets/mekanismgenerators/lang/en_us.json` | the jar's block/item/container/description/advancement keys for the heat generator |
| `data/mekanismgenerators/loot_table/blocks/heat_generator.json` | the 26.3 self-drop shape (the jar's `copy_components` list names Mekanism components this port does not register) |
| `data/mekanismgenerators/recipe/generator/heat.json` | the jar's shaped recipe, upgraded to the 26.3 ingredient shape (`"#c:ingots/copper"`, `"minecraft:furnace"`, `{"id","count"}` result) |
| `data/minecraft/tags/block/mineable/pickaxe.json` | the one block |
| `data/mekanismgenerators/horizon/block/heat_generator.json` | the barrel template with upstream's `strength(3.5, 16)`, `requiresCorrectToolForDrops`, stone sounds, non-occluding |
| `data/mekanismgenerators/horizon/creative_tab/main.json` | the tab upstream builds in code |

The recipe **loads but does not craft**: it needs `c:ingots/osmium` and the
`c:ingots/copper|iron` tags, and this port registers no osmium. That is a
recorded gap, not a silent change.

## Where the package is used

`mekanismgenerators.hmod` here is the **input** of the delivered `.hnmod`: a
byte-identical copy travels at
`../mixins/mekanismgenerators/data/mekanismgenerators-content.hmod` and the
`embedded-content` injection at `content_root` writes it out and loads it, so one
installed `.hnmod` gives the block, its item and the code (the registry stores
one file per project). Rebuild the `.hmod`, copy it over that data file, and
re-package; `../tools/package.py` refuses a stale copy and
`../tools/check-embedded.py` checks all three places agree.
