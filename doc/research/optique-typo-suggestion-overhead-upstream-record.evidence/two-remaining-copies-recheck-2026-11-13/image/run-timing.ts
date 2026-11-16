// Times the source arms baked into the recheck image on Deno, in rounds whose arm order rotates so no arm always
// runs first and every ratio comes from interleaved runs. One JSON line per run, and a last line saying it is done.
//
// Run inside the image: node /run-timing.ts --first-round 1 --rounds 5 [--steps small,sweep,...]

import { spawnSync, } from 'node:child_process';

const ALL = ['main', 'base', 'multiple', 'buffer', 'both', 'views-always', 'concat', 'slice-push',] as const;

interface Step {
  readonly name: string;
  readonly script: string;
  readonly args: readonly string[];
  readonly arms: readonly string[];
}

// The larger sizes leave out the arms that would take minutes per parse there without telling anything new.
const STEPS: readonly Step[] = [
  { name: 'small', script: 'token-scaling.ts', args: ['--counts', '100,1000', '--runs', '9', '--warmup', '2',], arms: ALL, },
  { name: 'sweep', script: 'token-scaling.ts', args: ['--counts', '2000,4000,8000,16000,32000', '--runs', '2', '--warmup', '1',], arms: ALL, },
  // token-scaling.ts refuses a single count (its fit needs two), so the base arms repeat 32,000 before 64,000.
  { name: 'large-base', script: 'token-scaling.ts', args: ['--counts', '32000,64000', '--runs', '1', '--warmup', '0',], arms: ['base', 'buffer',], },
  // slice-push ran here in round 1 only: it took as long as the base and told nothing more.
  {
    name: 'large',
    script: 'token-scaling.ts',
    args: ['--counts', '64000,128000', '--runs', '1', '--warmup', '0',],
    arms: ['multiple', 'both', 'views-always', 'concat',],
  },
  { name: 'mixed', script: 'mixed-scaling.ts', args: ['--counts', '6000,12000,24000,48000', '--runs', '2', '--warmup', '1',], arms: ['base', 'multiple', 'buffer', 'both',], },
  { name: 'mixed-views-always', script: 'mixed-scaling.ts', args: ['--counts', '3000,6000,12000', '--runs', '2', '--warmup', '1',], arms: ['base', 'views-always',], },
  { name: 'commit-paths', script: 'commit-paths.ts', args: ['--paths', '1000', '--runs', '7', '--warmup', '2',], arms: ALL, },
  { name: 'commit-paths-separator', script: 'commit-paths.ts', args: ['--paths', '1000', '--separator', '--runs', '7', '--warmup', '2',], arms: ALL, },
  {
    name: 'commit-paths-long',
    script: 'commit-paths.ts',
    args: ['--paths', '20000', '--runs', '2', '--warmup', '1',],
    arms: ['base', 'multiple', 'buffer', 'both', 'concat', 'slice-push',],
  },
  {
    name: 'commit-paths-long-separator',
    script: 'commit-paths.ts',
    args: ['--paths', '20000', '--separator', '--runs', '2', '--warmup', '1',],
    arms: ['base', 'multiple', 'buffer', 'both', 'concat', 'slice-push',],
  },
];

function option(name: string, fallback: string,): string {
  const index = process.argv.indexOf(name,);

  return index === -1 ? fallback : process.argv[index + 1] ?? fallback;
}

const firstRound = Number(option('--first-round', '1',),);
const rounds = Number(option('--rounds', '1',),);
const wanted = option('--steps', STEPS.map((step,) => step.name).join(',',),).split(',',);
const versions = spawnSync('sh', ['-c', 'node --version; deno --version | head --lines=1; bun --version; nproc',], { encoding: 'utf8', },);

console.log(JSON.stringify({ runtimes: versions.stdout.trim().split('\n',), firstRound, rounds, steps: wanted, },),);

for (let round = firstRound; round < firstRound + rounds; round += 1) {
  for (const step of STEPS) {
    if (!wanted.includes(step.name,)) continue;

    const offset = (round - 1) % step.arms.length;
    const order = [...step.arms.slice(offset,), ...step.arms.slice(0, offset,),];

    for (const arm of order) {
      const startedAt = Date.now();
      const child = spawnSync('deno', ['run', '--no-config', '--import-map', `/maps/recheck-${arm}.json`, step.script, ...step.args,], {
        cwd: '/repo',
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
        round,
        step: step.name,
        arm,
        script: step.script,
        args: step.args,
        exit: child.status,
        signal: child.signal,
        wallMs: Date.now() - startedAt,
        result,
        stderr: child.stderr.slice(0, 400,),
      },),);
    }
  }
}

console.log(JSON.stringify({ done: true, },),);
