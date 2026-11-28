# Claude Code 2.1.332 asks about every `sh -c` script that runs `rm` and that it cannot read, even under `--dangerously-skip-permissions`

## Symptom

A Bash call that hands a script to a shell with `-c`
(`sh -c '…'`,
`bash -c '…'`,
`podman exec "$cid" sh -c '…'`,
`env … bash -c '…'`)
and whose script contains `rm` stops at a permission prompt that reads:

```text
This command passes a shell -c script that runs rm, and Claude Code could not check the script for dangerous removals. Approve only if you have read the script.
```

Its short reason,
the line the user quoted on 2026-11-28,
is `This shell -c script runs rm and could not be checked`.

The prompt appears in every permission mode,
including sessions started with `claude --dangerously-skip-permissions`,
which is how the sessions on this machine are started (`/proc/<pid>/cmdline` of both live sessions on 2026-11-28).
No allow rule answers it.
In a `-p` run,
which has nobody to answer prompts,
the call is refused with the same text.

Agent-written commands in this repository's own transcripts hit it,
for example a `podman exec "$cid" sh -c '…'` status check
and an `env --ignore-environment … bash -c 'cd -- … && node …'` edit
(`~/.claude/projects/-home-user-Monochromatic/`,
searched for the message text on 2026-11-28).

## Root cause

Claude Code ships as a native executable with its JavaScript bundled,
not as open source.
The trace below reads the installed bundle,
`~/.local/share/claude/versions/2.1.332`,
with `rg --text --byte-offset --only-matching`.
Names are minified,
and each excerpt is cited by its byte offset in that file.

### The prompt is a hard-coded dangerous-removal safety check

Byte 343627504:

```js
// claude 2.1.332 bundle
function cVe(e,n){if(n===0)Sq("inline_shell_unchecked");return{behavior:"ask",message:e?"A permission rule denies a command inside this shell -c script, so Claude Code could not check it for dangerous removals. Approving runs the whole script, including that command.":"This command passes a shell -c script that runs rm, and Claude Code could not check the script for dangerous removals. Approve only if you have read the script.",decisionReason:{type:"safetyCheck",reason:e?"A permission rule denies a command in this shell -c script":"This shell -c script runs rm and could not be checked",classifierApprovable:!1,circuitBreaker:"dangerousRemoval",removalRule:"inline_shell_unchecked"},suggestions:[]}}
```

The answer is `ask` with `classifierApprovable:!1` and the `dangerousRemoval` circuit breaker,
and `suggestions:[]` offers no rule to add.
The breaker's properties sit at byte 329665780:

```js
// claude 2.1.332 bundle
Oyt={dangerousRemoval:{bypassImmune:!0,waivableInBypass:!0,classifierRouted:!0,autoModeDeny:!0,hostPersonOnly:!1,localProjectionOnly:!1,servedUnplaceable:!1},
```

`bypassImmune:!0` is why `--dangerously-skip-permissions` still shows the prompt.
`waivableInBypass:!0` is the hook for `CLAUDE_CODE_EVEN_MORE_DANGEROUSLY_SKIP_PERMISSIONS`,
covered under "What does not work".

### When the script counts as unreadable

The function that reads a `-c` script gives up on text it cannot follow.
Byte 343623077 is one of its exits:
a script containing the word `alias` comes back with no judged text.

```js
// claude 2.1.332 bundle
if(SVe(k).some(Uzs)||/\balias\b/.test(k)||/\bIFS\b/.test(k.replace(
```

Byte 343628371 drops scripts that name no `rm` early,
so only scripts that run `rm` can reach `cVe`:

```js
// claude 2.1.332 bundle
if(Q===void 0||!(Odo(Q.judged??Q.script)||Q.script.includes("$'")))return null;
```

The other exits (scripts over a length limit,
`IFS`,
zsh-only syntax,
too many directory changes)
were read only as far as their conditions and were not probed one by one.

### The switch that turns the whole check off

Byte 343632168,
the entry to the `-c` script analysis:

```js
// claude 2.1.332 bundle
async function pVe(e,n,o=[],g=[]){if(s.CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT||!Vte())return null;
```

With the variable set,
the analysis returns nothing for every `-c` script,
so neither this prompt nor its sibling "A permission rule denies a command in this shell -c script" is produced.
The current [environment variable reference][env-vars-switch] documents the switch:

> Set to 1 to stop Claude Code from reading scripts passed to a shell with -c,
> such as bash -c 'rm -rf ~',
> for critical-path removals.
> Claude Code still checks the shell variable and positional parameter targets in those scripts,
> and the other critical-path checks keep running.
> Set it in the environment that launches Claude Code,
> since Claude Code ignores a copy delivered through a settings env block.
> Requires Claude Code v2.1.288 or later

