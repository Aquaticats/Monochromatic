// Runs every published reproduction script on Node, Deno and Bun against two @optique/core versions, the way the
// repository's README says to run them, and prints one JSON line per run: the runtime, the arm, the script, the
// exit status, the script's own JSON line, and the start of its stderr.
//
// Timed comparisons are run in alternating rounds, one arm then the other, so each ratio comes from interleaved runs:
// commit-paths.ts --separator per runtime across the two npm versions, and on Deno both timed scripts across the
// two source checkouts (upstream main, and main with the input-trace prototype).

import { spawnSync, } from 'node:child_process';

const VERSIONS = ['1.3.6', '1.4.0-dev.2971',] as const;
const CHECKOUTS = ['before', 'after',] as const;
const RUNTIMES = ['node', 'deno', 'bun',] as const;
const ROUNDS = 5;

type Runtime = (typeof RUNTIMES)[number];
type Invocation = { command: string; argv: string[]; cwd: string; };

function npmInvocation(
  { runtime, version, script, args, }: { runtime: Runtime; version: string; script: string; args: string[]; },
): Invocation {
  // Deno resolves the package through an import map and installs nothing; Node and Bun read node_modules beside
  // the scripts.
  if (runtime === 'deno') {
    return {
      command: 'deno',
      argv: ['run', '--no-config', '--import-map', `maps/npm-${version}.json`, script, ...args,],
      cwd: '/repo',
    };
  }

  return { command: runtime, argv: [script, ...args,], cwd: `/arm/${version}`, };
}

function checkoutInvocation({ checkout, script, args, }: { checkout: string; script: string; args: string[]; },): Invocation {
  return {
    command: 'deno',
    argv: ['run', '--no-config', '--import-map', `/maps/src-${checkout}.json`, script, ...args,],
    cwd: '/repo',
  };
}

function run(
  { runtime, arm, script, args, round, invocation, }: {
    runtime: string;
    arm: string;
    script: string;
    args: string[];
    round: number;
    invocation: Invocation;
  },
): void {
  const child = spawnSync(invocation.command, invocation.argv, {
    cwd: invocation.cwd,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  },);
  let result: unknown = null;

  try {
    result = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
  }
  catch (error) {
    result = { unparsed: child.stdout.slice(0, 300,), reason: String(error,), };
  }

  console.log(JSON.stringify({
    runtime,
    arm,
    script,
    args,
    round,
    exit: child.status,
    signal: child.signal,
    result,
    stderr: child.stderr.slice(0, 400,),
  },),);
}

const versions = spawnSync('sh', ['-c', 'node --version; deno --version | head --lines=1; bun --version',], { encoding: 'utf8', },);

console.log(JSON.stringify({ runtimes: versions.stdout.trim().split('\n',), },),);

for (const runtime of RUNTIMES) {
  for (const version of VERSIONS) {
    for (const script of ['mismatch-count.ts', 'commit-paths.ts', 'token-scaling.ts',]) {
      run({ runtime, arm: version, script, args: [], round: 0, invocation: npmInvocation({ runtime, version, script, args: [], },), },);
    }
  }

  for (let round = 1; round <= ROUNDS; round += 1) {
    for (const version of VERSIONS) {
      const args = ['--separator',];

      run({ runtime, arm: version, script: 'commit-paths.ts', args, round, invocation: npmInvocation({ runtime, version, script: 'commit-paths.ts', args, },), },);
    }
  }
}

for (const checkout of CHECKOUTS) {
  run({ runtime: 'deno', arm: checkout, script: 'mismatch-count.ts', args: [], round: 0, invocation: checkoutInvocation({ checkout, script: 'mismatch-count.ts', args: [], },), },);
}

for (let round = 1; round <= ROUNDS; round += 1) {
  for (const checkout of CHECKOUTS) {
    for (const [script, args,] of [['token-scaling.ts', [],], ['commit-paths.ts', [],], ['commit-paths.ts', ['--separator',],],] as const) {
      run({ runtime: 'deno', arm: checkout, script, args: [...args,], round, invocation: checkoutInvocation({ checkout, script, args: [...args,], },), },);
    }
  }
}
