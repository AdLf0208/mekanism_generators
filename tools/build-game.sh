#!/usr/bin/env bash
# Build the launcher game this port's e2e needs: **this package plus the parent
# port `mekanism`**, so the energy emission can be proven against a real
# acceptor (`mekanism:basic_energy_cube`, a storage block that takes FE and
# never pulls) instead of a stub.
#
# `ports/tools/launcher-game.sh` builds one port; the launcher itself takes a
# folder of mods (`native-build FOLDER` — "a mod folder … or a folder of them"),
# which is what this script stages. Everything else (the launcher install, the
# Nightly, the export and the env block) is the wrapper's recipe, repeated here
# because the wrapper refuses a folder that is not one port.
#
#   tools/build-game.sh build [--instance NAME]   # stage, native-build, export
#   tools/build-game.sh env                       # print the exports for e2e.sh
#
# Both mods must pin the campaign's SDK; the launcher refuses a mixed build.
set -uo pipefail

CLONE=$(cd "$(dirname "$0")/../../.." && pwd)
TOOLS=$CLONE/ports/tools
PORT=$CLONE/ports/mekanismgenerators
PARENT=$CLONE/ports/mekanism
CAMPAIGN_PORTS=$CLONE/ports

LAUNCHER_VERSION=$(sed -n 's/^LAUNCHER_VERSION=//p' "$TOOLS/launcher-game.sh" | head -1)
LAUNCHER_DIR=$CAMPAIGN_PORTS/_shared/bin/launcher-$LAUNCHER_VERSION
LAUNCHER=$LAUNCHER_DIR/horizon-launcher
PIN=$(sed -n 's/^SDK_PIN=\([0-9a-f]\{7\}\).*/\1/p' "$TOOLS/launcher-game.sh" | head -1)
PIN=${PIN:-0a875da}
RES_VERSION=26.3
E2E_ROOT=${HORIZON_E2E_ROOT:-$HOME/.cache/horizon-e2e}
DATA=$E2E_ROOT/root/data
CONFIG=$E2E_ROOT/root/config
HOME_DIR=$E2E_ROOT/root/home
VERSIONS=$DATA/horizon-launcher/versions
INSTANCE=${HORIZON_E2E_INSTANCE:-mekanismgenerators-e2e}
MOD=mekanismgenerators
OUT=$E2E_ROOT/$PIN/$MOD

say() { printf '%s: %s\n' "${0##*/}" "$*" >&2; }
die() { printf '%s: %s\n' "${0##*/}" "$*" >&2; exit "${2:-1}"; }

launcher() {
    env XDG_DATA_HOME="$DATA" XDG_CONFIG_HOME="$CONFIG" HOME="$HOME_DIR" \
        "$LAUNCHER" "$@"
}

[ -x "$LAUNCHER" ] || die "no launcher: run ports/tools/launcher-game.sh ensure" 2

nightly_tag() {
    launcher versions --installed --json 2>/dev/null \
        | python3 -c 'import json,sys
print(next((v for v in json.load(sys.stdin).get("installed",[]) if v.endswith("-'$PIN'")), ""))'
}

# One mod tree without its build output: the launcher compiles the sources.
stage_mod() { # source-dir destination
    mkdir -p "$2"
    tar -C "$1" --exclude=target --exclude=Cargo.lock -cf - . | tar -C "$2" -xf -
}

export_version() { # tag outdir
    local tag=$1 out=$2 f
    rm -rf "$out"
    mkdir -p "$out" || die "cannot create $out" 3
    for f in "$VERSIONS/$tag"/horizon-client "$VERSIONS/$tag"/horizon-ctl \
             "$VERSIONS/$tag"/lib*.so "$VERSIONS/$tag"/horizon-wasm-std.wasm \
             "$VERSIONS/$tag"/research-lock.json "$VERSIONS/$tag"/PLAY.txt; do
        [ -e "$f" ] || continue
        ln -f "$f" "$out/" 2>/dev/null || cp -a "$f" "$out/"
    done
    [ -d "$VERSIONS/$tag/horizon-native-mods" ] && cp -a "$VERSIONS/$tag/horizon-native-mods" "$out/"
    [ -f "$VERSIONS/$tag/native-mods.json" ] && cp -a "$VERSIONS/$tag/native-mods.json" "$out/"
    ldd "$out/horizon-client" 2>/dev/null | grep -q 'not found' \
        && die "the exported game misses a library: $(ldd "$out/horizon-client" | grep 'not found' | tr '\n' ' ')" 9
    return 0
}

