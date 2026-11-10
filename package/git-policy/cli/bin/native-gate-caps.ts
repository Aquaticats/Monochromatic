/**
 The caps of one release gate: the transient systemd unit that runs it, and the container the tests run in.

 A gate is a full `native:test:container` run of the wrapper, which is the image build, the compile, the suite and
 Clippy, started in a unit of its own so it never holds the terminal or the control group of whoever started it (the
 owner, quoted in `doc/handover/cli-git-rust-implementation.md`, "Working rules for this effort": "You should (almost)
 never be waiting. There is always more work."). Two levels are capped, as a mutation campaign is
 (`package/mutation-test/cli-cargo-mutants/src/constants.ts`): the unit bounds the whole process tree, and the
 container's own process limit bounds the tests, which sit in a `libpod-*` scope outside the unit and therefore carry
 neither its `TasksMax` nor its `CPUQuota`.
 */
import { NativeGateError } from './native-gate-error.ts';

/**
 Everything one gate caps.
 */
export type GateCaps = {
  /**
   `CPUQuota` of the gate's unit, as systemd reads it.
   */
  readonly unitCpuQuota: string;
  /**
   `MemoryMax` of the gate's unit, as systemd reads it.
   */
  readonly unitMemoryMax: string;
  /**
   `TasksMax` of the gate's unit, in tasks, which a control group counts as threads and processes together.
   */
  readonly unitTasksMax: number;
  /**
   `IOWeight` of the gate's unit, from one to ten.
   */
  readonly unitIoWeight: number;
  /**
   Processes the gate container may have, which is its `--pids-limit`.
   */
  readonly containerPidsLimit: number;
};

/**
 Highest `IOWeight` a gate unit may carry.

 The owner's rule (`AGENTS.md`, rule `RXS`) is that a per-command cgroup never sets `IOWeight` above ten, and a unit left
 without the property gets systemd's default of a hundred.
 */
export const maximumUnitIoWeight = 10;

/**
 The caps every recorded gate of this repository ran with, which is what the promoted scratch scripts passed.

 Each one is overridable on the command line of `native:gate`; the defaults stay the values the recorded runs used, so a
 log of an earlier gate and a log of the next one are comparable.
 */
export const defaultGateCaps: GateCaps = {
  // Ten threads. This host has eight (measured 2026-11-10 with `nproc`), so the quota cannot bind here and the
  // container's own two CPUs (`native:test:container`'s default) are the bound that does; it stays as an upper bound
  // for a host with more threads than that.
  unitCpuQuota: '1000%',
  // The unit holds the image build, the container client and nextest. No recorded gate measured the unit's own peak
  // memory, so the value stays the one every recorded gate ran with; the tests themselves are capped at two gigabytes
  // by the container.
  unitMemoryMax: '20G',
  // A smaller control group refused `git-lfs` its filter process during two LFS controls, which is what this value is
  // sized from (`doc/handover/git-policy-cli-on-tokio.md`, "Step T11": both suites ran "with the run's own control
  // group at TasksMax=4000, because a smaller one refused `git-lfs`'s filter process to two LFS controls").
  unitTasksMax: 4_000,
  // The lowest disk weight every other capped unit of the repository uses (`AGENTS.md`, rule `RXS`).
  unitIoWeight: 10,
  // Measured 2026-11-09 inside the gate's own image and settings: the run peaked at 192 tasks against `pids.max` 8192,
  // and "`--pids-limit 512` stays what it is: 192 measured leaves it 2.7 times the room"
  // (`doc/planning/git-policy-cli-full-tokio.md`, "Step T12").
  containerPidsLimit: 512,
};

/**
 Characters a systemd resource value such as `20G`, `1000%` or `infinity` is made of.
 */
const systemdValueCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.%';

/**
 Why one text cap is unusable, or nothing when it can be read.

 @param value - the cap as the caller gave it

 @param name - the option that carried it

 @returns the refusal as the only entry, or no entry at all

 @example
 ```ts
 textCapProblem({ value: '20G', name: '--unit-memory-max' });
 ```
 */
function textCapProblem({
  value,
  name,
}: {
  readonly value: string;
  readonly name: string;
}): readonly string[] {
  if (value === '')
    return [`${name} is empty, and systemd reads a cap from a value such as 20G or 1000%.`];
  for (let index = 0; index < value.length; index += 1) {
    /**
     The character at this position.
     */
    const character = value.charAt(index);
    if (!systemdValueCharacters.includes(character))
      return [`${name} "${value}" holds a character systemd does not read in a cap value; use letters, digits, "." and "%" as in 20G or 1000%.`];
  }
  return [];
}

/**
 Why one whole-number cap is unusable, or nothing when it can be used.

 @param value - the cap as the caller gave it

 @param name - the option that carried it

 @returns the refusal as the only entry, or no entry at all

 @example
 ```ts
 wholeCapProblem({ value: 512, name: '--container-pids-limit' });
 ```
 */
function wholeCapProblem({
  value,
  name,
}: {
  readonly value: number;
  readonly name: string;
}): readonly string[] {
  return Number.isInteger(value) && (value >= 1)
    ? []
    : [`${name} ${String(value)} is not a whole number from one.`];
}

/**
 Why a gate's caps cannot be used, one line each, empty when they can.

 Checked before anything starts, because a cap systemd cannot read fails the start with a message about a property, and
 the two whole numbers are written into the unit's own shell command.

 @param caps - the caps to check

 @returns one line per refusal, in the order the options are listed

 @example
 ```ts
 gateCapsProblems({ caps: defaultGateCaps });
 ```
 */
export function gateCapsProblems({ caps }: { readonly caps: GateCaps; }): readonly string[] {
  return [
    ...textCapProblem({
      value: caps.unitCpuQuota,
      name: '--unit-cpu-quota',
    }),
    ...textCapProblem({
      value: caps.unitMemoryMax,
      name: '--unit-memory-max',
    }),
    ...wholeCapProblem({
      value: caps.unitTasksMax,
      name: '--unit-tasks-max',
    }),
    ...wholeCapProblem({
      value: caps.containerPidsLimit,
      name: '--container-pids-limit',
    }),
    ...(caps.unitIoWeight > maximumUnitIoWeight
      ? [`--unit-io-weight ${String(caps.unitIoWeight)} is above ${String(maximumUnitIoWeight)}, the highest disk weight a per-command control group may carry (rule RXS).`]
      : wholeCapProblem({
        value: caps.unitIoWeight,
        name: '--unit-io-weight',
      })),
  ];
}

/**
 Refuse caps systemd could not read, before anything starts.

 @param caps - the caps the caller asked for

 @returns the caps, unchanged

 @throws NativeGateError when a cap is not a value systemd reads, naming every unusable one

 @example
 ```ts
 const caps = checkedGateCaps({ caps: defaultGateCaps });
 ```
 */
export function checkedGateCaps({ caps }: { readonly caps: GateCaps; }): GateCaps {
  /**
   Every reason the caps are unusable.
   */
  const problems = gateCapsProblems({ caps, });
  if (problems.length > 0)
    throw new NativeGateError(`the gate's caps cannot be used: ${problems.join('\n')}`);
  return caps;
}
