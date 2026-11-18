# Licence provenance

- Mod: `mekanismgenerators` (Mekanism: Generators) by Aidan C. Brady and the
  Mekanism contributors — <https://github.com/mekanism/Mekanism>
- Licence: **MIT**, read by the campaign's jar audit from
  `MekanismGenerators-1.21.1-10.7.19.85.jar` (SHA-1
  `44c4d8006597580ff7e4101b5a22f7b426cdcaf6`, the pack's own file; the jar's
  `mods.toml` declares `MIT`) and from the repository's `LICENSE`. MIT is
  licence rank 0 (permissive).
- Upstream release read for the behaviour: tag `v1.21.1-10.7.19.85`, commit
  `a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`. Analysis only: this port is a
  clean-room Rust rewrite, no Java was copied or translated.
- Carried assets: the block's blockstate, models, textures and lang entries are
  the upstream mod's own files, and the three plate textures
  (`block/models/ports*.png`, `block/bottom.png`) are Mekanism's — same author,
  same MIT grant, moved into this mod's namespace because a content mod cannot
  reference another mod's assets. They are carried under the MIT grant in
  `LICENSE`; no upstream file is relicensed.
- This port's source is the tree beside this file; the licence text is the
  standard MIT text with both copyright lines (upstream and port).
