# Node owns the toml-edit CI scope decision

## Status

Accepted on 2026-09-09 through the request:
“Okay,
switch to node and fix it.”
This decision applies to issue 308 and the toml-edit workflow,
not a repository-wide workflow migration.

## Decision and rationale

Replace inline Bash decision logic with the dependency-free Node command
[`package/toml-edit/core.fuzz/src/ci-scope.ts`](../../package/toml-edit/core.fuzz/src/ci-scope.ts).
The [Bash usage investigation](../planning/bash-usage-boundary.md)
found no Bash requirement for this boundary.
Node execution before project setup was verified through the
[hosted runner evidence](../troubleshooting/github-actions-shell-selection.md).

The workflow uses `shell: node {0}`.
Its CommonJS launcher runs the checked-in TypeScript entry with the same Node executable.
This avoids depending on module detection for the runner's extensionless temporary script.
The entry and its local helpers are the deployed artifacts:
no build,
workspace imports,
or package installation is required before scope detection.
The bootstrap runtime floor is Node 22.18.0,
verified by running the source and actual launcher under that executable.

The user selected the already-used Node runtime,
not a new dependency or a new shell implementation.
Keeping Bash would preserve an implementation with no demonstrated requirement.
Adding a third-party path-filter action is unnecessary for the adopted boundary.

## Correctness contract

Push and pull-request events run verification as before.
For a merge group:

- Use the event's `base_sha` and `head_sha`,
  never a moving `origin/main` reference.
- Require full lowercase commit identifiers and available commit objects.
- Require the checkout to match the event head and the base to be its ancestor.
- Compare the exact trees with `git diff --name-only --no-renames -z`.
- Include deletions and both sides of moves.
- Interpret directory prefixes and exact contract filenames,
  not regular expressions over newline-separated output.
- Write `run=false` only after successful comparison proves no relevant paths changed.
- Propagate unavailable revisions,
  Git failures,
  signals,
  output overflow,
  and output-file errors as a failing step.

The event fields are documented in
[Octokit's merge-group payload schema][schema]:
`base_sha` is the merge group's parent commit,
and `head_sha` identifies the merge group.
Checkout retains full history.
The program validates availability rather than fetching a different revision or guessing a fallback.

## Bounds and failure recovery

Each Git call has a 30-second deadline and a 16-MiB output limit.
These are safety bounds,
not permission to truncate or skip.
Exceeding a bound blocks verification.
If a legitimate change set reaches a bound,
retain fail-closed behavior while investigating the input and choosing a measured limit or streaming implementation.

Missing or mismatched event revisions require fixing the checkout or event binding and rerunning the check.
A legitimate unrelated group still succeeds with `run=false`.
The no-op result is not interchangeable with an unavailable comparison.

## Verification

[`ci-scope.unit.test.ts`](../../package/toml-edit/core.fuzz/src/ci-scope.unit.test.ts)
copies the deployed source into disposable Git repositories without `node_modules`.
It extracts the actual workflow launcher,
checks the step's interpreter and event bindings,
and asserts process status and `GITHUB_OUTPUT` contents.

Coverage includes:

- Every declared relevant path and near-prefix nonmatches.
- Empty and irrelevant comparisons.
- Modifications,
  deletions,
  moves into and out of scope,
  embedded whitespace,
  quotes,
  and non-ASCII names.
- A moving `origin/main` that must not replace the event base.
- Missing or malformed revisions,
  checkout mismatch,
  invalid ancestry,
  missing Git,
  and a corrupted tree that fails only when diffing.
- Missing event or output inputs and an unwritable output destination.
- A deliberately fail-open copy whose successful skip is rejected by the failure oracle.

The mutation control changes only the disposable copy,
not committed source or the user's Git state.
Fixture extraction is intentionally tied to the current workflow's literal formatting;
a changed step shape fails the fixture rather than silently selecting unrelated code.

Local verification commands:

```sh
# From the repository root
mise run //package/toml-edit/core.fuzz:test:scope
mise run //package/toml-edit/core.fuzz:lint:scope
mise run //package/toml-edit/core.fuzz:lint:types
```

`SCOPE_TEST_NODE` selects a separate bootstrap executable for subprocess tests.
The installed Node 22.18.0 passed the complete scope suite while the harness ran on Node 26.8.1.
Scoped lint reported no warnings or errors.
[Hosted run 34421626248][hosted-run] passed at
`b5614800c278d7d3c68c2b51eb7ccd6f03a8fc9b`.
The log records `shell: /usr/local/bin/node {0}` for the pre-install scope step,
and the sidecar unit step records the complete scope regression suite passing.
Runtime tests,
fuzz smoke,
toml-test conformance,
and the coverage gate also passed.
The final source revision,
`acff1b695ff98f11dfed08d1aadda03c8ce93b2c`,
also passed [hosted run 34422039208][final-run],
including the pre-install scope step and every verification step.
Local tests on Node 22.18.0 and 26.8.1,
scoped lint,
type checking,
documentation lint,
and `git diff --check` passed.

## Boundaries and revisit conditions

The logger workflow's similar expression remains a separate incident.
This change does not authorize sweeping edits to shell configuration,
agent execution tools,
container entrypoints,
or third-party implementations.

Revisit when the hosted bootstrap runtime changes,
the merge-group event contract changes,
path ownership changes,
or measured inputs exceed the current bounds.
Do not restore an error-to-empty-list fallback as a rollback strategy.

[schema]: https://github.com/octokit/webhooks/blob/main/payload-schemas/api.github.com/merge_group/checks_requested.schema.json
[hosted-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/34421626248
[final-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/34422039208
