# timeout-cli 0.1.0 consumes `--version`, `--help`, `-v`, and `-k` when they directly follow the wrapped command

This document records an investigation done on 2026-11-01.
The `timeout` first on this repository's `PATH` is `timeout-cli` 0.1.0,
not GNU coreutils.
Every option that `timeout-cli` defines is still parsed when it is the first argument after the wrapped command,
so the command never receives that argument.
No `mise.toml` task triggers it today;
ad hoc commands such as `timeout 60 command-code --version` do.

Note 2026-11-09T16:55:17-05:00:
`package/git-policy/cli/src/native/binary_support.rs` is now
`package/git-policy/cli/src/native/binary/binary_support.rs`,
moved 2026-11-03 in `3ec1e1201b`.

## Symptom

A flag that spells one of `timeout-cli`'s own options,
placed as the first argument after the wrapped command,
is consumed by `timeout`.
The wrapped command either does not run or runs without that argument.

Host reproduction on 2026-11-01 (Fedora 44,
`PATH` as the repository toolchain sets it),
with `/usr/bin/true` from GNU coreutils 9.10 as the wrapped command:

```sh
# shell on the host, run from the main checkout of this repository
command -v timeout
timeout 5 /usr/bin/true --version
/usr/bin/timeout 5 /usr/bin/true --version
```

```text
<main checkout>/.cache/toolchain-bin/timeout
timeout 0.1.0
true (GNU coreutils) 9.10
```

The first `timeout` printed its own version and exited 0,
and `true` never ran.
The incident that started this investigation was `timeout 60 command-code --version` printing `timeout 0.1.0`
instead of the tool's version,
reported on 2026-11-01 and not re-run here.
A session on 2026-10-31 saw the same with `timeout 60 vibe --version`
(`doc/handover/file-enforcer-rust-rewrite.md:1956-1959`,
which recorded it as not yet investigated).

The failure has three variants,
split by what the consumed flag does.
The harness output in "Patterns that fail" shows each.

- Information flags (`-V`,
  `--version`,
  `-h`,
  `--help`):
  `timeout` prints its own version or help text on stdout and exits 0.
  The wrapped command never runs.
  The result looks like success,
  and the text is a plausible version string.
- Verbose flag (`-v`,
  `--verbose`):
  the command runs without the flag,
  and `timeout` writes `DEBUG:` lines to stderr.
  A repeated `-vv` exits 2 with `the argument '--verbose' cannot be used multiple times`.
- Kill-after flag (`-k N`,
  `--kill-after N`,
  `--kill-after=N`):
  the command runs without the flag and its value,
  and `timeout` applies the kill-after itself.
  A missing or non-numeric value exits 2 before the command runs.

Common tools show how this goes wrong in practice (real programs,
run on 2026-11-01 in the harness container):

- `timeout 5 df -h /` prints `timeout-cli`'s help text and exits 0,
  with no filesystem table.
- `timeout 5 grep -v . /etc/hostname` runs `grep` without `-v`,
  so it prints the matching line and exits 0 where GNU `timeout` gives no output and exit 1.
  The selection is inverted without any error.
- `timeout 5 du -k /etc/hostname` exits 2 with `invalid value '/etc/hostname' for '--kill-after <KILL_AFTER>'`.

What does not trigger it:

- A flag that is not the first argument after the command:
  `timeout 5 tool hello --version` reaches `tool` intact.
- A first argument that `timeout-cli` does not define:
  `--model x --print hello`,
  `--pid=1 --follow /dev/null`,
  `-f`,
  `-o`.
- A combined short group with at least one character `timeout-cli` does not define:
  `-lv` and `-Vx` pass through,
  while `-v` and `-vv` do not.

## Root cause

The fault is the argument layout in `timeout-cli`.
`clap` 4.5.43 behaves as its own documentation and maintainers describe,
and the same behavior holds on `clap` 4.6.7,
the newest 4.x release.
It is not a `clap` defect,
and a report belongs to `timeout-cli`.

### The argument layout

The wrapped command and its arguments are two separate positionals,
and only the second carries `trailing_var_arg`
([`src/main.rs:10-29`][tc-args],
the source that crates.io publishes as 0.1.0):

```rust
// src/main.rs:10-29 at e7be8fab08ef6efcb3423f26730a966dae7c25b1
#[derive(Parser)]
#[command(name = "timeout")]
#[command(about = "Run a command with a timeout")]
#[command(version)]
struct Args {
    #[arg(help = "Number of seconds to wait before timing out")]
    seconds: u64,

    #[arg(short = 'k', long = "kill-after", help = "Also send KILL signal after this many seconds")]
    kill_after: Option<u64>,

    #[arg(short = 'v', long = "verbose", help = "Print debug information")]
    verbose: bool,

    #[arg(help = "Command to execute", required = true)]
    command: String,

    #[arg(help = "Arguments for the command", trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}
```

`clap` adds `-h, --help` to every command by default and adds `-V, --version` because of `#[command(version)]`
([`command.rs:4722-4749`][clap-auto-flags]):

```rust
// clap_builder/src/builder/command.rs:4739-4745
        if !self.is_disable_version_flag_set() {
            debug!("Command::_check_help_and_version: Building default --version");
            let arg = Arg::new(Id::VERSION)
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help("Print version");
```

So the keymap of the parser holds exactly four options for this binary:
`-k/--kill-after`,
`-v/--verbose`,
`-h/--help`,
and `-V/--version`.
The installed binary's own help output lists the same four.

### The parser path for a long flag

`clap` switches to "everything that follows is a value" only when a value lands in the positional that has
`trailing_var_arg` ([`parser.rs:402-404`][clap-trailing-flip]):

```rust
// clap_builder/src/parser/parser.rs:402-404
                if arg.is_trailing_var_arg_set() {
                    trailing_values = true;
                }
```

The word after `5` is the `command` positional,
which has no `trailing_var_arg`.
`command` takes one value,
so the parser leaves `ParseState::ValuesDone` behind it
([`parser.rs:422-428`][clap-single-value-state]):

```rust
// clap_builder/src/parser/parser.rs:422-428
                        // Only increment the positional counter if it doesn't allow multiples
                        if !arg.is_multiple() {
                            pos_counter += 1;
                            ParseState::ValuesDone
                        } else {
                            ParseState::Pos(arg.get_id().clone())
                        }
```

The word `--version` after it therefore reaches `parse_long_arg` with `trailing_values` still false.
The early return that treats a flag as a value applies only when the parser state is an option or a positional
with `allow_hyphen_values` ([`parser.rs:768-773`][clap-long-prior-state]),
and the state is `ValuesDone`,
so it does not apply:

```rust
// clap_builder/src/parser/parser.rs:768-773
        #[allow(clippy::blocks_in_conditions)]
        if matches!(parse_state, ParseState::Opt(opt) | ParseState::Pos(opt) if
            self.cmd[opt].is_allow_hyphen_values_set())
        {
            debug!("Parser::parse_long_arg: prior arg accepts hyphenated values",);
            return Ok(ParseResult::MaybeHyphenValue);
        }
```

The keymap lookup then finds the registered flag first ([`parser.rs:791-793`][clap-long-keymap]):

```rust
// clap_builder/src/parser/parser.rs:791-793
        let arg = if let Some(arg) = self.cmd.get_keymap().get(long_arg) {
            debug!("Parser::parse_long_arg: Found valid arg or flag '{arg}'");
            Some((long_arg, arg))
```

