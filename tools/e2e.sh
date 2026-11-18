#!/usr/bin/env bash
# In-game check for Mekanism: Generators slice 1: the Heat Generator on a real
# game, for the flat SDK package (`mekanismgenerators.hmod`).
#
# The arena is a stone floor at y=100 in the Overworld, so every block read and
# command is in the player's own dimension. This package carries the block, its
# item and data AND the Rust that runs it, so the game must be built against it
# by the launcher:
#
#   horizon-launcher native-build . --instance <instance> --trust-native-code
#
# and `GAME` must name that build's client. There is no `--dev-mod` leg: the
# package is compiled in, and a content half loaded beside it would be a second
# copy of the same block.
#
# Run:
#  - control "unmodified game" (CONTROL=1, the default): the same scenario on a
#    launcher build WITHOUT this package — the block id must be unknown and
#    `/mekanismgenerators` must not be a command, so the main run's generator can
#    only come from this package. Set PLAIN_GAME to that build's client; with
#    PLAIN_GAME unset the leg is skipped with a note (CONTROL=0 does the same).
#  - two fresh generators make no FE for 120 ticks and stay at ambient
#    temperature;
#  - 64 coal in one generator's fuel slot (the container path, automation's way
#    in) converts to lava (80 mB a coal, one item a tick) and the generator makes
#    FE, settling at 112 FE/t (200 heat/t through the 0.5 Carnot conversion: a
#    695.3 K capacitor, 0.5685 efficient);
#  - the buffer grows over 100 ticks by about 112 x 100 FE;
#  - control "full buffer": at 160,000 FE it produces nothing, does not burn its
#    lava, and the lava level never falls;
#  - control "drain": with room again, the lava falls 10 mB/t and the buffer
#    grows, so the space is what gates the fuel;
#  - persistence: FE, lava and the capacitor's heat survive save + relaunch.
#
# SELFTEST=1 flips the "no fuel" assertion (it must FAIL), proving the scenario
# can fail. KEEP=1 keeps the scratch folder.
set -euo pipefail
export LC_ALL=C

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/mekanismgenerators
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}   # pin-matched shared ctl
GAME=${GAME:-}                       # a launcher build carrying this package
PLAIN_GAME=${PLAIN_GAME:-}           # a launcher build without it, for the control leg
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/mekanismgenerators-e2e.sock}
SELFTEST=${SELFTEST:-0}
CONTROL=${CONTROL:-1}
KEEP=${KEEP:-0}
WORLD=MekanismGeneratorsE2E
CONTROL_WORLD=MekanismGeneratorsControl
GENERATOR=mekanismgenerators:heat_generator

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "set GAME to a launcher build carrying ports/mekanismgenerators:" >&2
    echo "  horizon-launcher native-build $PORT --instance <i> --trust-native-code" >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/mekanismgenerators-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 80 >&2 || true; exit 1; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    echo "EXIT:$status"
    exit "$status"
}
trap cleanup EXIT

# info X Y Z: this block's status line, after a fresh reply arrives (the t=
# field makes each reply unique).
info() { # info X Y Z
    local out line before
    before=$(ctl messages 20 | jq -r '.chat[]' | grep -E " at $1,$2,$3:" | head -1)
    say "mekanismgenerators info $1 $2 $3"
    for _ in $(seq 1 25); do
        ctl wait 4 >/dev/null
        out=$(ctl messages 20)
        line=$(jq -r '.chat[]' <<<"$out" | grep -E " at $1,$2,$3:|No block entity" | head -1)
        if [ -n "$line" ] && [ "$line" != "$before" ]; then
            echo "$line"
            return
        fi
    done
    echo "NO-INFO $(jq -c '{chat,action_bar}' <<<"$out")"
}
# set_energy X Y Z AMOUNT: the harness control `/mekanismgenerators energy`,
# confirmed by the command's own reply (chat commands are asynchronous, and a
# producing generator gains FE again the tick after a set).
set_energy() { # X Y Z AMOUNT
    local want=$4 out
    say "mekanismgenerators energy $want $1 $2 $3"
    for _ in $(seq 1 20); do
        ctl wait 2 >/dev/null
        out=$(ctl messages 20 | jq -r '.chat[]')
        if grep -q "Energy set to $want FE." <<<"$out"; then
            echo "  set($1,$2,$3) = $want FE"
            return
        fi
    done
    fail "could not set $1,$2,$3 to $want FE"
}
expect_info() { # expect_info X Y Z SUBSTRING
    local line
    line=$(info "$1" "$2" "$3")
    echo "  info($1,$2,$3): $line"
    case "$line" in
        *"$4"*) ;;
        *) fail "expected /mekanismgenerators info to say '$4', got '$line'" ;;
    esac
}

