# Claude Code 2.1.317 rejects `claude --yolo` as an unknown option

## Symptom

On 2026-11-02,
the installed Claude Code `2.1.317` exited with status `1` for `claude --yolo`:

```text
error: unknown option '--yolo'
```

## Root cause and evidence boundary

The installed CLI does not accept this spelling.
Its `--help` lists `--dangerously-skip-permissions`,
and the current [CLI reference][cli-reference] describes that option as:

> Skip permission prompts.
> Equivalent to --permission-mode bypassPermissions.

The reference also distinguishes `--allow-dangerously-skip-permissions`:
it adds bypass mode to the mode cycle without starting in that mode.
The browser rendered the matching option rows;
the page text contained no `--yolo` entry.

This is an observed CLI contract,
not a source-level explanation of the proprietary parser.
No implementation-source claim or binary patch is needed for the local shorthand.

## Verification

The installed binary's SHA-256 was:

```text
eb224419e344575faa4a1f39b290b649425a10cd5ee2ef1d003480cd9ae57eb0
```

The failed invocation ran in a disposable directory with a disposable `CLAUDE_CONFIG_DIR`.
The command scope used `CPUQuota=100%`,
`MemoryMax=512M`,
`TasksMax=64`,
and `IOWeight=25`.

Private harness and captured output:
`${HOME}/temp/agent/claude-yolo.5Z9XjC4z/probe.ts`,
`live.stdout`,
and `live.stderr`.
The original harness invokes the first-position-only function in `bash --noprofile --norc`,
with a disposable executable named `claude` that records its argument array and exits with status `37`.

Passing cases:

- No arguments reach the executable unchanged.
- Leading `--yolo` becomes `--dangerously-skip-permissions`.
- `--yolo --continue "two words"` preserves the remaining arguments.
- Empty arguments,
  newlines,
  quotes,
  command-substitution text,
  semicolons,
  and glob characters remain literal argument data.
- `--system-prompt --yolo` remains unchanged because the shorthand is not first.
- `-- --yolo` remains unchanged.
- Every case returns the executable's status `37`.

Failing case:
the unwrapped installed executable rejects `--yolo` with the diagnostic in `Symptom`.

The real executable also ran with `--dangerously-skip-permissions`,
`--bare`,
empty setting sources,
no tools or configured MCP servers,
disposable home and configuration paths,
an inert API key,
and `ANTHROPIC_BASE_URL=http://127.0.0.1:9`.
Its stream initialization reported:

```json
{"type":"system","subtype":"init","permissionMode":"bypassPermissions"}
```

This is an excerpt of the initialization event.
The harness stopped the process at its configured deadline while it retried the unavailable local endpoint.
That verifies mode selection without an authenticated model turn.
It does not verify interactive onboarding or permission behavior under managed policies.

## Rejected workaround and corrected scope

The original Bash function passed the recorded argument and status checks,
but those checks omitted normal option ordering.
On 2026-11-02 the user required `claude --model sonnet --yolo` as well.
The function only examined the first argument,
so the recommendation did not satisfy the clarified requirement and is withdrawn.
The intermediate executable-wrapper recommendation was also superseded:
the user clarified that Bash functions remain acceptable and chose a reserved-token convention.
The [current proposal](../planning/claude-yolo-shorthand.md) contains the corrected,
tested Bash function.
No user shell configuration was changed during the proposal and prototype checks.

The troubleshooting record and proposal passed the scoped `format:monochromatic-lint` task
and a CommonMark-to-HTML-tree rendering check using the repository's installed renderer.
Rendered evidence is retained beside the private harness as `*.hast.json`.

### Corrected argument-grammar probes

`${HOME}/temp/agent/claude-yolo-native.RkvlTxjV/grammar-probe.ts` ran the installed Claude Code
with disposable configuration and a deliberately invalid sentinel option.
Every invocation exited during parsing,
before a model turn.

