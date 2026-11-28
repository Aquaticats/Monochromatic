# Claude Code 2.1.332 runs its Bash tool in zsh when `SHELL` is fish, and `CLAUDE_CODE_SHELL` makes it run bash

## Symptom

Every Bash tool command runs under zsh,
though no file on this machine picks zsh.
`ps -o args= -p $$` inside a Bash tool call printed,
on 2026-11-28 (shortened at the `…`):

```text
/usr/bin/zsh -c source /home/user/.claude/shell-snapshots/snapshot-zsh-1795858775390-ql4r8g.sh 2>/dev/null || true && \builtin setopt NO_EXTENDED_GLOB NO_BARE_GLOB_QUAL NO_BRACE_CCL NO_MAGIC_EQUAL_SUBST NO_RC_QUOTES NO_EQUALS 2>/dev/null || true && … && eval '<the command>' < /dev/null && pwd -P >| /tmp/claude-3649-cwd
```

The command also saw `SHELL=/usr/bin/zsh`.
The account's login shell is fish (`getent passwd user` ends in `/bin/fish`),
and Claude Code is started from bash in OdyTTY.
`/proc/<pid>/environ` of the running `claude`,
of the `bash --rcfile …/odytty.bash` that started it,
and of OdyTTY each hold `SHELL=/bin/fish` (read 2026-11-28).
So Claude Code chose zsh itself,
and the zsh `SHELL` value the command saw is Claude Code's own;
where the bundle sets it was not traced.

On 2026-11-28 the user asked that Claude Code's Bash tool always run bash.

## Root cause

Claude Code ships as a native executable with its JavaScript bundled,
not as open source.
The trace below reads the installed bundle,
`~/.local/share/claude/versions/2.1.332`,
with `rg --text --byte-offset --only-matching`.
Names are minified,
and each excerpt is cited by its byte offset in that file.

### Detection drops fish and tries zsh before bash

Byte 337395171,
the list of shells to try:

```js
// claude 2.1.332 bundle
function a(e,o){let r=[s.CLAUDE_CODE_SHELL,s.SHELL].filter((l)=>l!==void 0&&(l.includes("bash")||l.includes("zsh"))),n=hLr(s.SHELL??"")==="bash",i=["/bin","/usr/bin","/usr/local/bin","/opt/homebrew/bin"],h=n?["bash","zsh"]:["zsh","bash"],[u,f]=n?[o,e]:[e,o];return[...r,u,...h.flatMap((l)=>i.map((c)=>`${c}/${l}`)),f].filter((l)=>l!==null)}
```

Byte 337395628,
inside the function that walks that list (`e` and `o` are the `PATH` lookups of `zsh` and `bash`):

```js
// claude 2.1.332 bundle
async function S(){let[e,o]=await Promise.all([bf("zsh"),bf("bash")]),r;for(let i of a(e,o))if(await b(i)){r=i;break}let n=s.CLAUDE_CODE_SHELL;if(n)t(r===n?`Using shell override: ${n}`:`CLAUDE_CODE_SHELL="${n}" is not a valid bash/zsh path, falling back to detection`);
```

The list starts with `CLAUDE_CODE_SHELL` and then `SHELL`,
each kept only when its text contains `bash` or `zsh`.
After them come the standard locations,
bash first only when the last part of `SHELL` is `bash`,
and zsh first otherwise.
The first entry that is executable wins.
With `SHELL=/bin/fish` and `CLAUDE_CODE_SHELL` unset,
fish is dropped and zsh is tried before bash,
so the Bash tool runs zsh whenever zsh is installed.
CachyOS installs zsh by default (`/usr/bin/zsh`,
with `/usr/share/cachyos-zsh-config/cachyos-config.zsh` sourced by `~/.zshrc`).

The current [environment variable reference][env-vars-shell] says the same:

> Set the shell Claude Code uses to run Bash tool commands.
> Accepts a path to a bash or zsh binary,
> for example /opt/homebrew/bin/bash.
> Other shells such as fish are not supported.
> If the value is not a working bash or zsh path,
> Claude Code ignores it and falls back to auto-detection.
> Auto-detection uses your $SHELL when it points to bash or zsh,
> otherwise it picks the first working zsh then bash found on your PATH and standard install locations

### The choice is made once per process

Byte 337396228:

```js
// claude 2.1.332 bundle
function gGi(e){return e.bashToolShellPath??=S().then((o)=>(e.resolvedBashToolShellPath=o,o),
```

The result is kept for the life of the process,
so a running session keeps its shell until Claude Code is started again.

