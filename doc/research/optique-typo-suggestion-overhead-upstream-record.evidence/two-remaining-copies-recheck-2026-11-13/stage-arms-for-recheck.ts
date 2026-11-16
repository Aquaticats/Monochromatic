// Stages the source arms of @optique/core for the recheck of the prototype that removes the two per-token array
// copies left after the input-trace patch. Every arm is made here from upstream's commit and the diff files, in a
// throwaway worktree of this session's own disposable clone; nothing is taken from the prototype agent's build trees.
//
// Arms, each a copy of packages/core/src:
// - main: upstream main at the merge of pull request 1121;
// - base: main with the input-trace patch this repository holds;
// - multiple, buffer, both: base with the agent's diffs 01+02, 01+03, 01+02+03;
// - views-always: both, with the two thresholds set to zero, so every state and buffer is a view (a diagnostic arm);
// - concat, slice-push: base with a plain-array copy in multiple() that avoids the spread (this session's own idea).
//
// Also records: whether each diff applies, the tree of 01+02+03 against the tree of 04, whether the diffs remove any
// line from an existing test file, and how the agent's working tree differs from the tree staged here.
//
// Run: node stage-arms-for-recheck.ts > keep/stage-arms-for-recheck.jsonl

import { spawnSync, } from 'node:child_process';
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';

const HOME = process.env['HOME'];

if (HOME === undefined) throw new Error('HOME is not set',);

const CLONE = `${HOME}/temp/agent/upstream-prototype.O1iPjcqr/optique`;
const AGENT = `${HOME}/temp/agent/upstream-prototype.3lsGY9v7`;
const HERE = `${HOME}/temp/agent/optique-two-remaining-copies-recheck-2026-11-13`;
const ARMS = `${HERE}/arms`;
const MAIN = 'd26f1a244d4434f9dbfeb0317f6db32533cb8c32';
const TRACE = '/var/mnt/encrypted/repository/Monochromatic/doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.patch';
const DIFFS = `${AGENT}/keep/diffs`;
const SOURCE = 'packages/core/src';

function run(command: string, args: string[], cwd: string,): { status: number | null; stdout: string; stderr: string; } {
  const child = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);

  return { status: child.status, stdout: child.stdout, stderr: child.stderr, };
}

function must(args: string[], cwd = CLONE,): string {
  const result = run('git', args, cwd,);

  if (result.status !== 0) throw new Error(`git ${args.join(' ',)} failed: ${result.stderr.slice(0, 600,)}`,);

  return result.stdout;
}

function note(record: Record<string, unknown>,): void {
  console.log(JSON.stringify(record,),);
}

rmSync(ARMS, { recursive: true, force: true, },);
mkdirSync(ARMS, { recursive: true, },);

// region Upstream's main today

const fetched = run('git', ['fetch', '--no-tags', 'origin', '+refs/heads/main:refs/remotes/origin/main',], CLONE,);
const mainNow = must(['rev-parse', 'refs/remotes/origin/main',],).trim();

note({
  step: 'fetch-main',
  exit: fetched.status,
  stderr: fetched.stderr.slice(0, 300,),
  mainNow,
  isTheMergeOf1121: mainNow === MAIN,
  mergeTree: must(['rev-parse', `${MAIN}^{tree}`,],).trim(),
  commitsOnMainSinceTheMerge: must(['log', '--format=%H %cI %s', `${MAIN}..${mainNow}`,],).trim().split('\n',).filter((line,) => line.length > 0),
},);

// endregion Upstream's main today

// region Arms made by applying diffs

const applied = {
  main: [],
  base: [TRACE,],
  multiple: [TRACE, `${DIFFS}/01-array-view.patch`, `${DIFFS}/02-multiple-state.patch`,],
  buffer: [TRACE, `${DIFFS}/01-array-view.patch`, `${DIFFS}/03-input-buffer.patch`,],
  both: [TRACE, `${DIFFS}/01-array-view.patch`, `${DIFFS}/02-multiple-state.patch`, `${DIFFS}/03-input-buffer.patch`,],
  'both-from-04': [TRACE, `${DIFFS}/04-both-combined.patch`,],
} as const;
const trees: Record<string, string> = {};

