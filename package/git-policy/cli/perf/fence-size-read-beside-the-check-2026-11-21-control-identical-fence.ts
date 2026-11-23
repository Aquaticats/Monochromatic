// A control on the overlap's answer: two adds on one small source, one with the size read started
// beside the check and one without, must make the same fence decision and write the same worktree.
//
// What:  builds a small fixture whose `.gitattributes` gives a text rule, so the size proof decides
//        paths, then adds it twice with the same binary, the switch set and unset, and prints each
//        add's fence line plus the two destinations' status and a byte comparison against the source.
// Why:   the overlap changes where the sizes are read, not which they are; the fault it could
//        introduce is a map of another commit, and a decision that differs between the two arms is
//        what that would look like.
// Usage: node control-identical-fence.ts
import { spawnSync, } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync, } from 'node:fs';
import { join, } from 'node:path';

const SCRATCH = '/home/user/temp/agent/fence-size-read-beside-source-check-2026-11-21/scratch';
const ROOT = join(SCRATCH, 'control',);
const SOURCE = join(ROOT, 'src',);
const WORKTREE = '/home/user/Monochromatic/.claude/worktrees/fence-size-read-beside-source-check';
const BRANCH = join(SCRATCH, 'bin', 'branch-measurement',);
const SWITCH = 'MONOCHROMATIC_CLONE_SIZE_READ_BESIDE';
const HOME = join(ROOT, 'home',);
const RUNTIME = join(ROOT, 'runtime',);

function run(command: string, args: string[], cwd: string, env: NodeJS.ProcessEnv = {},): { status: number | null; stdout: string; stderr: string } {
  const done = spawnSync(command, args, { cwd, env: { ...process.env, HOME, XDG_RUNTIME_DIR: RUNTIME, GIT_CONFIG_NOSYSTEM: '1', ...env, }, maxBuffer: 1 << 26, },);
  return { status: done.status, stdout: (done.stdout ?? Buffer.alloc(0,)).toString(), stderr: (done.stderr ?? Buffer.alloc(0,)).toString(), };
}

function fixture(): void {
  rmSync(ROOT, { recursive: true, force: true, },);
  mkdirSync(HOME, { recursive: true, },);
  mkdirSync(RUNTIME, { recursive: true, mode: 0o700, },);
  mkdirSync(SOURCE, { recursive: true, },);
  writeFileSync(join(SOURCE, '.gitattributes',), '*.txt text eol=lf\n*.bin -text\n',);
  for (let index = 0; index < 12; index += 1) {
    // A carriage return in some files, so the rule and the size proof both matter.
    writeFileSync(join(SOURCE, `file-${index}.txt`,), index % 3 === 0 ? `line\r\n${index}\r\n` : `line\n${index}\n`,);
    writeFileSync(join(SOURCE, `blob-${index}.bin`,), Buffer.from([index, 0, 255, index,],),);
  }
  writeFileSync(join(SOURCE, 'plain.txt',), 'no rule touches this one\n',);
  run('git', ['init', '--quiet', '--initial-branch=main',], SOURCE,);
  run('git', ['config', 'user.name', 'Fixture',], SOURCE,);
  run('git', ['config', 'user.email', 'fixture@example.invalid',], SOURCE,);
  const added = run('git', ['add', '--all', '--no-enforce-bulk-add',], SOURCE,);
  if (added.status !== 0) throw new Error(added.stderr);
  const tree = run('git', ['write-tree',], SOURCE,).stdout.trim();
  const commit = run('git', ['commit-tree', tree, '-m', 'fixture',], SOURCE,).stdout.trim();
  run('git', ['update-ref', 'refs/heads/main', commit,], SOURCE,);
  // A fresh index's stat fields are racy for a moment, and the check refuses the source until they settle.
  spawnSync('sleep', ['8',],);
  run('git', ['update-index', '--really-refresh',], SOURCE,);
  const status = run('git', ['status', '--porcelain',], SOURCE,).stdout.trim();
  if (status !== '') console.log(`fixture stays dirty: ${status}`,);
}

/** Every regular file under `dir`, as path and bytes, for a byte-for-byte comparison. */
function tree(dir: string, prefix = '',): [string, string][] {
  const found: [string, string][] = [];
  for (const name of readdirSync(dir,)) {
    if (name === '.git') continue;
    const full = join(dir, name,);
    const relative = prefix === '' ? name : `${prefix}/${name}`;
    if (statSync(full,).isDirectory()) found.push(...tree(full, relative,),);
    else found.push([relative, readFileSync(full,).toString('base64',),],);
  }
  return found.sort((left, right,) => left[0].localeCompare(right[0],),);
}

function addWith(overlap: boolean,): { fence: string; status: string; destination: string; route: string | null; verbose: string[] } {
  const shim = join(ROOT, overlap ? 'shim-overlap' : 'shim-serial',);
  rmSync(shim, { recursive: true, force: true, },);
  mkdirSync(shim, { recursive: true, },);
  writeFileSync(join(shim, 'git',), readFileSync(BRANCH,), { mode: 0o755, },);
  const destination = join(ROOT, overlap ? 'dest-overlap' : 'dest-serial',);
  rmSync(destination, { recursive: true, force: true, },);
  const env: NodeJS.ProcessEnv = { PATH: `${shim}:/usr/bin:/bin:/usr/sbin:/sbin`, MONOCHROMATIC_VERBOSE: 'true', };
  if (overlap) env[SWITCH] = '1';
  const added = run('git', ['worktree', 'add', '--quiet', '-b', `wt-${overlap ? 'overlap' : 'serial'}`, destination,], SOURCE, env,);
  if (added.status !== 0) throw new Error(added.stderr.slice(-800,),);
  const fence = added.stderr.split('\n',).find((line,) => line.includes('the fence carries'),) ?? 'no fence line';
  const status = run('git', ['status', '--porcelain',], destination,).stdout;
  const record = JSON.parse(readFileSync(join(SOURCE, '.git', 'monochromatic-git-policy-cli-worktree-add-timing.jsonl',), 'utf8',).trim().split('\n',).at(-1,),);
  return { fence, status, destination, route: record.notes?.route ?? null, carried: record.steps ?? {}, verbose: added.stderr.split('\n',).filter((line,) => line.includes('fence'),).slice(0, 3,), };
}

fixture();
const serial = addWith(false,);
const overlap = addWith(true,);
const serialTree = tree(serial.destination,);
const overlapTree = tree(overlap.destination,);
const fenceOf = (line: string,): string => line.replace(/^.*?(the fence carries)/, '$1',).trim();
console.log(JSON.stringify({
  serial: { fence: fenceOf(serial.fence,), status: serial.status.trim(), route: serial.route, verbose: serial.verbose, },
  overlap: { fence: fenceOf(overlap.fence,), status: overlap.status.trim(), route: overlap.route, verbose: overlap.verbose, },
  sameTrees: JSON.stringify(serialTree,) === JSON.stringify(overlapTree,),
  files: serialTree.length,
  sameDecision: fenceOf(serial.fence,) === fenceOf(overlap.fence,),
}, null, 1,),);
