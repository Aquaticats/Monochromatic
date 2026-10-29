// Run a filtered set of the IDE's native tests many times at once on two starved CPUs and count failures per test.
// What: builds the test binary once in a container with the normal build bound, then starts a second GPU container
// that is limited to 2 CPUs and 2 GiB of memory, runs several `cargo nextest run --stress-count` invocations inside it
// at the same time (each test is its own process, as in `test:native`), and prints for every test how often it passed,
// was flaky (passed after a retry) and failed, with the first failure's panic text and the log files.
// Why: the native flakes appear only when tests contend for CPU and the GPU; the tests contend with each other inside
// the 2-CPU cap, so the run never loads the machine it runs on, and a rate means nothing without the runs behind it.
// A filtered `test:native` runs one test at a time (it passes --no-capture), which is why this tool exists.
// The pure helpers are tested in stress-native-report.test.ts; the containers are the only side effects.

import {
  spawn,
  spawnSync
} from 'node:child_process';
import {
  createWriteStream,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  statSync
} from 'node:fs';
import {
  homedir,
  loadavg
} from 'node:os';
import {
  join,
  resolve
} from 'node:path';
import { containerBounds } from './container-bounds.ts';
import { gpuArguments } from './gpu-arguments.ts';
import {
  summarize,
  type Tally
} from './stress-native-report.ts';

/**
 The options of one stress run, from the command line.
 */
type Options = {
  readonly filter: string;
  /**
   Iterations each copy runs one after the other (`--stress-count`).
   */
  readonly count: number;
  /**
   Nextest invocations that run at the same time in the container.
   */
  readonly copies: number;
  /**
   Retries per test (`--retries`); the run proves the harness shows a pass after retry as FLAKY.
   */
  readonly retries: number;
  /**
   Test processes each copy runs at once (`--test-threads`).
   */
  readonly threads: number;
  /**
   Longest wall time of the whole run, after which the container is removed.
   */
  readonly deadlineMinutes: number;
};

const usage = `Usage: node bin/stress-native.ts --filter <nextest filter expression> [--count 20] [--copies 4] [--retries 0] [--threads 2] [--deadline-minutes 60]
  --filter    nextest filter expression, for example 'test(rows_arriving_in_view)'; every selected test runs in each copy
  --count     iterations per copy, run one after the other
  --copies    concurrent nextest invocations in the 2-CPU container (each runs its tests in separate processes)
  --retries   nextest retries per test, 0 by default
  --threads   test processes each copy runs at once, 2 by default; copies times threads processes share 2 CPUs
  --deadline-minutes  the container is removed after this long, whatever has finished
The run never uses more than 2 CPUs and 2 GiB of memory, apart from the one build.`;

/**
 Read the command line.
 
 @param argv - arguments after the script name.
 
 @returns The options with their defaults.
 
 @throws {Error} When an option is unknown, repeated without a value, or not a whole number where one is required.
 */
function parse(argv: readonly string[]): Options {
  const values = new Map<string, string>();
  for (let index = 0; index < argv.length; index += 2) {
    const name = argv[index];
    const value = argv[index + 1];
    if ((!name?.startsWith('--')) || (value === undefined)) {
      throw new Error(`Unreadable argument ${name ?? ''}.\n${usage}`);
    }
    values.set(
      name.slice(2),
      value
    );
  }
  const known = new Set([
    'filter',
    'count',
    'copies',
    'retries',
    'threads',
    'deadline-minutes'
  ]);
  for (const name of values.keys()) {
    if (!known.has(name)) {
      throw new Error(`Unknown option --${name}.\n${usage}`);
    }
  }
  function whole(
    name: string,
    fallback: number
  ): number {
    const text = values.get(name);
    if (text === undefined) {
      return fallback;
    }
    if (!/^[0-9]{1,6}$/.test(text)) {
      throw new Error(`--${name} is "${text}", which is not a whole number.\n${usage}`);
    }
    return Number(text);
  }
  const filter = values.get('filter');
  if ((filter === undefined) || (filter === '')) {
    throw new Error(`--filter is required.\n${usage}`);
  }
  return {
    filter,
    count: Math.max(
      1,
      whole(
        'count',
        20
      )
    ),
    copies: Math.max(
      1,
      whole(
        'copies',
        4
      )
    ),
    retries: whole(
      'retries',
      0
    ),
    threads: Math.max(
      1,
      whole(
        'threads',
        2
      )
    ),
    deadlineMinutes: Math.max(
      1,
      whole(
        'deadline-minutes',
        60
      )
    ),
  };
}

