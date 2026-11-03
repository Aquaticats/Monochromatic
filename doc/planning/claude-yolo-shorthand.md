# Proposed `claude --yolo` shorthand

## Corrected requirements

On 2026-11-02 the user clarified:

> I want to make claude --model sonnet --yolo and so on work too.

The user also rejected modifying Claude:

> Patching Claude Code itself is not good.

The first-position-only Bash function was rejected.
The initial recommendation incorrectly treated the example `claude --yolo` as permission to restrict option order.
Its passing forwarding tests did not test the user's intended command grammar.
No shell function or binary patch was installed.

## Revised recommendation

Use an executable wrapper exposed as `claude`,
following the repository's existing command-shadow pattern.
It translates a parsed `--yolo` option to `--dangerously-skip-permissions`,
then replaces its process with the unchanged real Claude executable.
This is a design recommendation,
not an implemented or verified launcher.

The proposed argument contract includes:

```text
# Required translation examples; these are not shell installation commands.
claude --yolo --model sonnet
  => claude --dangerously-skip-permissions --model sonnet
claude --model sonnet --yolo
  => claude --model sonnet --dangerously-skip-permissions
claude --model=sonnet --yolo
  => claude --model=sonnet --dangerously-skip-permissions
claude --system-prompt '--yolo'
  => unchanged: --yolo is the required value of --system-prompt
claude -- '--yolo'
  => unchanged: --yolo is positional data after --
```

Preserve all other argument bytes and their order.
Use the existing shadow approach to real-executable resolution and recursion prevention;
do not replace Claude's installed executable or its updater-managed link.
Unix process replacement preserves the terminal streams and lets the caller observe Claude's own exit or signal.

The wrapper needs option-arity information to distinguish flags from values.
A global string replacement or a loop that rewrites every `--yolo` token is insufficient.
Required values,
optional values,
variadic options,
short forms,
the `--` delimiter,
and subcommands need coverage against the real CLI.
The option metadata source and its update strategy remain implementation design work.
Help-text parsing alone has not been established as a complete schema,
particularly for hidden options.
Do not present this proposal as upgrade-proof.

## Repository precedent and alternatives

[`pnpm-shadow`](../../package/pnpm-shadow/cli/README.md) documents executable resolution,
process replacement,
and recursion through managed launchers.
[`mpm-shadow`](../../package/mpm-shadow/cli/README.md) documents skipping option values while interpreting arguments,
including the maintenance requirement when upstream adds a value-taking option.
These are precedents,
not evidence that either existing wrapper already supports Claude.

An executable wrapper ranks ahead of a shell function for consistent behavior across launchers;
the tradeoff is an additional managed executable and option metadata maintenance.
A function that parses the same grammar could satisfy the demonstrated interactive commands,
but would still depend on shell initialization.
The rejected first-position-only function does not satisfy the clarified requirement.
Binary patching is excluded by the user's instruction.
A separate alias changes the requested spelling and is not a substitute for this task.

## Verification and remaining decisions

[Verification](../troubleshooting/claude-code-yolo-option.md) confirms the installed CLI's distinction
between the `--yolo` option position and the same text used as a value.
There is no new wrapper implementation to claim as tested.
Package ownership,
option metadata maintenance,
installation integration,
and completion integration remain unselected.
The current user permission default is `auto`;
the alias should select bypass mode only for the requested invocation.

## Proposed agent-guidance correction

Tighten existing `AGENTS.md` rule `OCG` to prevent inferring option-order restrictions from an example.
This is a proposed edit,
not an applied rule change:

> CLI options follow host grammar;
> never restrict order unasked.
> Output cardinality never decides occurrence grammar;
> sketch token encoding before repeated,
> delimited,
> or variadic forms.