# The energy in an info line: the number before "/<capacity> FE".
energy_of() { sed -nE 's/.* at [-0-9]+,[-0-9]+,[-0-9]+: ([0-9]+)\/[0-9]+ FE.*/\1/p' <<<"$1"; }
lava_of() { sed -nE 's/.*lava ([0-9]+)\/[0-9]+ mB.*/\1/p' <<<"$1"; }
produced_of() { sed -nE 's/.*producing ([0-9]+) FE\/t.*/\1/p' <<<"$1"; }
burning_of() { sed -nE 's/.*burning (true|false).*/\1/p' <<<"$1"; }

launch() { # launch WORLD DIRECTORY GAME
    local world=${1:-$WORLD} dir=${2:-$SCRATCH} game=${3:-$GAME}
    local attempt
    for attempt in 1 2 3; do
        rm -f "$SOCK"
        cd "$dir"
        if "$CTL" --socket "$SOCK" launch --game "$game" --world "$world" --seed 20261105 \
                --creative --headless --wait --resources "$RES" \
                >/dev/null 2>>"$dir/game-stderr.log"; then
            ctl wait-ready 120 >/dev/null
            ctl wait 40 >/dev/null
            return 0
        fi
        echo "  (launch attempt $attempt did not come up; waiting for the world lock)"
        sleep 10
    done
    fail "the game did not launch"
}

stop_game() { ctl quit >/dev/null 2>&1 || true; sleep 2; }

# A stone floor at y=99 with clear air above, in the Overworld, and the player
# standing on it at (0.5, 100, 3.5).
arena() {
    ctl tp 0.5 101 3.5 >/dev/null
    ctl wait 20
    ctl command "fill -8 99 -8 24 99 24 minecraft:stone" >/dev/null
    ctl command "fill -8 100 -8 24 110 24 minecraft:air" >/dev/null
    ctl command "kill @e[type=!player]" >/dev/null
    ctl tp 0.5 100 3.5 >/dev/null
    ctl wait 20
    local y feet floor
    y=$(ctl state | jq -r '.player.position[1] // 0')
    feet=$(ctl block 0 100 3 | jq -r '.name // .block // empty')
    floor=$(ctl block 0 99 3 | jq -r '.name // .block // empty')
    echo "  arena: y=$y floor(0,99,3)=$floor feet(0,100,3)=$feet"
    awk -v y="$y" 'BEGIN { exit !(y >= 99.9) }' || fail "the player is not on the arena floor (y=$y)"
    [ "$feet" = "minecraft:air" ] || fail "the arena volume is not clear ($feet at 0,100,3)"
    [ "$floor" = "minecraft:stone" ] || fail "the arena floor is missing ($floor at 0,99,3)"
}

place() { # place X Y Z ID
    ctl command "setblock $1 $2 $3 $4" >/dev/null
    ctl wait 4
    local got
    got=$(ctl block "$1" "$2" "$3" | jq -r '.name // .block // empty')
    echo "  block($1,$2,$3) = $got"
    [ "$got" = "$4" ] || fail "expected $4 at $1,$2,$3, got '$got'"
}

