# Writer calibration unit scope: the provider-free producer input runner of task 47

The bootstrap,
host launcher and fixed child owner that run input reconstruction in a bounded container,
their design constraints,
and the first real command-line run that reproduced the unqualified input artifact.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Native input runner under task 47

The next implemented operation is a specialized provider-free `producer-prepare` runner,
not a general command runner or live acquisition mode.
No existing named entry owns it.
`corpus-pass` includes excluded section,
title,
image and archive-repair operations;
`producer-calibrate` owns the later writer measurement.
The existing mutation-test Podman module owns shard reports and an arbitrary argv execution interface,
so its policy cannot substitute for this preparation launch contract.

The bootstrap will be separately packaged from application code.
Its host path owns an independently identified Podman invocation,
immutable image identity,
Node/bootstrap/application/library mounts,
clean loader environment and private output directory.
Its fixed private child path checks the recorded target,
resource and file identities before importing a dedicated inert preparation application entry.
It will not import the package's broad `index.mjs` barrel or expose an ungated application CLI.
The dedicated application calls the already verified `buildPreparationRootInputs` interface.

The trusted caller and host launcher are the authority for selecting the bootstrap and image.
Node and the dynamic linker necessarily run before child JavaScript can hash them;
child checks are cross-checks before application import,
not proof that libraries were validated before their first use.
Bootstrap self-hashes do not authenticate their creator.
Future live acquisition requires its own reviewed launch contract,
not a new mode silently added to this no-network operation.

After independently authenticating the launch roles,
the host may perform metadata-only topology preflight before creating output.
This is an explicit refinement of the initial no-I/O wording:
otherwise an output-parent alias could cause the first directory creation to write inside an input tree.
Preflight may inspect canonical directory paths,
directory device/inode identities and kernel-reported mount topology.
It may only reject,
not approve content or authorize an unregistered input.
It must not enumerate supporting files,
read or hash bodies,
invoke corpus Git operations or import the application.
Canonical paths alone do not identify bind-mount aliases;
those require topology evidence or refusal of ambiguity.
This remains trusted-host,
point-in-time observation,
not a hostile-host filesystem lease.

The run directory must be created exclusively and its marker synchronized before corpus or support-file body I/O.
Pinned directory identities are revalidated before those content operations.
The run retains fixed launch,
unqualified-input and terminal digest/extent records,
with content synchronization and no automatic resume.
It cannot reuse `createPreparationAttempt`'s `root-plan.json` marker,
because this result is not a reviewed correspondence root.
Host ownership of files and empty mount directories is checked explicitly.

The direct metadata probe `input-host-topology-probe-20260913.json` sees 41 host mount records.
The current runtime,
corpus,
supporting root and output parent all lie beneath mount ID `61` at `/var/home`.
The initial host-layout helper deliberately admits one shared unambiguous enclosing mount,
with no stacked ancestor or nested mount inside any participating directory tree.
Cross-mount or ambiguous layouts are refused rather than assigned guessed filesystem equivalence.
This restriction is explicit in `verify-host-layout` diagnostics;
it is not a claim that every refused layout is unsafe.
The helper compares canonical component boundaries and directory identities,
not a presumed equivalence between encoded mount devices and directory `st_dev` values.
The new mount parser and metadata revalidator are not yet wired to the production bootstrap or covered by its tests.

A fresh measurement confirms that all 63 reference locators remain inside the task40 support root,
using only its root and `preparation-pool-policy-reading-20260912` directory.
Their aggregate extent is 3290843 bytes;
the selection is 50399 bytes and still hashes to
`853fd30daef7464865356ac1ffa901de4ba5cb35b4ef80d28cf6a043c36be3be`.
The loader maps validated relative paths to fixed read-only mounts while preserving original locator strings.
Its aggregate byte allowance must come from the independently supplied launch contract.
The earlier 64 MiB probe envelope is not adopted as production policy.

