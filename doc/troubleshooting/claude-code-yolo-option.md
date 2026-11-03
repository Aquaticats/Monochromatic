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
The harness invokes the proposed function in `bash --noprofile --norc`,
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

## Verified workaround

The [proposed Bash function](../planning/claude-yolo-shorthand.md) passed the argument and status checks.
It supports `claude --yolo` in shells that load the function,
with the shorthand first.
It does not intercept programs that spawn the executable directly.
No user shell configuration was changed.

The troubleshooting record and proposal passed the scoped `format:monochromatic-lint` task
and a CommonMark-to-HTML-tree rendering check using the repository's installed renderer.
Rendered evidence is retained beside the private harness as `*.hast.json`.

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
  the local Bash function was tested;
  no upstream patch was created.

[cli-reference]: https://code.claude.com/docs/en/cli-reference#:~:text=--dangerously-skip-permissions
