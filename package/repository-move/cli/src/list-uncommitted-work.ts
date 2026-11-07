// Lists, for every worktree the repository knows, what a move would strand: uncommitted changes, untracked files,
// an unfinished merge or rebase, and a held index lock. Read-only: `git --no-optional-locks` never writes.
// Usage: node list-uncommitted-work.ts <main worktree> <output file> [per-worktree timeout in seconds]
import { spawnSync, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync, } from 'node:fs';
import { join, } from 'node:path';

const [mainWorktree, outputFile, timeoutText,] = process.argv.slice(2,);
if (mainWorktree === undefined || outputFile === undefined) {
  throw new Error('usage: list-uncommitted-work.ts <main worktree> <output file> [timeout seconds]');
}
const perWorktreeMilliseconds: number = Number(timeoutText ?? '120',) * 1000;
const realGit: string = '/usr/bin/git';

function git(directory: string, args: readonly string[],) {
  return spawnSync(realGit, ['--no-optional-locks', '-C', directory, ...args,], {
    encoding: 'utf8',
    timeout: perWorktreeMilliseconds,
    maxBuffer: 256 * 1024 * 1024,
  },);
}

const listing = git(mainWorktree, ['worktree', 'list', '--porcelain',],);
if (listing.status !== 0) {
  throw new Error(`cannot list worktrees: ${listing.stderr}`);
}
const paths: string[] = listing.stdout
  .split('\n',)
  .filter((line,) => line.startsWith('worktree ',))
  .map((line,) => line.slice('worktree '.length,),);

const rows: string[] = ['path\tstate\tchanged\tuntracked\tunfinished\tindexLock\thead\tindexSha256',];
let dirty: number = 0;
for (const path of paths) {
  if (!existsSync(path,)) {
    rows.push(`${path}\tmissing\t\t\t\t\t\t`,);
    continue;
  }
  const status = git(path, ['status', '--porcelain=v1', '--untracked-files=all',],);
  if (status.status !== 0) {
    rows.push(`${path}\tstatus-failed:${status.error?.message ?? status.stderr.trim().slice(0, 80,)}\t\t\t\t\t\t`,);
    continue;
  }
  const lines: string[] = status.stdout.split('\n',).filter((line,) => line !== '',);
  const untracked: number = lines.filter((line,) => line.startsWith('?? ',),).length;
  const changed: number = lines.length - untracked;
  const gitDirectory: string = git(path, ['rev-parse', '--absolute-git-dir',],).stdout.trim();
  const unfinished: string = ['MERGE_HEAD', 'CHERRY_PICK_HEAD', 'REVERT_HEAD', 'rebase-merge', 'rebase-apply', 'BISECT_LOG',]
    .filter((name,) => gitDirectory !== '' && existsSync(join(gitDirectory, name,),),)
    .join(',',);
  const indexLock: boolean = gitDirectory !== '' && existsSync(join(gitDirectory, 'index.lock',),);
  const head: string = git(path, ['rev-parse', 'HEAD',],).stdout.trim();
  const indexPath: string = join(gitDirectory, 'index',);
  const indexSha256: string = gitDirectory !== '' && existsSync(indexPath,)
    ? createHash('sha256',).update(readFileSync(indexPath,),).digest('hex',)
    : '';
  if (changed > 0 || untracked > 0 || unfinished !== '' || indexLock) {
    dirty += 1;
  }
  rows.push(`${path}\tok\t${changed}\t${untracked}\t${unfinished}\t${indexLock}\t${head}\t${indexSha256}`,);
}
writeFileSync(outputFile, `${rows.join('\n',)}\n`,);
console.log(`${paths.length} worktrees listed, ${dirty} with something a move would strand; table in ${outputFile}`,);