const options = parse(process.argv
  .slice(2));
const limits = containerBounds();
console.error(limits.summary);
const privateRoot = join(
  homedir(),
  'temp',
  'agent'
);
mkdirSync(
  privateRoot,
  {
    recursive: true,
    mode: 0o700
  }
);
if ((statSync(privateRoot)
  .mode
  & 0o077) !== 0) {
  throw new Error(`Scratch root must exclude group and other permissions: ${  privateRoot}`);
}
const logs = mkdtempSync(join(
  privateRoot,
  'stress-'
));
const name = `ide-stress-${  process.pid}`;
const buildName = `ide-stress-build-${  process.pid}`;
/**
 Bound of the container that runs the tests: starving the tests themselves is the only load the tool makes.
 */
const runBound = [
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=2048',
  '--ulimit',
  'nofile=4096:4096'
];
const image = 'localhost/monochromatic/ide';
const packageDirectory = resolve('.');
const rootClippyConfig = resolve(
  packageDirectory,
  '../../../clippy.toml'
);

function podman(
  args: readonly string[],
  inherit = false
) { return spawnSync(
  'podman',
  args,
  {
    encoding: 'utf8',
    stdio: inherit ? 'inherit' : [
      'ignore',
      'pipe',
      'pipe'
    ],
    maxBuffer: 64 * 1_024
      * 1_024
  }
); }

/**
 Remove both containers; safe to call twice.
 */
function cleanUp(): void {
  podman([
    'rm',
    '--force',
    '--time',
    '0',
    name
  ]);
  podman([
    'rm',
    '--force',
    '--time',
    '0',
    buildName
  ]);
}
process.on(
  'SIGINT',
  () => {
    cleanUp();
    process.exit(130);
  }
);
process.on(
  'SIGTERM',
  () => {
    cleanUp();
    process.exit(143);
  }
);

/**
 The video memory of the largest card, read from sysfs, or nothing when no card reports it.
 */
function videoMemory(): {
  used: number;
  total: number
} | undefined {
  let best: {
    used: number;
    total: number
  } | undefined;
  for (const card of readdirSync('/sys/class/drm')
    .filter(entry => /^card[0-9]+$/.test(entry))) {
    try {
      const total = Number(readFileSync(
        `/sys/class/drm/${card}/device/mem_info_vram_total`,
        'utf8'
      ));
      const used = Number(readFileSync(
        `/sys/class/drm/${card}/device/mem_info_vram_used`,
        'utf8'
      ));
      if ((best === undefined) || (total > best.total)) {
        best = {
          used,
          total
        };
      }
    } catch {
      // A card without the counters is not the one the tests render on.
    }
  }
  return best;
}

/**
 Peak video memory and load average seen while sampling, so a failure can be read against the pressure it met.
 */
const pressure = {
  vramPeak: 0,
  vramTotal: 0,
  loadPeak: 0,
  samples: 0
};
function sample(): void {
  const memory = videoMemory();
  if (memory !== undefined) {
    pressure.vramPeak = Math.max(
      pressure.vramPeak,
      memory.used
    );
    pressure.vramTotal = memory.total;
  }
  pressure.loadPeak = Math.max(
    pressure.loadPeak,
    loadavg()[0] ?? 0
  );
  pressure.samples += 1;
}

/**
 Run one command in a container and wait for it; output goes to `logFile`.
 */
function execute(
  container: string,
  command: readonly string[],
  logFile: string
): Promise<number> { return new Promise(done => {
  const child = spawn(
    'podman',
    [
      'exec',
      ...limits.environmentArguments,
      container,
      ...command
    ],
    { stdio: [
      'ignore',
      'pipe',
      'pipe'
    ] }
  );
    // The log grows while the run goes on, so its progress can be read from outside.
  const log = createWriteStream(logFile);
  child.stdout
    .pipe(
    log,
    { end: false }
    );
  child.stderr
    .pipe(
    log,
    { end: false }
    );
  child.on(
    'close',
    status => {
      log.end(() =>{  done(status ?? 1); });
    }
  );
}); }

