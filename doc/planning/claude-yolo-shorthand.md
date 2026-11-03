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

The user subsequently clarified:

> I'm not fundamentally against the bash fn idea.

The user selected `Reserve standalone --yolo (Recommended)` when asked whether every standalone `--yolo`
before `--` may be translated,
with literal option values written as `--system-prompt=--yolo`.
This selection removes the need to reproduce Claude's option-value grammar.
It does not authorize installing a shell change.

## Revised recommendation

For the personal Bash command,
use a function that reserves standalone `--yolo` before `--` as the shorthand.
It needs no list of Claude options,
no version check,
and no binary patch.
The tested definition is:

```bash
# Proposed personal configuration: ${HOME}/.bashrc.d/claude.sh
claude() {
  local arg
  local -a args=()
  local translate=1
  for arg in "$@"; do
    if [[ $arg == -- ]]; then
      translate=0
    elif [[ $translate == 1 && $arg == --yolo ]]; then
      arg=--dangerously-skip-permissions
    fi
    args+=("$arg")
  done
  command claude "${args[@]}"
}
```

Both `claude --yolo --model sonnet` and `claude --model sonnet --yolo` translate correctly.
Quoted arrays retain argument boundaries,
and `command claude` avoids recursively calling the function.
The function returns the real command's exit status.
The inspected `.bashrc` loads `.bashrc.d` files,
but the function has not been installed.

Literal text after `--` is unchanged.
`--system-prompt=--yolo` is also unchanged.
A standalone `--yolo` before `--` is rewritten even where Claude would otherwise consume it as a value;
therefore `--system-prompt '--yolo'` is outside the selected literal-value convention.
The function does not validate required values or repair malformed Claude invocations.

## Meaning of update maintenance

The previous proposal copied Claude's distinction between options and their values.
For that design,
adding a value-taking option could require extending the list if its value needed to be exactly `--yolo`.
That was a consequence of the proposed full-compatibility design,
not an inherent requirement for a personal shorthand.
The assistant overstated it as a likely maintenance burden without measuring relevant breakage frequency.

Under the accepted convention,
new options pass through untouched.
There is no per-option list to update.
Reasons to revisit the mapping include Claude changing or removing `--dangerously-skip-permissions`,
or introducing native `--yolo` semantics that the user wants instead.
No evidence gathered establishes that those changes happen frequently.

The inspected upstream changelog records [adding `--plugin-url <url>` in `2.1.129`][plugin-url-addition].
Such an addition illustrates the distinction:
the function forwards the new option and its URL without knowing either one's meaning.
The changelog inspection did not compare every release's parser or measure a future maintenance rate.

## Alternatives and verification

For the clarified personal-shell scope,
the Bash function ranks ahead of an executable wrapper because it needs no separate launcher installation.
An executable wrapper would cover programs that spawn `claude` directly;
that broader reach is not an established requirement.
The repository's [`pnpm-shadow`](../../package/pnpm-shadow/cli/README.md) remains relevant precedent if that need arises.
Binary patching is excluded by the user.

[Verification](../troubleshooting/claude-code-yolo-option.md) covers argument order,
the delimiter,
literal attached values,
argument boundaries,
an unknown future option,
exit status,
and real Claude parser acceptance after translation.
No authenticated model turn or installation check was performed for this revised function.

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

[plugin-url-addition]: https://github.com/anthropics/claude-code/blob/964d69565952a176ac1975ed218362991db71433/CHANGELOG.md?plain=1#L6413
