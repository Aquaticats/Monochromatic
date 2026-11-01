# cli-git scanner protocol mismatch after copying an older worktree artifact

## Symptom

A new owned worktree inherited a forbidden-strings 0.4.0 binary.
The current cli-git forbidden-strings plugin failed before staging:

```text
Forbidden-strings scanner exited with infrastructure status 2.
```

A direct invocation identified the rejected option:

```text
error: unexpected argument '--name-path' found
```

This was separate from the new worktree's initial `config-untrusted`
diagnostic.
Reviewing and trusting the exact repository configuration resolved the
trust gate,
not the subsequent scanner protocol failure.
No policy was disabled.

## Deciding source and executable evidence

`package/git-policy/cli/src/optional/forbidden-strings/scan-candidates.ts:327`
to `345` pairs logical candidate names with temporary content operands:

```ts
// package/git-policy/cli/src/optional/forbidden-strings/scan-candidates.ts
const nameArguments = logicalNames.flatMap(function nameOperand(name,): readonly string[] {
  return ['--name-path', name];
});
const scannerArguments = [
  ...(builtinRules ? ['--builtin-rules'] : []),
  ...nameArguments,
  ...materialized.paths,
];
```

The same file's line 382 throws the generic infrastructure diagnostic for
an exit other than the scanner's success/finding statuses.
The old artifact's `--help` lists no `--name-path` option,
and its real invocation exits 2 for that option.
The current forbidden-strings 0.4.1 artifact's `--help` lists it.
`package/forbidden-strings/cli/src/cli.rs:251` declares:

```rust
// package/forbidden-strings/cli/src/cli.rs
#[arg(long = "name-path", value_name = "PATH", allow_hyphen_values = true)]
pub name_paths: Vec<String>,
```

The private old artifact measured SHA-256
`3007f75aeacc5ffa64685ed42c63b179e8c7b2da0f9d40e450b428f6be365563`.
The compatible main-worktree artifact measured SHA-256
`73c909716de5d843dfd4792493df75cd23a60e7b8282698cb047bdb8cc1d2538`.
The cli-git worktree-copy output recorded ignored artifact copying;
that does not synchronize old native artifacts with the current wrapper's
protocol.

## Verification and scoped repair

The new first-run design worktree began at `a5560abb2`.
Its current Git wrapper sent logical-name operands to the copied scanner.
The direct old-scanner invocation with those operands exited 2;
the compatible 0.4.1 invocation scanned the actual debug fixture and exited 0.

Only the new worktree's ignored native scanner artifact was replaced with
the inspected main-worktree executable.
Its repository configuration,
rules,
source and the older prototype's artifact were not changed.
Scoped staging and committing then passed the normal guard.
No `--no-enforce-*` option or suppressed policy was used.

Additional disposable controls used one authoritative literal rule:

- Allowed content and an allowed logical name returned 0.
- The same allowed content with a prohibited logical name returned 1,
  with redacted `input=0` evidence.

This verifies both usable protocol and retained pathname enforcement.
The repair is artifact synchronization,
not a change to the scanner's decisions or an assertion that all copied
artifacts are current.
Rebuild via the package's owning task when a compatible,
inspected executable is not already available.

## What did not work

- Trusting the reviewed configuration did not fix the missing scanner
  option.
- Reusing the ignored 0.4.0 binary with current logical-name operands
  failed before scanning.
- Broadly bypassing the guard was not tried and is not a repair.

## Filing decision

Both components are owned by this repository,
not an external upstream.
No third-party issue or patch was warranted.
The observed mismatch and verified local artifact repair are recorded here;
this study did not implement a new artifact-version manager.
