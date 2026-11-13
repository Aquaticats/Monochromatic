/**
 The steps a container runs for `proof-private-runtime-directory.ts`, and the stand-in for the service manager's
 program.

 Kept in TypeScript rather than in a shell script file because repository rule `SCR` keeps shell script files out of
 the tree, and in a file of their own because both in the program would pass the line ceiling.
 Every path, count and bound the steps use is set inside them, so a person can re-run them in a container of their own
 by copying the two strings below.
 */

/**
 What:
  The three cases, what each prints, and the checks that make the container's exit status mean something.
 Why:
  The proof is only useful if a failing case says which one failed and why, and if the before and after arms run the
  same steps with only the executable changed.
 */
export const proofScript = `#!/bin/sh
# What:
#  Issue 6293's reproduction, in a container whose bus is its own:
#  what a run given a runtime directory of its own leaves in the account's manager and state directory,
#  and what a push to an address that can never be a remote does to its retry plan and to later commands.
# Why:
#  The two decisions of 2026-11-13 are "Private runtime dir is the boundary" and "End the plan at once, say so once".
#  Each case below prints what it found, and the proof fails when:
#   - a run with a runtime directory of its own asked a service manager anything, or left anything under the
#     account's state directory (case B);
#   - a push to the word DISABLED left a plan behind, or a second command printed its notice again (case B);
#   - the session whose own bus this is stopped asking its manager, or stopped keeping its state under the
#     account's state directory (case A, the control);
#   - a run with no runtime directory and no bus changed at all (case C, the control).
#  Nothing here can reach this machine's manager or session bus: the bus is the container's own and the manager's
#  program is a stand-in that records every call.
set -u

BUS_DIR=/run/session
PRIVATE_RUNTIME=/run/private
STATE_UNDER_HOME=/root/.local/state/monochromatic-git-policy-cli
RETRY_SECONDS=1
export CLI_GIT_PUSH_RETRY_SECONDS="$RETRY_SECONDS"
mkdir -p /proof "$BUS_DIR" "$PRIVATE_RUNTIME"
chmod 700 "$PRIVATE_RUNTIME"

failures=0
failed() {
    failures=$((failures + 1))
    echo "PROOF-FAILED: $*"
}

dbus-daemon --session --nofork --address="unix:path=$BUS_DIR/bus" >/proof/bus-address 2>/proof/bus-error &
BUS_PID=$!
for _try in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30; do
    [ -S "$BUS_DIR/bus" ] && break
    sleep 0.1
done
[ -S "$BUS_DIR/bus" ] || { echo "THE BUS DID NOT START: $(cat /proof/bus-error)"; exit 1; }
DBUS_SESSION_BUS_ADDRESS="unix:path=$BUS_DIR/bus" dbus-monitor >/proof/bus-monitor.txt 2>&1 &
MONITOR_PID=$!
sleep 0.7

echo "=== what this container has"
echo "pid 1: $(cat /proc/1/comm)"
echo "systemd: $(command -v systemd || echo none); systemctl: $(command -v systemctl || echo none)"
echo "the manager's program: $(command -v systemd-run || echo none) (a stand-in that records every call)"
echo "the bus: unix:path=$BUS_DIR/bus (the container's own); no service manager answers on it"
echo "the wrapper under test: $(git --version 2>&1 | head -1)"
echo "real git: $(/usr/bin/git --version)"

# A positive control, so the bus's own log is proved able to show a method call.
DBUS_SESSION_BUS_ADDRESS="unix:path=$BUS_DIR/bus" dbus-send --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames >/proof/control-call.txt 2>&1
sleep 0.5
echo "the bus log shows $(grep -c 'method call' /proof/bus-monitor.txt 2>/dev/null || true) method call(s) for a call of my own"
CONTROL_LINES=$(wc -l < /proof/bus-monitor.txt 2>/dev/null || echo 0)

# One repository, one commit that lands and one command after it.
case_run() {
    label=$1
    repo=/proof/$label
    rm -rf "$repo"
    mkdir -p "$repo"
    /usr/bin/git init --quiet --initial-branch=main "$repo/repo"
    cd "$repo/repo" || exit 1
    /usr/bin/git config user.email proof@example.invalid
    /usr/bin/git config user.name Proof
    printf 'base\\n' > a.txt
    /usr/bin/git add a.txt
    /usr/bin/git commit --quiet --message=base a.txt
    /usr/bin/git remote add origin DISABLED
    : > "$MANAGER_CALLS"
    printf 'base\\nmore\\n' > a.txt
    git commit --message="$label-one" a.txt > "$repo/commit.txt" 2>&1
    echo "  the commit: exit $?; subject now $(/usr/bin/git log -1 --format=%s)"
    echo "  manager calls for this case: $(wc -l < "$MANAGER_CALLS")"
}

# The notice of a failed push, as the next commands print it.
wait_for_the_notice() {
    waited=0
    while [ "$waited" -lt 30 ]; do
        git status --short > /proof/notice-first.txt 2>&1
        if grep -q 'backup push' /proof/notice-first.txt; then
            return 0
        fi
        sleep 0.5
        waited=$((waited + 1))
    done
    return 1
}

push_case() {
    label=$1
    runtime=$2
    state_root=$3
    note=$4
    echo "=== $label"
    echo "  $note"
    export XDG_RUNTIME_DIR="$runtime"
    export MANAGER_CALLS=/proof/manager-calls-$label.txt
    case_run "$label"
    echo "  the state directory this case's service wrote to, by folder:"
    for place in "$STATE_UNDER_HOME" "$runtime/monochromatic-git-policy-cli"; do
        echo "    $place: $(ls -1 "$place" 2>/dev/null | tr '\\n' ' ')"
    done
    echo "  the plan of a push waiting for a new try, in the private directory: $(ls -d "$runtime"/monochromatic-git-policy-cli/service/*/push-retries 2>/dev/null | wc -l) folder(s)"
    echo "  the plan of a push waiting for a new try, under the account's state directory: $(ls -d "$STATE_UNDER_HOME"/service/*/push-retries 2>/dev/null | wc -l) folder(s)"
    if wait_for_the_notice; then
        echo "  the first command after the failed push said:"
        sed 's/^/    /' /proof/notice-first.txt | head -6
    else
        echo "  NO NOTICE of the failed push arrived within the bound"
        failed "$label: the failed push left no notice"
    fi
    sleep 3
    git status --short > /proof/notice-second.txt 2>&1
    if grep -q 'backup push' /proof/notice-second.txt; then
        echo "  a second command printed the notice again:"
        sed 's/^/    /' /proof/notice-second.txt | head -6
        NOTICES_REPEATED=1
    else
        echo "  a second command printed no notice"
        NOTICES_REPEATED=0
    fi
}

echo
echo "=== case A: the bus is this session's own (the runtime directory the bus belongs to), the control"
export DBUS_SESSION_BUS_ADDRESS="unix:path=$BUS_DIR/bus"
push_case case-a "$BUS_DIR" "$STATE_UNDER_HOME" "the runtime directory is the one this bus belongs to"
A_CALLS=$(wc -l < /proof/manager-calls-case-a.txt)
A_STATE=$(ls -1 "$STATE_UNDER_HOME"/service 2>/dev/null | wc -l)
echo "  manager calls: $A_CALLS; instance folders under the account's state directory: $A_STATE"
[ "$A_CALLS" -ge 1 ] || failed "case A: the session's own manager was not asked for a scope at all"
[ "$A_STATE" -ge 1 ] || failed "case A: the session's own service left no journal under the account's state directory"
[ "$NOTICES_REPEATED" = 0 ] || failed "case A: a push to DISABLED printed its notice again on a later command"

echo
echo "=== case B: a runtime directory of the run's own, the bus inherited from another directory"
rm -rf "$STATE_UNDER_HOME"
push_case case-b "$PRIVATE_RUNTIME" "$PRIVATE_RUNTIME/monochromatic-git-policy-cli" "this is the run issue 6293 is about"
B_CALLS=$(wc -l < /proof/manager-calls-case-b.txt)
B_UNDER_HOME=$(ls -1 "$STATE_UNDER_HOME" 2>/dev/null | wc -l)
B_IN_PRIVATE=$(ls -1 "$PRIVATE_RUNTIME/monochromatic-git-policy-cli/service" 2>/dev/null | wc -l)
B_PLAN=$(ls -1 "$PRIVATE_RUNTIME"/monochromatic-git-policy-cli/service/*/push-retries 2>/dev/null | wc -l)
echo "  manager calls: $B_CALLS; entries under the account's state directory: $B_UNDER_HOME; instance folders in the private directory: $B_IN_PRIVATE"
[ "$B_CALLS" = 0 ] || failed "case B: a run with a runtime directory of its own asked a service manager"
[ "$B_UNDER_HOME" = 0 ] || failed "case B: a run with a runtime directory of its own left $B_UNDER_HOME entrie(s) under the account's state directory"
[ "$B_IN_PRIVATE" -ge 1 ] || failed "case B: no service journal was kept in the private runtime directory"
[ "$B_PLAN" = 0 ] || failed "case B: a push to DISABLED kept a retry plan"
[ "$NOTICES_REPEATED" = 0 ] || failed "case B: a push to DISABLED printed its notice again on a later command"

echo
echo "=== case C: no bus and no runtime directory at all, the control"
unset DBUS_SESSION_BUS_ADDRESS
unset XDG_RUNTIME_DIR
export MANAGER_CALLS=/proof/manager-calls-case-c.txt
case_run case-c
C_CALLS=$(wc -l < /proof/manager-calls-case-c.txt)
echo "  manager calls: $C_CALLS; sockets in the home's .run: $(ls -1 /root/.run/monochromatic-git-policy-cli 2>/dev/null | tr '\\n' ' ')"
echo "  the account's state directory holds: $(ls -1 "$STATE_UNDER_HOME"/service 2>/dev/null | wc -l) instance folder(s)"
[ "$C_CALLS" = 0 ] || failed "case C: a run with no bus and no runtime directory asked a service manager"

echo
echo "=== method calls the wrapper's own commands made on this container's bus"
sleep 0.5
kill "$MONITOR_PID" 2>/dev/null
tail -n "+$CONTROL_LINES" /proof/bus-monitor.txt > /proof/bus-monitor-after.txt
echo "method calls after the control: $(grep -c 'method call' /proof/bus-monitor-after.txt 2>/dev/null || true)"

echo
echo "=== the manager calls of each case"
for log in /proof/manager-calls-case-a.txt /proof/manager-calls-case-b.txt /proof/manager-calls-case-c.txt; do
    echo "--- $log"
    sed -e 's|/proof/[a-z-]*/||' "$log" 2>/dev/null | cut -c1-160 | sed 's/^/  /'
done

kill "$BUS_PID" 2>/dev/null
echo "failures: $failures"
if [ "$failures" -ne 0 ]; then
    echo "PROOF-FAILED"
    exit 1
fi
echo "PROOF-PASSED"`;

/**
 The manager's program, as a stand-in: it records every word it is given and then runs the command after `--`, which is
 what `systemd-run --user --scope` does.
 Why:
  A real manager would answer these calls and change the machine it is in, and the proof counts the calls: the count is
  what says whether a run of its own reached the account's manager.
 */
export const standInSystemdRun = `#!/bin/sh
target=$MANAGER_CALLS
[ -n "$target" ] || target=/proof/manager-calls.txt
printf '%s\\n' "$*" >> "$target"
while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do shift; done
[ "$#" -gt 0 ] && shift
exec "$@"
`;
