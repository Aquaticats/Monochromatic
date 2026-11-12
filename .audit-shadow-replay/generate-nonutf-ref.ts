import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve('.audit-shadow-replay/nonutf-ref');
const repo = resolve(root, 'repo');
const environment = { PATH: '/usr/bin:/bin', HOME: root, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_AUTHOR_NAME: 'Audit', GIT_AUTHOR_EMAIL: 'audit@example.invalid', GIT_COMMITTER_NAME: 'Audit', GIT_COMMITTER_EMAIL: 'audit@example.invalid' };
function git(args: string[], input?: Buffer): Buffer {
  const result = spawnSync('git', args, { cwd: repo, env: environment, input, timeout: 15000, killSignal: 'SIGKILL' });
  if (result.error || result.status !== 0) throw new Error(`${args}: ${result.error ?? result.stderr.toString()}`);
  return result.stdout;
}
mkdirSync(repo, { recursive: true });
git(['init', '--initial-branch=main']);
git(['commit', '--allow-empty', '-m', 'base']);
const base = git(['rev-parse', 'HEAD']).toString().trim();
git(['update-ref', '--stdin'], Buffer.concat([Buffer.from('create refs/tags/nonutf-'), Buffer.from([255]), Buffer.from(` ${base}\n`)]));
git(['commit', '--allow-empty', '-m', 'native control beside non-UTF-8 tag']);
console.log('native commit beside non-UTF-8 tag: exit=0');
console.log(`reference listing bytes: ${git(['for-each-ref', '--format=%(refname)']).toString('hex')}`);
writeFileSync(resolve(root, '..', 'nonutf-ref.json'), JSON.stringify({ repo, root }));
