/**
 What one gate unit is: the task it runs, the exit-status line it leaves in its log, and the `systemd-run` arguments
 that start it with its caps, its working directory and its image tag.

 The unit writes its own exit status as the last line of its log, because a transient unit started with `--collect` is
 unloaded when it ends and `systemctl show` then reports the defaults of a unit that does not exist, which read as a
 clean exit (`doc/troubleshooting/systemd-run-collect-reports-success-for-a-failed-unit.md`). A verdict is read from that
 line and from the log's nextest summary, never from the unit.
 */
import {
  shellProgram,
  taskRunnerProgram,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/constants.ts';

import type { GateCaps } from './native-gate-caps.ts';
import { gateImageTagVariable } from './native-gate-image-tag.ts';

/**
 The task whose run is a gate: the authority of the wrapper, which builds the image and runs the suite and Clippy in
 bounded containers with pacman's current Git.
 */
export const gateTask = '//package/git-policy/cli:native:test:container';

/**
 Text the gate's log carries on its last line, followed by the run's own exit status.

 A verdict looks for this text, so the writer and the reader share one name.
 */
export const gateExitStatusMarker = 'gate-exit-status=';

/**
 The command line the gate unit runs.

 The status is captured and written into the log before the same status ends the unit, so the log holds the verdict even
 though the collected unit is gone by the time anyone asks.

 @param caps - the caps of this gate, whose container process limit is passed to the task

 @returns the text a shell runs inside the unit

 @example
 ```ts
 gateUnitShellCommand({ caps: defaultGateCaps });
 ```
 */
export function gateUnitShellCommand({ caps }: { readonly caps: GateCaps; }): string {
  return `${taskRunnerProgram} run ${gateTask} -- --pids-limit ${String(caps.containerPidsLimit)}; status=$?; printf "\\n${gateExitStatusMarker}%s\\n" "$status"; exit "$status"`;
}

/**
 The arguments after the `systemd-run` program name, which start one gate as its own transient unit.

 The image tag is the unit's own name, so two gates never share a tested image, and the log is truncated at the start so
 a name reused in the same second never appends to an older verdict. The unit's standard error stays with the caller,
 which is where `systemd-run` itself reports a refused start.

 @param name - the unit's name

 @param worktree - absolute path of the worktree the gate runs in

 @param log - absolute path of the gate's log file

 @param caps - the unit's caps and the container's process limit

 @returns the complete argument array

 @example
 ```ts
 gateUnitArguments({ name, worktree, log, caps: defaultGateCaps });
 ```
 */
export function gateUnitArguments({
  name,
  worktree,
  log,
  caps,
}: {
  readonly name: string;
  readonly worktree: string;
  readonly log: string;
  readonly caps: GateCaps;
}): readonly string[] {
  return [
    '--user',
    `--unit=${name}`,
    '--collect',
    `--working-directory=${worktree}`,
    `--setenv=${gateImageTagVariable}=${name}`,
    `--property=CPUQuota=${caps.unitCpuQuota}`,
    `--property=MemoryMax=${caps.unitMemoryMax}`,
    `--property=TasksMax=${String(caps.unitTasksMax)}`,
    `--property=IOWeight=${String(caps.unitIoWeight)}`,
    `--property=StandardOutput=truncate:${log}`,
    '--property=StandardError=inherit',
    '--',
    shellProgram,
    '-c',
    gateUnitShellCommand({ caps }),
  ];
}