A new sealed artifact is required:
the old `c6375af734890a5f224e472c5e5e25bb2bd5ed73ee1635cd58a380f3ed742273` manifest
predates the semantic owner.
The execution copy contains only the declared runtime inventory and manifest,
not unneeded declarations or arbitrary extra files.
Verification must use the real CLI and corpus,
plus import-marker controls for every pre-import refusal,
image-loader environment controls,
output collisions/interruption and caller ownership.
No new paid operation is authorized by this work.

## Dedicated application, host lifecycle and first real run

The dedicated application,
host lifecycle,
fixed child owner and separate bootstrap CLI are now implemented and built.
The first real bundled CLI run succeeds at the unqualified input boundary.
Negative gate/lifecycle coverage and the final package verification remain unfinished.
`producer-input-application-RzZOcY` executes a frozen `producer-prepare-app.mjs` against the pinned corpus.
Its 162-file runtime manifest hashes to
`1cf95aa2a09d61b75ffdc4d901ebb18f9e3b8cba437e129b29a233161636e850`.
It writes 2821409 artifact bytes with SHA-256
`12c4f304dad2a7039c5452ae7f3de5a84160f759e27968e05e47494c0f3eb75a`.
The result preserves 40 parents,
30 entries,
63 references and 45 registrations.
A deliberate refused fetch first moves the counter;
reconstruction then records zero fetch calls.

The private comparison imports `index.mjs` only after the dedicated application completes,
then verifies exact serialized equality with a direct `buildPreparationRootInputs` call.
This is same-implementation output parity,
not independent parser correctness or a dedicated-entry-only claim for the entire comparison process.
The later bundled CLI runs in a separate process and is compared byte-for-byte with this retained native result.
That comparison is still not an independent implementation of parser or aligner semantics.
The recorded dedicated peak is 182272000 bytes;
the combined comparison process reaches 204419072 bytes with no recorded OOM events.
Its report explicitly says `privateGateOnly:true` and `productionBootstrapImplemented:false`.

Committed helper responsibilities include:

- Closed launch and sealed-runtime decoding,
  raw extent/hash checks,
  exact runtime inventory and running Node identity checks.
- Fixed container arguments with CSV-encoded bind fields.
- Exclusive private input-run control and output directories,
  synchronized launch records and no automatic reuse.
- Owned Podman configuration,
  removal of `CONTAINERS_CONF_OVERRIDE`,
  a private empty hooks directory and an explicit empty default mounts file.

The native Podman probes independently demonstrate config and subscription-mount injection,
then remove the respective fixture markers with controlled inputs.
Those initial probes did not exercise the committed helpers together,
prove an exact mount inventory or provide an OCI-hook positive control.
The real CLI now exercises the host and child owners together;
the adversarial OCI-hook control remains open.
`--default-mounts-file` is marked testing-only upstream;
its intended use depends on verified exact Podman bytes and fail-closed native invocation,
not a general compatibility promise.
The current host profile checks FIPS is not enabled without changing that setting.
See `doc/troubleshooting/translation-repair-native-runtime-bundling.md` for source excerpts and controls.