For `--version` that arg's action is `ArgAction::Version`,
whose handler returns the version error
([`parser.rs:1314-1323`][clap-version-action]),
which `clap` prints to stdout with exit status 0.
`--help` does the same through `ArgAction::Help`.
`--verbose` and `--kill-after` take the same lookup and are recorded as `timeout-cli`'s own settings,
which is why the command then runs with those tokens missing.

### Why unknown flags pass through

Only a long flag that is not in the keymap reaches the fallback that makes `args` start
([`parser.rs:858-866`][clap-long-fallback]):

```rust
// clap_builder/src/parser/parser.rs:858-866
        } else if self
            .cmd
            .get_keymap()
            .get(&pos_counter)
            .map(|arg| arg.is_allow_hyphen_values_set() && !arg.is_last_set())
            .unwrap_or_default()
        {
            debug!("Parser::parse_long_args: positional at {pos_counter} allows hyphens");
            Ok(ParseResult::MaybeHyphenValue)
```

`pos_counter` now points at `args`,
which has `allow_hyphen_values`,
so an unknown flag such as `--model` becomes
the first value of `args`.
That value sets `trailing_values` through `parser.rs:402-404`,
and every word after it is a value.
This is why `timeout 5 tool --model x --print hello` works and `timeout 5 tool --version` does not,
and why `timeout 5 tool hello --version` works:
`hello` starts `args` first.

### Short flags

The short-flag branch has one extra rule:
a group passes through when any character in it is not a defined short flag
([`parser.rs:901-913`][clap-short-fallback]):

```rust
// clap_builder/src/parser/parser.rs:901-913
        } else if self
            .cmd
            .get_keymap()
            .get(&pos_counter)
            .map(|arg| arg.is_allow_hyphen_values_set() && !arg.is_last_set())
            .unwrap_or_default()
            && short_arg
                .clone()
                .any(|c| !c.map(|c| self.cmd.contains_short(c)).unwrap_or_default())
        {
            debug!("Parser::parse_short_args: positional at {pos_counter} allows hyphens");
            return Ok(ParseResult::MaybeHyphenValue);
        }
```

`-lv` and `-Vx` contain `l` or `x`,
so they pass through.
`-v`,
`-V`,
`-h`,
`-k`,
and `-vv` contain only defined characters,
so the keymap lookup in the same function consumes them.
The doc comment for `trailing_var_arg` states the intended scope:
"everything that follows should be captured by it",
and says a flag-like first value needs `allow_hyphen_values` ([`arg.rs:544-553`][clap-trailing-doc]).
Nothing in that text covers flags that arrive before the trailing positional starts.

### Maintainers' view in clap

