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
It preserves command and control-flow roles, quoting, redirects, assignments, and nested execution.
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
