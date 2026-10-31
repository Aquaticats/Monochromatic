import {
  join,
  resolve,
} from 'node:path';
import { parseArgs } from 'node:util';

import type { ForeignBorrowed } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  defaultAttempts,
  defaultNamePrefix,
  defaultRetryMarker,
  defaultScopeFlag,
  defaultTagTemplate,
  defaultTimeoutSeconds,
  defaultUnitCpuQuota,
  defaultUnitMemoryMax,
  lockDirectoryEnvironmentVariable,
  rootEnvironmentVariable,
  scratchSegments,
  slotCountEnvironmentVariable,
  unitTemporarySegment,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  environmentEntry,
  nonEmptyText,
  plainName,
  positiveWholeNumber,
  systemdValue,
} from './validate.ts';

/**
 Environment as the launcher sees it.
 */
export type Environment = NodeJS.ProcessEnv;

/**
 Everything one launch needs, with every default and environment fallback already applied.
 */
export type LauncherOptions = {
  readonly label: string;
  readonly files: readonly string[];
  readonly task: string;
  readonly tagEnvironmentVariable: string;
  readonly tagTemplate: string;
  readonly extraEnvironment: readonly string[];
  readonly scopePrefix: string;
  readonly scopeFlag: string;
  readonly root: string;
  readonly logDirectory: string;
  readonly temporaryDirectory: string;
  readonly unitPrefix: string;
  readonly logPrefix: string;
  readonly gateSlotPath: string;
  readonly gateSlotArguments: readonly string[];
  readonly unitMemoryMax: string;
  readonly unitCpuQuota: string;
  readonly timeoutSeconds: number;
  readonly attempts: number;
  readonly retryMarker: string;
  readonly memory?: string;
  readonly cpus?: string;
  readonly home: string;
  readonly path?: string;
};

/**
 Usage text printed with every option error.
 */
export const launcherUsage: string = [
  'Usage: mutation-parallel.ts --task <mise task> --tag-env <name> [options] <label> <file>...',
  '',
  'Starts one capped systemd user unit per file; each runs the campaign task through the gate-slot limiter.',
  '',
  'Required:',
  '  --task <task>             mise task that runs one file campaign, e.g. //package/x/cli:native:mutation:scoped',
  '  --tag-env <name>          environment variable through which the task reads its image tag',
  'Campaign:',
  '  --scope-prefix <text>     prepended to each file to form its scope (default: none)',
  `  --scope-flag=<flag>       task flag that takes a scope, written with an equals sign (default: ${defaultScopeFlag})`,
  `  --timeout <seconds>       per-mutant timeout forwarded to the task (default: ${defaultTimeoutSeconds})`,
  '  --memory <size>           container memory limit forwarded to the task when given',
  '  --cpus <count>            container CPU count forwarded to the task when given',
  `  --attempts <count>        runs per file while the baseline fails (default: ${defaultAttempts})`,
  `  --retry-marker <text>     log text that requests a rerun (default: ${JSON.stringify(defaultRetryMarker)})`,
  `  --tag-template <text>     image tag from {label} and {index} (default: ${defaultTagTemplate})`,
  '  --setenv <NAME=value>     extra environment of every unit; repeatable',
  'Unit:',
  `  --unit-memory-max <value> systemd MemoryMax of each unit (default: ${defaultUnitMemoryMax})`,
  `  --unit-cpu-quota <value>  systemd CPUQuota of each unit (default: ${defaultUnitCpuQuota})`,
  `  --unit-prefix <text>      unit name prefix (default: ${defaultNamePrefix})`,
  'Locations:',
  `  --root <dir>              working tree the units start in (default: $${rootEnvironmentVariable}, then this checkout)`,
  '  --log-dir <dir>           where attempt logs go (default: <home>/temp/agent)',
  '  --tmp-dir <dir>           TMPDIR of each unit (default: <home>/temp/agent/tmp)',
  `  --log-prefix <text>       log file name prefix (default: ${defaultNamePrefix})`,
  'Gate slots:',
  `  --slot-count <count>      slots, overriding $${slotCountEnvironmentVariable} and the limiter default`,
  `  --lock-dir <dir>          slot lock directory, overriding $${lockDirectoryEnvironmentVariable}`,
  '  --gate-slot <path>        limiter script (default: bin/gate-slot.ts of this package)',
].join('\n');

/**
 Read the launcher's command line, apply defaults and environment fallbacks, and validate every value.

 Nothing is started or created here; the result feeds `planUnits`.

 @param argv - arguments after the script path
 
 @param environment - process environment
 
 @param home - current user's home directory, the base of every default location
 
 @param packageDirectory - directory of this package, which locates the default working tree and limiter script
 
 @returns validated options
 
 @throws CargoMutantsToolingError when a required argument is missing or a value is malformed, with the usage text
 
 @throws TypeError when `parseArgs` rejects an unknown flag or a flag without its value
 
 @example
 ```ts
 parseLauncherOptions({ argv: ['--task', 't', '--tag-env', 'E', 'w5', 'a.rs'], environment: {}, home: '/h', packageDirectory: '/p' });
 ```
 */