Independent review identified host binding,
interruption,
allocation and output-verification gaps.
The implementation now has fixed native-command deadlines with termination escalation,
late-signal refusal and signal-specific exit codes,
pre-start Node/library rechecks,
created-container timeout/healthcheck/rootless-map checks,
exact private output inventory and UID/GID/mode checks,
and a synchronized cleanup-complete record after native absence verification.
Launch and runtime manifests have a 1048576-byte host metadata ceiling;
the supporting-byte allowance remains independently supplied.
Podman receives only the measured local home,
fixed system PATH,
owned config and optional runtime-bus variables,
not a copied host environment.
These implementations still require their negative controls and final lint/test coverage.
The helper now writes `created.json` with `producer-preparation-input-created`,
so namespace creation is not labelled container startup.
The constructor now requests 4 GiB combined memory-plus-swap while retaining the 2 GiB memory cap.
This replaces the unverified equal-memory/swap argument,
using Podman's documented combined-limit meaning rather than claiming the older swap allowance was preserved.
The real CLI's child context checks the resulting limits before importing the application.
The shared child-environment helper now includes an explicit hostname and separate caller UID/GID bindings.
`runProducerInputChild` implements that comparison,
fixed executable/argv/cwd checks,
actual cgroup/network/capability/output-directory and mount checks,
then launch,
bootstrap,
Node,
library and full runtime-inventory byte checks before the dedicated application import.
Commit `2be081dc6` passes the exact development type check.
The first bundled CLI executes this child owner successfully.
Its negative gate tests and final full-suite verification remain open.
`initializeProducerInputHost` at `be4aaa681` now owns raw launch matching,
metadata topology preflight,
exclusive namespace creation,
runtime-file bindings and generated Podman configuration together.
Initialization failures after namespace creation retain a names-only pre-container terminal record.
`784c03642` wires the host lifecycle and separately packaged CLI;
`b2ab755fb` implements the independently reviewed hardening changes.
Its exact type check,
bootstrap build and current application seal all succeed in the bounded harness.
The new `bootstrap:seal` task does not change ordinary application output or approve a launch.
Package unit tests depend on that separate build.
A first read-only-artifact test attempt still ran that dependency because `--no-deps`
skips automatic dependency preparation,
not task dependencies.
The verified task-graph flag is `--skip-deps`;
callers using it must explicitly build both artifacts first.
The corrected invocation reaches the tests and exposes missing names-only inventory entries.
After the audited class and optional-locator expression entries are added,
`devtest-u73CI3` passes the built CLI grammar/privacy tests and complete names-only inventory.
This focused result does not cover native lifecycle or every pre-import refusal.

The native `input-child-context-FN03Xs` prototype independently observes the requested
2 GiB memory plus 2 GiB swap,
2 CPU and 512 PID bounds,
a 67108864-byte temporary filesystem,
zero capability words and `NoNewPrivs=1`.
It records the image's `/lib64` to `/usr/lib64` library mount alias and exact platform mount inventory.
These are native flag observations,
not invocation of the new child owner.
A no-writer FIFO blocks the old read-only open until the probe deadline;
`O_NONBLOCK` permits a descriptor-shape refusal and preserves the regular-file positive control.
The file observer now adds an initial nonregular-path rejection,
nonblocking/no-follow open and descriptor identity comparison.
Built file-helper race tests remain required.
The argument constructor no longer requests automatic container removal.
The host now records the stopped container and operation outcome before removal,
then writes `cleanup-complete.json` after its native absence check.
A failed or ambiguous create retains evidence without removing a container merely by name.

The frozen bootstrap in `input-bootstrap-audit-Wkrw97` has 67773 bytes and SHA-256
`9860eb668e5e446df6dfae51f53bf8fb1d9e6041bfabbe58be88b6fa27852180`.
The AST audit's positive control covers named static,
dynamic,
require and code-construction forms.
The built bootstrap has 10 Node-builtin static imports,
one dynamic import of `pathToFileURL(PRODUCER_INPUT_PATHS.application).href`,
and no detected require/eval/Function calls.
The complete compiled child-import function is read separately.
Built help exits `0`;
an out-of-context private child invocation exits `6` before application import.
This is an audit of those loader forms,
not authentication of the bootstrap's creator.

The separately read launch in `input-bootstrap-consumer-bs9muS` is 1443 bytes with SHA-256
`0332157771bc4f1050283140a0503991ded14c903e9bd5734d9de41e92bd0e79`.
It binds the current 162-file application runtime manifest
`b0495c956322e7b32c424f78f5d641c7377ba5acc535741f40aa10a669c0bcdb`,
unchanged selection,
corpus pin and 3290843-byte supporting allowance.
The actual CLI run `69b20af2-1dbe-4121-b1ae-8abbd819f51a` exits `0`.
Its artifact retains 40 parents,
30 entries,
63 references and 45 registrations;
2821409 bytes and SHA-256
`12c4f304dad2a7039c5452ae7f3de5a84160f759e27968e05e47494c0f3eb75a`.
A one-byte comparator control first proves that changed bytes are detectable,
then a separate host probe verifies exact equality with the retained native serialization.
Output directories are `0700`,
files `0600`,
UID/GID `1000`,
and home is empty.
The CLI and independent probe both verify that its container is absent after cleanup.
No reconstruction receipt is root,
phase,
writer or publication approval.
