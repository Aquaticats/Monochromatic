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
by spelling the real Git binary as a path,
which skips the `git` wrapper this repository puts on PATH.

The executable name is read from `@monochromatic-dev/git-executable-core`
rather than spelled here,
so an install outside `/usr/bin` is still recognized;
callers may inject executable names for fixtures.
A word counts as the real Git only when it carries a path separator and its last segment matches,
so a document under a directory named `git` and a differently named binary beside Git stay allowed.

Program wrappers are stripped before the subcommand is read:
`NAME=value` prefixes,
`env` (including `-S` split-string payloads),
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
up to eight wrapper layers.
A nested shell `-c` script string is analyzed too,
from a work list rather than by recursion.

Read-only subcommands and read-only argument shapes stay allowed:
plain `rev-parse`, `for-each-ref`, `log`, `show`, `status`, `ls-remote` and the rest of the read set,
plus `branch`/`tag`/`config`/`symbolic-ref` listing forms and the verb-first read forms of
`stash`, `remote`, `worktree`, `reflog`, `notes`, `submodule`, `bisect`, `bundle`, `sparse-checkout` and `commit-graph`.
Every other subcommand counts as mutating,
including one this package has never seen,
because only a proven read may skip the wrapper.

When real-Git resolution fails the guard stays inactive and logs at debug level:
a throwing hook would fail every Bash call.

## Pi dependency trust

PJE:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.
