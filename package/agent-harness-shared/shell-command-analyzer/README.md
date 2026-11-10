# Shell command analyzer

Shared shell-command analysis for agent harness guardrails.

The package parses Bash source with `unbash` and returns command records,
redirects,
expansion flags,
parse status,
and execution context.
Argument and redirect records pair parsed values with original shell source spellings.
Literal `for` loop bindings preserve both forms for expansion-provenance checks.
Callers can distinguish commands that run while the script is evaluated
from commands only stored inside function bodies.

## Every word and expression form

A nested command is reported wherever the shell grammar can put one.
The traversal descends each word part that carries a child list
(`parts` in the parser's types),
which covers `DoubleQuoted`,
`LocaleString`,
`BraceExpansion`,
`ExtendedGlob`,
and a `bracket` `Glob`,
and it descends an `ArithmeticWord` for the same reason.
`ParameterExpansion` is read through its own operation table:
`Default`,
`Remove`,
`CaseModification`,
and `Transform` carry one operand,
`Replace` carries a pattern and a replacement,
`Slice` carries an offset and an optional length,
`Names` carries none,
and the parser's `Unknown` operation is reported as unread.
Here-document bodies,
here-string words,
case patterns,
`for` word lists,
assignment values,
array elements,
and redirect targets are read as words too,
so `echo {a,$(bun test)}` reports the inner `bun test`.

The parts the grammar keeps literal are named as leaves rather than guessed at:
`Literal`,
`SingleQuoted`,
`SimpleExpansion`,
`AnsiCQuoted` (which decodes backslash escapes and expands nothing),
and the `star` and `question` `Glob` kinds,
whose `parts` field is typed `undefined`.

## A node kind the traversal does not know

`ShellCommandAnalysis.notUnderstoodNodeKinds` names every parser node kind
the walk met and had no visitor for.
An empty list means the answered tree was walked whole;
a non-empty list means part of the command was named instead of read,
so `commands` is not proof that nothing else runs.

A limit this list does not cover:
past the parser's nesting budget (`MAX_SYNTAX_NESTING`,
256 in unbash 6.0.4),
a sub-field keeps its raw text and has no parts to walk,
so a command nested that deep is neither reported nor named.
That depth takes a command written to reach it.

A limit this list does not cover:
past the parser's nesting budget (`MAX_SYNTAX_NESTING`,
256 in unbash 6.0.4),
a sub-field keeps its raw text and has no parts to walk,
so a command nested that deep is neither reported nor named.
That depth takes a command written to reach it.
`parsed` still reports parser success,
because unparsable syntax and unread syntax are different answers.
`hasNotUnderstoodShellSyntax` and `NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON`
give guards the refusal that keeps that answer from becoming an allow.
Bash rejects the reachable unread form itself
(`printf %s ${value;text}` is a bad substitution),
so refusing it costs a rewrite and never a working command.

The adapter consumes the `unbash` 6 syntax tree,
including redirected compounds,
command-wrapper nodes,
declaration assignments,
and parameter operations.
Allocated descriptor variables are reported as `fileDescriptorVariable`,
separately from numeric `fileDescriptor` values.
Nested scripts and their diagnostics come from the parser;
the analyzer does not execute shell expressions or reparse removed legacy fields.

`parseShellReviewStructure` projects the same parser tree for model permission review.
It preserves command and control-flow roles,
quoting,
redirects,
assignments,
and nested execution.
Complete source scripts and source locations stay local.
Literal arguments and quoted here-document bodies remain data.
Runtime expansion values and effects remain unestablished by parsing.
`ShellReviewSyntaxError` rejects incomplete scripts and unsupported parameter operations.
Those expected syntax rejections retain debug diagnostics and their original thrown value.
Unexpected parser failures retain error diagnostics and the original cause.
The caller decides whether rejected syntax belongs to a current proposal or historical data;
historical recovery does not grant execution permission.

See the [AST adapter troubleshooting record](../../../doc/troubleshooting/unbash-5-ast-adapter.md)
for the original failure and verification commands.