for (const [arm, patches,] of Object.entries(applied,)) {
  const worktree = `${HERE}/worktrees/${arm}`;

  must(['worktree', 'add', '--detach', worktree, MAIN,],);

  const results = patches.map((patch,) => {
    const result = run('git', ['apply', '--index', patch,], worktree,);

    return { patch: patch.split('/',).at(-1,), applies: result.status === 0, stderr: result.stderr.slice(0, 300,), };
  },);

  trees[arm] = must(['write-tree',], worktree,).trim();
  cpSync(`${worktree}/${SOURCE}`, `${ARMS}/${arm}/${SOURCE}`, { recursive: true, },);

  if (arm === 'both') {
    // The agent's clone, as it left it, against the tree made here from its diff files.
    for (const folder of [SOURCE, 'changes.d',]) {
      const difference = run('diff', ['--recursive', '--brief', `${AGENT}/optique/${folder}`, `${worktree}/${folder}`,], HERE,);

      note({ step: 'agent-working-tree-against-staged-both', folder, exit: difference.status, differences: difference.stdout.trim().split('\n',).filter((line,) => line.length > 0), },);
    }
  }

  must(['worktree', 'remove', '--force', worktree,],);
  note({ step: 'arm', arm, tree: trees[arm], results, },);
}

note({ step: 'combined-diff-equals-the-three', same: trees['both'] === trees['both-from-04'], },);
rmSync(`${ARMS}/both-from-04`, { recursive: true, force: true, },);

// endregion Arms made by applying diffs

// region Lines the diffs remove from existing test files

for (const name of ['01-array-view.patch', '02-multiple-state.patch', '03-input-buffer.patch',]) {
  const sections = readFileSync(`${DIFFS}/${name}`, 'utf8',).split(/^diff --git /mu,).slice(1,);

  for (const section of sections) {
    const path = section.split('\n', 1,)[0]?.split(' b/',).at(-1,) ?? '';
    const isNew = section.includes('\nnew file mode',);
    const removed = section.split('\n',).filter((line,) => line.startsWith('-',) && !line.startsWith('---',)).length;
    const added = section.split('\n',).filter((line,) => line.startsWith('+',) && !line.startsWith('+++',)).length;

    note({ step: 'diff-section', diff: name, path, isNew, added, removed, },);
  }
}

// endregion Lines the diffs remove from existing test files

// region Arms made by editing a staged arm

function derive({ arm, from, edits, }: { arm: string; from: string; edits: { file: string; find: string; replace: string; times: number; }[]; },): void {
  cpSync(`${ARMS}/${from}`, `${ARMS}/${arm}`, { recursive: true, },);

  for (const edit of edits) {
    const path = `${ARMS}/${arm}/${SOURCE}/${edit.file}`;
    const text = readFileSync(path, 'utf8',);
    const found = text.split(edit.find,).length - 1;

    if (found !== edit.times) throw new Error(`${arm}: ${edit.file} holds the text to replace ${found} times, expected ${edit.times}`,);

    writeFileSync(path, text.replaceAll(edit.find, edit.replace,),);
  }

  note({ step: 'derived-arm', arm, from, edits: edits.map((edit,) => ({ file: edit.file, find: edit.find, replace: edit.replace, }),), },);
}

derive({
  arm: 'views-always',
  from: 'both',
  edits: [
    { file: 'internal/persistent-vector.ts', find: 'export const VIEW_THRESHOLD = 1024;', replace: 'export const VIEW_THRESHOLD = 0;', times: 1, },
    { file: 'internal/input-buffer-view.ts', find: 'export const VIEW_THRESHOLD = 16384;', replace: 'export const VIEW_THRESHOLD = 0;', times: 1, },
  ],
},);

const SPREAD = [
  '        state: annotateFreshArray(context.state, [',
  '          ...(added ? context.state : context.state.slice(0, -1)),',
  '          nextItemState,',
  '        ]),',
].join('\n',);

derive({
  arm: 'concat',
  from: 'base',
  edits: [{
    file: 'modifiers.ts',
    find: SPREAD,
    replace: [
      '        state: annotateFreshArray(',
      '          context.state,',
      '          (added ? context.state : context.state.slice(0, -1)).concat([',
      '            nextItemState,',
      '          ]),',
      '        ),',
    ].join('\n',),
    times: 2,
  },],
},);
derive({
  arm: 'slice-push',
  from: 'base',
  edits: [{
    file: 'modifiers.ts',
    find: SPREAD,
    replace: [
      '        state: annotateFreshArray(',
      '          context.state,',
      '          (() => {',
      '            const items = context.state.slice(',
      '              0,',
      '              added ? context.state.length : -1,',
      '            );',
      '            items.push(nextItemState);',
      '            return items;',
      '          })(),',
      '        ),',
    ].join('\n',),
    times: 2,
  },],
},);

// endregion Arms made by editing a staged arm