### Why a settings file cannot set it

The variable is in a set (`rla`) that the settings loader drops from every settings source.
Byte 333843568,
inside the loop over a settings file's `env` block:

```js
// claude 2.1.332 bundle
if(rla.has(E.toUpperCase())&&!Ct(E,g,n))continue;
```

Byte 333841306,
the only exception,
which is for managed policy settings:

```js
// claude 2.1.332 bundle
function Ct(e,n,o){return o==="policySettings"&&gt.has(e.toUpperCase())&&pi(n)}
```

The drop is silent for `~/.claude/settings.json`;
for a project's `.claude/settings.json` the loader logs
"no settings file can set this key,
only the environment Claude Code is started with or Claude Code itself."
The probe's third arm under "Verification" confirms the drop for a user settings file.

## Verification

Claude Code `2.1.332`,
`~/.local/share/claude/versions/2.1.332`,
SHA-256 `a2df1f41bb00250992926420d0063f89b127cb81abdd284dbefb80c50ff1d026`.

### Harness

Each arm ran the installed executable once in `-p` mode with `--dangerously-skip-permissions`,
`--model haiku` and `--output-format stream-json --verbose`,
with a disposable `HOME` and `CLAUDE_CONFIG_DIR` under
`~/temp/agent/claude-code-shell-c-rm-prompt-2026-11-28/arms/<arm>/`,
an environment built from nothing but `HOME`,
`CLAUDE_CONFIG_DIR`,
`PATH`,
`LANG`,
`DISABLE_AUTOUPDATER`,
and `ANTHROPIC_API_KEY` taken from the repository's test key `CLAUDE_CODE_TEST_CLAUDE_API_KEY`.
The prompt asked the model to run one command unchanged;
every arm's recorded `tool_use` input equals the command asked for.
The runs went in `systemd-run --user --scope` units capped at `CPUQuota=200%`,
`MemoryMax=3G`,
`TasksMax=512` and `IOWeight=1`,
between 2026-11-28T03:18:13-05:00 and 2026-11-28T03:19:49-05:00.

The harness,
as run:

```ts
// ~/temp/agent/claude-code-shell-c-rm-prompt-2026-11-28/keep/inline-shell-rm-switch-probe.ts
// Probe: does CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT stop Claude Code's
// "This shell -c script runs rm and could not be checked" safety prompt,
// and is it honoured from the launch environment versus a settings file's env block?
// Each arm runs the installed Claude Code once in -p mode with a disposable HOME and CLAUDE_CONFIG_DIR.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const root = '/home/user/temp/agent/claude-code-shell-c-rm-prompt-2026-11-28';
const claudeBinary = '/home/user/.local/share/claude/versions/2.1.332';
const keep = join(root, 'keep');
mkdirSync(keep, { recursive: true });
const testKey = process.env.CLAUDE_CODE_TEST_CLAUDE_API_KEY;
if (!testKey) throw new Error('CLAUDE_CODE_TEST_CLAUDE_API_KEY is not set in the probe environment');
const switchName = 'CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT';

type Arm = { name: string; launchEnv: Record<string, string>; settingsEnv?: Record<string, string>; shape: 'unchecked' | 'runtime-target' | 'runtime-target-recursive' | 'whole-variable-target' };
const arms: Arm[] = [
  { name: 'unchecked-script-switch-unset', launchEnv: {}, shape: 'unchecked' },
  { name: 'unchecked-script-switch-in-launch-env', launchEnv: { [switchName]: '1' }, shape: 'unchecked' },
  { name: 'unchecked-script-switch-in-user-settings-env', launchEnv: {}, settingsEnv: { [switchName]: '1' }, shape: 'unchecked' },
  { name: 'runtime-target-switch-unset', launchEnv: {}, shape: 'runtime-target' },
  { name: 'runtime-target-switch-in-launch-env', launchEnv: { [switchName]: '1' }, shape: 'runtime-target' },
  { name: 'recursive-runtime-target-switch-unset', launchEnv: {}, shape: 'runtime-target-recursive' },
  { name: 'recursive-runtime-target-switch-in-launch-env', launchEnv: { [switchName]: '1' }, shape: 'runtime-target-recursive' },
  { name: 'whole-variable-target-switch-unset', launchEnv: {}, shape: 'whole-variable-target' },
  { name: 'whole-variable-target-switch-in-launch-env', launchEnv: { [switchName]: '1' }, shape: 'whole-variable-target' },
];

const only = process.argv.slice(2);
const summary: unknown[] = [];
for (const arm of arms) {
  if (only.length > 0 && !only.includes(arm.name)) continue;
  const armDir = join(root, 'arms', arm.name);
  const home = join(armDir, 'home');
  const configDir = join(home, '.claude');
  const work = join(armDir, 'work');
  mkdirSync(configDir, { recursive: true });
  mkdirSync(work, { recursive: true });
  const victim = join(work, 'victim.txt');
  writeFileSync(victim, 'delete me\n');
  const emptyFile = join(work, 'empty.txt');
  writeFileSync(emptyFile, '');
  if (arm.settingsEnv) writeFileSync(join(configDir, 'settings.json'), JSON.stringify({ env: arm.settingsEnv }, null, 2));
  // unchecked: the word alias makes Claude Code give up analysing the script (jzs returns no judged text).
  // runtime-target: the rm target is built from a command output that is empty at run time,
  // so it names /claude-code-probe-nonexistent-target, which does not exist and which this user cannot write.
  const command = arm.shape === 'unchecked'
    ? `sh -c 'alias q=true; rm -f ${victim}'`
    : arm.shape === 'runtime-target'
      ? `sh -c 'X=$(cat ${emptyFile}); rm -f "$X/claude-code-probe-nonexistent-target"'`
      : arm.shape === 'runtime-target-recursive'
        ? `sh -c 'X=$(cat ${emptyFile}); rm -rf "$X/claude-code-probe-nonexistent-target"'`
        // X is empty at run time, so this runs rm -rf "" and deletes nothing.
        : `sh -c 'X=$(cat ${emptyFile}); rm -rf "$X"'`;
  const prompt = `This is a test of a permission check. Use the Bash tool exactly once to run the following command, character for character, without changing, splitting or quoting it differently. Then reply with the single word DONE. If the tool call is refused, reply with the single word REFUSED.\n\n${command}`;
  const started = new Date().toISOString();
  const result = spawnSync(claudeBinary, ['-p', '--model', 'haiku', '--output-format', 'stream-json', '--verbose', '--dangerously-skip-permissions', '--max-turns', '4', prompt], {
    cwd: work,
    env: { HOME: home, CLAUDE_CONFIG_DIR: configDir, PATH: process.env.PATH ?? '/usr/bin:/bin', LANG: 'C.UTF-8', ANTHROPIC_API_KEY: testKey, DISABLE_AUTOUPDATER: '1', ...arm.launchEnv },
    encoding: 'utf8',
    timeout: 300000,
    maxBuffer: 64 * 1024 * 1024,
  });
  writeFileSync(join(keep, `${arm.name}.stdout.jsonl`), result.stdout ?? '');
  writeFileSync(join(keep, `${arm.name}.stderr.txt`), result.stderr ?? '');
  const lines = (result.stdout ?? '').split('\n').filter(Boolean).flatMap((line) => { try { return [JSON.parse(line)]; } catch { return []; } });
  const toolUses = lines.flatMap((event: any) => (event?.message?.content ?? []).filter((c: any) => c?.type === 'tool_use').map((c: any) => c.input?.command));
  const toolResults = lines.flatMap((event: any) => (Array.isArray(event?.message?.content) ? event.message.content : []).filter((c: any) => c?.type === 'tool_result').map((c: any) => (typeof c.content === 'string' ? c.content : JSON.stringify(c.content)).slice(0, 1200)));
  const final = lines.find((event: any) => event?.type === 'result');
  const record = {
    arm: arm.name, started, ended: new Date().toISOString(), exitStatus: result.status, signal: result.signal, spawnError: result.error?.message,
    askedCommand: command, toolUseCommands: toolUses, commandUnchanged: toolUses.includes(command), toolResults,
    permissionDenials: final?.permission_denials, finalText: final?.result,
    victimStillExists: arm.shape === 'unchecked' ? existsSync(victim) : undefined,
    mentionsUncheckedPrompt: toolResults.some((t: string) => t.includes('shell -c script that runs rm')),
    mentionsRuntimeTargetPrompt: toolResults.some((t: string) => t.includes('Dangerous rm operation in a shell -c script')),
  };
  summary.push(record);
  writeFileSync(join(keep, `${arm.name}.summary.json`), JSON.stringify(record, null, 2));
}
const tag = only.length > 0 ? 'selected-arms' : 'all-arms';
writeFileSync(join(keep, `summary-${tag}.json`), JSON.stringify(summary, null, 2));
writeFileSync(join(keep, `done-${tag}`), new Date().toISOString());
```

Run it as `node inline-shell-rm-switch-probe.ts [arm names…]` with `CLAUDE_CODE_TEST_CLAUDE_API_KEY` in the environment.

### Arms that prompt

- `sh -c 'alias q=true; rm -f <work>/victim.txt'` with the switch unset:
  the tool result is the prompt text quoted under "Symptom",
  `permission_denials` lists the call,
  and the victim file is still there.
  This is the positive control.
