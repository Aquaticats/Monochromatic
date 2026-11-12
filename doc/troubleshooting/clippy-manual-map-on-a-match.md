# Clippy's `manual_map` on a match that rebuilds `Option::map`

Status as of 2026-11-11T18:40:00-05:00:
measured after the daily update of 2026-11-11,
fixed on `main` by `574063773d`,
and no longer a break.

## Symptom

`mise run //package/git-policy/cli:lint:clippy`
fails with:

```text
error: manual implementation of `Option::map`
   --> src/native/worktree_copying/worktree_copy_prune.rs:258:41
    |
258 |               let parent: Option<&[u8]> = match index.checked_sub(1) {
    | _________________________________________^
259 | |                 Some(before) => Some(components[before]),
260 | |                 None => None,
261 | |             };
    | |_____________^ help: try: `index.checked_sub(1).map(|before| components[before])`
    |
    = help: for further information visit https://clippy.rust-lang.org/main/index.html#manual_map
    = note: `-D clippy::manual-map` implied by `-D warnings`
```

The code built a fresh `Option` from a `match` whose arms only rebuild the
same `Option`,
which is what `Option::map` does.

## When it appeared

The daily update of 2026-11-11 moved the Rust toolchain from
nightly-2026-11-10 to nightly-2026-11-11
(`toolchain.lock.json`,
row `rust`),
and issue 6284 reported this check newly red.

Everything else this check reads stayed the same:

- `package/git-policy/cli/src` and its `Cargo.lock` are identical between the
  commit the check last passed at (`c51b81f981`) and the update commit
  (`637a834afd`).
- The update commits changed `toolchain.lock.json`,
  `pnpm-lock.yaml` and `pnpm-workspace.yaml` and nothing else;
  the Cargo locks group failed and every lock it touched was restored.
- The daily update's own pass before the update,
  on nightly-2026-11-10,
  passed this check in 155.3 seconds;
  its pass after the update,
  on the same source,
  failed.

So the newer nightly's Clippy sees the match on `Option` as a `manual_map`
that the older one did not.

## Remedy

Write the map the lint asks for:

```rust
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_prune.rs
let parent: Option<&[u8]> = index.checked_sub(1).map(|before| return components[before]);
```

`checked_sub` answers `None` exactly where the `match` did,
and the closure returns the parent component,
so the value does not change.

That landed on `main` as `574063773d`
("satisfy clippy manual_map in the worktree copy prune"),
which also means the daily update's fix-forward issue had no code left to fix
by the time it was read.
`mise run //package/git-policy/cli:lint:clippy` then finished clean,
in 3 minutes 0 seconds on the same nightly.

## Limits

- The lint is reported per crate and compilation stops after the error,
  so a crate that fails can raise this finding again for another site once
  the first one is gone.
  A clean `--all-targets` run is what says the crate
  has none left.
- The Rust row follows the newest nightly on purpose
  (`doc/planning/toolchain-always-latest.md`),
  so a newer nightly may extend a lint like this one into more shapes of
  the same code style.
  That is a fix-forward break,
  never a reason to move the toolchain back or
  to allow the lint.
- The fix-forward report could not show this diagnostic when it was opened,
  because the check printed 173,538 characters of cargo loop warnings before
  it;
  see `doc/troubleshooting/cargo-loop-warnings-before-the-first-error.md`.