## Real-Git mutation guard

`invokesMutatingGitThroughRealGitPath` reports shell source that mutates a repository
without going through the `git` wrapper this repository puts on PATH.

### What counts as the real Git

The executable name and the folder come from `@monochromatic-dev/git-executable-core`
rather than being spelled here,
so an install outside `/usr/bin` is still recognized.
A word counts as the real Git only when it carries a path separator
and its last segment matches,
so a document under a directory named `git`
and a differently named binary beside Git stay allowed.
The folder that holds that executable is the one a search-path assignment must name
to reach it.

Callers may inject executable names and folders for fixtures.
An injected name is enough to classify a path word;
an injected folder is what a `PATH` assignment is compared against.

### Path words, search paths, and default search paths

Three spellings reach the real binary,
and each one is reported:

- A path word,
  as in `/usr/bin/git commit`.
- A `PATH` assignment on the same command line,
  as in `PATH=/usr/bin:$PATH git commit`,
  `env PATH=/usr/bin git commit`,
  a standalone `PATH=/usr/bin; git commit` statement,
  or `export PATH=/usr/bin; git commit`.
  A record is read in traversal order,
  so an assignment reaches the commands after it and never the ones before it.
  The assignment reaches a nested shell too:
  `PATH=/usr/bin bash -c 'git commit'` is reported.
- A search path the repository cannot install the wrapper into:
  `command -p` (the platform default path),
  `env -i` and `env --ignore-environment` (no environment at all),
  and `env -u PATH` or `env --unset=PATH` (no search path at all).

A search-path decision never reads shell state.
An entry the shell only resolves at run time (`$PATH`,
`~/bin`,
`$(dirname "$0")`)
names no readable folder and never matches,
so `PATH=$HOME/.local/bin:$PATH git commit` stays allowed.
The guard also does not look at what a folder contains:
a folder is compared by spelling,
so a directory that happens to hold a copy of Git is reported only when it is
the folder the resolver reports.

### Wrappers

Program wrappers are stripped before the subcommand is read,
one entry per program,
for every wrapper layer a command line spells.
Two shapes are tabled,
in `src/program-wrappers-that-own-only-options.ts` and
`src/program-wrappers-that-own-leading-words.ts`:

- Wrappers whose words before the program are their own options:
  `NAME=value` prefixes,
  `env` (including `-S` and `--split-string` payloads,
  attached as `-S'...'`/`--split-string=...` or separated),
  `sudo`,
  `doas`,
  `command`,
  `builtin`,
  `exec`,
  `nohup`,
  `setsid`,
  `time`,
  `nice`,
  `ionice`,
  `stdbuf`,
  `xargs`,
  `systemd-run`,
  `watch`,
  `eval`,
  and `busybox`.
- Wrappers that own a positional word before the program,
  which a reader must drop before the program name:
  `timeout` (duration),
  `flock` (lock file),
  `ssh` (host),
  and the two that own a number,
  `taskset` (affinity mask or cpu list) and `chrt` (priority).

A split-string payload carries a command line,
and GNU `env` appends the words after the payload to that command line,
so `env -S'/usr/bin/git' commit` is read as `/usr/bin/git commit`.
`flock -c` carries one whole command string instead,
which is that command and nothing after it.
`eval` and `watch` join the words after their options into one shell source line
instead of taking them as one program and its arguments,
so `eval '/usr/bin/git commit'` and `watch '/usr/bin/git commit'` are read as that command.
`watch -x` and `watch --exec` switch it to running those words as a program
and its arguments,
which is read as those words.
`busybox` names its applet in its first word,
so `busybox sh -c '/usr/bin/git commit'` is read as `sh -c '/usr/bin/git commit'`.
`systemd-run -E NAME=VALUE` and `--setenv=NAME=VALUE` count as assignments
for the command the unit runs,
the way an `env` prefix does.
A nested shell `-c` script string is analyzed too,
from a work list rather than by recursion,
and it inherits the search path its launcher was given.

