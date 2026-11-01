# Logger host source scope correction

The frozen source requires four distinct runner-label contexts across its two host workflows.
The fs-id workflow uses `ubuntu-latest`, `macos-latest` and `windows-latest` with Node `26.5.0`.
The semantic workflow uses `ubuntu-latest`, `macos-15` and `windows-latest` with source-locked Node `26.11.1`.
The earlier Linux/Windows-only semantic interpretation is retracted.
Its source workflow blob is `a091f76946056c0e0920c98fa657219be7ec3f71`.

The unified candidate at `6aefa5d651` omitted the explicit semantic `macos-15` context.
Its historical three-OS development commands remain genuine successes,
but `macos-latest` does not prove the separate authored label or image.
The source matrix and manifest declaration must both be read before classifying a platform as overhead.

## Prepared route and result receipts

Ordered child commits `5db0268856c5179314e6a73125772a5841067eaa` and
`dc5423d3616744d8ac2ea9ff052f02a94f656a2d` preserve the exact source runner-label union:

- `ubuntu-latest` and `windows-latest` run fs-id and both semantic commands.
- `macos-latest` runs fs-id only.
- `macos-15` runs both semantic commands only.

The source command strings and paired source/cohort guards remain unchanged.
Semantic Node setup follows fs-id, so each applicable source command keeps its original runtime.
The result helper records runner label, applicability, declared/setup versions and actual prerequisite runtime metadata.
It requires success for applicable commands and guards, and skipped outcomes for inapplicable commands.
Every other helper, including changed-Cargo formatting, is byte-identical to the parent base.
The primary logger manifest is unchanged in this child patch; its correction and digest regeneration belong
to the parent.

Artifact follow-up `face4f61997c698f1ab1d2f77dfc09ed2f9a7abe` adds the original runner label to upload names.
A same-architecture simulation produces three unique legacy names for four rows,
while the revised names are all distinct.
Only the artifact name changes, and the result helper remains byte-identical to its 75-control version.
This is a preflight counterexample, not an observed provider collision.

## Verification and remaining proof

Four valid actual-Mise result fixtures pass, 70 invalid fixtures fail at their intended diagnostic boundaries,
and the failed semantic outcome is accepted only after removing its exact outcome guard.
The fixture platform metadata is simulated; these controls execute no source host test.
An initial sensitivity fixture lacked `monorepo_root=true`; that setup failure is retained separately and corrected.
Structured workflow/source comparisons and scoped formatting pass.
The companion JSON retains every control, source workflow text/blob and protected-helper hash.

Incident [#1168](https://github.com/Aquaticats/Monochromatic/issues/1168) remains open.
Inspect the exact source matrices, command strings, runtime/applicability gates and sensitivity result when reviewing.
The parent must correct the primary platform declaration and freeze bindings,
then retain actual source/cohort/runtime/outcome proof from the final explicit `macos-15` provider row.
Reconciliation must keep runner label with check ID and platform: both macOS rows report `darwin`.
No duplicate development or final provider dispatch occurred.
