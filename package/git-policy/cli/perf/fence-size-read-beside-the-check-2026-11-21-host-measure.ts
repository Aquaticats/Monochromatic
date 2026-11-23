// The host-side measurement of rung 2 (the clone rung) on this host's ZFS home dataset.
//
// What:  builds one fixture source of this repository's tracked tree, then drives one `git worktree add`
//        per arm per round through a shim of that arm's binary, reading the record line the add appends.
//        The destination sits on the same dataset as the source, so the clone rung applies and the fence's
//        size read runs beside the copies.
// Why:   rung 2's read is already beside the copies, so this rung says whether starting it beside the
//        source check too changes anything, and its step records say where the time went.
// Usage: node host-measure.ts fixture
//        node host-measure.ts adds <layout> <rounds>
//        node host-measure.ts gc
import { spawnSync, } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';
import { join, } from 'node:path';

const SCRATCH = '/home/user/temp/agent/fence-size-read-beside-source-check-2026-11-21/scratch';
const ROOT = join(SCRATCH, 'host-zfs',);
const SOURCE = join(ROOT, 'src',);
const DESTS = join(ROOT, 'dests',);
const HOMES = join(ROOT, 'homes',);
const TAR = join(SCRATCH, 'tree-40783c23ea.tar',);
const RECORD = join(SOURCE, '.git', 'monochromatic-git-policy-cli-worktree-add-timing.jsonl',);
const WORKTREE = '/home/user/Monochromatic/.claude/worktrees/fence-size-read-beside-source-check';
const BINARIES: Record<string, string> = {
  main: join(SCRATCH, 'target-main', 'release', 'monochromatic-git-policy-cli',),
  branch: join(SCRATCH, 'bin', 'branch-measurement',),
};
/** The variable one binary reads to start the size read beside the source check (measurement builds only). */
const SWITCH = 'MONOCHROMATIC_CLONE_SIZE_READ_BESIDE';
type Arm = { name: string; binary: string; overlap: boolean };
const ARMS: Arm[] = [
  { name: 'main', binary: 'main', overlap: false, },
  { name: 'branch-serial', binary: 'branch', overlap: false, },
  { name: 'branch-overlap', binary: 'branch', overlap: true, },
  { name: 'branch-serial-again', binary: 'branch', overlap: false, },
];
const PLAIN_HOME = join(HOMES, 'plain',);

/** Run one command, failing loudly unless asked to tolerate a failure. */
function run(command: string, args: string[], options: { cwd?: string; env?: NodeJS.ProcessEnv } = {},): string {
  const spawned = spawnSync(command, args, { cwd: options.cwd, env: options.env, maxBuffer: 1 << 28, },);
  if (spawned.status !== 0)
    throw new Error(`${command} ${args.join(' ',)} in ${options.cwd ?? process.cwd()} failed: ${spawned.stderr?.toString().slice(-800,)}`);
  return (spawned.stdout ?? Buffer.alloc(0,)).toString();
}

/** The environment every fixture-building Git run takes: its own home and no system configuration. */
function fixtureEnvironment(): NodeJS.ProcessEnv {
  return { ...process.env, HOME: PLAIN_HOME, XDG_RUNTIME_DIR: '/run/user/1000', GIT_CONFIG_NOSYSTEM: '1', };
}

