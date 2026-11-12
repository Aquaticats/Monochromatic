import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve('.audit-shadow-replay');
const environment = { PATH: '/usr/bin:/bin', HOME: root, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_AUTHOR_NAME: 'Audit', GIT_AUTHOR_EMAIL: 'audit@example.invalid', GIT_COMMITTER_NAME: 'Audit', GIT_COMMITTER_EMAIL: 'audit@example.invalid', GIT_AUTHOR_DATE: '2001-01-01T00:00:00Z', GIT_COMMITTER_DATE: '2001-01-01T00:00:00Z' };
function git(repo: string, args: string[], input?: string | Buffer): Buffer {
  const result = spawnSync('git', args, { cwd: repo, env: environment, input, timeout: 15000, killSignal: 'SIGKILL', maxBuffer: 1024 * 1024 });
  if (result.error || result.status !== 0) throw new Error(`${args.join(' ')}: ${result.error ?? result.stderr.toString()}`);
  return result.stdout;
}
function tree(repo: string, entries: Record<string, string | { text: string, mode: string }>): string {
  const records = Object.entries(entries).map(([name, value]) => {
    const text = typeof value === 'string' ? value : value.text;
    const mode = typeof value === 'string' ? '100644' : value.mode;
    const oid = git(repo, ['hash-object', '-w', '--stdin'], text).toString().trim();
    return `${mode} blob ${oid}\t${name}\0`;
  });
  return git(repo, ['mktree', '-z'], records.join('')).toString().trim();
}
function commit(repo: string, entries: Record<string, string | { text: string, mode: string }>, base?: string): string {
  return git(repo, ['commit-tree', tree(repo, entries), ...(base ? ['-p', base] : []), '-m', 'audit']).toString().trim();
}
function scenario(name: string, baseEntries: Record<string, string | { text: string, mode: string }>, landedEntries: Record<string, string | { text: string, mode: string }>, preparedEntries: Record<string, string | { text: string, mode: string }>, expectedPath: string, expected: string, config: string[][] = [], cwd = '') {
  const repo = resolve(root, name);
  mkdirSync(repo, { recursive: true });
  git(repo, ['init', '--initial-branch=main']);
  for (const pair of config) git(repo, ['config', ...pair]);
  const base = commit(repo, baseEntries);
  const landed = commit(repo, landedEntries, base);
  const prepared = commit(repo, preparedEntries, base);
  git(repo, ['update-ref', 'refs/heads/main', landed]);
  git(repo, ['update-ref', 'refs/heads/prepared', prepared]);
  if (cwd) mkdirSync(resolve(repo, cwd), { recursive: true });
  const manifest = { repo, base, landed, prepared, expectedPath, expected, cwd: cwd ? resolve(repo, cwd) : repo };
  writeFileSync(resolve(root, `${name}.json`), JSON.stringify(manifest));
  console.log(`${name}: ${JSON.stringify(manifest)}`);
}
mkdirSync(root, { recursive: true, mode: 0o700 });
console.log(git(root, ['--version']).toString().trim());
const base = 'a\nb\nc\nd\n';
const landed = 'a\nB\nc\nd\n';
const prepared = 'a\nB\nC\nd\n';
scenario('path-collision', { 'é.txt': base, 'Ã©.txt': 'unrelated base\n' }, { 'é.txt': landed, 'Ã©.txt': landed }, { 'é.txt': prepared, 'Ã©.txt': 'unrelated base\n' }, 'Ã©.txt', landed);
scenario('ascii-control', { 'a.txt': base, 'b.txt': 'unrelated base\n' }, { 'a.txt': landed, 'b.txt': landed }, { 'a.txt': prepared, 'b.txt': 'unrelated base\n' }, 'b.txt', landed);
scenario('relative-config', { 'f.txt': base }, { 'f.txt': landed }, { 'f.txt': 'a\nMINE\nc\nd\n' }, 'f.txt', 'CONFLICT', [['diff.relative', 'true']], 'subdir');
scenario('crlf-control', { 'f.txt': base.replaceAll('\n', '\r\n') }, { 'f.txt': landed.replaceAll('\n', '\r\n') }, { 'f.txt': prepared.replaceAll('\n', '\r\n') }, 'f.txt', prepared.replaceAll('\n', '\r\n'));
scenario('binary-control', { 'f.txt': 'a\0base' }, { 'f.txt': 'a\0landed' }, { 'f.txt': 'a\0prepared' }, 'f.txt', 'CONFLICT');
scenario('mode-control', { 'f.txt': base }, { 'f.txt': {text: landed, mode:'100755'} }, { 'f.txt': prepared }, 'f.txt', prepared);
scenario('link-control', { 'f.txt': {text: 'base', mode:'120000'} }, { 'f.txt': {text: 'landed', mode:'120000'} }, { 'f.txt': {text:'prepared', mode:'120000'} }, 'f.txt', 'CONFLICT');
