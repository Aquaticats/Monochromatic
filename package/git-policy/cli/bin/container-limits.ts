/**
 Memory, CPU and process limits of every container the verification runners start, with the Podman arguments they become.

 The defaults are the repository rule for test containers: 2 GiB, 2 CPUs and 128 processes.
 A task overrides them with `--memory <size>`, `--cpus <count>` and, where its container has a process cap,
 `--pids-limit <count>`.
 Image builds take the CPU bound as a CFS period and quota (`quota = cpus * period`) because `podman build` has no `--cpus`.
 Every option is checked with a linear scan over its characters, so a malformed option fails before any container starts.
 */
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 Memory every container may use unless a task overrides it: 2 GiB.
 */
export const defaultMemory = '2g';

/**
 CPUs every container may use unless a task overrides it.
 */
export const defaultCpus = 2;

/**
 Processes every container with a process cap may run unless a task overrides it.
 */
export const defaultPidsLimit = 128;

/**
 Length of one CFS scheduling period in microseconds, which is also Podman's own default period.
 */
export const cpuPeriodMicroseconds = 100_000;

/**
 Smallest CPU count a task may request.
 The kernel refuses a CFS quota under 1000 microseconds, which is 0.01 CPU at the default period.
 */
const minimumCpus = 0.01;

/**
 Most decimals a CPU count may carry, which keeps its quota a whole number of microseconds.
 */
const maximumCpuDecimals = 2;

/**
 Memory units Podman accepts after the number: bytes, kibibytes, mebibytes and gibibytes.
 */
const memoryUnits = 'bkmg';

/**
 Decimal digits.
 */
const decimalDigits = '0123456789';

/**
 Options this module reads, in the order the usage messages list them.
 */
const limitFlags = [
  '--memory',
  '--cpus',
  '--pids-limit',
] as const;

/**
 One option this module reads.
 */
type LimitFlag = typeof limitFlags[number];

/**
 Limits of one container.
 */
export type ContainerLimits = {
  /**
   Podman memory size: digits and an optional unit, such as `2g` or `512m`.
   */
  readonly memory: string;
  /**
   CPUs the container may use; fractions allowed.
   */
  readonly cpus: number;
  /**
   Processes the container may run; ignored where the container has no process cap.
   */
  readonly pidsLimit: number;
};

/**
 Limits every task starts from.
 */
export const defaultContainerLimits: ContainerLimits = {
  memory: defaultMemory,
  cpus: defaultCpus,
  pidsLimit: defaultPidsLimit,
};

/**
 Check that a text is a non-empty run of decimal digits without a leading zero.

 @param text - candidate whole number

 @returns true for `1` and `128`; false for `0`, `01`, `1.5` and the empty text
 */
function isPositiveWholeNumber(text: string): boolean {
  if ((text.length === 0) || text.startsWith('0',))
    return false;
  for (const character of text) {
    if (!decimalDigits.includes(character,))
      return false;
  }
  return true;
}

/**
 Check for a Podman memory size.

 @param text - `--memory` value

 @returns true for a positive whole number with an optional `b`, `k`, `m` or `g` unit
 */
function isMemorySize(text: string): boolean {
  if (text.length === 0)
    return false;
  /**
   Last character, which is the unit when it is not a digit.
   */
  const last = text.slice(-1,);
  return isPositiveWholeNumber(memoryUnits.includes(last,) ? text.slice(
    0,
    -1,
  ) : text,);
}

/**
 Check for a CPU count of at least 0.01 with at most two decimals.

 @param text - `--cpus` value

 @returns true for `2`, `1.5` and `0.25`; false for `0`, `0.001`, `1.`, `.5`, `-1` and `1e1`
 */
function isCpuCount(text: string): boolean {
  /**
   Digits before and after the decimal point.
   */
  const [whole = '', fraction, ...surplus] = text.split('.',);
  if ((surplus.length > 0) || ((fraction !== undefined) && ((fraction.length === 0) || (fraction.length > maximumCpuDecimals))))
    return false;
  for (const character of `${whole}${fraction ?? ''}`) {
    if (!decimalDigits.includes(character,))
      return false;
  }
  if ((whole.length === 0) || ((whole.length > 1) && whole.startsWith('0',)))
    return false;
  return Number(text,) >= minimumCpus;
}

/**
 CFS quota that limits a container to a number of CPUs per period.

 @param cpus - CPUs the container may use

 @returns microseconds of CPU time per period, rounded because floating point turns `0.1 * 100000` into `10000.000000000002`

 @example
 ```ts
 const quota = cpuQuotaMicroseconds(1.5);
 ```
 */
export function cpuQuotaMicroseconds(cpus: number): number {
  return Math.round(cpus * cpuPeriodMicroseconds,);
}

/**
 Memory bound alone, for the build of an image whose compilation is not CPU bound.

 @param limits - container limits

 @returns `['--memory=2g']` for the defaults

 @example
 ```ts
 const bound = memoryArguments(defaultContainerLimits);
 ```
 */
export function memoryArguments(limits: ContainerLimits,): string[] {
  return [`--memory=${limits.memory}`,];
}

/**
 Resource bounds of `podman build`, which has no `--cpus` option.

 @param limits - container limits

 @returns memory, CFS period and the quota derived from the CPUs

 @example
 ```ts
 const bounds = buildLimitArguments(defaultContainerLimits);
 ```
 */
export function buildLimitArguments(limits: ContainerLimits,): string[] {
  return [
    ...memoryArguments(limits,),
    `--cpu-period=${String(cpuPeriodMicroseconds,)}`,
    `--cpu-quota=${String(cpuQuotaMicroseconds(limits.cpus,),)}`,
  ];
}