function fixture(): void {
  mkdirSync(PLAIN_HOME, { recursive: true, },);
  unbuild();
  mkdirSync(DESTS, { recursive: true, },);
  run('tar', ['--extract', '--file', TAR, '--directory', SOURCE,],);
  const attributes = join(SOURCE, '.gitattributes',);
  const kept = readFileSync(attributes, 'utf8',).split('\n',).filter((line,) => !line.includes('filter=lfs',),).join('\n',);
  writeFileSync(attributes, kept,);
  // No policy runs on the fixture's own commits, so a measurement never pays a scanner over this tree.
  writeFileSync(join(SOURCE, 'monochromatic-git-policy-cli.config.jsonc',), '{\n  "policies": {}\n}\n',);
  const env = fixtureEnvironment();
  run('git', ['init', '--quiet', '--initial-branch=main',], { cwd: SOURCE, env, },);
  run('git', ['config', 'user.name', 'Fixture',], { cwd: SOURCE, env, },);
  run('git', ['config', 'user.email', 'fixture@example.invalid',], { cwd: SOURCE, env, },);
  run('git', ['config', 'commit.gpgsign', 'false',], { cwd: SOURCE, env, },);
  run('git', ['add', '--all', '--no-enforce-bulk-add',], { cwd: SOURCE, env, },);
  const tree = run('git', ['write-tree',], { cwd: SOURCE, env, },).trim();
  const commit = run('git', ['commit-tree', tree, '-m', 'fixture',], { cwd: SOURCE, env, },).trim();
  run('git', ['update-ref', 'refs/heads/main', commit,], { cwd: SOURCE, env, },);
  run('git', ['update-index', '--really-refresh',], { cwd: SOURCE, env, },);
  const status = run('git', ['status', '--porcelain',], { cwd: SOURCE, env, },).trim().split('\n',).filter((line,) => line !== '',);
  const tracked = run('git', ['ls-files',], { cwd: SOURCE, env, },).trim().split('\n',).length;
  console.log(JSON.stringify({ phase: 'fixture', tracked, statusLines: status.length, commit, },),);
  console.log(run('git', ['count-objects', '-v',], { cwd: SOURCE, env, },).trim(),);
}

/** Remove the fixture source and the destinations left from an earlier run. */
function unbuild(): void {
  rmSync(SOURCE, { recursive: true, force: true, },);
  rmSync(DESTS, { recursive: true, force: true, },);
  mkdirSync(SOURCE, { recursive: true, },);
}

function gc(): void {
  const env = fixtureEnvironment();
  run('git', ['gc', '--quiet',], { cwd: SOURCE, env, },);
  run('git', ['update-index', '--really-refresh',], { cwd: SOURCE, env, },);
  const status = run('git', ['status', '--porcelain',], { cwd: SOURCE, env, },).trim().split('\n',).filter((line,) => line !== '',);
  console.log(JSON.stringify({ phase: 'gc', statusLines: status.length, },),);
  console.log(run('git', ['count-objects', '-v',], { cwd: SOURCE, env, },).trim(),);
}

/** One arm's shim, home and environment for one add. */
function armEnvironment(arm: Arm, tag: string,): NodeJS.ProcessEnv {
  const shim = join(HOMES, `shim-${arm.name}`,);
  mkdirSync(shim, { recursive: true, },);
  const binary = BINARIES[arm.binary]!;
  rmSync(join(shim, 'git',), { force: true, },);
  cpSync(binary, join(shim, 'git',),);
  const home = join(HOMES, `${arm.name}-${tag}`,);
  mkdirSync(home, { recursive: true, },);
  const runtime = join(HOMES, `runtime-${arm.name}-${tag}`,);
  mkdirSync(runtime, { recursive: true, mode: 0o700, },);
  const env: NodeJS.ProcessEnv = { PATH: `${shim}:/usr/bin:/bin:/usr/sbin:/sbin`, HOME: home, XDG_RUNTIME_DIR: runtime, GIT_CONFIG_NOSYSTEM: '1', };
  if (arm.overlap) env[SWITCH] = '1';
  return env;
}

type Row = {
  layout: string;
  arm: string;
  overlap: boolean;
  tag: string;
  counted: boolean;
  round?: number;
  position?: number;
  wallMs: number;
  totalMs: number | null;
  route: string | null;
  exitCode: unknown;
  failed: boolean;
  stderr: string;
  steps: Record<string, number>;
  work: Record<string, number>;
};

function lastRecord(): { totalMs?: number; steps?: Record<string, { ms: number }>; work?: Record<string, { ms: number }>; notes?: Record<string, string>; exitCode?: unknown } {
  const lines = readFileSync(RECORD, 'utf8',).trim().split('\n',);
  return JSON.parse(lines.at(-1)!,);
}

