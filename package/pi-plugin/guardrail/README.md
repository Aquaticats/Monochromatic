# pi-guardrail

Tool-call guardrails for pi.

The extension blocks misleading `bun test` Bash tool calls,
 refuses Bash tool calls that run a mutating Git subcommand through a path to a
 real Git binary,
 refuses Bash, `write`, `edit` and `codemode` calls that delete a path the shell
 computes instead of spelling it,
 and refuses `edit` or `write` calls for protected paths matched with
 `.gitignore` semantics.

## Pi dependency trust

PJY:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/guardrail"
  ]
}
```

Keep project `.pi/settings.json` package-free unless the guardrail becomes
shared project policy.

## Built-in behavior

### `bun test` Bash blocking

The guardrail blocks Bash tool calls that invoke `bun test` at a shell command
segment boundary.

Use package mise tasks instead:

```sh
mise run //package/<path>:test:unit
```

When no such task exists,
 add one to the target package's `mise.toml` first.
For ad-hoc single-file runs,
 use `node <file>` directly.

### Absolute-path real Git blocking

`git` on `PATH` in this repository is a wrapper.
 It applies the repository's policies,
 runs the commit-time checks,
 and pushes after each commit.
A command that spells the Git binary by path,
 such as `/usr/bin/git commit`,
 starts the real binary instead,
 so every one of those policies is skipped.
So does a command that takes the wrapper off the search path:
`PATH=/usr/bin:$PATH git commit`,
`export PATH=/usr/bin; git commit`,
`command -p git commit`,
`env -i git commit`,
and `env -u PATH git commit`.

The guardrail therefore blocks a Bash tool call that runs a mutating Git
subcommand outside that wrapper.
 The refusal names the subcommand to run instead:

```sh
git -C <repository-root> commit -m "message" path/to/file
```

`git -C <root>` is the same `git` the wrapper provides,
 so it works from any directory.
Without `-C`,
 plain `git` works from the repository root.

Read-only probes through the same absolute path,
 or under any of those search-path forms,
 stay allowed,
 because they change nothing:
 `status`,
 `log`,
 `show`,
 `diff`,
 `rev-parse`,
 `for-each-ref`,
 `ls-remote`,
 `cat-file`,
 `ls-files`,
 and `merge-base`.
A subcommand whose effect the analyzer cannot prove read-only counts as
mutating,
 so only a proven read passes.

The check parses shell grammar rather than matching text,
 so quoting,
 chaining,
 and wrappers such as `env`,
 `sudo`,
 `timeout`,
 or a shell `-c` string do not hide a mutation.

A Git path that only the shell knows at run time stays undetected:
`bash -c "$CMD commit"` and `G=/usr/bin/git; $G commit` are not refused,
because the check reads command text and runs no shell evaluator.
The shared analyzer README lists every undetected form.

Unlike the `bun test` guard,
 this one has no config switch:
 running the real binary skips guards that protect committed work.

### Literal `rm` path blocking

The repository's rule is that an `rm` command writes every path literally:
a path spelled through a variable, a substitution, or an arithmetic expression
can widen when that value is unset or empty,
while the command text still looks narrow.
The shared
`@monochromatic-dev/agent-harness-shared-shell-command-analyzer`
package holds the reader,
and its README lists every form it reads and every form it does not.

#### The Bash surface

A `bash` call is refused when it would run such a deletion,
including one behind a program wrapper (`sudo`, `env`, `timeout`, `systemd-run`,
`flock`, `nice`, `command`, `xargs`),
one written in a here-document body,
one inside a `bash -c` string or an `eval` or `ssh` command line,
one inside a function body,
one `find -exec` runs,
and one inside a script file the command hands a shell
(`bash deploy.sh`, `source env.sh`, `./cleanup` with a shell shebang),
read from disk relative to the session's working directory.

#### The `write` and `edit` surface

A `write` or `edit` call is refused when the file it would leave on disk is a shell
script holding a deletion the file does not hold now.
A script is recognized by `.sh`, `.bash`, `.zsh`, `.ksh`, `.dash`, or `.bats`,
or by a first line that asks for a shell.
The whole resulting file is judged,
so two edits cannot split one violation between them,
and a change that repairs an old script is never refused.
The protected-path rules above still run first and keep their own messages.

#### The `codemode` surface

Code mode runs a script a model wrote,
and Pi dispatches the `codemode` call itself through `tool_call`,
with its input `{ code }`,
so this extension reads the script before any of it runs.
A script is refused when it builds an `rm` command at run time:

- a template literal holding a `${` substitution whose literal text puts `rm` in
  command position,
- a string holding an `rm` command joined with `+`,
- a string that is exactly `rm`.

The nested calls a script makes arrive as ordinary tool calls carrying their
parent's id,
so a script that reads its command from somewhere this extension cannot see still
meets the Bash guard at the nested `bash` call,
which is refused,
and the script receives the rejection.
That was measured through a real Pi session on 2026-11-24:
`src/pi-runtime-verifier-literal-rm.ts` records every dispatched call.

What stays undetected on this surface:
a command the script assembles from values no literal spells (a file's text through
`tools.read`, a stored value through `load`) is not read by the script guard,
a literal that puts `sh` or another shell in command position
(`` `sh -c \`rm -rf ${d}\`` ``) names no `rm` of its own,
`rmdir`, `unlink`, `shred` and `find -delete` are not read,
and a regular expression in the script is read as arithmetic rather than skipped,
so a quote inside one can hide a literal after it.