A wrapper owns one positional number at most,
which is why `taskset 1 cc` reads `cc` as the program
and not as a second mask.

### Programs that run a command named among their arguments

`find` runs a command listed between one of its command options and that option's terminator,
so those words are read as their own command line:
`-exec`,
`-execdir`,
`-ok`,
and `-okdir`,
Each ends at a `;` word.
`-exec` and `-execdir` also end at a `+` word,
but only directly after the `{}` placeholder,
because any other argument may spell `+` as data.
`find . -exec /usr/bin/git commit {} \;` is refused,
and `find . -exec git commit {} \;` reaches the wrapper.

### A program handed to another host

`ssh` joins its arguments into one command line the remote login shell runs,
so that line is read the way a local script string is:
`ssh build-host /usr/bin/git commit` and
`ssh build-host 'cd /repo && /usr/bin/git commit -m x'` are refused,
while `ssh build-host git commit` stays allowed,
because the remote search path,
and whether that host installs the wrapper,
is not in the command text.
A path word is therefore the only spelling that proves a wrapper is skipped there,
and a `PATH` assignment before `ssh` is not read as applying to the remote program.

### Read-only subcommands

Read-only subcommands and read-only argument shapes stay allowed in all of those forms:
plain `rev-parse`,
`for-each-ref`,
`log`,
`show`,
`status`,
`ls-remote` and the rest of the read set,
plus `branch`/`tag`/`config`/`symbolic-ref` listing forms and the verb-first read forms of
`stash`,
`remote`,
`worktree`,
`reflog`,
`notes`,
`submodule`,
`bisect`,
`bundle`,
`sparse-checkout` and `commit-graph`.
Every other subcommand counts as mutating,
including one this package has never seen,
because only a proven read may skip the wrapper.

### What stays undetected

A Git path known only at run time is not detected,
because the guard reads command text and runs no shell evaluator:

- `bash -c "$CMD commit"` and `sh -c "$GIT commit"`.
- `G=/usr/bin/git; $G commit` and `"$G" commit`.
- `eval "$G commit"` and `$(command -v git) commit`.

A shell script string a shell reads from standard input is not followed:
`bash <<< '/usr/bin/git commit'` and `printf '%s' '/usr/bin/git commit' | bash`
run a script the guard never reads.

Programs that run another program but carry no entry here are not followed either:
`su -c '/usr/bin/git commit'`,
`runuser -c '/usr/bin/git commit'`,
`unshare --mount /usr/bin/git commit`,
`nsenter --target 1 /usr/bin/git commit`,
`chroot /mnt /usr/bin/git commit`,
`systemd-cat /usr/bin/git commit`,
`script -c '/usr/bin/git commit'`,
`tmux new-session '/usr/bin/git commit'`,
`docker exec container /usr/bin/git commit`,
`podman run image /usr/bin/git commit`,
and an interpreter that runs a command the guard would have to execute,
such as `python -c` or `node -e`.

A `git` word handed to another host with `ssh` stays allowed,
as the section on that form says.

A short-option cluster whose last letter takes a separate value hides the program:
`ssh -vp 22 build-host /usr/bin/git commit`,
`systemd-run -qu build -- /usr/bin/git commit`,
`watch -tn 1 '/usr/bin/git commit'`,
and `flock -xw 5 /repo.lock /usr/bin/git commit`
read that value as a program or as the wrapper's own leading word.
A `find` expression whose operand spells one of those command options
(`find . -name '-exec' -exec /usr/bin/git commit {} \;`) is read as that option too.
A unit property that carries the environment
(`systemd-run --property=Environment=PATH=/usr/bin -- sh -c 'git commit'`)
is read for its option only,
never for its property value.

A `sudo` that resets the search path to its own `secure_path` is not detected:
that value lives in the sudoers configuration,
not in the command text.
`local PATH=...` inside a function is not tracked as setting the search path.
`git` reached through a relative path or a symlink whose target is the real binary
is read by spelling,
never resolved.

When real-Git resolution fails the guard stays inactive and logs at debug level:
a throwing hook would fail every Bash call.

