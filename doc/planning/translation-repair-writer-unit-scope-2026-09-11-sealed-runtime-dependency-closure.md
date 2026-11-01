# Writer calibration unit scope: execution dependency closure and the sealed runtime

The experiment that bundles the package runtime so a copied artifact runs without workspace access,
the failures from Satteri's native loader,
the dedicated sealed build and its manifest,
and the standalone consumer and location comparison that verified it.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Execution dependency closure and the sealed runtime

Execution dependency closure still needs evidence.
`corpus-run/pipeline-digest.ts` excludes external runtime dependencies.
Its historical comment says closing that requires repo-wide build configuration,
but the current repository-owned `nodeConfig` accepts a per-package `external` override,
and `nodeExternal` accepts bundle-inclusion patterns.
The deciding source is `package/config/rolldown/src/index.node.ts:145`
and `package/config/rolldown/src/package-externals.ts:185`.
A package-only bundled-runtime experiment can test whether this dissolves the dependency-snapshot problem.
Do not infer dependency closure from a build alone:
inspect emitted imports and exercise a copied artifact without workspace or `node_modules` access.
No build-policy change has been adopted by this observation.

The owned runtime-closure worktree starts at `f896c779f`.
Its package-only JavaScript-bundling prototype is committed as `cb3101583`,
with no auto-push because HEAD is detached.
The native Git trust and scanner-input setup is recorded separately in
[temporary policy inputs](../troubleshooting/cli-git-temporary-worktree-policy-inputs.md).
No guard was disabled;
that temporary trust must be revoked during cleanup.

The JavaScript-only build and types pass,
but the full suite fails while importing Satteri's native loader,
before ordinary test assertions:
`MODULE_NOT_FOUND` for `./satteri_napi.linux-x64-gnu.node`
and `@bruits/satteri-linux-x64-gnu` relative to the emitted chunk.
The actual installed Satteri package and native package are `0.9.5`.
Its generated loader contains a `0.8.1` version-check literal;
that literal is not proof that a different package version was resolved.
Direct resolution from both worktrees confirms the same `0.9.5` JavaScript entry.
Do not follow the loader's generic npm-reinstallation advice for this relocation failure.

The next prototype emits the exact resolved Linux x64 GNU native asset beside the bundled JavaScript
and records emitted import/module metadata.
`runtime-closure-build-20260913.out` retains the failed JavaScript-only run.
The baseline and failed bundle are separately copied and hash-checked before rebuilding;
`artifact-copies.json` in the worktree's `.guard-sandbox` records 380 copied files.
The new run uses fresh `r2` output and runtime-ledger paths under the same resource limits.
The R7 matched-environment full suite passes build,
types and all 629 test-entry processes on Node `26.8.2`.
`test:unit exit 0` was read at line 10374 of `.guard-sandbox/r7/test-unit.out`.
Peak container memory is 1032114176 bytes.
Harness corrections preserve the real command defaults:
independent `/tmp`,
default-path libatomic and a readonly corpus at the temporary home's normal corpus location,
not a global clone-directory override.
The separate failures and their controls remain in
[native runtime bundling](../troubleshooting/translation-repair-native-runtime-bundling.md).

The copied R2 artifact exercises the complete semantic-input consumer with no workspace or `node_modules` mount.
The original artifact fails for missing `p-limit`,
and removing only the native asset from the copied candidate fails for missing native binding.
All 192 copied R2 files match the R7 artifact by name and bytes.
The comparator's actual one-byte-file-change positive control is retained in `candidate-parity-controlled.json`.

The emitted/import audit examines 150 JavaScript files,
790 static AST edges and 794 emitted edges.
Static edges name only Node builtins or existing local output files.
Its 87 indirect observations retain Satteri's native override,
platform and fallback loaders plus literal Node dynamic imports.
These must be classified against an enforced target/environment contract;
static closure is not complete closure.