export function parseLauncherOptions({
  argv,
  environment,
  home,
  packageDirectory,
}: ForeignBorrowed<{
  readonly argv: readonly string[];
  readonly environment: Environment;
  readonly home: string;
  readonly packageDirectory: string;
}>): LauncherOptions {
  /**
   Flag values and positional arguments.
   */
  const {
    values,
    positionals
  } = parseArgs({
    args: [...argv],
    allowPositionals: true,
    strict: true,
    options: {
      'task': { type: 'string' },
      'tag-env': { type: 'string' },
      'tag-template': { type: 'string' },
      'setenv': {
        type: 'string',
        multiple: true,
      },
      'scope-prefix': { type: 'string' },
      'scope-flag': { type: 'string' },
      'timeout': { type: 'string' },
      'memory': { type: 'string' },
      'cpus': { type: 'string' },
      'attempts': { type: 'string' },
      'retry-marker': { type: 'string' },
      'unit-memory-max': { type: 'string' },
      'unit-cpu-quota': { type: 'string' },
      'unit-prefix': { type: 'string' },
      'root': { type: 'string' },
      'log-dir': { type: 'string' },
      'tmp-dir': { type: 'string' },
      'log-prefix': { type: 'string' },
      'slot-count': { type: 'string' },
      'lock-dir': { type: 'string' },
      'gate-slot': { type: 'string' },
    },
  });
  /**
   Positional label and files.
   */
  const [label, ...files] = positionals;
  if ((label === undefined) || (files.length === 0))
    throw new CargoMutantsToolingError(`A label and at least one file are required.\n\n${launcherUsage}`);
  if ((values.task === undefined) || (values['tag-env'] === undefined))
    throw new CargoMutantsToolingError(`--task and --tag-env are required.\n\n${launcherUsage}`);
  /**
   Scratch directory under the home directory.
   */
  const scratch = join(
    home,
    ...scratchSegments,
  );
  /**
   Slot count from the flag or, failing that, the environment; the units do not inherit the environment.
   */
  const slotCount = values['slot-count'] ?? environment[slotCountEnvironmentVariable];
  /**
   Lock directory from the flag or, failing that, the environment.
   */
  const lockDirectory = values['lock-dir'] ?? environment[lockDirectoryEnvironmentVariable];
  /**
   Limiter options forwarded only when named, so the limiter's own defaults apply otherwise.
   */
  const gateSlotArguments = [
    ...(slotCount === undefined
      ? []
      : [
        '--slot-count',
        String(positiveWholeNumber({
          option: '--slot-count',
          value: slotCount,
        })),
      ]),
    ...(lockDirectory === undefined
      ? []
      : [
        '--lock-dir',
        resolve(lockDirectory),
      ]),
  ];
  /**
   Launcher PATH, which the units need because they start with a bare environment.
   */
  const path = environment.PATH;
  return {
    label: plainName({
      option: 'label',
      value: label,
    }),
    files: files.map(function checkFile(file) {
      return nonEmptyText({
        option: 'file',
        value: file,
      });
    }),
    task: nonEmptyText({
      option: '--task',
      value: values.task,
    }),
    tagEnvironmentVariable: nonEmptyText({
      option: '--tag-env',
      value: values['tag-env'],
    }),
    tagTemplate: nonEmptyText({
      option: '--tag-template',
      value: values['tag-template'] ?? defaultTagTemplate,
    }),
    extraEnvironment: (values.setenv ?? []).map(function checkEntry(entry) {
      return environmentEntry(entry);
    }),
    scopePrefix: values['scope-prefix'] ?? '',
    scopeFlag: nonEmptyText({
      option: '--scope-flag',
      value: values['scope-flag'] ?? defaultScopeFlag,
    }),
    root: resolve(values.root ?? environment[rootEnvironmentVariable]
      ?? join(
        packageDirectory,
        '..',
        '..',
        '..',
      )),
    logDirectory: resolve(values['log-dir'] ?? scratch),
    temporaryDirectory: resolve(values['tmp-dir'] ?? join(
      scratch,
      unitTemporarySegment,
    )),
    unitPrefix: plainName({
      option: '--unit-prefix',
      value: values['unit-prefix'] ?? defaultNamePrefix,
    }),
    logPrefix: plainName({
      option: '--log-prefix',
      value: values['log-prefix'] ?? defaultNamePrefix,
    }),
    gateSlotPath: resolve(values['gate-slot'] ?? join(
      packageDirectory,
      'bin',
      'gate-slot.ts',
    )),
    gateSlotArguments,
    unitMemoryMax: systemdValue({
      option: '--unit-memory-max',
      value: values['unit-memory-max'] ?? defaultUnitMemoryMax,
    }),
    unitCpuQuota: systemdValue({
      option: '--unit-cpu-quota',
      value: values['unit-cpu-quota'] ?? defaultUnitCpuQuota,
    }),
    timeoutSeconds: values.timeout === undefined
      ? defaultTimeoutSeconds
      : positiveWholeNumber({
        option: '--timeout',
        value: values.timeout,
      }),
    attempts: values.attempts === undefined
      ? defaultAttempts
      : positiveWholeNumber({
        option: '--attempts',
        value: values.attempts,
      }),
    retryMarker: nonEmptyText({
      option: '--retry-marker',
      value: values['retry-marker'] ?? defaultRetryMarker,
    }),
    ...(values.memory === undefined ? {} : {
      memory: nonEmptyText({
        option: '--memory',
        value: values.memory,
      }),
    }),
    ...(values.cpus === undefined ? {} : {
      cpus: nonEmptyText({
        option: '--cpus',
        value: values.cpus,
      }),
    }),
    home,
    ...(path === undefined ? {} : { path }),
  };
}
