import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve('.audit-shadow-replay');
const repo = resolve(root, 'interhunk-config');
const environment = { PATH: '/usr/bin:/bin', HOME: root, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_AUTHOR_NAME: 'Audit', GIT_AUTHOR_EMAIL: 'audit@example.invalid', GIT_COMMITTER_NAME: 'Audit', GIT_COMMITTER_EMAIL: 'audit@example.invalid' };
function git(args: string[], input?: string): string {
  const result = spawnSync('git', args, { cwd: repo, env: environment, input, timeout: 15000, killSignal: 'SIGKILL', encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error(`${args}: ${result.error ?? result.stderr}`);
  return result.stdout.trim();
}
function commit(text: string, base?: string): string {
  const blob = git(['hash-object', '-w', '--stdin'], text);
  const tree = git(['mktree'], `100644 blob ${blob}\tf.txt\n`);
  return git(['commit-tree', tree, ...(base ? ['-p', base] : []), '-m', 'audit']);
}
mkdirSync(repo, { recursive: true });
git(['init', '--initial-branch=main']);
git(['config', 'diff.interHunkContext', '1']);
const base = commit('a\nb\nc\nd\ne\n');
const landed = commit('a\nB\nc\nD\ne\n', base);
const prepared = commit('a\nB\nD\nd\ne\n', base);
git(['update-ref', 'refs/heads/main', landed]);
git(['update-ref', 'refs/heads/prepared', prepared]);
console.log('Landed patch under the inherited setting:');
console.log(git(['diff-tree', '-p', '--unified=0', base, landed]));
writeFileSync(resolve(root, 'interhunk-config.json'), JSON.stringify({ repo, base, landed, prepared, cwd: repo, expectedPath: 'f.txt', expected: 'a\nB\nD\nD\ne\n' }));