This exact layout (an executable positional,
then a trailing variable argument) is clap issue [#1538][clap-1538].
A maintainer wrote that `clap` starts processing the trailing argument "as soon as its the next positional argument,
even if that means we can't process any flags" (emphasis removed) and raised the compatibility risk of starting it on
the first unknown flag,
because "any new flag added breaks compatibility" ([comment][clap-1538-epage]).
One commenter's workaround was to drop the executable positional and take it from the first item of the trailing
argument ([comment][clap-1538-workaround]).
That is also the fix in the prototype recorded under "Have we prototyped a minimal fix compatible with their
architecture?".
The issue was closed by pull request [#4187][clap-4187] (`fix(parser): Resolve problems around allow_hyphen_values`,
merged 2022-09-07).
Flags that the command defines are still parsed right after the executable positional on `clap` 4.5.43 and 4.6.7,
as the harness shows.
Issue [#6308][clap-6308] (open,
filed 2026-03-17) asks for the case where flags after a positional should be collected,
and describes `bun run foo.js --help` printing help instead.
It has one maintainer comment asking for a minimal reproduction.

### What GNU timeout does instead

GNU coreutils 9.10 starts option parsing with `+`,
which stops at the first non-option word,
the duration
([`src/timeout.c:522`][gnu-getopt]):

```c
// src/timeout.c:522
  while ((c = getopt_long (argc, argv, "+fk:ps:v", long_options, NULL))
```

Everything after the duration,
including the command's own flags,
belongs to the command.
That is the habit every caller of `timeout` has,
and `timeout-cli` does not share it.

### A reading that was wrong

The first hypothesis was that `clap`'s automatic `--version` and `--help` flags were still recognized after the
trailing command.
The catalog disproves the narrow form.
`-v` and `-k` are options that `timeout-cli` declares itself,
and they are consumed by the same path.
The flags are also not "after the trailing command":
the trailing positional,
`args`,
has not started when they arrive.
They sit between `command` and `args`.

## Verification

### Version under test

- `timeout-cli` 0.1.0,
  the only release on crates.io,
  published 2025-08-12.
  The `.crate` archive checksum is `e5e3bf8a86dc5c449a820b953bd23ade441c9c5d589159eacd839cec75911102`
  (downloaded from `https://crates.io/api/v1/crates/timeout-cli/0.1.0/download` and checksummed on 2026-11-01).
- The archive records its source commit as `e7be8fab08ef6efcb3423f26730a966dae7c25b1`,
  and its `src/main.rs` is byte-identical to that commit's
  (sha256 `40459a2830d7aa9693ebf8ccb3443feae8e1ec455a151d00eafccf973a817e64`).
  The tip of `main`,
  `f11480d258107d3724a20fe634b61db3bf982248`,
  differs in `src/main.rs` only by `rustfmt`
  reflow (the `Args` struct is equal after whitespace normalization),
  so the line numbers in this document cite `e7be8fa`.
- The installed binary is `~/.local/share/monochromatic-dev/toolchain/cargo/timeout-cli/0.1.0/bin/timeout`,
  sha256 `2abb217a09b80f293d677b4f38281c4d60e299da87bdd3f839cd08689a4c6e36`,
  linked from `.cache/toolchain-bin/timeout` in the main checkout.
  Its `.crates2.json` records a source build (`rustc 1.101.0-nightly (19833af72 2026-10-20)`,
  profile `release`),
  not a prebuilt download.
  `strings` on the binary shows `clap_builder-4.5.43` and `clap_lex-0.7.5`,
  which match `Cargo.lock` at `e7be8fa` ([`Cargo.lock:98-99`][tc-lock]).
- `clap_builder` 4.5.43 from crates.io is identical to the `v4.5.43` tag checkout
  (commit `520beb5ec2d2bb5dd11912d27127df4e97027965`) for `parser.rs`,
  `arg.rs`,
  and `command.rs`,
  so the `clap` citations use that commit.
- Comparison implementations:
  GNU coreutils 9.7 (Debian trixie,
  inside the container),
  GNU coreutils 9.10 (host),
  and uutils `coreutils` 0.12.0 (`timeout` applet,
  the repository toolchain's `cargo:coreutils`).

### Harness

Each case runs in its own container:
`docker.io/library/rust:1-trixie` (`rustc 1.99.1 (58cf8f9c3 2026-10-13)`,
manifest `sha256:d841af61827b29b3efa46f5363e8f2e972b379cc0463c319e8f320d9f0c8eb0c`),
with `--network none --cpus 1 --memory 128m --pids-limit 32 --read-only`,
the harness directory as the only mount (read-only),
and no environment variable passed in.
`--security-opt label=disable` is needed because the first compile failed with
`permission denied when opening file` on the mounted scratch directory without it;
SELinux labeling is the likely cause and was not checked in the audit log.
The wrapped command is a stand-in that prints its argv,
because real tools handle `--version` themselves
(GNU `echo --version` prints coreutils' version,
so it cannot show whether the flag arrived).

Compile the stand-in once,
in a capped container,
then run the harness with `node harness.ts <group> [subject set]`:

```rust
// argv_dump.rs, compiled with: podman run --rm --network none --cpus 1 --memory 1g --pids-limit 128 ... rustc --edition 2024 -O
// Stand-in child for the timeout-cli harness: prints the argv it received, one entry per line.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("child-argv-count={}", args.len());
    for (index, arg) in args.iter().enumerate() {
        println!("child-argv[{index}]={arg}");
    }
}
```

```ts
// Harness for the timeout-cli 0.1.0 flag-consumption finding.
// Runs each case against the timeout implementations of one subject set, one capped, networkless, secret-free
// container per run. The child is usually a stand-in that prints its argv, so "the child ran and received X" is
// observable without a real tool.
// Usage: node harness.ts <group|all> [installed|prototype|latest-clap]
//   groups: clean, intercepted, workaround, repo, adjacent, realtools
//   subject sets: installed (default), prototype (unpatched vs patched build), latest-clap (clap 4.5.43 vs 4.6.7)
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';

const image = 'docker.io/library/rust:1-trixie';
const harnessDirectory = `${homedir()}/temp/agent/timeout-cli/h`;
const childPath = '/opt/h/argv-dump';

type Subject = { readonly label: string; readonly invocation: readonly string[]; };

const subjectSets: Readonly<Record<string, readonly Subject[]>> = {
  installed: [
    { label: 'timeout-cli', invocation: ['/opt/h/timeout-cli',], },
    { label: 'coreutils  ', invocation: ['/usr/bin/timeout',], },
    { label: 'uutils     ', invocation: ['/opt/h/uutils-coreutils', 'timeout',], },
  ],
  prototype: [
    { label: 'baseline-build', invocation: ['/opt/h/timeout-baseline-build',], },
    { label: 'patched-build ', invocation: ['/opt/h/timeout-patched',], },
    { label: 'coreutils     ', invocation: ['/usr/bin/timeout',], },
  ],
  'latest-clap': [
    { label: 'installed-clap-4.5.43', invocation: ['/opt/h/timeout-cli',], },
    { label: 'rebuilt-clap-4.6.7   ', invocation: ['/opt/h/timeout-clap-4.6.7-build',], },
  ],
};

// Each case is the argument vector after `timeout`; the token CHILD stands for the argv-dump stand-in.
const groups: Readonly<Record<string, readonly (readonly string[])[]>> = {
  clean: [
    ['5', 'CHILD',],
    ['5', 'CHILD', 'hello', '--version',],
    ['5', 'CHILD', 'hello', '-v', '-k', '3', '--help',],
    ['5', 'CHILD', '--model', 'x', '--print', 'hello',],
    ['5', 'CHILD', '--pid=1', '--follow', '/dev/null',],
    ['5', 'CHILD', '-f',],
    ['5', 'CHILD', '-lv',],
    ['5', 'CHILD', '-Vx',],
    ['5', 'CHILD', '--unknown', '--version',],
    ['5', 'CHILD', '--unknown', '-v', '-k', '3',],
    ['1', '/usr/bin/sleep', '3',],
  ],
  intercepted: [
    ['5', 'CHILD', '--version',],
    ['5', 'CHILD', '-V',],
    ['5', 'CHILD', '--help',],
    ['5', 'CHILD', '-h',],
    ['5', 'CHILD', '--verbose',],
    ['5', 'CHILD', '-v',],
    ['5', 'CHILD', '-v', 'hello',],
    ['5', 'CHILD', '-vv',],
    ['5', 'CHILD', '--verbose', '--version',],
    ['5', 'CHILD', '-k', '3',],
    ['5', 'CHILD', '--kill-after', '3',],
    ['5', 'CHILD', '--kill-after=3', 'hello',],
    ['5', 'CHILD', '-k',],
    ['5', 'CHILD', '-k', 'abc',],
  ],
  workaround: [
    ['--', '5', 'CHILD', '--version',],
    ['--', '5', 'CHILD', '-v', 'hello',],
    ['-k', '3', '--', '5', 'CHILD', '--version',],
    ['5', '--', 'CHILD', '--version',],
    ['5', 'CHILD', '--', '--version',],
    ['5', 'CHILD', '--', '-v', 'hello',],
    ['5', 'env', 'CHILD', '--version',],
    ['5', 'env', 'CHILD', '-v',],
    ['5', '/usr/bin/env', '--', 'CHILD', '--version',],
  ],
  repo: [
    ['3600', 'CHILD', '--model', 'openai/gpt-6.1-sol', '--print', '--no-tools', '--no-skills', '--no-themes', '--thinking', 'xhigh', 'question text',],
    ['600', 'CHILD', '--pid=12345', '--follow', '/dev/null',],
    ['35', 'CHILD', '--time1', 'input',],
    ['20', 'CHILD', '-o', 'BatchMode=yes', 'm1', 'pmset -g thermlog',],
    ['60', 'CHILD', '--version',],
  ],
  adjacent: [
    ['10s', 'CHILD', 'hello',],
    ['5.5', 'CHILD', 'hello',],
    ['--signal=KILL', '5', 'CHILD', 'hello',],
    ['--preserve-status', '5', 'CHILD', 'hello',],
    ['--foreground', '5', 'CHILD', 'hello',],
    ['0', 'CHILD', 'hello',],
  ],
  // Real coreutils programs in place of the stand-in. The output is the program's own, so only stdout and exit show.
  realtools: [
    ['5', '/usr/bin/df', '-h', '/',],
    ['5', '/usr/bin/grep', '-v', '.', '/etc/hostname',],
    ['5', '/usr/bin/du', '-k', '/etc/hostname',],
  ],
};

const clip = (text: string, limit: number,): string => text.length > limit ? `${text.slice(0, limit,)}...` : text;

const runCase = (subject: Subject, args: readonly string[],): string => {
  const resolved = args.map((token,) => token === 'CHILD' ? childPath : token,);
  const result = spawnSync('podman', [
    'run', '--rm', '--network', 'none', '--security-opt', 'label=disable',
    '--cpus', '1', '--memory', '128m', '--pids-limit', '32', '--read-only',
    '--volume', `${harnessDirectory}:/opt/h:ro`, image, ...subject.invocation, ...resolved,
  ], {
    encoding: 'utf8',
    timeout: 60000,
    // No token or credential from the caller's environment reaches podman or the container.
    env: { PATH: process.env['PATH'] ?? '', HOME: process.env['HOME'] ?? '', XDG_RUNTIME_DIR: process.env['XDG_RUNTIME_DIR'] ?? '', },
  },);
  const stdoutLines = result.stdout.split('\n',).filter((line,) => line.length > 0,);
  const stderrFirst = result.stderr.split('\n',).find((line,) => line.length > 0,) ?? '';
  const ranChild = stdoutLines[0]?.startsWith('child-argv-count=',) ?? false;
  const childArgv = stdoutLines.filter((line,) => line.startsWith('child-argv[',),).map((line,) => line.slice(line.indexOf('=',) + 1,),);
  const outcome = ranChild
    ? `child=ran argv=${JSON.stringify(childArgv,)}`
    : `stdout=${JSON.stringify(clip(stdoutLines[0] ?? '', 70,),)} stdout-lines=${String(stdoutLines.length,)}`;
  const stderrNote = stderrFirst.length > 0 ? ` stderr=${JSON.stringify(clip(stderrFirst, 90,),)}` : '';
  return `${subject.label} exit=${String(result.status,).padEnd(3,)} ${outcome}${stderrNote}`;
};

const requested = process.argv[2] ?? 'all';
const subjectSetName = process.argv[3] ?? 'installed';
const subjects = subjectSets[subjectSetName];
if (subjects === undefined) throw new Error(`unknown subject set ${subjectSetName}`,);
const selected = requested === 'all' ? Object.keys(groups,) : [requested,];
for (const groupName of selected) {
  const cases = groups[groupName];
  if (cases === undefined) throw new Error(`unknown group ${groupName}`,);
  console.log(`## ${groupName}`,);
  for (const args of cases) {
    console.log(`$ timeout ${args.join(' ',)}`,);
    for (const subject of subjects) console.log(`  ${runCase(subject, args,)}`,);
  }
}
```

The harness directory held `argv-dump` (the compiled stand-in),
`timeout-cli` (a copy of the installed binary),
`uutils-coreutils` (a copy of the toolchain's `coreutils` 0.12.0),
and for the comparison sets the two builds
described in "The same behavior on the newest clap" and in the filing artifact.
Positive controls:
every `clean` case behaves identically on all three implementations,
and `timeout 1 /usr/bin/sleep 3` exits 124 on all three,
so a difference elsewhere is not a harness artifact.

### Patterns that work cleanly

Output of `node harness.ts clean` (rows for `timeout-cli` and GNU coreutils 9.7;
uutils matched coreutils in every row):

```text
$ timeout 5 CHILD hello --version
  timeout-cli exit=0   child=ran argv=["hello","--version"]
  coreutils   exit=0   child=ran argv=["hello","--version"]
$ timeout 5 CHILD --model x --print hello
  timeout-cli exit=0   child=ran argv=["--model","x","--print","hello"]
  coreutils   exit=0   child=ran argv=["--model","x","--print","hello"]
$ timeout 5 CHILD --pid=1 --follow /dev/null
  timeout-cli exit=0   child=ran argv=["--pid=1","--follow","/dev/null"]
  coreutils   exit=0   child=ran argv=["--pid=1","--follow","/dev/null"]
$ timeout 5 CHILD -lv
  timeout-cli exit=0   child=ran argv=["-lv"]
  coreutils   exit=0   child=ran argv=["-lv"]
$ timeout 5 CHILD -Vx
  timeout-cli exit=0   child=ran argv=["-Vx"]
  coreutils   exit=0   child=ran argv=["-Vx"]
$ timeout 5 CHILD --unknown --version
  timeout-cli exit=0   child=ran argv=["--unknown","--version"]
  coreutils   exit=0   child=ran argv=["--unknown","--version"]
$ timeout 5 CHILD --unknown -v -k 3
  timeout-cli exit=0   child=ran argv=["--unknown","-v","-k","3"]
  coreutils   exit=0   child=ran argv=["--unknown","-v","-k","3"]
```

The last two rows show that one unknown flag in first position protects every flag after it,
as the trace predicts.

### Patterns that fail

Output of `node harness.ts intercepted`,
split by variant.
The `coreutils` row of each case shows the expected behavior;
the uutils row matched it in every case.

Information flags:
the command does not run and `timeout` exits 0.

```text
$ timeout 5 CHILD --version
  timeout-cli exit=0   stdout="timeout 0.1.0" stdout-lines=1
  coreutils   exit=0   child=ran argv=["--version"]
$ timeout 5 CHILD -V
  timeout-cli exit=0   stdout="timeout 0.1.0" stdout-lines=1
  coreutils   exit=0   child=ran argv=["-V"]
$ timeout 5 CHILD --help
  timeout-cli exit=0   stdout="Run a command with a timeout" stdout-lines=11
  coreutils   exit=0   child=ran argv=["--help"]
$ timeout 5 CHILD -h
  timeout-cli exit=0   stdout="Run a command with a timeout" stdout-lines=11
  coreutils   exit=0   child=ran argv=["-h"]
$ timeout 5 CHILD --verbose --version
  timeout-cli exit=0   stdout="timeout 0.1.0" stdout-lines=1
  coreutils   exit=0   child=ran argv=["--verbose","--version"]
```

Verbose flag:
the command runs without the flag,
with a `DEBUG:` line on stderr;
`-vv` fails.

```text
$ timeout 5 CHILD -v hello
  timeout-cli exit=0   child=ran argv=["hello"] stderr="DEBUG: Starting timeout: 5s, kill-after: Nones, command: /opt/h/argv-dump"
  coreutils   exit=0   child=ran argv=["-v","hello"]
$ timeout 5 CHILD -vv
  timeout-cli exit=2   stdout="" stdout-lines=0 stderr="error: the argument '--verbose' cannot be used multiple times"
  coreutils   exit=0   child=ran argv=["-vv"]
```

Kill-after flag:
the command runs without the flag and its value;
a missing or bad value exits 2.

```text
$ timeout 5 CHILD -k 3
  timeout-cli exit=0   child=ran argv=[]
  coreutils   exit=0   child=ran argv=["-k","3"]
$ timeout 5 CHILD --kill-after=3 hello
  timeout-cli exit=0   child=ran argv=["hello"]
  coreutils   exit=0   child=ran argv=["--kill-after=3","hello"]
$ timeout 5 CHILD -k abc
  timeout-cli exit=2   stdout="" stdout-lines=0 stderr="error: invalid value 'abc' for '--kill-after <KILL_AFTER>': invalid digit found in string"
  coreutils   exit=0   child=ran argv=["-k","abc"]
```

### Real programs

Output of `node harness.ts realtools` (real coreutils programs in the container,
no stand-in):

```text
$ timeout 5 /usr/bin/df -h /
  timeout-cli exit=0   stdout="Run a command with a timeout" stdout-lines=11
  coreutils   exit=0   stdout="Filesystem      Size  Used Avail Use% Mounted on" stdout-lines=2
$ timeout 5 /usr/bin/grep -v . /etc/hostname
  timeout-cli exit=0   stdout="cc2cd323adcf" stdout-lines=1 stderr="DEBUG: Starting timeout: 5s, kill-after: Nones, command: /usr/bin/grep"
  coreutils   exit=1   stdout="" stdout-lines=0
$ timeout 5 /usr/bin/du -k /etc/hostname
  timeout-cli exit=2   stdout="" stdout-lines=0 stderr="error: invalid value '/etc/hostname' for '--kill-after <KILL_AFTER>': invalid digit found ..."
  coreutils   exit=0   stdout="4\t/etc/hostname" stdout-lines=1
```

### The same behavior on the newest clap

The unpatched source of `main` was rebuilt in a capped container without its lockfile,
which resolved `clap` 4.6.7
(published 2026-09-14,
the newest release on crates.io on 2026-11-01) and `clap_lex` 1.1.1.
`node harness.ts intercepted latest-clap` and `node harness.ts clean latest-clap` gave identical rows for the
installed binary (`clap` 4.5.43) and the rebuild (`clap` 4.6.7) in every case.

### Adjacent differences from GNU timeout

These are other gaps of the same tool,
not this bug.
Output of `node harness.ts adjacent`:

```text
$ timeout 10s CHILD hello
  timeout-cli exit=2   stdout="" stdout-lines=0 stderr="error: invalid value '10s' for '<SECONDS>': invalid digit found in string"
$ timeout --signal=KILL 5 CHILD hello
  timeout-cli exit=2   stdout="" stdout-lines=0 stderr="error: unexpected argument '--signal' found"
$ timeout 0 CHILD hello
  timeout-cli exit=124 stdout="" stdout-lines=0
```

`--preserve-status` and `--foreground` fail like `--signal`,
and `5.5` fails like `10s`.
GNU `timeout` and uutils ran the command to completion with exit 0 in every case.
For `timeout 0` they ran it normally,
while `timeout-cli` ended the run at once with exit 124 and the stand-in printed nothing.

## Repository impact

### Why the repository installs timeout-cli

The entry is `cargo:timeout-cli` at `toolchain.plan.jsonc:214-220`:

```jsonc
// toolchain.plan.jsonc:214-220
    "cargo:timeout-cli": {
      "request": "cargo:timeout-cli@latest",
      "route": "cargo-binstall",
      "bins": [
        "timeout"
      ]
    },
```

The reason is recorded in one place.
Commit `1bd8b72911` (2025-11-21,
message `mise`) added `"cargo:timeout-cli" = "latest"` to `mise.toml`
under the comment `# For reliable timeout`,
alongside `rust` and `cargo-binstall`,
in the same commit as a `timeout.nu` nushell implementation,
a `task_timeout = "5s"` setting,
and tasks such as `timeout 5 pnpm exec eslint ...`.
That file also records `# asdf -> Doesn't support Windows` and a nushell dependency "for executing tasks cross-platform".
Cross-platform availability of a `timeout` command is therefore the likely motive,
but no commit message or document
states it,
so treat it as an inference.
The entry then moved through the toolchain generator commits (`a7e48350e3`,
`e7e3be50ab`,
`a5cdc3158d`)
and into the plan in `20f3c28eec` (2026-10-31),
none of which gives a new reason.

Today no `mise.toml` task invokes the `timeout` binary with a duration
(the pattern `(^|[^-_./a-zA-Z])timeout +[0-9$<{]` matched in no `mise.toml`).
As far as this search shows,
the tool is used only by agents and by hand,
and the documentation was already adjusted to it once:
commit `e6045beb05` (2026-08-16) changed `timeout 1h` to `timeout 3600` after `timeout-cli` rejected `1h`.

On the host,
`which -a timeout` lists the toolchain `timeout-cli`,
then the `mise` shim,
then `/usr/bin/timeout`.
The shim is a symlink to `mise` and resolved to GNU coreutils 9.10 when run on 2026-11-01
(`~/.local/share/mise/shims/timeout --version` printed `timeout (GNU coreutils) 9.10`,
exit 0).

### Repository commands checked

Searches on 2026-11-01 for `timeout` followed by a duration in `AGENTS.md`,
`.config/file-enforcer/claude-preamble.md`,
`.agents`,
`doc`,
every `mise.toml`,
`toolchain.plan.jsonc`,
and `package` source (`rg` with the pattern
`(^|[^-_./a-zA-Z])timeout +[0-9$<{]` plus a spawn-by-name search) found these prescribed commands.
The `repo` group of the harness ran their argument shapes against the stand-in on all three implementations.

- `.config/file-enforcer/claude-preamble.md:18`,
  `timeout 3600 pi --model openai/gpt-6.1-sol --print ...`:
  not affected,
  because the first argument after `pi` is `--model`,
  which `timeout-cli` does not define.
- `AGENTS.md:554` (and `package/agent-harness-shared/shell-command-analyzer/src/process-name-wait.ts:42`,
  which prints it),
  `timeout 600 tail --pid=<pid> --follow /dev/null`:
  not affected,
  because the first argument is `--pid=<pid>`.
- Other documented commands in `doc` have a non-flag first argument or an undefined flag first
  (`timeout 35 repro --time1 {}`,
  `timeout 20 ssh -o BatchMode=yes ...`,
  `timeout 1800 lake env lean ...`,
  `timeout 3 node ...`,
  `timeout 240 mise install rust`):
  not affected.
- `package/git-policy/cli/src/native/binary_support.rs:254` runs `/usr/bin/timeout --signal=KILL`
  by absolute path,
  so it never meets `timeout-cli`
  (which would reject `--signal` with exit 2,
  see "Adjacent differences from GNU timeout").

`package/git-policy/cli/src/native/binary_support.rs` moved on 2026-11-03 to
`package/git-policy/cli/src/native/binary/binary_support.rs` in `3ec1e1201b`
(refactor(git-policy-cli):
move the binary-level controls into a binary folder).

So no documented repository command is hit today.
Both prescribed commands would break silently if their first argument became `--help`,
`-h`,
`-v`,
`-V`,
or `-k`,
and any new ad hoc `timeout N <tool> --version` or `timeout N df -h` is hit.

### Options for the toolchain plan (not applied)

The plan and lock are unchanged.
Distinct options,
with the cost of each:

- Option A,
  keep `timeout-cli` and write the portable form (`timeout -- 5 tool --version`):
  no plan change and the form works on all three implementations.
  The hazard stays for every caller who forgets the `--`,
  and the failure is silent with exit 0.
- Option B,
  remove the `cargo:timeout-cli` entry:
  `timeout` then resolves through the `mise` shim to GNU coreutils
  (verified on this host only),
  which has the `--signal`,
  `--preserve-status`,
  and duration-suffix syntax the
  documentation elsewhere already assumes.
  No task uses the binary today.
  It drops the cross-platform motive,
  and this investigation did not check any macOS or Windows host that relies on it.
- Option C,
  pin a patched `timeout-cli` build:
  keeps the cross-platform binary and fixes this bug,
  but needs a fork or an upstream release first,
  and the upstream has been quiet since 2025-08-12.

Ranking:
B > A > C.
B outranks A because it removes the hazard at its source and the repository has no task that needs the binary,
while A leaves a silent failure for every future command.
A outranks C because A works today with no new maintenance,
while C waits on an upstream change or a fork to maintain.
The choice belongs to the repository owner.

### Decision and outcome

The owner chose on 2026-11-01,
in the question tool,
"uutils timeout (Recommended)",
a fourth option offered then:
replace `cargo:timeout-cli` with `cargo:uu_timeout` (uutils,
bin `timeout`).
It keeps a cross-platform binary and parses like GNU `timeout`.
Applied the same day:
the repository plan entry is now `cargo:uu_timeout`,
the lock moved one row (`timeout-cli` 0.1.0 out,
`uu_timeout` 0.12.0 in),
and `toolchain:install` reported 1 installed,
63 unchanged,
0 failed.
`.cache/toolchain-bin/timeout` now resolves to `toolchain/cargo/uu_timeout/0.12.0/bin/timeout`.
Positive check against `/usr/bin/timeout`:
`timeout 5 echo --version`,
`timeout 5 df -h /`,
`timeout 5 grep -v x <file>`
and `timeout 5 du -k <file>`
each gave the same exit status and output as GNU coreutils 9.10.
The owner also chose "Don't file (Recommended)" for the upstream report,
so the draft in this document stays unposted.

## Verified workarounds

All tested on 2026-11-01 with the harness `workaround` group,
on `timeout-cli` 0.1.0,
GNU coreutils 9.7,
and uutils 0.12.0.

### Put `--` before the duration (portable)

```sh
# works on timeout-cli, GNU coreutils, and uutils
timeout -- 5 tool --version
timeout -k 3 -- 5 tool --version
```

```text
$ timeout -- 5 CHILD --version
  timeout-cli exit=0   child=ran argv=["--version"]
  coreutils   exit=0   child=ran argv=["--version"]
  uutils      exit=0   child=ran argv=["--version"]
$ timeout -k 3 -- 5 CHILD --version
  timeout-cli exit=0   child=ran argv=["--version"]
  coreutils   exit=0   child=ran argv=["--version"]
  uutils      exit=0   child=ran argv=["--version"]
```

On `timeout-cli`,
`--` at the start makes every word after it positional,
so nothing is consumed.
Tradeoffs:
the placement is unfamiliar and easy to forget,
and the bug returns silently without it.
`timeout`'s own options must come before the `--` (`-k 3 --`),
because after it they are positional words.
The child's argv is clean on all three implementations.

### Call the system `timeout` by path

```sh
/usr/bin/timeout 5 tool --version
```

Run on the host 2026-11-01:
`/usr/bin/timeout 5 /usr/bin/true --version` printed `true (GNU coreutils) 9.10`.
`package/git-policy/cli/src/native/binary_support.rs:254` already does this.
Tradeoffs:
the path is hard-coded and exists only where coreutils installs it (Linux,
not stock macOS or Windows),
and it bypasses the repository-pinned tool,
so a `timeout` option that only `timeout-cli` has is not available.
`timeout-cli` has no such option,
so nothing is lost on this host.

### Run the command through `env`

```sh
timeout 5 env tool --version
```

The first argument after the command becomes `env`,
so the command's flags are never first.
It passed on all three implementations,
including `-v` (`timeout 5 env CHILD -v`).
Tradeoffs:
`env` must be on `PATH`,
and a command name containing `=` is read by `env` as an assignment.
Every command line gains a word,
and the wrapped command's name is no longer the first word that `timeout` sees,
so any tool that reads the `timeout` command line for the wrapped command's name sees `env` instead.

### Skip `timeout` for commands that cannot hang

`tool --version` and `df -h` finish immediately,
so the wrapper adds nothing.
This avoids the bug without changing the form of any other command.
Tradeoff:
no bound if the tool does hang,
which is the reason the wrapper exists.

## What does not work

- `timeout 5 tool -- --version`:
  works on `timeout-cli` (it swallows the `--`),
  but GNU coreutils and uutils forward the
  `--` to the command,
  so `tool` receives `["--","--version"]`.
  It is correct on one implementation and wrong on the other two.
- `timeout 5 -- tool --version`:
  works on `timeout-cli` and uutils,
  but GNU coreutils 9.7 exits 127 with
  `timeout: failed to run command '--': No such file or directory`,
  because its option parsing stops at the duration.
- A `timeout-cli` flag or environment variable that turns the consumption off:
  none exists.
  The binary defines exactly `-k`,
  `-v`,
  `-h`,
  and `-V`,
  and the source reads no environment variable.
- Blaming `clap` or filing against it:
  the behavior is the documented one,
  and clap issue #1538 records the design
  discussion.
- Checking only `--version` and `--help`:
  `-v` and `-k` are consumed too,
  and the `grep -v` inversion is the worst case.

## Not verified

- Behavior on macOS and Windows:
  the parsing code is platform independent,
  but no run was made there.
- The `command-code --version` incident itself was reported,
  not re-run;
  the tool was not run in this investigation.
- Whether a `timeout-cli` release newer than 0.1.0 exists:
  crates.io lists one version and the repository has one branch.
- Whether any host outside this machine depends on the toolchain's `timeout` for cross-platform tasks.
- The line permalinks into `timeout-cli`,
  `clap`,
  and GNU coreutils were each loaded in `agent-browser`,
  and the rendered text of the first and last cited line matched the local copy of that exact source
  (every one of them matched on 2026-11-01);
  the comment anchors of clap issue 1538 were confirmed by screenshots of the highlighted comment.
  `agent-browser eval` was not available in this session,
  so `get text` and screenshots were used instead.
- The `#:~:text=` fragment on the clap issue 6308 link could not be confirmed to scroll or highlight:
  a text fragment on a rendered README page behaved the same way in this headless browser,
  so the browser does not apply fragment directives to navigations it starts itself.
  The quoted sentence ("bun run foo.js --help should collect --help") was confirmed present in the issue body text.

## Upstream filing artifact

### Upstream filing decision

All six constraints hold,
with the prototype.
This document makes no recommendation to file:
readiness item 7 was not run (see "Readiness check").
Filing is the poster's decision and carries the risk named in "Risk to the poster".

#### Is it really upstream's fault?

Yes,
for `timeout-cli`.
The behavior is the program's argument layout,
not wording,
and not a `clap` restriction:
`clap` 4.5.43 and 4.6.7 behave as documented,
and a single trailing positional avoids the problem
(see the prototype).
The upstream README states that `timeout-cli` "Supports commands with flags" ([`README.md:153`][tc-readme-flags])
and "aims to be compatible with the standard Unix `timeout` command" ([`README.md:210`][tc-readme-compat]),
which this behavior contradicts.

#### Can upstream fix it?

Yes.
The fix is local to `src/main.rs`:
fold `command` and `args` into one trailing positional.
No architectural limit blocks it.

#### Are they supporting this use case?

Yes,
softly.
The README usage line is `timeout [OPTIONS] <SECONDS> <COMMAND> [ARGS]...` ([`README.md:54`][tc-readme-usage]),
and its feature list claims argument handling for commands with flags.
The only test of a command with flags is `ls -la` ([`tests/integration_tests.rs:39-44`][tc-test-flags]),
whose first argument is not a defined flag,
so the suite cannot see this bug.

#### Would the repo welcome our contribution?

No policy against it was found.
The repository has no `CONTRIBUTING.md`,
no issue template,
no pull request template,
no code of conduct,
and no policy text about AI-assisted work.
The README asks issue reporters for the operating system and version,
the command,
expected versus actual behavior,
and error messages ([`README.md:176-185`][tc-readme-issues]).
The packaged commit `e7be8fa` is itself marked `Generated with Claude Code` and `Co-Authored-By: Claude`,
so AI assistance is not alien to the project.
Checked 2026-11-01 against tip `f11480d`:
the repository tree (`gh api repos/yaleman/timeout-cli/git/trees/main?recursive=1`),
`.github/workflows/ci.yml` and `release.yml`,
`README.md`,
`CHANGELOG.md`,
and `RELEASE_CHECKLIST.md`.

#### Will they likely fix it?

Unknown,
and not a failure.
No document or maintainer statement declines it.
The repository has no issues and no pull requests (open or closed),
one branch,
no forks,
and its last push was
2025-08-12 (`pushed_at`),
so there is no signal either way.
It has one star and one maintainer.

#### Have we prototyped a minimal fix compatible with their architecture?

Yes.
The prototype folds the two positionals into one and is saved as
[`timeout-cli-consumes-flags-right-after-command.patch`](timeout-cli-consumes-flags-right-after-command.patch)
(it applies to `f11480d`,
the tip of `main`;
two files,
45 insertions and 15 deletions).
The disposable clone was created with `mktemp --directory` under the scratch root,
cloned with `gh repo clone`,
verified to have `origin` `https://github.com/yaleman/timeout-cli.git` at `f11480d258107d3724a20fe634b61db3bf982248`,
with its push URL set to `DISABLED`.
All cargo work ran in `docker.io/library/rust:1-trixie` with `--cpus 2 --memory 3g --pids-limit 512`,
only that clone mounted,
no environment variable passed,
and the network used only to fetch crates.

The core change:

```rust
// src/main.rs (the Args struct after the patch)
    // One trailing positional holds the command and its arguments, so the command's own flags
    // (such as `--version` or `-v`) are never matched against timeout's flags.
    #[arg(
        value_name = "COMMAND",
        help = "Command to execute, followed by its arguments",
        required = true,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    command_and_args: Vec<String>,
```

`main` then splits the first value off as the command.

Verification,
all on 2026-11-01:

- A regression test (`test_timeout_flags_right_after_the_command_belong_to_the_command`) runs `echo` with each of
  `-V`,
  `--version`,
  `-h`,
  `--help`,
  `-v`,
  `--verbose`,
  `-k 3`,
  `--kill-after 3`
  (always with a second word so GNU `echo` does not treat a lone `--help` as its own request)
  and asserts that stdout equals the words and stderr is empty.
  On unpatched `f11480d` it failed on the first case:
  `stdout="timeout 0.1.0\n"` where `-V x` was expected.
  On the patched tree it passed.
- The full suite on the patched tree:
  `test result: ok. 26 passed; 0 failed` (25 existing tests and the new one).
- `cargo fmt --all -- --check` and `cargo clippy --locked --all-targets --all-features -- -D warnings`
  (the upstream CI commands) both exited 0;
  the first run of `fmt` asked for one macro call to be reflowed,
  and the patch contains the reflowed form.
- The harness `prototype` set ran every group against an unpatched release build of `f11480d`,
  the patched build,
  and GNU coreutils.
  The patched build matched GNU coreutils in every `intercepted` and `repo` case
  (for example `-v hello` gave `argv=["-v","hello"]`,
  `--version` gave `argv=["--version"]`,
  `-k` and `-k abc` ran the command with those arguments).
  On the `realtools` group the patched build matched GNU coreutils for `df -h /` (a filesystem table),
  `grep -v . /etc/hostname` (exit 1,
  no output),
  and `du -k` (exit 0 with the size line),
  where the unpatched build printed its help text,
  inverted the `grep`,
  and exited 2.
  The `adjacent` rows were unchanged between the unpatched and patched builds,
  as intended.

One behavior change comes with the patch,
and the pull request should name it:
`timeout 5 tool -- --version` now forwards the `--` to the command (`argv=["--","--version"]`),
as GNU `timeout` does,
where the unpatched build swallowed it.
The harness `workaround` group showed this on the patched build;
`-- 5 tool --version` and `5 -- tool --version`
still work.

### Duplicate search

Run 2026-11-01 against `yaleman/timeout-cli`:

- `gh issue list --state all --limit 100` returned no issues.
- `gh pr list --state all --limit 100` returned no pull requests.
- `gh search issues` with `version`,
  `trailing`,
  and `help`,
  and `gh search prs` with `args`,
  each returned nothing
  (the repository has no issues or pull requests at all,
  so the empty search is expected).
- `gh api repos/yaleman/timeout-cli/forks` and `.../branches` returned no forks and only `main`.

No duplicate exists,
so the filing artifact is a new-issue draft.
In `clap-rs/clap`,
issues [#1538][clap-1538] (closed 2022-09-07),
[#6308][clap-6308] (open) and [#4187][clap-4187]
(the pull request that closed #1538) were read in full.
They cover the generic trailing-argument design,
not `timeout-cli`,
and they are context,
not duplicates.

### Work in progress check

Run 2026-11-01.
No open or draft pull request touches `src/main.rs` (there are no pull requests).
The only branch is `main`,
and there are no forks.
No issue exists to carry an "I'll take this" claim or an assignee.
The commit list for the repository (`gh api repos/yaleman/timeout-cli/commits`) shows seven commits,
the newest on 2025-08-12,
so there is no fresh maintainer churn in the code path.

### Trust and policy search

Run 2026-11-01 at commit `f11480d`.
The tree search `gh api 'repos/yaleman/timeout-cli/git/trees/main?recursive=1'` for `VOUCH`,
`DENOUNCE`,
`CODEOWNERS`,
`CONTRIBUTING`,
and `AI_POLICY` found none,
and the only files under `.github` are the two workflows.
`ci.yml` runs on pushes and pull requests to `main`;
`release.yml` builds release artifacts.
Neither contains a trust list,
an auto-close step,
or an issue-triggered job
(`rg` for `vouch|denounce|blocklist|auto-close|stale|issues:|pull_request_target` in both files found nothing).
The poster must check the live repository again on the day of posting.

### Out-of-scope check

`.out-of-scope/` was read on 2026-11-01:
`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`.
A search of the directory for `timeout-cli`,
`clap`,
`yaleman`,
`timeout`,
and `trailing_var_arg` matched only
unrelated uses of the word upstream.
No exemption covers `timeout-cli` or this bug class.

### Readiness check

Recorded 2026-11-01 against `timeout-cli` `f11480d` and `clap` 4.5.43 and 4.6.7.

1.  Latest release and other lines:
    `timeout-cli` has one release (0.1.0) and one branch.
    It reproduced on the installed 0.1.0 binary,
    on an unpatched build of `f11480d` with the locked `clap` 4.5.43,
    and on the same source rebuilt with `clap` 4.6.7,
    the newest 4.x on crates.io.
    Not run:
    macOS and Windows.
2.  Prior art:
    no merged or closed pull request or issue exists in `timeout-cli`,
    so there is no maintainer
    reason for the current behavior to argue with.
    The `clap` thread is cited under "Maintainers' view in clap".
3.  Intake:
    no template or policy exists.
    The README's requested fields are answered in the draft from what was run
    (operating systems,
    the command,
    expected versus actual,
    and the messages).
    The pull request is drafted together with the issue because a prototype passed its verification.
4.  Automation:
    none that acts on filings (see "Trust and policy search").
5.  Day-of re-run:
    the duplicate search,
    the work-in-progress check,
    and the trust search ran today,
    and must be re-run on the day of posting,
    together with a check that `src/main.rs:24-28` still exists on `main`.
6.  Draft reproduction run as written:
    done,
    and the output is recorded under this list.
7.  Runnable clean-machine reproduction (a minimal repository with a CI workflow):
    not run.
    It needs a new repository in the poster's GitHub account,
    which is a write outside this repository.
    The brief for this investigation limits the work to a local document
    ("Don't file anything upstream.
    The draft stays in the doc."),
    and `AGENTS.md` `PX3` lists what may be done without asking:
    "Act + report on local work,
    current-repo GitHub mutations,
    and solely user-controlled resources.
    Else need authorization;
    unsure -> ask."
    Whether a new private repository in the poster's account counts as solely user-controlled was unclear,
    so it was left for the poster.
    The draft's reproduction is a single `cargo install` followed by five commands,
    which any machine with Rust can run.
8.  Verdict:
    not recommended for filing from this document,
    for the reason in item 7.
    The blocker is the missing CI reproduction repository,
    and it clears when the poster authorizes a session to create it as a private repository,
    or decides it is not wanted.
    Everything else the agent can close is closed,
    and the draft needs no other change.

The draft's reproduction block,
copied exactly from the draft,
was run on 2026-11-01 in a fresh
`docker.io/library/rust:1-trixie` container (network on for crates only,
nothing mounted,
no environment passed):

```text
   Installed package `timeout-cli v0.1.0` (executable `timeout`)
timeout 0.1.0
timeout 0.1.0
DEBUG: Starting timeout: 5s, kill-after: Nones, command: /usr/bin/echo
DEBUG: Spawning command: /usr/bin/echo ["x"]
DEBUG: Waiting for command result...
DEBUG: Timeout thread started, sleeping for 5s
DEBUG: Command spawned successfully with PID: 379
x
DEBUG: Command exited with code: 0, term_sent: false
DEBUG: Process completed normally
DEBUG: Command result received: Completed(0)
DEBUG: Command completed normally with exit code 0
DEBUG: Exiting with code: ExitCode(unix_exit_status(0))
Run a command with a timeout

Usage: timeout [OPTIONS] <SECONDS> <COMMAND> [ARGS]...

Arguments:
  <SECONDS>  Number of seconds to wait before timing out
  <COMMAND>  Command to execute
  [ARGS]...  Arguments for the command

Options:
  -k, --kill-after <KILL_AFTER>  Also send KILL signal after this many seconds
  -v, --verbose                  Print debug information
  -h, --help                     Print help
  -V, --version                  Print version
--version x
```

The output reads in command order.
After the install line,
the first `timeout 0.1.0` answers `timeout --version`,
and the second is `timeout 5 /usr/bin/echo --version x`,
which should have printed `--version x` and did not run `echo` at all.
The `DEBUG:` block and the lone `x` come from `-v x`,
the help text from `df -h /`,
and the last line is the system `timeout` running `echo` correctly.

### Risk to the poster

Filing puts the poster's GitHub account in front of a stranger's repository.
This repository has no trust or blocklist system (see "Trust and policy search"),
a single maintainer,
and no stated policy.
An agent-written report that its recipient reads as low effort is the case such systems exist for,
and a reproduction,
a source trace,
and a verified prototype lower that risk and do not remove it.
This document does not recommend filing,
and any filing is the poster's own decision.
The draft carries a disclosure line that an AI agent prepared it;
the poster decides whether to keep it.

### Draft issue

Do not file without re-running the day-of checks in "Readiness check".
The reproduction block is exactly the one that was run.

~~~md
Title: A flag right after the command (`--version`, `-h`, `-v`, `-k`) is consumed by timeout instead of the command

timeout-cli 0.1.0 (crates.io checksum e5e3bf8a86dc5c449a820b953bd23ade441c9c5d589159eacd839cec75911102,
source commit e7be8fab08ef6efcb3423f26730a966dae7c25b1), also reproduced on `main` at f11480d.

This report was prepared by an AI agent (Claude) for the account that posts it.
Every command in this report was run, and the output shown is real.

## What happens

When the first argument after the wrapped command is one of timeout's own options, timeout takes it and the command
never gets it.
`timeout 5 tool --version` prints `timeout 0.1.0` and exits 0 without running `tool`.
`-h`/`--help` print timeout's help.
`-v`/`--verbose` and `-k N`/`--kill-after N` are removed from the command's arguments.
GNU `timeout 5 tool --version` runs `tool --version`, and the README says timeout-cli "Supports commands with flags"
and "aims to be compatible with the standard Unix `timeout` command".

Real programs go wrong quietly:
`timeout 5 df -h /` prints timeout's help, and `timeout 5 grep -v . file` runs `grep` without `-v`.

## Reproduction

```sh
cargo install timeout-cli --version 0.1.0 --locked
timeout --version
timeout 5 /usr/bin/echo --version x
timeout 5 /usr/bin/echo -v x
timeout 5 /usr/bin/df -h /
/usr/bin/timeout 5 /usr/bin/echo --version x
```

Output:

```text
   Installed package `timeout-cli v0.1.0` (executable `timeout`)
timeout 0.1.0
timeout 0.1.0
DEBUG: Starting timeout: 5s, kill-after: Nones, command: /usr/bin/echo
DEBUG: Spawning command: /usr/bin/echo ["x"]
DEBUG: Waiting for command result...
DEBUG: Timeout thread started, sleeping for 5s
DEBUG: Command spawned successfully with PID: 379
x
DEBUG: Command exited with code: 0, term_sent: false
DEBUG: Process completed normally
DEBUG: Command result received: Completed(0)
DEBUG: Command completed normally with exit code 0
DEBUG: Exiting with code: ExitCode(unix_exit_status(0))
Run a command with a timeout

Usage: timeout [OPTIONS] <SECONDS> <COMMAND> [ARGS]...

Arguments:
  <SECONDS>  Number of seconds to wait before timing out
  <COMMAND>  Command to execute
  [ARGS]...  Arguments for the command

Options:
  -k, --kill-after <KILL_AFTER>  Also send KILL signal after this many seconds
  -v, --verbose                  Print debug information
  -h, --help                     Print help
  -V, --version                  Print version
--version x
```

The first `timeout 0.1.0` answers `timeout --version`.
The second is `timeout 5 /usr/bin/echo --version x`: `echo` never ran, and the last line, from the system `timeout`,
is what it should have printed.
The `DEBUG:` block and the lone `x` come from `-v x`, and the help text from `df -h /`.

Environment: Debian GNU/Linux 13 (trixie) container (`rust:1-trixie`, rustc 1.99.1), and a Fedora 44 host.

## Cause

`command` and `args` are separate positionals and only `args` has `trailing_var_arg`
(https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/src/main.rs#L10-L29).
clap starts "everything that follows is a value" only after a value lands in the `trailing_var_arg` positional,
so a defined flag that arrives between `command` and `args` is parsed as an option.
Flags that timeout does not define pass through, which is why `timeout 5 tool --model x` works.
This is clap's documented behavior (clap-rs/clap#1538), not a clap bug.
The same happens with clap 4.6.7.

## Suggested fix

Fold `command` and `args` into one positional with `trailing_var_arg = true`, `allow_hyphen_values = true`,
and `required = true`, and take the first value as the command in `main`
(`src/main.rs`, the `Args` struct and the use of `args.command` and `args.args`).
A patch with a regression test is ready: with it, the existing 25 tests and the new one pass,
and `cargo fmt --check` and `cargo clippy -D warnings` are clean.
It changes one behavior: `timeout 5 tool -- --version` now passes the `--` to the tool, as GNU timeout does.
I can open a pull request.
~~~

### Draft pull request

~~~md
Title: Pass flags that follow the command to the command

Fixes #<issue number, known only after the issue is filed>.

Merges `command` and `args` into one trailing positional so that `timeout 5 tool --version`, `-h`, `-v`, and `-k N`
reach `tool` instead of being parsed as timeout's options.

- `src/main.rs`: one `command_and_args` positional (`required`, `trailing_var_arg`, `allow_hyphen_values`),
  the first value is the command.
- `tests/integration_tests.rs`: a regression test over every option timeout defines; it fails on `main` and passes here.
- Behavior change: `timeout 5 tool -- x` now forwards `--` to `tool`, matching GNU timeout.

Verified: `cargo test` (26 passed), `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`.

Prepared by an AI agent (Claude) for the account that posts it; the commands in this pull request were run in a container.
~~~

[tc-args]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/src/main.rs#L10-L29
[tc-lock]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/Cargo.lock#L98-L99
[tc-readme-usage]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/README.md?plain=1#L54
[tc-readme-flags]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/README.md?plain=1#L153
[tc-readme-compat]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/README.md?plain=1#L210
[tc-readme-issues]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/README.md?plain=1#L176-L185
[tc-test-flags]: https://github.com/yaleman/timeout-cli/blob/e7be8fab08ef6efcb3423f26730a966dae7c25b1/tests/integration_tests.rs#L39-L44
[clap-auto-flags]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/builder/command.rs#L4722-L4749
[clap-trailing-flip]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L402-L404
[clap-single-value-state]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L422-L428
[clap-long-prior-state]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L768-L773
[clap-long-keymap]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L791-L793
[clap-version-action]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L1314-L1323
[clap-long-fallback]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L858-L866
[clap-short-fallback]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/parser/parser.rs#L901-L913
[clap-trailing-doc]: https://github.com/clap-rs/clap/blob/520beb5ec2d2bb5dd11912d27127df4e97027965/clap_builder/src/builder/arg.rs#L544-L553
[clap-1538]: https://github.com/clap-rs/clap/issues/1538
[clap-1538-epage]: https://github.com/clap-rs/clap/issues/1538#issuecomment-1028473211
[clap-1538-workaround]: https://github.com/clap-rs/clap/issues/1538#issuecomment-1025053372
[clap-6308]: https://github.com/clap-rs/clap/issues/6308#:~:text=bun%20run%20foo.js%20--help%20should%20collect
[clap-4187]: https://github.com/clap-rs/clap/pull/4187
[gnu-getopt]: https://github.com/coreutils/coreutils/blob/89b2cd58ac895e3fc0d24d8f10e7e4ba132e7fb6/src/timeout.c#L522