### A settings file can deliver `CLAUDE_CODE_SHELL`

The settings loader drops the variables in the set `rla` from every settings file's `env` block
(byte 333843568,
`if(rla.has(E.toUpperCase())&&!Ct(E,g,n))continue`;
`doc/troubleshooting/claude-code-shell-c-script-rm-prompt.md` traces that loader).
`rla` (byte 332644847) is a literal list plus the spread of `YJt`
(byte 329187878,
`YJt=["ANTHROPIC_UNIX_SOCKET","CLAUDE_CODE_PROVIDER_MANAGED_BY_HOST","CLAUDE_CODE_HOST_AUTH_ENV_VAR"]`).
Neither holds `CLAUDE_CODE_SHELL`:
`rg --only-matching 'rla=new Set\(\[[^\]]*\]\)'` over the bundle's strings,
piped into `rg --fixed-strings CLAUDE_CODE_SHELL`,
found nothing.
The settings arm under "Verification" confirms it.

## Verification

Claude Code `2.1.332`,
`~/.local/share/claude/versions/2.1.332`,
SHA-256 `a2df1f41bb00250992926420d0063f89b127cb81abdd284dbefb80c50ff1d026`.
GNU bash `5.3.20(1)-release` at `/usr/bin/bash`.

### Harness

Each arm ran the installed executable once in `-p` mode with `--dangerously-skip-permissions`,
`--model haiku` and `--output-format stream-json --verbose`,
with a disposable `CLAUDE_CONFIG_DIR`,
`SHELL=/bin/fish` as on this machine,
and `ANTHROPIC_API_KEY` taken from the repository's test key `CLAUDE_CODE_TEST_CLAUDE_API_KEY`.
The first three arms also had a disposable `HOME`.
The last two used the real `HOME`,
so the shell snapshot was built from the real `~/.bashrc` or `~/.zshrc`,
and a copy of the real `~/.claude/settings.json`,
so its hooks ran;
one copy was taken before the edit under "Verified workaround" and one after.
Every arm's recorded `tool_use` input equals the command asked for.
The runs went in `systemd-run --user --scope` units capped at `CPUQuota=200%`,
`MemoryMax=3G`,
`TasksMax=512` and `IOWeight=1`,
between 2026-11-28T04:44:09-05:00 and 2026-11-28T04:47:20-05:00.

The harness,
as run:

```ts
// Probe: which shell does Claude Code's Bash tool run when the launch SHELL is fish,
// and does CLAUDE_CODE_SHELL=/usr/bin/bash change it from a user settings env block and from the launch environment?
// Each arm runs the installed Claude Code once in -p mode with a disposable CLAUDE_CONFIG_DIR.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync, readdirSync, copyFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const root = '/home/user/temp/agent/claude-code-bash-tool-shell-override-probe-2026-11-28';
const claudeBinary = '/home/user/.local/share/claude/versions/2.1.332';
const keep = join(root, 'keep');
mkdirSync(keep, { recursive: true });
const testKey = process.env.CLAUDE_CODE_TEST_CLAUDE_API_KEY;
if (!testKey) throw new Error('CLAUDE_CODE_TEST_CLAUDE_API_KEY is not set in the probe environment');
const override = { CLAUDE_CODE_SHELL: '/usr/bin/bash' };

type Arm = { name: string; launchEnv: Record<string, string>; settingsEnv?: Record<string, string>; realHomeWithSettingsCopy?: boolean; command?: string };
// The real user settings turn on code mode, where the model writes JavaScript whose template literals would read ${...};
// the real-settings arms therefore ask for a command without ${...}, and also show which git and PATH the shell resolves.
const realSettingsCommand = 'ps -o args= -p $$; command -v git; echo $PATH';
const arms: Arm[] = [
  { name: 'fish-shell-no-override', launchEnv: {} },
  { name: 'fish-shell-override-in-user-settings-env', launchEnv: {}, settingsEnv: override },
  { name: 'fish-shell-override-in-launch-env', launchEnv: override },
  { name: 'fish-shell-real-home-with-copy-of-real-user-settings-before-edit', launchEnv: {}, realHomeWithSettingsCopy: true, command: realSettingsCommand },
  { name: 'fish-shell-real-home-with-copy-of-real-user-settings-after-edit', launchEnv: {}, realHomeWithSettingsCopy: true, command: realSettingsCommand },
];

const only = process.argv.slice(2);
const summary: unknown[] = [];
for (const arm of arms) {
  if (only.length > 0 && !only.includes(arm.name)) continue;
  const armDir = join(root, 'arms', arm.name);
  const home = arm.realHomeWithSettingsCopy ? '/home/user' : join(armDir, 'home');
  const configDir = join(armDir, 'claude-config');
  const work = join(armDir, 'work');
  mkdirSync(configDir, { recursive: true });
  mkdirSync(work, { recursive: true });
  if (!arm.realHomeWithSettingsCopy) mkdirSync(home, { recursive: true });
  if (arm.settingsEnv) writeFileSync(join(configDir, 'settings.json'), JSON.stringify({ env: arm.settingsEnv }, null, 2));
  if (arm.realHomeWithSettingsCopy) copyFileSync('/home/user/.claude/settings.json', join(configDir, 'settings.json'));
  const command = arm.command ?? 'ps -o args= -p $$; echo "bash=${BASH_VERSION-unset} zsh=${ZSH_VERSION-unset}"';
  const prompt = `This is a test of which shell runs your shell commands. Run the following shell command exactly once, character for character, with whatever tool you have for running shell commands. Then reply with its output verbatim and nothing else.\n\n${command}`;
  const started = new Date().toISOString();
  const result = spawnSync(claudeBinary, ['-p', '--model', 'haiku', '--output-format', 'stream-json', '--verbose', '--dangerously-skip-permissions', '--max-turns', '6', prompt], {
    cwd: work,
    env: { HOME: home, CLAUDE_CONFIG_DIR: configDir, PATH: process.env.PATH ?? '/usr/bin:/bin', LANG: 'C.UTF-8', SHELL: '/bin/fish', ANTHROPIC_API_KEY: testKey, DISABLE_AUTOUPDATER: '1', ...arm.launchEnv },
    encoding: 'utf8',
    timeout: 300000,
    maxBuffer: 64 * 1024 * 1024,
  });
  writeFileSync(join(keep, `${arm.name}.stdout.jsonl`), result.stdout ?? '');
  writeFileSync(join(keep, `${arm.name}.stderr.txt`), result.stderr ?? '');
  const lines = (result.stdout ?? '').split('\n').filter(Boolean).flatMap((line) => { try { return [JSON.parse(line)]; } catch { return []; } });
  const toolUses = lines.flatMap((event: any) => (Array.isArray(event?.message?.content) ? event.message.content : []).filter((c: any) => c?.type === 'tool_use').map((c: any) => ({ name: c.name, input: c.input })));
  const toolResults = lines.flatMap((event: any) => (Array.isArray(event?.message?.content) ? event.message.content : []).filter((c: any) => c?.type === 'tool_result').map((c: any) => (typeof c.content === 'string' ? c.content : JSON.stringify(c.content)).slice(0, 1500)));
  const final = lines.find((event: any) => event?.type === 'result');
  const snapshotDir = join(configDir, 'shell-snapshots');
  const record = {
    arm: arm.name, started, ended: new Date().toISOString(), exitStatus: result.status, signal: result.signal, spawnError: result.error?.message,
    askedCommand: command, toolUses, toolResults, finalText: final?.result, isError: final?.is_error,
    shellSnapshots: existsSync(snapshotDir) ? readdirSync(snapshotDir) : [],
  };
  summary.push(record);
  writeFileSync(join(keep, `${arm.name}.summary.json`), JSON.stringify(record, null, 2));
}
const tag = only.length > 0 ? only.join('+') : 'all-arms';
writeFileSync(join(keep, `summary-${tag}.json`), JSON.stringify(summary, null, 2));
console.log(JSON.stringify(summary, null, 2));
```

Run it as `node bash-tool-shell-override-probe.ts [arm names…]` with `CLAUDE_CODE_TEST_CLAUDE_API_KEY` in the environment.
Each arm's `<arm>.summary.json` and raw `<arm>.stdout.jsonl` land in `keep/`.

### Arms that run zsh

- `fish-shell-no-override`:
  `/usr/bin/zsh -c source …/snapshot-zsh-1795859052900-dtgbk6.sh …`,
  and `bash=unset zsh=5.9.2`.
  This is the positive control.
- `fish-shell-real-home-with-copy-of-real-user-settings-before-edit`:
  `/usr/bin/zsh -c source …/snapshot-zsh-1795859210568-rq7ga4.sh …`.

### Arms that run bash

- `fish-shell-override-in-user-settings-env`,
  with only `{"env":{"CLAUDE_CODE_SHELL":"/usr/bin/bash"}}` in the disposable `settings.json`:
  `/usr/bin/bash -c source …/snapshot-bash-1795859059709-xi3d1z.sh 2>/dev/null || true && \builtin shopt -u extglob …`,
  and `bash=5.3.20(1)-release zsh=unset`.