# The differential: the same scenario on a launcher build without this package.
# Nothing of the mod may exist there — not the block, not the item, not the
# command — so the main run's generator can only come from the package.
control_leg() {
    echo "== control: a launcher build without this package =="
    local dir=$SCRATCH/control
    mkdir -p "$dir"
    (
        export HORIZON_HOME=$dir/game HORIZON_LAUNCHER_HOME=$dir/launcher \
               XDG_DATA_HOME=$dir/data XDG_CONFIG_HOME=$dir/config
        mkdir -p "$XDG_CONFIG_HOME"
        launch "$CONTROL_WORLD" "$dir" "$PLAIN_GAME"
        arena
        ctl command "setblock 0 100 0 $GENERATOR" >/dev/null || true
        ctl give "$GENERATOR" 1 >/dev/null 2>&1 || true
        ctl wait 6
        local placed
        placed=$(ctl block 0 100 0 | jq -r '.name // .block // ""')
        echo "  placed block: $placed"
        [ "$placed" = "minecraft:air" ] || fail "the control game holds $placed, so the block id was known"
        ctl inventory | jq -e --arg item "$GENERATOR" \
            '[(.hotbar // [])[], (.main // [])[] | select(.item == $item)] | length == 0' >/dev/null \
            || fail "the control game gave a heat generator item"
        local line
        line=$(info 0 100 0)
        echo "  native command: $line"
        case "$line" in
            NO-INFO*) ;;
            *) fail "the control game answered /mekanismgenerators (got '$line')" ;;
        esac
        echo "  without the package: no heat generator block, item or command"
        ctl quit >/dev/null 2>&1 || true
    )
    sleep 2
    rm -rf "$dir"
}

if [ "$CONTROL" = 1 ]; then
    if [ -n "$PLAIN_GAME" ]; then
        control_leg
    else
        echo "== control skipped: set PLAIN_GAME to a launcher build without this package =="
    fi
fi

echo "== launch (Overworld arena at y=100, the launcher build carrying this package) =="
launch
arena

echo "== place the generator and an unfuelled control =="
place 0 100 0 mekanismgenerators:heat_generator
place 0 100 4 mekanismgenerators:heat_generator

echo "== control: no fuel means no FE and no heat =="
line=$(info 0 100 0)
echo "  generator: $line"
[ "$(energy_of "$line")" = 0 ] || fail "a generator with no fuel stored FE: $line"
[ "$(burning_of "$line")" = false ] || fail "a generator with no fuel is burning: $line"
[ "$(produced_of "$line")" = 0 ] || fail "a generator with no fuel is producing: $line"
ctl wait 120
line=$(info 0 100 0)
echo "  after 120 ticks: $line"
[ "$(energy_of "$line")" = 0 ] || fail "a generator with no fuel made FE: $line"
if [ "$SELFTEST" = 1 ]; then
    echo "  SELFTEST: expecting the unfuelled generator to have made FE (this must fail)"
    [ "$(energy_of "$line")" != 0 ] || fail "SELFTEST flipped: the unfuelled generator made no FE, as it should"
fi
expect_info 0 100 4 "0/160000 FE, lava 0/24000 mB, temp 300.0 K"

echo "== coal into the fuel slot becomes lava and the generator burns it =="
# A coal is 80 mB (1600 burn ticks / 20), i.e. 8 burning ticks; the slot takes
# one stack of 64, so three top-ups give about 1,500 burning ticks.
ctl container 0 100 0 minecraft:coal 64 >/dev/null
ctl wait 70
ctl container 0 100 0 minecraft:coal 64 >/dev/null
ctl wait 70
ctl container 0 100 0 minecraft:coal 64 >/dev/null
ctl wait 6
# one item became 80 mB on the first tick
line=$(info 0 100 0)
echo "  $line"
lava=$(lava_of "$line")
[ "${lava:-0}" -gt 0 ] || fail "the coal did not become lava: $line"
[ "$(burning_of "$line")" = true ] || fail "the generator is not burning: $line"

