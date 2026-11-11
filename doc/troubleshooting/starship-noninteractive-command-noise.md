# Starship 1.26.0 and 1.27.0 log a TERM=dumb error into shell output

## Symptom

With `TERM=dumb`,
Starship writes two strings,
on two different streams:

```text
# stderr, colourised, once per Starship session
[ERROR] - (starship::print): Under a 'dumb' terminal (TERM=dumb).
# stdout, on every prompt
Starship disabled due to TERM=dumb > 
```

The stdout line is Starship's documented fallback prompt,
so it appears wherever a prompt is drawn.
The stderr line is a log record,
so it appears in any command output that captures standard error,
which is what made agent command output unreadable at 1.26.0,
commit `fca92d8`.

Both were reported against 1.26.0.
The probes below ran against 1.27.0,
commit `94dfccf`.

## Root cause

Four steps,
each measured.

1. The login shell reaches the local Starship integration.
   `~/.bash_profile:4` through `~/.bash_profile:5` source `~/.bashrc`,
   so every login shell,
   interactive or not,
   ran the file.
2. `~/.bashrc` evaluated `starship init bash` unconditionally,
   which defines `starship_precmd` and points `PROMPT_COMMAND` at it.
   Measured before the fix,
   in a noninteractive login shell:
   `declare -F starship_precmd` printed `starship_precmd` and
   `PROMPT_COMMAND` read `starship_precmd 2>&10`.
3. Starship's prompt builder logs an error when the terminal is dumb,
   at [`src/print.rs#L97-L101`][print-rs] (1.27.0,
   `94dfccf`):

   ```rust
   // starship/src/print.rs:97-101
   match context.get_env("TERM") {
       Some(term) if term == "dumb" => {
           log::error!("Under a 'dumb' terminal (TERM=dumb).");
           buf.push_str("Starship disabled due to TERM=dumb > ");
           return buf;
       }
   ```

   The module path in the record is the one the message shows,
   `starship::print`.
4. Starship's logger prints every enabled record to stderr,
   at [`src/logger.rs#L207-L220`][log-stderr],
   and decides "enabled" by comparing the record's level with
   `self.log_level` at [`src/logger.rs#L107-L118`][log-levels].
   That parse accepts five words only:

   ```rust
   // starship/src/logger.rs:107-118
   log_level: env::var("STARSHIP_LOG").map_or_else(
       |_| Level::Warn,
       |level| match level.to_ascii_lowercase().as_str() {
           "trace" => Level::Trace,
           "debug" => Level::Debug,
           "info" => Level::Info,
           "warn" => Level::Warn,
           "error" => Level::Error,
           _ => Level::Warn,
       },
   ),
   ```

   The lowest level it can select is `Error` itself,
   so no `STARSHIP_LOG` value can silence an `error!` record.
   `off` is not a level:
   it falls into the `_` arm and selects `Warn`.

One more step explains why the stderr line appeared once and then stopped:
a record whose text is already in the session log file is not printed again,
at [`src/logger.rs#L163-L178`][log-dupes].
The session log is per `STARSHIP_SESSION_KEY`,
so a fresh key prints the line again.

## Verification

Installed binary:
Starship 1.27.0,
commit `94dfccf`,
at `~/.local/share/monochromatic-dev/toolchain-global/bin/starship`.

The harness gives Starship a private `HOME`,
a fresh `STARSHIP_SESSION_KEY` so the duplicate check starts empty,
and separates the two streams:

```sh
# Reproduce both strings; the key must be fresh for the stderr line.
probe=$(mktemp --directory "${HOME}/temp/agent/starship-probe.XXXXXXXX")
env -i HOME="$probe" TERM=dumb \
  PATH="$HOME/.local/share/monochromatic-dev/toolchain-global/exposure:/usr/bin:/bin" \
  STARSHIP_SESSION_KEY="probe-$RANDOM" \
  starship prompt >/dev/null
```

Measured,
with the fresh key:

- stderr:
  `[ERROR] - (starship::print): Under a 'dumb' terminal (TERM=dumb).`
- stdout:
  `Starship disabled due to TERM=dumb > `
- the same key a second time:
  nothing on stderr,
  which is the duplicate check
- a second fresh key:
  the stderr line again,
  which is the per-session record

`STARSHIP_LOG` was tried at five values,
each with its own fresh key and `HOME`:
`off`,
`error`,
`warn`,
`starship::print=off` and `trace`.
The first four printed the `[ERROR]` line unchanged.
`trace` printed it and dozens of `TRACE` records besides,
which shows the probes do differ and the variable is read.