function oneAdd(arm: Arm, tag: string, layout: string, counted: boolean,): Row {
  const env = armEnvironment(arm, tag,);
  const destination = join(DESTS, `${arm.name}-${tag}`,);
  const branch = `wt-${arm.name}-${tag}`;
  const began = process.hrtime.bigint();
  const added = spawnSync('git', ['worktree', 'add', '--quiet', '-b', branch, destination,], { cwd: SOURCE, env, maxBuffer: 1 << 26, timeout: 300_000, },);
  const wallMs = Number(process.hrtime.bigint() - began,) / 1e6;
  const record = lastRecord();
  const failed = added.status !== 0;
  const steps: Record<string, number> = {};
  for (const [step, value,] of Object.entries(record.steps ?? {},)) steps[step] = value.ms;
  const work: Record<string, number> = {};
  for (const [step, value,] of Object.entries(record.work ?? {},)) work[step] = value.ms;
  const row: Row = {
    layout,
    arm: arm.name,
    overlap: arm.overlap,
    tag,
    counted,
    wallMs: Math.round(wallMs * 10,) / 10,
    totalMs: record.totalMs ?? null,
    route: record.notes?.['route'] ?? null,
    exitCode: record.exitCode ?? null,
    failed,
    stderr: failed ? (added.stderr?.toString() ?? '').slice(-300,) : '',
    steps,
    work,
  };
  spawnSync('git', ['worktree', 'remove', '--force', destination,], { cwd: SOURCE, env, },);
  spawnSync('git', ['worktree', 'prune',], { cwd: SOURCE, env, },);
  spawnSync('git', ['branch', '-D', branch,], { cwd: SOURCE, env, },);
  rmSync(destination, { recursive: true, force: true, },);
  rmSync(env['HOME']!, { recursive: true, force: true, },);
  rmSync(env['XDG_RUNTIME_DIR']!, { recursive: true, force: true, },);
  return row;
}

function median(values: number[],): number {
  const ordered = [...values,].sort((a, b,) => a - b,);
  return ordered[Math.floor(ordered.length / 2)]!;
}

function adds(layout: string, rounds: number,): void {
  const rows: Row[] = [];
  // One uncounted warm-up add per arm, so no arm's first run pays a cold binary or a cold page.
  for (const arm of ARMS) rows.push(oneAdd(arm, `warm-${arm.name}`, layout, false,),);
  for (let round = 0; round < rounds; round += 1) {
    const shift = round % ARMS.length;
    const base = Math.floor(round / ARMS.length) % 2 === 0 ? [...ARMS,] : [...ARMS,].reverse();
    const order = [...base.slice(shift,), ...base.slice(0, shift,),];
    for (const [at, arm,] of order.entries()) {
      const row = oneAdd(arm, `r${round + 1}p${at + 1}`, layout, true,);
      row.round = round + 1;
      row.position = at + 1;
      rows.push(row,);
    }
  }
  writeFileSync(join(SCRATCH, `${layout}-host-adds.jsonl`,), `${rows.map((row,) => JSON.stringify(row,),).join('\n',)}\n`,);
  const counted = rows.filter((row,) => row.counted,);
  const summary: Record<string, unknown> = {};
  for (const arm of ARMS) {
    const mine = counted.filter((row,) => row.arm === arm.name && !row.failed,);
    const steps: Record<string, number[]> = {};
    const work: Record<string, number[]> = {};
    for (const row of mine) {
      for (const [step, ms,] of Object.entries(row.steps,)) (steps[step] ??= []).push(ms,);
      for (const [step, ms,] of Object.entries(row.work,)) (work[step] ??= []).push(ms,);
    }
    const stepMedians: Record<string, number> = {};
    for (const [step, values,] of Object.entries(steps,)) stepMedians[step] = Math.round(median(values,) * 10,) / 10;
    const workMedians: Record<string, number> = {};
    for (const [step, values,] of Object.entries(work,)) workMedians[step] = Math.round(median(values,) * 10,) / 10;
    summary[arm.name] = {
      n: mine.length,
      failed: counted.filter((row,) => row.arm === arm.name && row.failed).length,
      routes: [...new Set(mine.map((row,) => row.route),),],
      medianTotalMs: mine.length > 0 ? median(mine.map((row,) => row.totalMs ?? Number.NaN),) : null,
      totals: mine.map((row,) => row.totalMs,),
      walls: mine.map((row,) => row.wallMs,),
      orders: mine.map((row,) => [row.round ?? null, row.position ?? null,],),
      stepMedians,
      workMedians,
    };
  }
  console.log(JSON.stringify({ phase: 'adds', layout, summary, }, null, 1,),);
}

