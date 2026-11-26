# Claude Code 2.1.332 `--add-dir` consumes a following print prompt unless it is placed earlier or after `--`

## Symptom

A `claude --print` invocation that places a positional prompt after a multi-directory `--add-dir`
fails before starting a session.
The emitting tool is the Claude Code CLI,
which prints two diagnostics:

```text
"Act as a fresh final design gate review…" was read as a value of --add-dir, not as a prompt · put a prompt before --add-dir or after --
Error: Input must be provided either through stdin or as a prompt argument when using --print
```

The first quoted text varies with the prompt.
The failure is argument parsing,
not model execution,
authentication,
or hook behavior.

## Root cause

`claude --help` declares the option as variadic:

```text
--add-dir <directories...>            More directories Claude can access
```

A variadic option consumes following tokens as values until another option or the `--` separator stops it.
A positional prompt placed after the directory list is therefore consumed as another directory.
The parser's own first diagnostic identifies this exact token assignment and gives both valid remedies.
The second diagnostic follows because no prompt remains for `--print`.

The decisive evidence is the installed CLI's help and parser diagnostic.
No claim about an uninspected implementation is needed.

## Verification

Version under test:

```bash
claude --version
# 2.1.332 (Claude Code)
```

The failing shape places the prompt after the directory values without `--`:

```bash
claude --print \
  --add-dir /home/user/Monochromatic /home/user/temp/agent \
  'Return ok'
```

Claude Code reports that `Return ok` was read as an `--add-dir` value,
then reports that no prompt was provided.
The actual failure observed during a design review used a longer prompt and exited 1 before any review output existed.

The corrected shape inserts `--` before the prompt:

```bash
claude --print \
  --add-dir /home/user/Monochromatic /home/user/temp/agent \
  -- \
  'Return ok'
```

A second corrected shape places the prompt before the option:

```bash
claude --print \
  'Return ok' \
  --add-dir /home/user/Monochromatic /home/user/temp/agent
```

The separator form was used for the final Claude Sonnet design review recorded in
`doc/planning/claude-code-bash-blocking-timeout.md`.
Its session started after the failing invocation had produced no output.

## Verified workarounds

### Put `--` before the prompt

Use the separator after the final directory and before the first positional prompt token.
Its tradeoff is that every later token is positional,
so no further Claude Code options can follow the prompt.

### Put the prompt before `--add-dir`

Use the CLI diagnostic's other accepted form.
Its tradeoff is that readers must keep the prompt and its quoting together before the directory list;
later options still work normally.

## What does not work

- Repeating `--add-dir` once per directory does not by itself protect a trailing prompt:
  the last variadic occurrence can still consume it.
  Keep the separator or prompt-first ordering.
- Adding only `< /dev/null` does not repair the token assignment.
  It prevents the unrelated stdin wait warning,
  but the prompt has already been consumed as a directory.
- Treating the failure as a hook or model problem sends diagnosis in the wrong direction.
  No session starts,
  so neither hooks nor a model can influence the result.

## Upstream filing decision

`.out-of-scope/claude-code-upstream-bugs.md`
exempts Claude Code upstream bug filing because upstream is recorded as unresponsive.
This is also documented variadic CLI behavior with an explicit parser remedy,
not a defect requiring a patch.

The six-constraint audit is therefore:

1. **Upstream fault:** no;
   the diagnostic describes the accepted escaping rules.
2. **Upstream fix capability:** not assessed;
   consumer-side argument ordering already solves the invocation.
3. **Supported use case:** yes;
   the parser itself names both valid forms.
4. **Contribution welcome:** not assessed because the exemption stops filing.
5. **Likely fix:** not assessed because the exemption stops filing.
6. **Minimal fix prototype:** none;
   the verified workaround is invocation syntax at our boundary.

No duplicate search or work-in-progress search is needed under the exemption.
There is no issue or comment draft and nothing to add upstream.