- `--model sonnet --yolo` and `--model=sonnet --yolo` produced `error: unknown option '--yolo'`.
- `--system-prompt --yolo` consumed `--yolo` as the required value,
  then rejected the sentinel option.
- `--system-prompt=--yolo` also rejected the sentinel option rather than the value.
- With the sentinel before `-- --yolo`,
  Claude rejected the sentinel option.
  This last probe does not independently prove delimiter handling,
  because parsing can reject the sentinel before examining the later tokens.

The required-value probe rules out blindly replacing every matching argument as a transparent alias.
No claim about a complete wrapper follows from these parser probes.
The user later chose an explicit reserved-token convention,
so complete transparency for a separate literal option value is no longer required.

### Reserved-token function verification

The user selected replacement of standalone `--yolo` before `--`,
with literal values written using `--system-prompt=--yolo`.
`${HOME}/temp/agent/claude-yolo.5Z9XjC4z/reserved-probe.ts` tests the exact function in the current proposal.
Its disposable fixture is `${HOME}/temp/agent/claude-yolo-reserved.BIezsd`.

The recording executable verified:

- `--yolo` before or after `--model sonnet`,
  and after `--model=sonnet`.
- Every standalone alias before `--` is rewritten,
  including the deliberately reserved separate-value spelling.
- `--system-prompt=--yolo`,
  text containing `--yolo`,
  and `--yolo=true` remain unchanged.
- `-- --yolo` remains unchanged,
  and aliases on opposite sides of `--` follow the selected rule.
- An invented `--future-option value` passes through while a following `--yolo` is translated.
- Empty arguments,
  newlines,
  quotes,
  command-substitution text,
  separators,
  and globs retain their original argument boundaries.
- The executable's exit status `37` is preserved throughout.

The real Claude `2.1.317` parser also accepted the translated alias before and after `--model sonnet`,
and accepted the attached literal `--system-prompt=--yolo`.
A deliberate invalid sentinel option then stopped each invocation during parsing.
These are parser checks,
not authenticated model sessions.
No user configuration was changed during these prototype checks.

### Installation

The user subsequently authorized installing the function.
On 2026-11-03,
the exact tested definition was written to `${HOME}/.bashrc.d/claude.sh` with mode `0644`.
The write required the destination to be absent,
and the existing `.bashrc` drop-in loading remains unchanged.
Post-installation checks are pending.

This function has no list of Claude options to keep synchronized.
The earlier maintenance warning concerned a different,
fully transparent parser-copying design.
No measured release-to-release breakage rate supports treating the selected function as costly to maintain.

### Abandoned binary inspection

The user explicitly rejected patching Claude Code itself.
An in-progress `tweakcc unpack` inspection was terminated;
it returned exit status `143` and produced no extracted file.
No repack or patch command ran,
and no installed binary was modified.
This cancellation establishes no conclusion about current `tweakcc` compatibility.

## What does not work

The literal `--yolo` option fails on the installed executable.
Replacing it with `--allow-dangerously-skip-permissions` would not select bypass mode at startup,
according to the current CLI reference.
That latter alternative was rejected from its documented semantics,
not from a live mode-selection probe.

## Upstream filing decision

Nothing to file under the [Claude Code exemption](../../.out-of-scope/claude-code-upstream-bugs.md).
The finding is a local shorthand request,
not evidence that the documented upstream option is broken.

- Upstream fault:
  not established;
  the supported option works in the mode-selection probe.
- Fixability:
  an upstream alias is conceivable;
  implementation source was not inspected.
- Supported use case:
  bypass mode is documented;
  `--yolo` is not listed.
- Contribution policy:
  not investigated because upstream tracking is excluded.
- Likely upstream action:
  not assessed;
  no tracker or duplicate search was performed under the exemption.
- Prototype:
  the first-position-only Bash function was rejected;
  the corrected reserved-token function passed argument and parser checks;
  no upstream patch was created.

[cli-reference]: https://code.claude.com/docs/en/cli-reference#:~:text=--dangerously-skip-permissions
