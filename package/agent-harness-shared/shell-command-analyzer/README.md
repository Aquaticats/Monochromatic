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

The adapter consumes the `unbash` 5 syntax tree,
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

Program wrappers are stripped before the subcommand is read:
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
`timeout`,
and `xargs`,
for every wrapper layer a command line spells.
A split-string payload carries a command line,
and GNU `env` appends the words after the payload to that command line,
so `env -S'/usr/bin/git' commit` is read as `/usr/bin/git commit`.
A nested shell `-c` script string is analyzed too,
from a work list rather than by recursion,
and it inherits the search path its launcher was given.

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

Programs that run another program but carry no wrapper entry here are not followed either:
`find . -exec /usr/bin/git commit {};`,
`flock lock /usr/bin/git commit`,
`eval '/usr/bin/git commit'`,
`busybox sh -c '/usr/bin/git commit'`,
`systemd-run -- /usr/bin/git commit`,
`watch '/usr/bin/git commit'`,
`ssh host git commit`,
and `bash <<< '/usr/bin/git commit'`.

A `sudo` that resets the search path to its own `secure_path` is not detected:
that value lives in the sudoers configuration,
not in the command text.
`local PATH=...` inside a function is not tracked as setting the search path.
`git` reached through a relative path or a symlink whose target is the real binary
is read by spelling,
never resolved.

When real-Git resolution fails the guard stays inactive and logs at debug level:
a throwing hook would fail every Bash call.

## Pi dependency trust

PJE:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.
