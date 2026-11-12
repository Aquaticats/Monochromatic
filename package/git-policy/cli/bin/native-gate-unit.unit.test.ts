import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { defaultGateCaps } from './native-gate-caps.ts';
import {
  gateExitStatusMarker,
  gateTask,
  gateUnitArguments,
  gateUnitShellCommand,
} from './native-gate-unit.ts';

/**
 The unit name a test starts.

 @returns the name
 */
function unitName(): string {
  return 'gpc-gate-wrapper-next-alpha-on-47eb745abb-20261110T180203-0500';
}

/**
 The argument array the scratch launcher this command replaced passed to `systemd-run`, which the promoted command has to
 keep passing.

 @returns the arguments after the program name
 */
function recordedArguments(): readonly string[] {
  return [
    '--user',
    `--unit=${unitName()}`,
    '--collect',
    '--working-directory=/repo/.claude/worktrees/gate-wrapper-next-alpha',
    `--setenv=GIT_POLICY_NATIVE_IMAGE_TAG=${unitName()}`,
    '--property=CPUQuota=1000%',
    '--property=MemoryMax=20G',
    '--property=TasksMax=4000',
    '--property=IOWeight=10',
    '--property=StandardOutput=truncate:/home/a/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-47eb745abb-20261110T180203-0500.log',
    '--property=StandardError=inherit',
    '--',
    'sh',
    '-c',
    String.raw`mise run //package/git-policy/cli:native:test:container -- --pids-limit 512; status=$?; printf "\ngate-exit-status=%s\n" "$status"; exit "$status"`,
  ];
}

await describe({
  name: 'the unit of one gate',
  children: [
    it({
      name: 'runs the wrapper\'s authority task, the one whose run a release is gated on',
      fn: async () => {
        expect(gateTask,).toBe('//package/git-policy/cli:native:test:container',);
        expect(gateUnitShellCommand({ caps: defaultGateCaps, }),).toContain(`mise run ${gateTask}`,);
      },
    }),
    it({
      name: 'names the exit-status line it writes, which a verdict reads instead of the collected unit',
      fn: async () => {
        expect(gateExitStatusMarker,).toBe('gate-exit-status=',);
        expect(gateUnitShellCommand({ caps: defaultGateCaps, }),).toContain(gateExitStatusMarker,);
      },
    }),
    it({
      name: 'writes the run\'s own status into the log and then exits with it, so a failed unit leaves evidence',
      fn: async () => {
        expect(gateUnitShellCommand({ caps: defaultGateCaps, }),).toBe(recordedArguments().at(-1),);
      },
    }),
    it({
      name: 'passes the container process limit the task was recorded with, and the one the caller asked for',
      fn: async () => {
        expect(gateUnitShellCommand({ caps: { ...defaultGateCaps, containerPidsLimit: 64, }, }),).toContain('--pids-limit 64',);
      },
    }),
    it({
      name: 'starts the unit with the caps, the worktree, the image tag and the log of the recorded gate, argument for argument',
      fn: async () => {
        expect(gateUnitArguments({
          name: unitName(),
          worktree: '/repo/.claude/worktrees/gate-wrapper-next-alpha',
          log: '/home/a/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-47eb745abb-20261110T180203-0500.log',
          caps: defaultGateCaps,
        }),).toEqual(recordedArguments(),);
      },
    }),
    it({
      name: 'names its own unit as the image tag, so two gates never share a tested image',
      fn: async () => {
        /**
         The arguments of this gate.
         */
        const args = gateUnitArguments({
          name: unitName(),
          worktree: '/w',
          log: '/l',
          caps: defaultGateCaps,
        });
        expect(args,).toContain(`--setenv=GIT_POLICY_NATIVE_IMAGE_TAG=${unitName()}`,);
      },
    }),
    it({
      name: 'truncates its own log at the start, so a name reused in the same second never reads an older verdict',
      fn: async () => {
        expect(gateUnitArguments({
          name: 'u',
          worktree: '/w',
          log: '/l/u.log',
          caps: defaultGateCaps,
        }),).toContain('--property=StandardOutput=truncate:/l/u.log',);
      },
    }),
  ],
});
