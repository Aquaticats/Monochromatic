# Oxlint 1.73.0 help output aborts when its stdout reader closes early

## Symptom

If a downstream pipeline command exits before reading Oxlint help output,
Oxlint aborts with status `134` and prints:

```text
thread 'tokio-rt-worker' panicked at library/std/src/io/stdio.rs:1165:9:
failed printing to stdout: Broken pipe (os error 32)
```

The incident was triggered by `oxlint --help | rg <pattern> .`.
The explicit `.` made `rg` search the repository instead of reading stdin,
so it closed the help-output pipe while Oxlint was writing.

## Root cause

Oxc commit `8de6fcaac7037d37e7f971e67a474b3ae442513a` delegates argument-parser output to bpaf in
`apps/oxlint/src/run.rs:180`:

```rust
match cmd.run_inner(&*args) {
    Ok(cmd) => cmd,
    Err(e) => {
        e.print_message(100);
```

bpaf `0.9.26` commit `3a265c5c6c5d57da82cabd1e88bfef899da2ea71` writes successful parser output through
`src/error.rs:237`:

```rust
ParseFailure::Stdout(msg, full) => {
    println!("{}", msg.render_console(*full, color, max_width));
}
```

Rust's standard `println!` path panics when stdout reports `EPIPE`.
Oxlint invokes `print_message` inside its Tokio worker,
so the panic appears as `tokio-rt-worker` even though the failure is ordinary pipe closure.

## Verification

The installed versions were Oxlint `1.73.0` and bpaf `0.9.26`.
This deterministic harness reproduced the abort:

```sh
set -o pipefail
oxlint --help | (exit 0)
printf 'exit=%s\n' "$?"
```

```text
failed printing to stdout: Broken pipe (os error 32)
Aborted (core dumped)
exit=134
```

### Patterns that complete cleanly

- `oxlint --help` with an attached terminal.
- `oxlint --help | rg 'ignore|threads'` with `rg` reading stdin.
- `oxlint --help | head --lines 1` in the observed run,
  because the short output fit the pipe before the reader exited.

### Patterns that abort

- `oxlint --help | (exit 0)` deterministically closes the reader before output.
- `oxlint --help | rg <pattern> .` can close the pipe while Oxlint is still writing,
  because `rg` searches `.` rather than consuming stdin.

## Verified workarounds

Use `rg` without an explicit search path when filtering command output:

```sh
oxlint --help | rg 'ignore|threads'
```

This makes `rg` consume stdin.
The tradeoff is that the command no longer searches repository files,
which is the intended behavior for help filtering.

For automation that intentionally stops reading early,
capture help into a file first and then query the file.
This avoids `EPIPE` at the cost of one disposable file.

## What does not work

- Adding `.` to the `rg` invocation does not mean stdin;
  it switches `rg` to filesystem search.
- Assuming every short-reader pipeline reproduces the abort is unreliable because pipe buffering can let Oxlint finish
  before the reader exits.
- Treating `tokio-rt-worker` as evidence of semantic-plugin failure is wrong.
  The source trace reaches bpaf's stdout `println!` before lint setup.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** The abort originates in bpaf's use of `println!`,
    but the observed incident also contained an incorrect `rg` pipeline.
2.  **Can upstream fix it?
    ** Yes.
    bpaf or Oxlint could handle `BrokenPipe` as successful early consumer termination.
3.  **Are they supporting this use case?
    ** No explicit bpaf or Oxlint guarantee for graceful early pipe closure was found.
4.  **Would the repo welcome our contribution?
    ** Not evaluated because the supported-use-case gate failed.
5.  **Will they likely fix it?
    ** No maintainer signal was collected for this incidental misuse-triggered report.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** No.
    The auto-prototype gate did not apply because constraint 3 failed.

Nothing should be filed upstream.