A catalog that renders no prompt,
for contrast:

```sh
# Not a prompt: no record and no fallback line.
env -i HOME="$probe" TERM=dumb starship module directory
```

## Verified workaround: Starship only in interactive shells

`~/.bashrc` now initialises Starship only when the shell is interactive.
Line numbers in this file shift as the file changes,
so the block is quoted rather than located:

```bash
# ~/.bashrc: Starship exists only where a prompt is drawn.
if [[ $- == *i* ]]; then
  eval "$(starship init bash)"
fi
```

Measured in two disposable homes whose `.bash_profile` sources `.bashrc`,
so each arm runs the whole chain:

- Stopgap arm,
  noninteractive login shell:
  `starship_precmd`,
  `PROMPT_COMMAND=starship_precmd 2>&10`,
  the state described in "Root cause".
- Guarded arm,
  same shell:
  `no-hook`,
  `PROMPT_COMMAND=unset`.
- Guarded arm,
  `bash -i`:
  `starship_precmd`,
  `PROMPT_COMMAND=starship_precmd`.

The same three measurements on the real `~/.bashrc` after the change:

- `env -i HOME=/var/home/user TERM=dumb bash -lc 'echo hi'` printed `hi`,
  `declare -F starship_precmd` printed `no-hook`,
  `PROMPT_COMMAND` read
  `_mise_hook_prompt_command`,
  which is another tool's hook and stays,
  and file descriptor 10,
  which the stopgap opened,
  no longer exists.
- `bash -i -c 'declare -F starship_precmd'` printed `starship_precmd`.

Tradeoffs:

- Nothing is drawn by Starship in a noninteractive shell,
  so a noninteractive consumer of the prompt cannot get one.
  None was found:
  a noninteractive shell draws no prompt.
- A wrapper that calls `starship prompt` itself,
  such as a harness that
  renders a prompt for a status line,
  still gets both strings.
  The guard removes the shell integration,
  not the binary's behaviour.
- Interactive shells keep every diagnostic,
  which is what the stopgap lost.

Not measured:
the harness that printed the `[ERROR]` line at 1.26.0.
A noninteractive `bash -lc` does not draw a prompt,
and it printed neither string before or after the guard in these probes.
The guard removes the class (Starship wired into a shell that draws no prompt),
not one identified caller.

## Superseded workaround: mute Starship's stderr

The 1.26.0 stopgap discarded the initialisation's standard error
and pointed the prompt hook's diagnostics at a descriptor on `/dev/null`.
It suppressed the noise and also hid every useful Starship diagnostic,
so it was replaced by the interactive-only guard above.
Its text is in this file's history.

`~/.bashrc:52` and `~/.bashrc:54` to `~/.bashrc:57`,
which an earlier reading named as the initialisation and the filter,
no longer hold those statements:
the guard replaced them on 2026-11-10.

## What does not work

- `STARSHIP_LOG=off`:
  the word is not a level,
  so it selects `Warn` and the `error!` record still prints.
  The same holds for `error`,
  `warn` and `starship::print=off`,
  because the lowest selectable level is the record's own.
- Muting standard error around the initialisation and the prompt hook:
  it works,
  and it also discards the diagnostics a user needs.
- The earlier probe `env STARSHIP_LOG=off starship prompt` proved nothing:
  it ran in a session whose log file already carried the record,
  and it set no `TERM`,
  so the dumb-terminal branch was not the one under test.

## Upstream filing decision

Nothing to file.
The behaviour is documented,
tested
at [`src/print.rs#L566-L575`][print-test] as "a dumb terminal gets the standard
fallback",
and no setting promises to silence it;
the defect was in this machine's shell configuration,
which ran Starship where no prompt exists.
Constraint 1 fails,
so the remaining five were not assessed.

[print-rs]: https://github.com/starship/starship/blob/94dfccf79eeb6b79805dcfbdcc2d788f6fb4c248/src/print.rs#L97-L101
[log-levels]: https://github.com/starship/starship/blob/94dfccf79eeb6b79805dcfbdcc2d788f6fb4c248/src/logger.rs#L107-L118
[log-dupes]: https://github.com/starship/starship/blob/94dfccf79eeb6b79805dcfbdcc2d788f6fb4c248/src/logger.rs#L163-L178
[log-stderr]: https://github.com/starship/starship/blob/94dfccf79eeb6b79805dcfbdcc2d788f6fb4c248/src/logger.rs#L207-L220
[print-test]: https://github.com/starship/starship/blob/94dfccf79eeb6b79805dcfbdcc2d788f6fb4c248/src/print.rs#L566-L575