- The same command with the switch only in the disposable `~/.claude/settings.json` as
  `{"env":{"CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT":"1"}}`:
  same prompt text,
  same denial,
  victim file still there.

### Arms that run

- The same command with `CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT=1` in the launch environment:
  the tool result is `(Bash completed with no output)`,
  `permission_denials` is empty,
  and the victim file is gone.

### Arms that measured nothing

Three shapes whose `rm` target is built from a value known only at run time
(`X=$(cat <work>/empty.txt)` followed by `rm -f "$X/claude-code-probe-nonexistent-target"`,
`rm -rf "$X/claude-code-probe-nonexistent-target"` and `rm -rf "$X"`)
ran without a prompt with the switch unset as well as set.
They have no positive control,
so whether the switch also lifts the check on such targets inside `-c` scripts is not measured here;
the reference quoted under "Root cause" says those targets are still checked.

## Verified workaround

Set the variable in the environment Claude Code starts with.
On this machine Claude Code runs from interactive `bash` in OdyTTY,
and OdyTTY is started by the systemd user manager,
whose environment comes from `~/.config/environment.d/`
(the home `~/.bashrc` already names for this machine's environment variables).
On 2026-11-28 this file was written:

```ini
# ~/.config/environment.d/60-claude-code-inline-shell-rm-prompt.conf
# Turns off Claude Code's "This shell -c script runs rm and could not be checked" prompt,
# which neither allow rules nor --dangerously-skip-permissions can answer.
# Claude Code reads this variable only from the environment it is started with:
# it ignores a copy in a settings file's env block.
# Claude Code's other dangerous-rm checks keep running.
# Record: /home/user/Monochromatic/doc/troubleshooting/claude-code-shell-c-script-rm-prompt.md
# Takes effect for processes started after the next login.
CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT=1
```

and loaded into the running user manager with
`systemctl --user set-environment CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT=1`.
Checks on 2026-11-28:
`/usr/lib/systemd/user-environment-generators/30-systemd-environment-d-generator` prints the line,
`systemctl --user show-environment` lists it,
and `systemd-run --user --wait --pipe printenv CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT` prints `1`.

### Tradeoffs

- Claude Code no longer reads `-c` scripts for dangerous removals at all.
  A `bash -c 'rm -rf ~'` is then judged like any other Bash call:
  under `--dangerously-skip-permissions` it runs,
  and in auto mode the classifier decides.
  The `rm` checks on commands outside a `-c` script keep running.
- The sibling prompt for a `-c` script that contains a command a deny rule blocks goes away too,
  because it comes from the same analysis.
- A running Claude Code keeps the environment it started with.
  The OdyTTY process that hosts the terminal tabs also keeps its own,
  so new tabs in it get the variable only after OdyTTY is restarted or the user logs in again.
  Until then,
  `export CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT=1` in a shell before `claude` applies it to that launch.
- The switch is read from a minified,
  closed-source bundle;
  re-check it after a Claude Code update with
  `rg --text --fixed-strings --count CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT ~/.local/share/claude/versions/<version>`
  and the first arm of the harness.

## What does not work

- Allow rules:
  the decision carries `suggestions:[]` and the reference says this class of check cannot be auto-allowed;
  the bundle's own variable-target message says "cannot be auto-allowed by permission rules".
- `--dangerously-skip-permissions`:
  the `dangerousRemoval` breaker is `bypassImmune`.
- `CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT` in `~/.claude/settings.json` `env`,
  where this machine keeps its other `CLAUDE_CODE_*` variables:
  dropped by the loader,
  shown by the probe's settings arm.
- `CLAUDE_CODE_EVEN_MORE_DANGEROUSLY_SKIP_PERMISSIONS`:
  it waives every dangerous-removal prompt in bypass mode,
  which is far wider than this one prompt,
  and the bundle keeps the prompts whenever the session has deny or ask rules or a `PermissionRequest` hook
  (byte 334746249 prints "CLAUDE_CODE_EVEN_MORE_DANGEROUSLY_SKIP_PERMISSIONS has no effect in this session:
  dangerous rm commands keep their usual checks.").
  `~/.claude/settings.json` has deny rules,
  so it was not tried.

## Upstream filing decision

Nothing to file.
`.out-of-scope/claude-code-upstream-bugs.md` puts Claude Code bug reports out of scope for this project,
and this is not a defect in any case:
the prompt is deliberate,
and upstream documents the switch that turns it off.
The six constraints were therefore not walked,
and no draft is kept.

[env-vars-switch]: https://code.claude.com/docs/en/env-vars#:~:text=CLAUDE_CODE_DISABLE_INLINE_SHELL_RM_PROMPT
