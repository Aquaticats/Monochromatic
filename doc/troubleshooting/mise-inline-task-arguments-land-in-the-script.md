# mise 2026.11.9 appends a task's `-- --help` arguments to its inline `node -e` script, so the task dies in TypeScript instead of receiving them

## Symptom

`mise run toolchain:install -- --help` fails before the task reaches its arguments:

```text
[//:toolchain:install] $ const { existsSync, readdirSync, statSync } = await import('node:fs')
[eval]:43
runMpmShadow(['install']) --help
                          ^^^^
SyntaxError [ERR_INVALID_TYPESCRIPT_SYNTAX]: Expected ';', '}' or <eof>
    at parseTypeScript (node:internal/modules/typescript:61:36)
```

The failure needs all three of these at once:

- the task declares a `usage` spec, so mise owns its arguments;
- its body runs under `shell = "node --input-type=module-typescript -e"`;
- the invocation passes `-h` or `--help` after `--`, as in `mise run <task> -- --help`.

Any other argument reaches the task as a `usage_*` environment variable and works. A
shebang or file task receives the same words as argv and works. Both catalogs are under
[Verification](#verification).

Two error variants exist, and the difference matters. A body whose last line is an
expression fails while node parses the script, so nothing runs (18 of the 22 root tasks
in this repository). A body that ends with a block or a statement instead reaches the
pasted words as code:

```text
[eval]:131
--help
^^
ReferenceError: help is not defined
```

That variant runs the body first, so a `buildAndTest`, `test` or
`fetch:rust-dependencies` invocation with `-- --help` builds, runs suites or fetches crates
before failing. See [What does not work](#what-does-not-work) for why the obvious fixes do
not close this.

## Root cause

mise documents two different argument paths for a TOML task, and a usage spec does not
always select the first one. From `docs/tasks/running-tasks.md`:

> - If `run` is an array, the arguments go only to its last entry.
> - For an inline command, mise appends the arguments to the command, quoted for the shell.
> - A shebang script or a file task receives them as script arguments, such as `$1` and `$@` in Bash.

([jdx/mise permalink, `docs/tasks/running-tasks.md` lines 164 to 166](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/docs/tasks/running-tasks.md#L164-L166).)

The first path is taken whenever the usage parser is bypassed. `Task::should_bypass_usage_parser`
decides that, and `-h`/`--help` after `--` bypasses it on purpose:

```rust
// src/task/mod.rs:1926-1939
    /// True when mise should not run the usage parser against this task's
    /// args. Either the task opted in via `raw_args = true`, or the user
    /// passed `--help`/`-h` after `--` for an ad-hoc passthrough.
    ///
    /// The `--` separator is the boundary between mise parsing and task
    /// parsing, so task-side help flags remain literal task arguments.
    pub(crate) fn should_bypass_usage_parser(&self) -> bool {
        if self.raw_args {
            return true;
        }
        self.trailing_args
            .iter()
            .any(|a| a == "--help" || a == "-h")
    }
```

([jdx/mise permalink, `src/task/mod.rs` lines 1926 to 1939](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/src/task/mod.rs#L1926-L1941).)

The bypass was introduced for `mise run task -- --help` by
[jdx/mise#9118](https://github.com/jdx/mise/pull/9118) (commit `c4b11b59`, 2026-04-16),
which also added `raw_args = true`. [jdx/mise#15573](https://github.com/jdx/mise/pull/15573)
later taught the *pre-`--`* help check to consult the spec, and left the post-`--` bypass
as it was.

With the parser bypassed, `TaskExecutor::get_cmd_program_and_args` appends the words to
the script text itself rather than passing argv:

```rust
// src/task/task_executor.rs:1954-1963
        // Shared (Unix, and Windows POSIX shells like `bash -c`): append the
        // forwarded args to the command string so they reach an inline `-c` shell
        // as part of the command — the documented behavior for inline TOML
        // scripts — rather than as positional parameters. Passing them as separate
        // argv to `bash -c` on Windows shifted the user's first arg into `$0`.
        // See #9355.
        let mut script = script.to_string();
        if !args.is_empty() {
            script = format!("{script} {}", shell_words::join(args));
        }
```

([jdx/mise permalink, `src/task/task_executor.rs` lines 1954 to 1963](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/src/task/task_executor.rs#L1954-L1965).)

`inline_args_style` decides that appending is a POSIX command text edit on Unix for every
inline shell, because only Windows (and a `cmd.exe` program) branches into
`SeparateArgv`:

```rust
// src/task/task_executor.rs:349-355
    #[cfg(windows)]
    {
        if !crate::path::is_posix_shell_program(Path::new(program)) {
            return InlineArgsStyle::SeparateArgv;
        }
    }
    #[cfg(not(windows))]
    let _ = (program, shell_args);
    InlineArgsStyle::PosixCommandText
```

So on Unix the last shell argument is `node --input-type=module-typescript -e`'s payload,
and the appended words land inside the evaluated TypeScript. `node` never sees them as
argv: with `-e`, `process.argv` is `["/usr/bin/node"]` (measured, see
[Verification](#verification)), and the eval source is what carries the words.

The repository's own `doc/decision/mise-task-node-invocations.md` recorded an earlier
reading that is wrong for this case: "A task with a plain `run` and no `usage` spec
receives post-`--` arguments from mise appended as a real argv vector, preserving spaces.
A `usage` spec instead consumes the args into `usage_*` env vars and they do not append."
That holds only while the usage parser runs. The e2e test that pins the bypass says the
opposite for help flags, and prints the mechanism our reproduction shows:

```sh
# e2e/tasks/test_task_raw_args:36-38
# Bug fix: `-- --help` now bypasses the usage parser even without raw_args.
# Previously the usage crate intercepted --help regardless of `--`.
assert "mise run normal -- --help" "got: --help"
```

([jdx/mise permalink, `e2e/tasks/test_task_raw_args` lines 37 to 38](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/e2e/tasks/test_task_raw_args#L37-L38).)

That is not a regression since 2026.11.9: the bypass dates from 2026-04-16, and the
append path is older still. What exposes it here is a task body written in TypeScript,
which mise's inline-command append was never intended to feed.

## Verification

Tested with mise 2026.11.9 (linux-x64, released 2026-11-17) and node v26.13.1. Source read
at jdx/mise 2026.11.11, commit `55853bbade5f1194731b4a47d17b4c85b8378066`; every citation
above was loaded in `agent-browser` and shows the quoted text.

Harness, in a scratch directory of its own:

```toml
# mise.toml
[tasks.probe]
usage = 'arg "[args]" var=#true help="x"'
shell = "node --input-type=module-typescript -e"
run = """
const source = process.execArgv.at(-1) ?? ''
console.log('tail40:', JSON.stringify(source.slice(-40)), 'usage_args:', JSON.stringify(process.env.usage_args))
// mise appends forwarded arguments after this comment
"""
```

Commands that work cleanly:

- `mise run probe`: `tail40` ends at the marker, `usage_args` is undefined, and
  `process.argv` is `["/usr/bin/node"]`.
- `mise run probe -- alpha 'two words'`: node evaluates the script unchanged, and
  `usage_args` is `"alpha 'two words'"`, so the words arrive shell-escaped as designed.
- `mise run probe -- --foo` and `mise run probe -- -x`: same path, `usage_args` is
  `"--foo"` / `"-x"`. Only the two help spellings bypass the parser.
- `mise run probe --help` (without `--`): mise answers with its own task help, as
  `docs/tasks/task-configuration.md` lines 230 to 233 describe
  ([permalink](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/docs/tasks/task-configuration.md#L230-L233)).

Commands that fail, by variant:

- `mise run probe -- --help` and `mise run probe -- -h` where the body's last line is an
  expression: `SyntaxError [ERR_INVALID_TYPESCRIPT_SYNTAX]: Expected ';', '}' or <eof>`,
  with the pasted words visible in node's `[eval]` output, for example
  `show() --help`.
- The same invocation where the body ends with a block or a `;` statement:
  `ReferenceError: help is not defined`, printed after the body's own work finished.

Repository-wide catalog, run with `-- --help` on the branch of this fix, each task under a
preloaded `node:child_process` stub that records calls instead of making them so nothing
had effects: all 22 root tasks that interpolate `vars.parse_usage_args` failed before the
workaround (18 with the syntax error, 4 with the reference error), and after it all 22
recovered `--help` into the argument vector that reaches their program (21 through a
recorded child-process call, `lint:skills` by naming `--help` as a skill root). The
details are in the report that accompanies the fix commit, and the durable checks live in
`package/monorepo-task/core-exec/src/root-task-shared-shapes.unit.test.ts`.

## Verified workarounds

### End the body with a marker comment and read the words from node's eval source

This repository's fix. The last line of the body renders as a comment, so the appended
words are comment text instead of code, and the parser reads them back from
`process.execArgv.at(-1)`, which holds the whole eval source:

```ts
// [vars].parse_usage_args, abridged; mise.toml in this repository
const forwardedArgumentsMarker = '// mise appends forwarded arguments after this comment'
const forwardedArgumentWords = () => {
  const source = process.execArgv.at(-1) ?? ''
  const markerLine = source.slice(source.lastIndexOf('\n') + 1)
  if (!markerLine.startsWith(forwardedArgumentsMarker)) { return '' }
  return markerLine.slice(forwardedArgumentsMarker.length)
}
const usageArgWords = () => process.env.usage_args ?? forwardedArgumentWords()
```

Tradeoffs: every body that wants the words must end with the marker line, and the whole
weight of the workaround sits in one shared variable; it depends on mise trimming the
script it evaluates, so the words land on the marker's own line (measured above), and on
`node` exposing its eval source through `process.execArgv`; it only recovers words, not
the `usage_*` variables a real parse would have set, so a task whose spec has named
arguments reads its own `usage_<name>` first and the recovered words as the fallback; and
a body without the marker silently receives no words, which is only visible when mise
appended some (it then fails as TypeScript, as before). Remove it when mise forwards
arguments as separate argv to a non-shell interpreter.

### Make the task a file task

`docs/tasks/running-tasks.md` gives file and shebang tasks real argv. A TOML task can name
a script with `file = "..."` ([`task-configuration.md` lines 116 to 129](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/docs/tasks/task-configuration.md#L116-L129)), or
a `mise-tasks/` file task can carry `#USAGE` lines. Tradeoffs: the logic leaves the task
file and becomes a script with an extension, so the repository's inline-script rule
(`SCR` in `AGENTS.md`) forbids it here; and an inline `run` that starts with `#!` does not
help, because mise writes it to an extensionless temporary file which node then runs as
CommonJS without type stripping.

### Ask for help without the separator

`mise run <task> --help` prints mise's own help, generated from the task's usage spec, and
never appends anything. Tradeoffs: that is the task's interface, not the wrapped
program's; a tool's own `--help` still needs one of the other two routes.

## What does not work

- `raw_args = true`: it bypasses the usage parser for every invocation, so every argument
  lands in the script text. The issue gets worse, not better.
- Ending the body with an unterminated block comment or a quote: node reports a syntax
  error for the unterminated construct, which is the same failure with a different message.
- Reading arguments from `process.argv`: with `-e` mise passes no argv. Measured:
  `process.argv` is `["/usr/bin/node"]` while `process.execArgv` is the eval source.
- An inline `run` that starts with `#!/usr/bin/env -S node --input-type=module-typescript`:
  mise writes it to a temporary file named `script`, and `--input-type` applies only to
  `--eval` and stdin, so node treats the file as CommonJS and the repository's typed
  bodies fail with `SyntaxError: Missing initializer in destructuring declaration`.
  Forwarding works, the TypeScript does not.
- Any `shell` whose program is a real shell, for example `shell = "sh -c 'node ... -e
  \"$0\"'"`: mise still appends to the script text, so the words still end up inside the
  evaluated code, and the shell string is no longer portable to Windows.
- Relying on mise's display: `mise run --dry-run <task> -- --help` shows the same command
  before and after the workaround, because the workaround changes the body, not mise's
  append. Only running the task shows the arguments arriving.

## Upstream filing decision

Checked `.out-of-scope/` in this repository first: it holds entries for bun, cargo
workspace, Claude Code, Codex, JSR, lightningcss, low-impact TypeScript formatting, the
module ES monolith, pi, terminal-title fork parity tests and TypeScript project references,
and no mise entry, so mise tracking is not exempt.

Duplicate search, 2026-11-20, on jdx/mise: `gh search issues` for `node --input-type`,
`shell = "node -e"`, `usage_args`, `appended to the command`, `raw_args` and `args after --`
returned one unrelated hit ([#4379](https://github.com/jdx/mise/issues/4379), a `run` array
and flag error) and nothing matching. A positive control (`gh search issues ... 'tasks'`)
returned hits, so the empty results are the search's answer, not a broken query. Work in
progress: `gh pr list --state all --search 'task_executor inline args'` and the recent
commit list of `src/task/task_executor.rs` show no change to this path after 2026-11-20.
Prior art read in full: [#9118](https://github.com/jdx/mise/pull/9118) (adds the bypass),
[#15573](https://github.com/jdx/mise/pull/15573) (spec decides the pre-`--` help check),
[#12277](https://github.com/jdx/mise/pull/12277) (forwards a file task's arguments through a
`-c` shell, a neighbouring bug with a different shape).

The six constraints, walked explicitly:

1. Is it really upstream's fault? Yes as a mechanism, not as a mistake: mise appends to an
   inline command by documented design, and its own e2e test pins `-- --help` forwarding.
2. Can upstream fix it? Yes. `InlineArgsStyle::SeparateArgv` already exists for Windows
   programs that are not POSIX shells, and the same decision could cover a non-shell
   `-e`-style program on Unix.
3. Are they supporting this use case? No. Nothing in the docs or tests covers a usage spec
   plus a non-shell interpreter receiving forwarded words as argv, and the e2e comment in
   `test_task_raw_args` states the split as intended ("unlike inline TOML scripts where
   args are appended to the command string",
   [lines 49 to 50](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/e2e/tasks/test_task_raw_args#L49-L50)).
   [`docs/contributing.md` lines 13 to 16](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/docs/contributing.md#L8-L16)
   also says bug reports are only for problems run into while using mise, which this one
   is, so a report would not be refused on that ground; the missing support is the
   constraint that fails.
4. Would the repo welcome our contribution? For an issue, yes, with the disclosure line
   [`docs/contributing.md` lines 21 to 26](https://github.com/jdx/mise/blob/55853bbade5f1194731b4a47d17b4c85b8378066/docs/contributing.md#L8-L16)
   requires, and not for a pull request: the same page says "mise does not accept external
   pull requests, and GitHub does not let anyone outside the maintainer team create one."
5. Will they likely fix it? No signal either way: no matching issue, no recent churn in the
   path, and no stated non-goal.
6. Have we prototyped a minimal fix compatible with their architecture? No. Since
   constraint 3 fails, the audit stops before the auto-prototype step; a patch could not be
   contributed as a pull request in any case.

Decision: do not file as-is, and keep no draft. Filing would also put the poster's account
in front of the maintainers over a use case their documentation does not claim to support,
which is the risk this decision avoids; a future session may revisit if mise ever documents
argument forwarding for a non-shell interpreter, in which case the report would need the
reproduction and a maintainer-side patch sketch.

## Related

- `doc/decision/mise-task-node-invocations.md` records the inline-script shapes and now
  carries a correction pointing here.
- `doc/troubleshooting/node-eval-module-detection.md` covers why these bodies spell the
  shell as `node --input-type=module-typescript -e`.
