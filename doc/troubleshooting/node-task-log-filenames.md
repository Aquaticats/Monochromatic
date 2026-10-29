# Task labels need filename encoding before execution

## Exact diagnostic

The Node research logger in public run `39141147245` used the task label
`//package/test-harness/core:build` in a log basename after replacing only colons.
Node `fs.writeFileSync` emitted `ENOENT` on both Mac and Windows
for a path ending in `004-/package/module/test-build.log`
(with the corresponding Windows separators).
The child had already executed, but its output and receipt row were not retained.
Those lost rows do not qualify the complete shared-build or logger owner.
The immutable first-handoff raw logs are retained in
[`depot-current-host-first-handoff-2026-10-18.evidence.json.gz`](../audit/depot-current-host-first-handoff-2026-10-18.evidence.json.gz).

## Root cause and verified correction

A task label is not a filesystem basename.
The generated logger left slashes in the label, creating unintended path components.
The corrected harness `aa501f4a5c0e74a0a993b668c98b00fe3be2da19` replaces non-basename characters,
limits the basename length, verifies the basename and exclusively creates the log before child execution.
An existing destination therefore prevents child execution before output can be lost.
The actual unchanged logger function passed a bounded local adversarial-name control.
Its negative existing-log control produced `EEXIST` and its child marker remained absent.
The control used Node26.12, 2 CPU, 2 GiB, 128 PIDs,
no network, a read-only root and a 90-second bound; its owned container is absent.
The public recovery artifact independently retains all 63 Mac command logs with matching byte hashes.

## Remaining boundaries

This correction qualifies research logging, not application test success.
The recovered Mac logger owner still has two failed file-batch children.
Its Nextest owner reports 1,495 executions,
1,468 passed and 27 failed, with one skipped.
Passing shared builds and an emitted binary version consumer do not erase these source failures.
The separate hidden-root upload correction is documented in
[`github-artifact-hidden-sdk-resources.md`](github-artifact-hidden-sdk-resources.md).

## Upstream filing decision

This is an integration filename-encoding error,
not an established Node filesystem defect.
No upstream issue or external comment was sent.