### `pnpm-lock.yaml` edit and write blocking

The guardrail ships this protected-path rule:

```json
{
  "pnpm-lock.yaml": "edit pnpm-workspace.yaml and package.json files then run `pnpm install`"
}
```

Because matching follows `.gitignore` semantics,
 `pnpm-lock.yaml` matches that
basename anywhere under pi's current working directory.

## Global config

Optional global config lives at:

```text
~/.pi/agent/extensions/pi-guardrail.json
```

Short form maps `.gitignore`-style patterns to refusal messages:

```json
{
  "Cargo.lock": "edit Cargo.toml then run `cargo update` intentionally",
  "package-lock.json": "edit package.json then run `npm install`"
}
```

Advanced form can also disable the `bun test` guard:

```json
{
  "blockBunTest": false,
  "pathRules": {
    "Cargo.lock": "edit Cargo.toml then run `cargo update` intentionally"
  }
}
```

Rules are appended after built-in defaults.
 Use `.gitignore` negation to unguard
a built-in rule:

```json
{
  "!pnpm-lock.yaml": ""
}
```

## Source structure

```text
src/
  index.ts                         # Extension entry point and tool_call registration
  bash-guard.ts                    # Shared analyzer backed `bun test`, absolute real-Git, and literal-`rm` checks
  script-file-guard.ts             # Literal-`rm` check for `write` and `edit` on shell scripts
  codemode-script-guard.ts         # Refuses a code-mode script that builds an `rm` command at run time
  codemode-script-literals.ts      # Reads the string and template literals one script spells
  path-guard.ts                    # `ignore`-backed protected-path matching
  path-normalize.ts                # Tool path extraction and cwd-relative normalization
  config.ts                        # Global config loader orchestration
  config-file.ts                   # Optional JSON config reading
  config-normalize.ts              # Config shape validation and normalization
  config-paths.ts                  # Global config path helper
  constants.ts                     # Built-in rules and messages
  value.ts                         # Shared value-shape helpers
  types.ts                         # Shared config and decision types
  *.unit.test.ts                   # Unit tests
  mise.verify-extension.ts         # Built extension smoke verification
  mise.verify-pi-runtime.ts        # Real Pi session in a throwaway repository
  pi-runtime-verifier-literal-rm.ts # The bash, write and codemode arms of that session
  pi-runtime-verifier-literal-rm-session.ts # The session, its code-mode extension, and its recorder
  pi-runtime-verifier-session.ts   # Real agent session running scripted Bash calls
  pi-runtime-verifier-scripted-provider.ts # Pi faux provider playing those calls
  pi-runtime-verifier-repository.ts # Throwaway Git repository and its reads
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/pi-plugin/guardrail:build
mise run //package/pi-plugin/guardrail:test:unit
mise run //package/pi-plugin/guardrail:lint
mise run //package/pi-plugin/guardrail:verify:extension
mise run //package/pi-plugin/guardrail:verify:pi-runtime
```

`verify:pi-runtime` drives a real Pi session in a throwaway repository.
 It runs two arms:
 the absolute real-Git sequence below,
 and a literal-`rm` sequence that refuses a computed deletion on `bash`,
 refuses a `write` of a violating script and lands a literal one,
 refuses a `codemode` script that builds its command,
 and shows a nested `bash` call refused inside a script that reads its command
 from a file.
 A recorder extension beside the guardrail logs every dispatched `tool_call`,
 and the run prints them, which is how the codemode answer above was measured.
 The session asks its Bash tool to run `/usr/bin/git commit`,
 which is refused with the remedy text,
 and then `/usr/bin/git status`,
 which runs and prints the fixture.
The same commit then runs through the PATH wrapper and lands,
 which shows the refused call would otherwise have committed.
