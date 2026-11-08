# guardrail

PreToolUse hook that guards Agent and Bash tool calls with four checks:
resume polling prevention,
`bun test` blocking,
process-name wait loop blocking,
and mutating real-Git path blocking.

## Problem

Claude sometimes polls background agents via `resume` calls instead of waiting
for automatic completion notifications,
 which wastes context tokens on repeated
error messages.
 Separately,
 `bun test` misreports under this repo's custom
`@monochromatic-dev/test-harness-core` harness:
 the harness runs tests as a side
effect of module import,
 so `bun test <file>` prints `PASS` lines and then
reports `0 pass / 0 fail` (bun's runner finds no `bun:test` registrations),
which looks like a broken run when every test passed.

General-purpose Agent calls are no longer blocked here.
 The ban was lifted once
the Claude Code UI let a human observe and message subagent sessions directly;
see `doc/decision/general-purpose-subagent-ban.md`.

## Solution

The handler implements denial responses for four cases:

1. **Resume blocking**:
    when an Agent call includes a `resume` parameter,
   it is denied with a message explaining that background agents notify
   automatically.

2. **`bun test` blocking**:
    when a Bash call invokes `bun test`,
    it is denied
   with a message directing the caller to `mise run //package/<path>:test:unit`
   (or `node <file>` for ad-hoc single-file runs).

3. **Process-name wait loop blocking**:
   when a Bash call runs `pgrep` or `pidof` inside a `while` or `until` loop,
   including inside a `bash -c` script string,
   it is denied
   with a message explaining the self-match and pointing at PID-based waits
   (`timeout 600 tail --pid=<pid> --follow /dev/null`,
   or `systemd-run --user --wait`).
   Straight-line `pgrep` calls and `kill -0 "$pid"` loops stay allowed.
   `ps | grep` loops are not detected.

4. **Mutating real-Git path blocking**:
    when a Bash call runs a mutating Git subcommand outside the repository's `git` wrapper
   (as in `/usr/bin/git commit`,
   `PATH=/usr/bin:$PATH git commit`,
   `command -p git commit`,
   `env -i git commit`,
   or `env -u PATH git commit`),
   it is denied
   with a message naming `git -C <repository-root> <subcommand>` instead.
   The repository's `git` on PATH is a wrapper that applies commit-time policies
   and pushes after a commit;
   each of those forms starts the real binary and skips all of them.

## Mutating real-Git paths

The check lives in
`@monochromatic-dev/agent-harness-shared-shell-command-analyzer`
as `invokesMutatingGitThroughRealGitPath`,
beside `invokesBunTest`,
and it reads the executable name from
`@monochromatic-dev/git-executable-core`
rather than spelling a path.
Covered spellings include a quoted path,
`env X=1 /usr/bin/git commit`,
`sudo /usr/bin/git reset`,
`cd x && /usr/bin/git add .`,
`bash -c '/usr/bin/git commit'`,
a function body,
Git global options in front of the subcommand,
a `PATH` assignment on the same command line or in an earlier statement,
an attached or separated `env -S` payload,
and the default search path forms `command -p`,
`env -i` and `env -u PATH`.
Only a word carrying a path separator matches,
so `/usr/share/doc/git/RelNotes` and `/usr/bin/gitk` stay allowed,
and so does the PATH wrapper,
including `git -C <root> commit`.
Read-only subcommands (`for-each-ref`,
`rev-parse`,
`log`,
`show`,
`status`,
`ls-remote`,
`diff`,
`cat-file`,
`ls-files`,
`merge-base` and the rest of the read set)
stay allowed through the same absolute path and under any of those search-path forms.
A subcommand this analyzer has never seen counts as mutating.

A Git path known only at run time stays undetected:
`bash -c "$CMD commit"` and `G=/usr/bin/git; $G commit` are not refused,
because the check reads command text and runs no shell evaluator.
The shared analyzer README lists every undetected form.

The handler is asynchronous because the check resolves the real Git executable;
the hook runtime awaits every handler.
When resolution fails,
the check reports nothing and logs at debug level,
so a host without a resolvable Git does not fail every Bash call.

## Process-name wait loops

The Bash tool runs each command inside a wrapper shell whose command line contains the command text.
`pgrep --full <pattern>` therefore matches that wrapper,
and a loop waiting for the pattern to disappear never exits.
Measured on 2026-10-19:
`bash -c 'pgrep --list-full --full <unique-string>'` listed both the wrapper shell and the inner `bash -c`.
The check was verified through the live Claude Code host,
which denied a bounded probe loop.

## Resume blocking status

On 2026-10-15,
the user reported that resume blocking does not appear to work in Claude Code.
This is a user-observed discrepancy,
not a reproduced failure or an established explanation of the host's behavior.
The handler still returns a denial for an Agent input containing a string `resume` field.
That source behavior alone does not prove Claude Code invokes the hook or enforces its response.

The user wants resumes allowed but deferred removal of the existing check.
Their concern is that Claude Code's non-open-source implementation makes unproven changes undesirable.
Keep the check unchanged until host-level evidence supports a change.
No such runtime verification was performed for this observation.

Follow the [Claude Code upstream bugs policy](../../../.out-of-scope/claude-code-upstream-bugs.md):
do not file or track upstream Claude Code bugs as GitHub issues,
or develop elaborate local workarounds for upstream defects.
This observation does not establish an upstream defect.
Document the uncertainty without treating it as a request for investigation or a speculative workaround.

## Setup

Add to `.claude/settings.json`:

```jsonc
{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [{ "type": "command", "command": "ccgr" }],
      },
    ],
  },
}
```

## Binary

**`ccgr`**:
 Claude Code GuardRail

## Maintenance

### Rebuilding the bundle

`bundle/node/index.mjs` is tracked because Claude Code's marketplace install runs it with no build step.
Rebuild it with `mise run //package/claude-code-plugin/guardrail:build`,
never a raw bundler.
`git diff` on the bundle is a line diff over minified lines that are tens of kilobytes each,
so a rebuild that changes six lines can print a patch of hundreds of kilobytes.
Compare token by token instead:
split both files after `;`,
`{` and `}`,
then run `diff` on the split copies.
The rebuild on 2026-11-07 changed 6 lines (+604 bytes),
all attributable to sources.

The bundle inlines more than this repository's sources.
It carries the `unbash` parser,
so a catalog bump of `unbash` changes the bundle with no source commit in `package/`.
It also inlines `@monochromatic-dev/caught-value-core` from its gitignored `dist/final/neutral/index.mjs`,
because `@monochromatic-dev/logger-core` imports that package by its root export,
not its `/ts` subpath.
A fresh clone must build `//package/caught-value/core` before this bundle can be rebuilt faithfully.

Attributing drift:
list the sources with a transitive import walk from `src/index.ts`,
run `git log <last bundle commit>..main -- <those files> pnpm-lock.yaml pnpm-workspace.yaml`,
and diff the `unbash` package versions between the lock at the bundle's last commit and now.
A `git log` over this repository's sources alone misses dependency bumps.

### Why the bundle drifted

Nothing in the commit path builds it.
The bundle changes only when someone remembers to rebuild:
the last three bundle commits are `689c6e624d` (shipped with a source change),
`3597fe2748` (a sweep onto a dependency change) and `d2e83c095e` (a sweep onto a logging change).
Source commits that landed in between left the bundle behind:
`2619ea3af7` for the refusal text,
and the `unbash` bumps in `581190711e` and `d48e67b24e`.
Their effects stayed out of every session until this rebuild.
`file-enforcer check --outputs committed` is the check this repository has that committed outputs match their sources
(a search of the mise tasks,
the workflows and the file-enforcer rules found no other),
and CI runs it in `.github/workflows/advisory-core.yml`;
its rule kinds (concat,
mirror,
mode-only,
toml-keys and the like) have none that runs a bundler,
so the bundles are outside it.
`doc/decision/dist-in-git.md` accepted the "forgot to rebuild" risk
and names stale committed output as a condition to revisit.
This README does not edit that record.

### Proposal: check that the bundle matches its sources

Not built yet;
the owner picks.
Every option needs the same two preconditions,
and both hold today:
the rolldown output is byte-for-byte deterministic
(a second build after the 2026-11-07 commit left `git status` clean),
and the `caught-value-core` `dist` exists before the build.

Option A,
a cluster mise task plus a CI step:
a task in `package/claude-code-plugin` builds every bundle into a scratch directory
and compares each file with its tracked copy.
CI runs it in `.github/workflows/advisory-core.yml` after `prepare:pnpm`,
next to the file-enforcer drift steps.
Pros:
it covers all plugin bundles at once
(`bash-output-filter` and `terminal-title` inline `unbash` too and had drifted the same way until their
2026-11-07 rebuilds),
it needs no new tool,
and it fails on the change that causes the drift.
Cons:
it reports drift after the push,
not at commit time,
and it must build into scratch so the tracked file is not rewritten.

Option B,
a file-enforcer rule kind that runs a command and compares its output:
Pros:
it reuses the existing `check` and `write` pair,
so `mise run sync:files` would repair the bundles too.
Cons:
it makes a file-enforcer rewrite carry a build toolchain,
widens a tool that today renders text and TOML only,
and makes every `sync:files` run pay for bundling.

Option C,
a `monochromatic-git-policy-cli` guard at commit time
that refuses a commit touching bundle inputs while the bundle is stale:
Pros:
it stops the drift before it enters history.
Cons:
the input set includes the lock and the catalog,
so many commits would trigger a build,
and it adds that build to the commit path.

Ranking:
A > B > C.
A beats B because it needs no new rule kind and keeps bundling out of the file renderer.
B beats C because it checks in CI and by one local command
instead of adding a bundle build to the commit path.
