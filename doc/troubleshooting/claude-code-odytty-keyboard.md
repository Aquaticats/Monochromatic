# Claude Code 2.1.317 handles Shift+Enter in OdyTTY 0.17.5 without the historical patch

## Symptom and current status

The historical [OdyTTY Shift+Enter runbook](../runbook/claude-code-shift-enter-odytty.md)
describes Claude Code `2.1.215` submitting on Shift+Enter in OdyTTY `0.9.1`.
That report remains historical evidence.
The behavior did not reproduce with the installed Claude Code `2.1.317` and OdyTTY `0.17.5`
on 2026-11-02.
Shift+Enter inserted a newline,
Ctrl+C cleared the input,
and Enter submitted the input.
No Claude binary or OdyTTY preference was patched.

## Why the historical check is insufficient

The installed Claude binary still contains the terminal-name allowlist from the old report,
without `odytty`.
Searching only that literal would incorrectly imply that the workaround remains necessary.
The same binary also contains capability-based keyboard checks:

```js
// Installed Claude Code 2.1.317 native bundle, matched functions preserved verbatim.
function y(r){if(r!=null)return z6o.includes(r);return HA().now("extendedKeys")}
function Nze(){return!gJ()&&(y()||HA().now("kittyKeyboard"))?mTs:""}
```

These distributed-binary excerpts identify the alternative capability checks;
they do not establish which upstream commit introduced them.
Claude Code's implementation source is not available in this repository.
The live input check is the evidence that this installed combination needs no patch.
Do not infer a first fixed Claude version from this result.

## Verification

The OdyTTY AppImage SHA-256 was
`e833268ceaa54996707f4af062f2f5ccd2610f214b8b307f74cfdd587297a725`,
matching upstream release `v0.17.5`.
The installed Claude binary SHA-256 was
`eb224419e344575faa4a1f39b290b649425a10cd5ee2ef1d003480cd9ae57eb0`.

Private fixtures and screenshots are under
`${HOME}/temp/agent/odytty-switch-20261102`.
`launch-claude.ts` starts the installed applications in `monochromatic-nested-wayland-session`
with disposable XDG and Claude configuration directories.
It uses `--bare`,
empty setting sources,
no tools,
and no configured MCP servers.
The test API key is an inert placeholder,
and `ANTHROPIC_BASE_URL=http://127.0.0.1:9` prevents submitting the test input to a model provider.
The compositor supplies keyboard events through its private socket.

The successful sequence was:

- Type `alpha`,
  press Shift+Enter,
  then type `beta`.
  Both lines remain in the input box;
  no request is submitted.
- Press Ctrl+C.
  The input becomes empty.
- Type `local-probe` and press Enter.
  Claude moves it into the transcript and attempts the intentionally unavailable local API endpoint.
  Its `Connection refused` diagnostic confirms the submission reached the request path.
- Quit the nested compositor.
  The probe exits with status `0`.

The lower-level `probe-launch.ts launcher-verified` test independently checks terminal input bytes:
legacy Shift+Enter sends `0d`,
while Kitty disambiguation mode sends `1b5b31333b3275`.
It also verifies command arguments,
working directory,
title,
application ID,
hold behavior,
and clean exit through the built `terminal-exec` consumer.

The failing catalog is limited to the historical versions in the runbook.
No current Shift+Enter failure was observed.
The test does not cover SSH,
multiplexers,
custom Claude keybindings,
or future releases.
No `keybindings.json` was present in the host's Claude configuration directory.

## Workaround and rejected approach

No workaround is required for this tested combination.
Reapplying the old allowlist patch solely because `odytty` is absent from that list is rejected:
the unmodified installed binary passed the live test.
Retain the old procedure only for a version where the historical failure is reproduced.

## Upstream filing decision

Nothing to report:
the tested combination works.
The repository's [Claude Code exemption](../../.out-of-scope/claude-code-upstream-bugs.md)
also excludes upstream bug tracking and elaborate workarounds.
No issue,
comment,
or patch was submitted.
