# Writer calibration unit scope: the final verification of the producer input runner

The rebuilt bootstrap and application manifests,
the real command-line run and its parity check,
the race and signal controls,
and the retained proof with its cleanup audit.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Current input-runner verification

The final rebuilt bootstrap is frozen at `.frozen-producer-bootstrap-WWrgVV`:
69196 bytes,
SHA-256 `b83a852e6a1dc203a5df1185a257a34db9c20a4ac661dc649359bb876afd46f7`.
The application runtime is `.frozen-producer-input-runtime-fOagX0`,
with 162 files and manifest digest
`b0495c956322e7b32c424f78f5d641c7377ba5acc535741f40aa10a669c0bcdb`.
The bootstrap audit has nine builtin static imports and only the fixed guarded application import.

Launch `input-bootstrap-consumer-5wRE0Y/launch.json` is separately read before execution:
1443 bytes,
SHA-256 `e65934692b540d8c0e6b7f5f3af17774af5520d1a1494b44acd1ce15f0c9070f`.
Its real CLI run is `producer-input-17ff000b-53c2-4b14-b688-c79e4df45232`.
It exits zero and produces 2821409 bytes with digest
`12c4f304dad2a7039c5452ae7f3de5a84160f759e27968e05e47494c0f3eb75a`.
An independent consumer rechecks byte equality with the retained native serialization,
including a one-byte comparator control,
40 parents,
30 entries,
63 references,
45 registrations,
private UID/GID/modes,
closed inventory,
empty home and native container absence.
This remains same-implementation parity and unqualified input reconstruction.

`devfull-WIzgTX` and the later fault-regression `devfull-QpDeVU`
report actual unit exit zero and 631 expected/observed test entries on Node `v26.8.2`.
The ledger does not claim coverage of descendants that remove its preload or environment.
The committed CLI regression now includes unexpected-fault exit `5` without leaking a private error canary.

Fresh frozen JavaScript verification drivers at `.frozen-input-native-checks-EjQapl`
rerun the child,
host,
retention and hook qualification suites against this bootstrap.
Their results are retained under `input-final-native-proof-cQXfCG`.
`.frozen-input-race-checks-iXJdVk` adds launch-file race and repeated-signal evidence
under `input-final-race-proof-OAi94X`.
The regular-file positive observes body reading;
known/replaced FIFOs,
replacement files and symlinks refuse before body reading where expected.
Post-read replacement/modification refuses at the file boundary.
An exclusive UUID collision preserves the existing directory and marker.
Repeated SIGINT/SIGTERM in both orders terminates by the second native signal,
retains files and does not forge completed runner cleanup.
The verification owner independently cleans those fixtures.

`input-final-startup-signals-wUUrF1` adds an application marker,
malformed created inspection,
pre-start bootstrap drift and first interruption during create,
remove and absence verification.
The pre-start refusals produce no start command or application marker.
Interrupted create retains the unconfirmed container;
interrupted cleanup preserves completed output while returning the signal-specific status.
The initial phase fixture selected image inspection too broadly;
its correction requires the exact `inspect-created` command record.

The formatter AST review covers only `79ed34c87` to `7c106717c`.
It identifies Error-brand checking,
nonzero-to-positive checks for nonnegative lengths/extents,
and `import.meta.filename` at the Node CLI boundary.
Other compared ASTs differ only in excluded location/comment/parenthesis representation.
It does not prove later refactors or independent runtime semantics.

Regression ownership and native qualification limits are recorded in
`package/module/translation-repair/producer-input-verification.md` (removed before 2026-10-18).
Private native suites are artifact qualification,
not portable committed native CI coverage or exhaustive package-path coverage.

Initial retention at `input-runner-proof-HbhvIx` copies and verifies 8149 regular files,
343209361 bytes,
with 772 empty directories,
no foreign owners and two metadata-only FIFO records.
Its audited removal plan identifies 69 stopped owned containers and excludes the unrelated container.
No removal occurs before retention is verified.
All worktrees,
corpus files,
frozen originals and the owner's `mise.lock` remain outside cleanup scope.

A scoped attempt to commit five extra terminal newlines creates no commit.
Those agent-owned newline-only differences are reconciled with exact committed bytes,
and only their staging entries are reset.
`input-module-ending-reconciliation-20260913.json` records the before/after hashes.
The reconciliation verification sequence `input-reconciled-verification-20260914.out`
finishes types,
normal build,
bootstrap build,
runtime seal,
complete default unit suite and full lint with native exit zero.
It also verifies that bootstrap and application-manifest hashes remain exactly unchanged.

Final retained proof is `input-runner-proof-kIIDf7`:
9429 copied regular files,
359249852 bytes,
871 empty directories,
no foreign owners and two metadata-only FIFO records.
Every copied regular file is hashed against its source;
source symlinks and special files are recorded without following or opening them.
The matching-owned-process check finds no surviving bootstrap,
verification or diagnostic worker process.
Only the 75 audited stopped container IDs are removed,
and every ID is independently checked absent.
Cleanup completes at `2026-09-14T04:02:28.318Z`.
The unrelated `davincibox` container,
all worktrees,
proof roots,
frozen originals,
corpus data and owner `mise.lock` are preserved.
The root sentinel/ignored-state audit finds no sentinel artifacts to remove;
no mutating Git cleanup runs.

Task47's current-artifact verification is complete subject to final documentation checks.
Its scope remains input reconstruction and artifact-bound native qualification,
not exhaustive package-path coverage,
portable native CI,
root/phase review,
writer calibration or publication readiness.