/**
 Start a detached container that sleeps until the deadline; builds and tests run in it through `podman exec`.
 */
function startContainer(
  container: string,
  bound: readonly string[]
): void {
  const started = podman([
    'run',
    '--detach',
    '--rm',
    '--network=none',
    '--name',
    container,
    ...gpuArguments,
    ...bound,
    '--security-opt',
    'label=disable',
    '--volume',
    `${packageDirectory  }:/work`,
    '--volume',
    'ide-cargo:/cargo',
    '--volume',
    `${rootClippyConfig  }:/clippy-conf/clippy.toml:ro`,
    '--env',
    'CLIPPY_CONF_DIR=/clippy-conf',
    '--workdir',
    '/work',
    ...limits.environmentArguments,
    '--env',
    'SLINT_EMIT_DEBUG_INFO=1',
    '--env',
    'HELIX_RUNTIME=/work/target/debug/runtime',
    image,
    'sleep',
    String(options.deadlineMinutes * 60),
  ]);
  if (started.status !== 0) {
    throw new Error(`Could not start the container: ${  started.stderr}`);
  }
}

try {
  // One build of the test binary and the scripted language server, with the normal build bound (the only run that may
  // use more than 2 CPUs), so the copies below only run it.
  startContainer(
    buildName,
    limits.limitArguments
  );
  for (const build of [
    [
      'cargo',
      'build',
      '--bin',
      'ide-scripted-lsp'
    ],
    [
      'cargo',
      'nextest',
      'run',
      '--bin',
      'monochromatic-ide',
      '--no-run'
    ]
  ]) {
    const status = await execute(
      buildName,
      build,
      join(
        logs,
        'build.log'
      )
    );
    if (status !== 0) {
      process.stderr
        .write(readFileSync(join(
          logs,
          'build.log'
        )));
      throw new Error(`The build failed: ${  build.join(' ')}`);
    }
  }
  podman([
    'rm',
    '--force',
    '--time',
    '0',
    buildName
  ]);
  startContainer(
    name,
    runBound
  );
  console.error('Run bound: 2 CPUs, 2g memory, 2048 processes; tests contend with each other, nothing else loads the host');
  const began = Date.now();
  sample();
  const sampler = setInterval(
    sample,
    5_000
  );
  const runs = Array.from(
    { length: options.copies },
    (
      _,
      copy
    ) =>
      execute(
        name,
        [
          'cargo',
          'nextest',
          'run',
          '--bin',
          'monochromatic-ide',
          '--filter-expr',
          options.filter,
          '--no-fail-fast',
          '--stress-count',
          String(options.count),
          '--retries',
          String(options.retries),
          '--test-threads',
          String(options.threads),
          '--status-level',
          'pass',
          '--final-status-level',
          'none',
          '--failure-output',
          'immediate',
        ],
        join(
          logs,
          `copy-${copy}.log`
        )
      )
  );
  await Promise.all(runs);
  clearInterval(sampler);
  sample();
  const tally: Tally = summarize(Array.from(
    { length: options.copies },
    (
      _,
      copy
    ) => readFileSync(
      join(
        logs,
        `copy-${copy}.log`
      ),
      'utf8'
    )
  ));
  console.log(`Stress run: filter ${options.filter}, ${options.copies} copies x ${options.count} iterations, 2 CPUs, retries ${options.retries}, threads ${options.threads} per copy, ${Math.round((Date.now() - began) / 1_000)} s`);
  console.log(`Logs: ${logs}`);
  console.log(`Pressure: video memory peak ${(pressure.vramPeak / (2
    ** 30)).toFixed(2)} of ${(pressure.vramTotal / (2
      ** 30)).toFixed(2)} GiB (all programs), load average peak ${pressure.loadPeak
        .toFixed(1)}, ${pressure.samples} samples`);
  console.log(tally.report);
  process.exitCode = tally.failures > 0 ? 1 : 0;
} finally {
  cleanUp();
}