/**
 Resource bounds of a started container.

 @param limits - container limits

 @param withPidsLimit - whether the container has a process cap

 @returns memory, CPUs and, when capped, the process limit

 @example
 ```ts
 const bounds = runLimitArguments({ limits: defaultContainerLimits, withPidsLimit: true });
 ```
 */
export function runLimitArguments({
  limits,
  withPidsLimit,
}: {
  readonly limits: ContainerLimits;
  readonly withPidsLimit: boolean;
},): string[] {
  return [
    ...memoryArguments(limits,),
    `--cpus=${String(limits.cpus,)}`,
    ...(withPidsLimit ? [`--pids-limit=${String(limits.pidsLimit,)}`,] : []),
  ];
}

/**
 Parallel jobs a build inside the container may start: one per whole CPU.

 @param limits - container limits

 @returns the CPUs rounded down, at least one

 @example
 ```ts
 const jobs = parallelJobs(defaultContainerLimits);
 ```
 */
export function parallelJobs(limits: ContainerLimits,): number {
  return Math.max(
    1,
    Math.floor(limits.cpus,),
  );
}

/**
 One limit option as given on the command line.
 */
type GivenLimit = {
  readonly flag: LimitFlag;
  readonly value: string;
};

/**
 Whether an argument is one of the options this module reads.

 @param argument - command-line argument

 @returns true for `--memory`, `--cpus` and `--pids-limit`
 */
function isLimitFlag(argument: string,): argument is LimitFlag {
  return limitFlags.some(function isSame(flag,) {
    return flag === argument;
  },);
}

/**
 Read one option's value into the limits.

 @param limits - limits read so far

 @param flag - option being read

 @param value - text after the option, empty when the option ends the arguments

 @returns the limits with that option applied
 */
function applyLimit({
  limits,
  flag,
  value,
}: {
  readonly limits: ContainerLimits;
  readonly flag: LimitFlag;
  readonly value: string;
},): ContainerLimits {
  /**
   Text the diagnostic quotes back.
   */
  const got = value === '' ? 'no value' : value;
  if (flag === '--memory') {
    if (!isMemorySize(value,))
      throw new NativeVerificationError(`--memory must be a whole number of bytes with an optional unit b, k, m or g (for example 512m or 2g), got ${got}.`,);
    return {
      ...limits,
      memory: value,
    };
  }
  if (flag === '--cpus') {
    if (!isCpuCount(value,))
      throw new NativeVerificationError(`--cpus must be a number of CPUs from ${String(minimumCpus,)} with at most ${String(maximumCpuDecimals,)} decimals; fractions are allowed (for example 2 or 1.5), got ${got}.`,);
    return {
      ...limits,
      cpus: Number(value,),
    };
  }
  if (!isPositiveWholeNumber(value,))
    throw new NativeVerificationError(`--pids-limit must be a whole number of processes from 1 (for example 128), got ${got}.`,);
  return {
    ...limits,
    pidsLimit: Number(value,),
  };
}

/**
 Split `--memory <size>`, `--cpus <count>` and `--pids-limit <count>` from the other arguments of a task.

 The last repeat of an option wins.
 Other arguments are returned in order, an option with its value kept together, for the task's own parser.

 @param options - command-line arguments after the script path

 @param acceptPidsLimit - whether the task's container has a process cap, so `--pids-limit` means something

 @param leadingOnly - whether to read options only until the first argument that is not one, so a trailing argument
   list forwarded to another program (the interop driver) keeps its own `--memory`

 @returns the limits with the defaults for options not given, and the arguments left over

 @throws NativeVerificationError when an option has no value or a value outside the accepted forms, or when
   `--pids-limit` is given to a task without a process cap

 @example
 ```ts
 const { limits, rest } = parseContainerLimitOptions({
   options: ['--memory', '512m', '--timeout', '60'],
   acceptPidsLimit: true,
   leadingOnly: false,
 });
 ```
 */
export function parseContainerLimitOptions({
  options,
  acceptPidsLimit,
  leadingOnly,
}: {
  readonly options: readonly string[];
  readonly acceptPidsLimit: boolean;
  readonly leadingOnly: boolean;
},): {
  readonly limits: ContainerLimits;
  readonly rest: readonly string[];
} {
  /**
   Limit options in command-line order, each with its value.
   */
  const given: GivenLimit[] = [];
  /**
   Arguments that are not limit options.
   */
  const rest: string[] = [];
  for (let index = 0; index < options.length; index += 2) {
    /**
     The option at this position.
     */
    const flag = options[index] ?? '';
    /**
     The text after the option, empty when the option ends the arguments.
     */
    const value = options[index + 1] ?? '';
    /**
     Whether the argument after the option exists.
     */
    const hasValue = (index + 1) < options.length;
    if (!isLimitFlag(flag,)) {
      if (leadingOnly) {
        rest.push(...options.slice(index,),);
        break;
      }
      rest.push(
        flag,
        ...(hasValue ? [value,] : []),
      );
      continue;
    }
    if ((flag === '--pids-limit') && (!acceptPidsLimit))
      throw new NativeVerificationError('--pids-limit is not accepted by this task: its container has no process cap.',);
    given.push({
      flag,
      value,
    },);
  }
  return {
    limits: given.reduce(
      function applyGiven(
        limits,
        {
          flag,
          value,
        },
      ) {
        return applyLimit({
          limits,
          flag,
          value,
        },);
      },
      defaultContainerLimits,
    ),
    rest,
  };
}