env_block() { # game ctl control resources
    printf 'export GAME=%q\n' "$1"
    printf 'export CTL=%q\n' "$2"
    printf 'export PLAIN_GAME=%q\n' "$3"
    printf 'export RES=%q\n' "$4"
    printf 'export HORIZON_RES=%q\n' "$4"
    printf 'export E2E_TMP=%q\n' "$E2E_ROOT/scratch"
    printf 'export SOCK=%q\n' "$E2E_ROOT/scratch/$MOD-e2e.sock"
    printf 'export GSOCK=%q\n' "$E2E_ROOT/scratch/$MOD-e2e-guest.sock"
}

cmd_build() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --instance) INSTANCE=$2; shift 2 ;;
            *) die "unknown option $1" 64 ;;
        esac
    done
    local tag stage before built out rc
    tag=$(nightly_tag)
    [ -n "$tag" ] || die "the Nightly at $PIN is not installed: run ports/tools/launcher-game.sh build $MOD" 5
    [ -f "$VERSIONS/$tag/.local/horizon-client/$RES_VERSION/horizon-resources.json" ] \
        || die "$tag is not prepared: run ports/tools/launcher-game.sh build $MOD" 7
    launcher instance list --json 2>/dev/null | grep -q "\"$INSTANCE\"" \
        || launcher instance create "$INSTANCE" --json >&2 || die "instance create $INSTANCE failed" 8
    stage=$E2E_ROOT/scratch/$MOD-two-mods
    rm -rf "$stage"
    stage_mod "$PORT" "$stage/$MOD"
    stage_mod "$PARENT" "$stage/mekanism"
    say "native-build $stage ($MOD + mekanism) against $tag"
    launcher native-build "$stage" --instance "$INSTANCE" --trust-native-code --json >&2 \
        || die "horizon-launcher native-build failed" 10
    # The version the launcher *selected for the instance* is the build, whether
    # it compiled or reused an identical one (a reused build is not new in the
    # installed list, so "the newest new tag" would be wrong — it once exported
    # a previous, deliberately mutated build).
    built=$(launcher instance list --json 2>/dev/null | python3 -c '
import json, sys
data = json.load(sys.stdin)
instances = data.get("instances", data) if isinstance(data, dict) else data
for entry in instances:
    if entry.get("id") == "'"$INSTANCE"'":
        print(entry.get("version") or "")
        break')
    [ -n "$built" ] || die "native-build reported success but the instance plays no version" 10
    cmp -s "$VERSIONS/$built/horizon-native-mods/$MOD/horizon-mod.toml" "$PORT/horizon-mod.toml" \
        || say "note: the built package's manifest differs from the port's"
    ls -1 "$VERSIONS/$built/horizon-native-mods" | sed 's/^/  carries /' >&2
    export_version "$built" "$OUT"
    export_version "$tag" "$E2E_ROOT/$PIN/control"
    say "built $built; exported to $OUT"
    grep -c . "$OUT/native-mods.json" >/dev/null 2>&1 && say "native-mods.json: $(cat "$OUT/native-mods.json")"
    rm -rf "$stage"
    return 0
}

cmd_env() {
    tag=$(nightly_tag)
    [ -n "$tag" ] || die "the Nightly at $PIN is not installed" 5
    [ -x "$OUT/horizon-client" ] || die "no exported game; run: ${0##*/} build" 2
    env_block "$OUT/horizon-client" "$OUT/horizon-ctl" \
              "$E2E_ROOT/$PIN/control/horizon-client" \
              "$VERSIONS/$tag/.local/horizon-client/$RES_VERSION"
}

case "${1:-}" in
    build) shift; cmd_build "$@" ;;
    env) cmd_env ;;
    *) die "usage: ${0##*/} build [--instance NAME] | env" 64 ;;
esac