Independent review recommends a dedicated sealed build,
not changing the default package build.
The experimental `runtime-imports.json` contains source-location paths and must stay outside the sealed artifact.
The native-asset build helper must not remain under published `src` subpaths.
Remaining requirements are GNU-libc/Node/native-asset identity,
pre-import rejection or binding of loader overrides,
final lint,
matched-environment standalone verification and a second-worktree byte comparison.
The development worktree now has a separate `runtime:seal` package task and `rolldown.sealed.config.ts`,
introduced in `67e5468c4` and refined through `382dfe702`.
Named runner entries are shared with the ordinary config without changing its externalization policy.
The candidate output is `package/module/translation-repair/node_modules/.sealed-runtime-candidate`,
outside published `dist/final` and `src`.
It is not an execution directory:
copy it into a fresh frozen location before any reviewed use.

The build checks Linux x64 GNU libc,
compares actual Satteri JavaScript/native package versions,
and records the native asset plus executed Node binary byte identities.
Its relative-path `sealed-runtime.json` excludes source-location paths and timestamps.
Loader-environment absence requirements and system-library responsibility are declarations,
not an implemented pre-import gate.
The native asset's `readelf --dynamic` entries name `libgcc_s.so.1`,
`libpthread.so.0`,
`libc.so.6` and `ld-linux-x86-64.so.2`;
these remain runner/image inputs rather than JavaScript dependencies.

The first independent dedicated-build check passes the ordinary build,
confirms all 150 ordinary runtime files are byte-identical to the frozen baseline,
then passes sealed build and types.
Its physical-file comparison detects declaration chunks incorrectly included in the runtime manifest.
Those are excluded in `5a483507c` rather than counting type-only output as executable identity.
The formatter's readonly-host-callback finding is resolved with an explicit deeply readonly view of consumed fields;
the repeated package formatter reports zero findings.

`sealed-runtime-dedicated-verification-r2-20260913.out` records successful bounded verification.
It confirms the 150 normal runtime files remain unchanged,
checks the manifest against all 151 executable/native files,
and runs all 629 test-entry processes against a copy of the actual sealed artifact.
`test:unit exit 0` was read at line 10378 of `.guard-sandbox/dedicated-r2/test-unit.out`.
The manifest digest is `c6375af734890a5f224e472c5e5e25bb2bd5ed73ee1635cd58a380f3ed742273`.
The normal artifact is restored by asynchronous disposal after the sealed test run.

A second main-created worktree at
`/var/home/user/temp/agent/translation-repair-sealed-location-20260913`
uses source `382dfe702` and the same bounded build environment.
`sealed-location-comparison-20260913.out` confirms identical names and bytes for all 192 emitted files,
including the manifest and declarations.
No policy trust was granted to this build-only worktree.

The final copied consumer checks the independently recorded manifest hash,
Node binary identity,
target,
loader environment and every runtime file before importing the application.
It then reproduces all semantic-input checks with no workspace or `node_modules` mount
and default-path libatomic rather than an `LD_LIBRARY_PATH` override.
Its report is `/var/home/user/temp/agent/runtime-closure-standalone-e3Yxki/report.json`.
This remains a probe of the owning gate contract,
not a production pre-import gate.
Final package lint reports zero warnings and errors across 1485 files.

The durable proof is `/var/home/user/temp/agent/sealed-runtime-proof-20260913`:
744 hash-checked retained copies,
66926586 copied bytes,
151 executable/native files,
629 verified test entries and the independent build-location comparison.
Native policy trust and its recursive authority were revoked before cleanup.
Empty container bind-target directories required checked,
nonrecursive user-namespace ownership repair after Git's partial removal;
the [cleanup observation][input-cleanup-observation]
retains the details and proposed `GCL` clarification.
Both worktree directories and registrations are absent at `2026-09-13T08:34:22.034Z`.

A pre-import gate and native correspondence-root/phase owner remain unfinished.
Place the gate in the actual owning runner before loading the sealed application;
do not introduce a generic execution framework or treat build metadata as approval.

[input-cleanup-observation]:
  ../troubleshooting/cli-git-temporary-worktree-policy-inputs.md#container-backed-cleanup-observation