echo "== the generator makes FE and settles at its Carnot rate =="
ctl wait 300
line=$(info 0 100 0)
echo "  generator: $line"
energy=$(energy_of "$line")
lava_now=$(lava_of "$line")
produced=$(produced_of "$line")
echo "  energy $energy FE, lava $lava_now mB, producing $produced FE/t"
[ "${energy:-0}" -gt 25000 ] || fail "300 fuelled ticks made only ${energy:-0} FE: $line"
[ "$produced" = 112 ] || fail "the generator should settle at 112 FE/t, got $produced: $line"
[ "$lava_now" -lt 15360 ] || fail "the fuel was not consumed: $line"
[ "$lava_now" -gt 2000 ] || fail "the fuel was consumed too fast: $line"
expect_info 0 100 4 "burning false"

echo "== the buffer grows by about 112 FE/t =="
before=$(energy_of "$(info 0 100 0)")
ctl wait 100
after_line=$(info 0 100 0)
after=$(energy_of "$after_line")
delta=$((after - before))
echo "  $before -> $after FE over ~100 ticks (delta $delta)"
[ "$delta" -ge 9000 ] || fail "the buffer grew only $delta FE in ~100 ticks: $after_line"
[ "$delta" -le 15000 ] || fail "the buffer grew $delta FE in ~100 ticks, faster than 112/t: $after_line"

echo "== control: a full buffer produces nothing and never burns its lava =="
set_energy 0 100 0 160000
ctl wait 20
line=$(info 0 100 0)
full_lava=$(lava_of "$line")
echo "  full: $line"
[ "$(energy_of "$line")" = 160000 ] || fail "the full buffer changed: $line"
[ "$(produced_of "$line")" = 0 ] || fail "a full buffer still produced: $line"
[ "$(burning_of "$line")" = false ] || fail "a full buffer still burned lava: $line"
ctl wait 100
line=$(info 0 100 0)
echo "  after 100 ticks full: $line"
[ "$(energy_of "$line")" = 160000 ] || fail "a full buffer changed: $line"
[ "$(lava_of "$line")" -ge "$full_lava" ] || fail "a full buffer burned lava: $(lava_of "$line") < $full_lava"

echo "== control: with room again the lava falls 10 mB/t and FE grows =="
drained_lava=$(lava_of "$line")
set_energy 0 100 0 0
ctl wait 60
line=$(info 0 100 0)
echo "  drained: $line"
[ "$(energy_of "$line")" -gt 0 ] || fail "the drained generator made no FE: $line"
[ "$(lava_of "$line")" -lt "$drained_lava" ] || fail "the drained generator did not burn lava: $line"

echo "== persistence: save, quit and relaunch read the block entity back =="
before_line=$(info 0 100 0)
echo "  before save: $before_line"
before_energy=$(energy_of "$before_line")
before_lava=$(lava_of "$before_line")
[ "$before_energy" -gt 0 ] || fail "nothing to persist: $before_line"
ctl save | sed 's/^/  /'
stop_game
launch
ctl command "execute in minecraft:overworld run tp @s 0.5 100 3.5" >/dev/null
ctl wait 40 >/dev/null
after_line=$(info 0 100 0)
echo "  after relaunch: $after_line"
after_energy=$(energy_of "$after_line")
after_lava=$(lava_of "$after_line")
# The block keeps ticking between the two readings, so allow it to grow but
# never to lose what it had.
[ "$after_energy" -ge "$before_energy" ] || fail "the saved FE was lost: $before_energy -> $after_energy"
[ "$after_lava" -le "$before_lava" ] || fail "the saved lava was lost: $before_lava -> $after_lava"
echo "  (the saved compound survives: FE $before_energy -> $after_energy, lava $before_lava -> $after_lava)"

if [ "$SELFTEST" = 1 ]; then
    echo "SELFTEST finished without failing (this is a bug in the test)"
    exit 1
fi
echo "PASS: a fuelled heat generator converts coal into lava, burns 10 mB/t and makes 112 FE/t into its own 160,000 FE buffer, an unfuelled one makes nothing, a full buffer produces nothing and burns no lava while a drained one resumes, and the FE, lava and heat survive a save and relaunch"
