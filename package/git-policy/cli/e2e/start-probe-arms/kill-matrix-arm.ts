/**
 M3's kill matrix: every named hold point, each of its processes killed alone, and
 the raised-deadline control, which runs the feature build.

 A hold point makes the probe stand at one step, so a kill there is a kill at a
 known place. Each kill is followed by a run that has to pass, a mount table that
 has to be unchanged, no namespace that still lists the source, and no process left
 behind named by the pattern.

 @example
 ```ts
 await m3Kill({ chosen });
 ```
 */

import {
  type GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';


import {
  longArmTimeoutMilliseconds,
  probeScript,
} from './probe-fixture.ts';

/**
 The log the kill matrix writes inside the run's log folder.
 */
const killLog = 'guest-m3-kill.log';

/**
 Run the kill matrix.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await m3Kill({ chosen });
 ```
 */
export async function m3Kill({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The matrix: one control run per hold point, then five kills per hold point,
   then the raised-deadline control and an immediate kill of the mode.
   */
  const script = probeScript({
    lines: [
      'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m3-table-before.txt',
      'rm -f /root/m3-summary.txt',
      'check_after() { label="$1"; sleep 1.2; sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m3-table-after.txt; if cmp -s /root/m3-table-before.txt /root/m3-table-after.txt; then t=table-same; else t=TABLE-DIFFERS; fi; l=$(leaks | tail -n 1); left=$(pgrep -f "$PAT" | wc -l); st=$(grep -m1 "^state " $REC | cut -d" " -f2); $H start-probe > /root/m3-next.log 2>&1 || true; if grep -q "^verdict passed" $REC; then nx=next-run-passed; else nx=NEXT-RUN-DID-NOT-PASS; fi; echo "KILL $label $t $l leftover-processes=$left record-state-after-kill=$st $nx" | tee --append /root/m3-summary.txt; pkill -KILL -f "$PAT" || true; }',
      'for step in unshare propagate-private scratch-tmpfs scratch-tree walk-slot walk-refusal-controls attach-context attach-source attach-options attach-exclusive-create attach-mount-descriptor attach-place verify-mount scan-while-held scan-after-holder move unmount; do',
      '  CLI_GIT_TEST_ONLY_START_PROBE=hold-$step $H start-probe > /root/m3-control.log 2>&1 || true',
      '  echo "CONTROL hold-$step last-finished=$(grep "^last-finished " $REC | cut -d" " -f2) in-flight=$(grep "^in-flight " $REC | cut -d" " -f2) verdict=$(grep "^verdict " $REC | cut -d" " -f2)"',
      'done',
      'for step in unshare propagate-private scratch-tmpfs scratch-tree walk-slot walk-refusal-controls attach-context attach-source attach-options attach-exclusive-create attach-mount-descriptor attach-place verify-mount scan-while-held scan-after-holder move unmount; do',
      '  for target in mode worker both holder child; do',
      '    CLI_GIT_TEST_ONLY_START_PROBE=hold-$step $H start-probe > /root/m3-run.log 2>&1 &',
      '    mode=$!',
      '    sleep 0.2',
      '    worker=$(pgrep -P $mode -o || true)',
      '    holder=$(pgrep -P $mode -n || true)',
      '    child=""; if test -n "$worker"; then child=$(pgrep -P $worker -n || true); fi',
      '    case $target in',
      '      mode) kill -KILL $mode 2>/dev/null || true ;;',
      '      worker) test -n "$worker" && kill -KILL $worker 2>/dev/null || true ;;',
      '      both) kill -KILL $mode 2>/dev/null || true; test -n "$worker" && kill -KILL $worker 2>/dev/null || true ;;',
      '      holder) if test -n "$holder" && test "$holder" != "$worker"; then kill -KILL $holder 2>/dev/null || true; else echo "SKIP hold-$step holder no-holder-yet" | tee --append /root/m3-summary.txt; fi ;;',
      '      child) if test -n "$child"; then kill -KILL $child 2>/dev/null || true; else echo "SKIP hold-$step child no-child-yet" | tee --append /root/m3-summary.txt; fi ;;',
      '    esac',
      '    wait $mode 2>/dev/null || true',
      '    check_after "hold-$step kill-$target"',
      '  done',
      'done',
      'CLI_GIT_TEST_ONLY_START_PROBE=hold-unmount CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=900000 $R start-probe > /root/m3-raised.log 2>&1 || true',
      'if grep -q "^verdict passed" $REC && grep -q "^step unmount passed" $REC; then echo "M3 raised-deadline-control-passed"; else echo "M3 raised-deadline-control-failed"; cat $REC; exit 1; fi',
      '$H start-probe > /root/m3-run.log 2>&1 & mode=$!; kill -KILL $mode 2>/dev/null || true; wait $mode 2>/dev/null || true; check_after "kill-immediately mode"',
      'if grep -q TABLE-DIFFERS /root/m3-summary.txt; then echo "M3 a kill changed the host table"; exit 1; fi',
      'if grep -Eq "leaks=[1-9]" /root/m3-summary.txt; then echo "M3 a kill left the source in a namespace"; exit 1; fi',
      'echo "M3 kill matrix done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: killLog,
    timeoutMilliseconds: longArmTimeoutMilliseconds,
  },);
}
