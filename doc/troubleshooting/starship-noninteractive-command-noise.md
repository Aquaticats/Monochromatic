# Starship 1.26.0: login-shell commands print TERM=dumb diagnostics

## Symptom

Noninteractive login-shell commands repeatedly printed:

```text
# Diagnostic emitted by Starship before the command's own output.
[ERROR] - (starship::print): Under a 'dumb' terminal (TERM=dumb).
```

## Root cause

The login shell reaches the local `~/.bashrc` Starship integration.
Its active initialization is at `~/.bashrc:52`,
and prompt diagnostic routing is at `~/.bashrc:54` through `~/.bashrc:57`.
The relevant code is recorded in the workaround.
Its existing prompt diagnostic filter excluded selected directory-scan warnings,
but did not exclude the observed `TERM=dumb` diagnostic.
This traces the local shell integration;
no upstream defect is established.

## Verification

Installed Starship was 1.26.0,
commit `fca92d8`.

- Before the stopgap, login-shell commands printed the diagnostic.
- `env STARSHIP_LOG=off starship prompt` still printed it.
- A non-login command printed its intended output without that diagnostic.
- After the stopgap, a fresh login-shell `pwd` printed only the working directory.

The installed-binary probe can be repeated with:

```sh
# Run against the installed Starship binary in a TERM=dumb environment.
env STARSHIP_LOG=off starship prompt
```

## Verified workaround

The user explicitly requested temporary suppression.
The local `~/.bashrc` now redirects Starship initialization stderr
and the existing prompt-hook diagnostic descriptor to `/dev/null`:

```bash
# Relevant local ~/.bashrc settings; do not replace the whole configuration.
eval "$(starship init bash 2>/dev/null)" 2>/dev/null
if [[ -z ${STARSHIP_CONTEXT_FILTER_FD-} ]]; then
  exec {STARSHIP_CONTEXT_FILTER_FD}>/dev/null
fi
PROMPT_COMMAND="starship_precmd 2>&${STARSHIP_CONTEXT_FILTER_FD}"
```

Tradeoff:
this also hides useful Starship shell-integration diagnostics.
It is a stopgap, not a durable logging policy.

## What does not work

`STARSHIP_LOG=off` did not suppress the message in the installed-binary probe.
The earlier scan-warning filter did not match this error.
The separate cli-git timeout incident does not establish a cause for this diagnostic.

## Upstream filing decision

- Upstream fault: not established.
- Upstream fixability: not assessed.
- Supported upstream use case: not assessed.
- Contribution policy: not assessed.
- Maintainer intent: not assessed.
- Prototype: none; the user requested a stopgap without further investigation.

No upstream report is proposed.
The user explicitly requested repository-local follow-up,
which is tracked in [issue #994](https://github.com/Aquaticats/Monochromatic/issues/994).