- `fish-shell-override-in-launch-env`,
  with `CLAUDE_CODE_SHELL=/usr/bin/bash` in the launch environment:
  `/usr/bin/bash -c source …/snapshot-bash-1795859064036-7s21ep.sh …`,
  and `bash=5.3.20(1)-release zsh=unset`.
- `fish-shell-real-home-with-copy-of-real-user-settings-after-edit`:
  `/usr/bin/bash -c source …/snapshot-bash-1795859237086-woj6pk.sh …`.

The two real-settings arms printed the same `git`,
`/home/user/Monochromatic/.cache/monochromatic-git-policy-cli-bin/git`,
and the same `PATH`
(516 characters in both,
equal SHA-256 prefix `eff5c9d9cfb641a4` over the `git` and `PATH` lines).
So the switch changed the interpreter and nothing that resolves commands.

### Not measured

- Interactive sessions:
  every arm ran in `-p` mode.
  Interactive mode reaches the same `S()` through the same memoized lookup,
  which is a reading of the bundle,
  not a run.
- Hook commands,
  the status line and stdio MCP servers:
  which shell starts them was not traced.
- Commands typed after `!` in the input box:
  the `defaultShell` setting
  (`Default shell for input-box ! commands. Defaults to 'bash' on all platforms`,
  from the bundled settings schema)
  may govern them,
  and they were not run.

## Verified workaround

`~/.claude/settings.json` got one key in its `env` block,
at 2026-11-28T04:47:05-05:00,
on the user's request of that day:

```json
// ~/.claude/settings.json, the env block (other keys unchanged)
"CLAUDE_CODE_EXPERIMENTAL_CODE_MODE": "1",
"CLAUDE_CODE_SHELL": "/usr/bin/bash"
```

`diff` against the file as it was shows only that line and the comma before it,
and the file still parses as JSON with the same top-level keys.
The `after-edit` arm under "Verification" ran a copy of the edited file.

### Tradeoffs

- Only sessions started after the edit run bash.
  A running session,
  including the one that made the edit,
  keeps zsh until it is started again
  (see "The choice is made once per process").
- Only a Claude Code that reads `~/.claude/settings.json` gets it.
  A run with another `CLAUDE_CONFIG_DIR`,
  such as the harness here,
  falls back to detection and runs zsh.
- A value that is not an executable bash or zsh stops working without any visible message:
  Claude Code writes only the debug line
  `CLAUDE_CODE_SHELL="…" is not a valid bash/zsh path, falling back to detection`
  and goes back to zsh.
  If bash ever moves,
  change the value.
- Commands written for zsh behave differently under bash.
  Agents here write bash syntax,
  so this removes a mismatch rather than adding one.
- The bundle refuses some commands under an old bash
  (byte 341053897,
  a `case "${BASH_VERSINFO-}" in '2'|'3'|'4')` guard,
  `'2'|'3'` in one variant,
  that exits 126).
  Bash here is 5.3.20,
  so the guard does not fire.

### Re-checking after a Claude Code update

The switch and the detection order are read from a minified,
closed-source bundle.
After an update,
run `rg --text --count --fixed-strings CLAUDE_CODE_SHELL ~/.local/share/claude/versions/<version>`
(30 matches in 2.1.332)
and the harness's `fish-shell-override-in-user-settings-env` and `fish-shell-no-override` arms.

## What does not work

Nothing was tried and rejected.
Two other ways were weighed and not used:

- Exporting `CLAUDE_CODE_SHELL` from `~/.config/environment.d/` and `~/.bashrc`,
  as `doc/troubleshooting/claude-code-shell-c-script-rm-prompt.md` does for a variable the loader drops.
  The launch-environment arm shows it would work,
  but it needs a new login,
  reaches only processes started from those environments,
  and duplicates a value a settings file can carry.
- Setting `SHELL` to bash.
  The detection code would take it,
  but `SHELL` also tells every other program which shell to open for the user,
  and the user asked about Claude Code's Bash tool only.
  Not run.

## Upstream filing decision

Nothing to file.
`.out-of-scope/claude-code-upstream-bugs.md` puts Claude Code bug reports out of scope for this project,
and this is not a defect in any case:
the detection order and the override are documented in the environment variable reference quoted under "Root cause".
The six constraints were therefore not walked,
and no draft is kept.

[env-vars-shell]: https://code.claude.com/docs/en/env-vars#:~:text=Set%20the%20shell%20Claude%20Code%20uses%20to%20run%20Bash%20tool%20commands