## Process-name wait guard

`waitsOnProcessName` reports shell source whose repeating loop looks a process up by name,
which this repository bans because the Bash tool runs the command inside a wrapper shell
whose command line contains the command text,
so `pgrep --full <pattern>` matches that wrapper and the loop never sees the process exit.
`PROCESS_NAME_WAIT_BAN_REASON` carries the replacements,
which wait on a PID under a bound.

### What the ban reads

Loop membership covers the condition list and the loop body,
so `until ! pgrep -f x; do sleep 1; done` and `while true; do pgrep -f x || break; done` both match,
while `pgrep -af x` on its own stays allowed.
A lookup word is read by its last path segment,
so `/usr/bin/pgrep` and `pg"rep"` are the same lookup as `pgrep`.
A shell's `-c` script string is read as a command line,
so `timeout 60 bash -c 'until ! pgrep -f x; do sleep 1; done'` matches.

### A command the parser rejects

A command that does not parse is read from its raw words instead of answering "no",
because the visible text is the only evidence such a command carries.
The reader reports the runs of words that begin where a command can begin,
named wrapper layers are stripped,
and any `pgrep` or `pidof` standing where a program can stand is refused,
because the loop around that lookup could not be read at all.
`until ! pgrep x; do`,
`env pgrep -f build "unterminated` and
`timeout 60 bash -c 'while pgrep -f x; do sleep 1; done' "unterminated` are refused for that reason.
So is straight-line `pgrep -af build "unterminated`,
which the parsed path allows:
an unparsable command gives the guard no way to tell a loop from a
single lookup.
Prose that never reaches a program position stays allowed,
so `printf pgrep pidof "unterminated` and `echo "use pgrep in a while loop` are not refused.
A redirection standing before the lookup or attached to it is read as a redirection,
so it cannot hide the lookup:
`while >/dev/null pgrep -f x; do sleep 1; done "unterminated`,
`while 2>/dev/null pidof x; do sleep 1; done "unterminated` and
`while pgrep</dev/null -f x; do sleep 1; done "unterminated` are refused.
Only the program word hands a script to a shell,
so `printf bash -c pgrep "unterminated` is not refused.

### What stays undetected or over-refused

The raw reader never follows the redirect that declares a here-document,
so a here-document body is read as command text:
`cat <<EOF` whose body spells `while pgrep x; do sleep 1; done` is refused
even though the body is document text rather than a command.
The same quote bookkeeping cuts the other way:
a here-document body or any unparsable word carrying an unbalanced quote
hides everything after that quote,
so a real loop spelled later in that command is missed.

A lookup spelled through a variable (`while "$LOOKUP" -f x; do sleep 1; done`)
is not read,
in a parsed command or an unparsable one.
A shell script read from standard input (`bash <<< 'until ! pgrep x; do sleep 1; done'`)
is not followed either.
The raw reader hides everything inside quotes,
so a lookup spelled inside one is unread in an unparsable command:
`while test "$(pgrep -f x)"; do sleep 1; done "unterminated`,
`while $'pgrep' -f x; do sleep 1; done "unterminated`,
and a lookup split by a line continuation (a `pg` word,
a backslash,
a newline,
then `rep`).
A parsed command reads all three,
because the parser hands back the commands inside a substitution and resolves the quoting itself.
The reader also ends a run at `&`,
so a redirection that names a descriptor splits it away from the command behind it:
`while >&2 pgrep -f x; do sleep 1; done "unterminated` is not refused.

A parsed command is read by its own program word and by shell `-c` script strings only,
so a wrapper or a command string in front of the lookup is not followed there:
`while sudo pgrep -f x; do sleep 1; done`,
`while env pgrep -f x; do sleep 1; done`,
`while timeout 5 pgrep -f x; do sleep 1; done`,
`eval "while pgrep -f x; do sleep 1; done"`
and `find . -exec pgrep -f x \;` stay allowed.
The unparsable path does strip those wrapper layers,
so the same spellings are refused
after a parse error.

## Pi dependency trust

PJE:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.
