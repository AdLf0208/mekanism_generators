#!/usr/bin/env python3
"""Check the port ships as one file: its content embedded in the native bundle.

The registry stores one file per project (`ports/_shared/horizon-mods/AGENT_PLAYBOOK.md`
§1.0), so a content+code port carries its built `.hmod` at
`mixins/<id>/data/<id>-content.hmod` and writes it out from a `content_root`
injection (the AE2 pattern, `docs/native-mods.md` "Data files").

This checks the three things that make that true, without running the game:

  1. the embedded copy exists and is byte-identical to `content/<id>.hmod`;
  2. the plan has one HEAD `content_root` injection whose helper compiles the
     same file in with `native_data!`;
  3. the delivered `.hnmod` carries that data file byte-identically (when it
     has been packaged).

    python3 tools/check-embedded.py            # exit 0 when all hold
    python3 tools/check-embedded.py --json
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
MOD = "mekanismgenerators"
CONTENT = ROOT / "content" / f"{MOD}.hmod"
BUNDLE = ROOT / "mixins" / MOD
EMBEDDED = BUNDLE / "data" / f"{MOD}-content.hmod"
HELPER = BUNDLE / "embedded_content.rs"
HNMOD = ROOT / f"{MOD}.hnmod"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def injection_problems() -> list[str]:
    """The plan's `content_root` injection, as the mixins parser will read it."""
    manifest = tomllib.loads((BUNDLE / "horizon-mixin.toml").read_text())
    roots = [i for i in manifest.get("injection", [])
             if i.get("target") == "horizon_client::game::mods::content_root"]
    if len(roots) != 1:
        return [f"expected exactly one content_root injection, found {len(roots)}"]
    root = roots[0]
    problems = []
    if root.get("kind") != "HEAD":
        problems.append(f"the content_root injection must be HEAD, not {root.get('kind')!r}")
    if root.get("cancellable") is not True:
        problems.append("the content_root injection must set cancellable = true")
    if root.get("helper") != "embedded_content.rs":
        problems.append(f"the content_root injection names helper {root.get('helper')!r}")
    wanted = {"root": "&std::path::Path",
              "enabled": "&std::collections::BTreeSet<String>",
              "dev": "&[std::path::PathBuf]"}
    got = {b.get("name"): b.get("ty") for b in root.get("bindings", [])}
    if got != wanted:
        problems.append(f"the content_root bindings are {got}, expected {wanted}")
    return problems


def helper_problems() -> list[str]:
    if not HELPER.is_file():
        return [f"missing helper {HELPER.relative_to(ROOT)}"]
    source = HELPER.read_text()
    problems = []
    needle = rf'native_data!\(\s*"{MOD}"\s*,\s*"{MOD}-content\.hmod"\s*\)'
    if not re.search(needle, source):
        problems.append('the helper does not compile the bundle\'s data in '
                        f'(native_data!("{MOD}", "{MOD}-content.hmod"))')
    if "core::ops::ControlFlow::Break(content_root(root, enabled, &with))" not in source:
        problems.append("the helper does not hand the file back through content_root")
    return problems


def bundle_problems() -> list[str]:
    if not HNMOD.is_file():
        return []                      # nothing packaged yet: not a failure here
    with zipfile.ZipFile(HNMOD) as archive:
        names = archive.namelist()
        inner = f"mixins/{MOD}/data/{MOD}-content.hmod"
        if inner not in names:
            return [f"the .hnmod does not carry {inner}"]
        if archive.read(inner) != CONTENT.read_bytes():
            return [f"{inner} in the .hnmod is not the built {CONTENT.name}"]
        manifest = json.loads(archive.read("horizon-native.json"))
        if manifest.get("id") != MOD:
            return [f"the .hnmod's id is {manifest.get('id')!r}, expected {MOD!r}"]
    return []


def check() -> dict:
    problems = []
    if not CONTENT.is_file():
        problems.append(f"missing the built content mod {CONTENT.relative_to(ROOT)}")
    elif not EMBEDDED.is_file():
        problems.append(f"missing {EMBEDDED.relative_to(ROOT)}: copy the built content mod into data/")
    elif EMBEDDED.read_bytes() != CONTENT.read_bytes():
        problems.append(f"{EMBEDDED.relative_to(ROOT)} is not the built {CONTENT.relative_to(ROOT)}")
    problems += injection_problems() + helper_problems() + bundle_problems()
    report = {"ok": not problems, "mod": MOD, "problems": problems}
    if CONTENT.is_file():
        report["content"] = {"path": str(CONTENT.relative_to(ROOT)),
                             "size": CONTENT.stat().st_size, "sha256": digest(CONTENT)}
    if EMBEDDED.is_file():
        report["embedded"] = {"path": str(EMBEDDED.relative_to(ROOT)),
                              "size": EMBEDDED.stat().st_size, "sha256": digest(EMBEDDED),
                              "identical_to_content": EMBEDDED.read_bytes() == CONTENT.read_bytes()}
    if HNMOD.is_file():
        with zipfile.ZipFile(HNMOD) as archive:
            report["hnmod"] = {"path": str(HNMOD.relative_to(ROOT)),
                               "size": HNMOD.stat().st_size, "sha256": digest(HNMOD),
                               "entries": len(archive.namelist())}
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    report = check()
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"embedded content: {'ok' if report['ok'] else 'FAILED'}")
        for problem in report["problems"]:
            print(f"  - {problem}")
        for key in ("content", "embedded", "hnmod"):
            if key in report:
                item = report[key]
                print(f"  {key}: {item['size']} B sha256 {item['sha256']}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
