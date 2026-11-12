import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve('.audit-shadow-replay/signing-config');
const repo = resolve(root, 'repo');
const environment = { PATH: '/usr/bin:/bin', HOME: root, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_AUTHOR_NAME: 'Audit', GIT_AUTHOR_EMAIL: 'audit@example.invalid', GIT_COMMITTER_NAME: 'Audit', GIT_COMMITTER_EMAIL: 'audit@example.invalid' };
function run(command: string, args: string[]): string {
  const result = spawnSync(command, args, { cwd: repo, env: environment, timeout: 15000, killSignal: 'SIGKILL', encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error(`${command} ${args}: ${result.error ?? result.stderr}`);
  return result.stdout.trim();
}
mkdirSync(repo, { recursive: true });
run('git', ['init', '--initial-branch=main']);
run('git', ['commit', '--allow-empty', '-m', 'base']);
const key = resolve(root, 'audit-key');
run('ssh-keygen', ['-t', 'ed25519', '-N', '', '-C', 'disposable-audit-key', '-f', key]);
const include = resolve(root, 'signing.config');
writeFileSync(include, `[commit]\n\tgpgSign = true\n[gpg]\n\tformat = ssh\n[user]\n\tsigningKey = ${JSON.stringify(key)}\n`);
run('git', ['config', `includeIf.gitdir:${repo}/.git.path`, include]);
run('git', ['commit', '--allow-empty', '-m', 'native signed control']);
const raw = run('git', ['cat-file', 'commit', 'HEAD']);
console.log(`real Git honors exact gitdir include: signed=${raw.includes('\ngpgsig ')}\n`);
writeFileSync(resolve(root, '..', 'signing-config.json'), JSON.stringify({ repo, root }));