/**
 * What:
 *  The same adds on a source whose check refuses it:
 *  one staged path, which no rung serves on ZFS, so the add falls to Git's own checkout.
 * Why:
 *  The overlap starts its read before the check, so a refused source pays a read the fence never takes;
 *  this phase prices that, and every arm's record shows whether the read ran at all.
 *
 * In TS you'd write (pseudocode):
 * ```ts
 * stage('refusal-marker.txt'); for (const arm of ARMS) once(arm); unstage('refusal-marker.txt');
 * ```
 */
function refusedAdds(rounds: number,): void {
  const env = fixtureEnvironment();
  const marker = 'refusal-marker.txt';
  writeFileSync(join(SOURCE, marker,), 'staged, so the index of the source is not its HEAD tree\n',);
  run('git', ['add', '--', marker,], { cwd: SOURCE, env, },);
  const rows: Row[] = [];
  for (let round = 0; round < rounds; round += 1) {
    const shift = round % ARMS.length;
    const order = [...ARMS.slice(shift,), ...ARMS.slice(0, shift,),];
    for (const [at, arm,] of order.entries()) {
      const row = oneAdd(arm, `refused-r${round + 1}p${at + 1}`, 'refused', true,);
      row.round = round + 1;
      row.position = at + 1;
      rows.push(row,);
    }
  }
  run('git', ['rm', '--cached', '--quiet', '--', marker,], { cwd: SOURCE, env, },);
  rmSync(join(SOURCE, marker,), { force: true, },);
  writeFileSync(join(SCRATCH, 'refused-host-adds.jsonl',), `${rows.map((row,) => JSON.stringify(row,),).join('\n',)}\n`,);
  const summary: Record<string, unknown> = {};
  for (const arm of ARMS) {
    const mine = rows.filter((row,) => row.arm === arm.name && !row.failed,);
    const read = mine.map((row,) => (row.steps['git ls-tree'] ?? 0) + (row.work['fenceSizesBesideTheCheck'] ?? 0),);
    summary[arm.name] = {
      n: mine.length,
      routes: [...new Set(mine.map((row,) => row.route,),),],
      totals: mine.map((row,) => row.totalMs,),
      medianTotalMs: mine.length > 0 ? median(mine.map((row,) => row.totalMs ?? Number.NaN),) : null,
      readMs: read,
      checkSourceMs: mine.map((row,) => row.steps['checkSource'],),
    };
  }
  console.log(JSON.stringify({ phase: 'refused', summary, }, null, 1,),);
}

const phase = process.argv[2];
if (phase === 'fixture') fixture();
else if (phase === 'gc') gc();
else if (phase === 'adds') adds(process.argv[3]!, Number(process.argv[4] ?? '8',),);
else if (phase === 'refused') refusedAdds(Number(process.argv[3] ?? '2',),);
else if (phase === 'probe') {
  // One add per arm, printed whole: the shape check before the counted rounds.
  for (const arm of ARMS) {
    if (!existsSync(BINARIES[arm.binary]!)) {
      console.log(`${arm.name}: binary not built yet`,);
      continue;
    }
    console.log(JSON.stringify(oneAdd(arm, `probe-${arm.name}`, 'probe', false,),),);
  }
} else throw new Error(`unknown phase ${phase}`);
