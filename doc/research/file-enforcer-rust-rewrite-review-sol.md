# Outside review of the Rust file-enforcer plan: openai/gpt-6.1-sol

Review of [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md) as of 2026-10-29,
run through `pi --print --no-tools --thinking xhigh` with the planning doc, the four research reports and two decisions attached.
Findings are unverified model output until checked against the repository.

This review uses the attached records only.
No repository or runtime verification was performed.
Findings are ranked by severity, with coverage of the requested sections listed at the end.

## High severity

### 1. The new update command stops before updating anything that installers consume

`doc/planning/file-enforcer-rust-rewrite.md`, “Lock writer”:

> `mpm-shadow update` writes only `toolchain.resolution.json`

The same section says:

> file-enforcer renders `toolchain.lock.json` from `toolchain.plan.jsonc` plus `toolchain.resolution.json`

The toolchain-split research establishes that the replacement installer can consume the lock,
without needing file-enforcer on the installation path.
However, no decision defines the orchestration that turns a successful update into an updated lock and manifests.

A direct replacement of the existing `toolchain:update` task with `mpm-shadow update`
leaves the installation lock unchanged.
The next sync installs the old versions.
There is also no explicit requirement that update reads the **current plan** rather than declarations in the stale lock.
Changing a pinned request could therefore resolve the old request and then render a contradictory lock.

Smallest fix:
keep the accepted resolution-only binary command,
but define `toolchain:update` as resolution followed by file-enforcer regeneration.
Define the corresponding global operation.
Require update to read current declarations,
and reject missing resolutions or resolutions inconsistent with explicit pins before rendering the lock.

### 2. In-memory checking needs an output dependency model that the design does not specify

`doc/planning/file-enforcer-rust-rewrite.md`, “Adopted from the request and repository rules”:

> `check` renders every output in memory and compares,
> needing no prior write

The inventory identifies an existing ordering dependency:

> hidden ordering via a global read cache

It also requires the catalog operation to collect problems before writing:

> All problems collected first, then written, so a failure writes nothing.

The replacement still has dependent transformations:

- Plan and resolution produce the lock; the lock produces the mpm manifests.
- Cargo policy edits and dependency-catalog edits affect the same `Cargo.toml`.
- A missing local forbidden-strings seed must be planned before the concatenated rules file can consume it.
- Catalog adoption can change `rust-catalog.toml` before workspace dependency tables are rendered.

If rules independently read disk,
`check` can render downstream outputs from stale upstream files.
If separate rules independently replace a Cargo manifest,
one transformation can overwrite the other.

Smallest fix:
specify a discovery and planning phase with explicit ordering,
a virtual view of planned outputs,
and composed transformations per destination.
`check` and `write` must use the same final plan.
The dependency-catalog library must return its complete catalog-and-manifest edit batch
only after validating all catalog conflicts.

### 3. The single-switch gate requires published artifacts, not merely successful local builds

`doc/planning/file-enforcer-rust-rewrite.md`, “Cutover”:

> Everything is built first

“Bootstrap” says:

> `mpm-shadow` becomes an archive-route plan entry with release URLs and SHA-256 in the lock

“Adopted defaults” says:

> Distribution:
> published to crates.io and installed by `mpm-shadow sync`

A locally built implementation does not satisfy either installation route.
The cutover can leave Docker or CI fetching an unpublished archive,
or sync installing a published version that lacks the new commands.
Publishing the new file-enforcer binary also requires its new library dependencies to be available first.

There is a second fresh-machine gap.
The accepted fallback is:

> `cargo install monochromatic-mpm-shadow` is the other documented path,
> and the only one on platforms without a prebuilt archive.

The toolchain-split research explicitly identifies Rust as something sync currently provisions.
A machine without a supported archive and without Cargo cannot execute that fallback.

Smallest fix:
make publication and retrieval verification explicit pre-cutover gates.
Publish dependency libraries, the new shadow, and file-enforcer before switching consumers.
Record the bootstrap version and artifact hashes.
For the Cargo fallback, document an independent Rust bootstrap prerequisite.
Specify Docker and CI ordering so neither a prepare task nor file enforcement runs before its binary is installed.

This preserves the owner's single cutover commit.

### 4. The global conversion does not create the new resolution input or define its renderer

`doc/planning/file-enforcer-rust-rewrite.md`, “Global toolchain plan”:

> a one-off converter writes `toolchain.plan.jsonc` beside it,
> keeping comments,
> at mode 0600,
> and replaces the `.data.ts`.

But the toolchain-split research lists both:

> `plan.tools.data.ts`, `plan.artifacts.data.ts`

The new lock requires `toolchain.resolution.json`.
Converting only declarations does not seed versions and artifacts into that file.
The existing global lock can initially conceal this omission,
but its next regeneration has no specified resolution input.

The design also does not assign global lock-and-manifest rendering to a particular per-user config.
`mpm-shadow update --global` is forbidden from writing those files under the sole-writer decision.

Smallest fix:
define a complete global migration:
declarations to JSONC,
versions and artifacts to the resolution file,
then lock and manifest rendering through an explicitly located per-user rule configuration.
Back up both source modules.
Preserve the accepted XDG locations and restrictive permissions.
Define distinct default config paths for repository commands and `user` commands.

### 5. Preserving the bin-paths JSON does not preserve the IDE consumer

`doc/planning/file-enforcer-rust-rewrite.md`, “bin-paths”:

> `mpm-shadow bin-paths` prints the same JSON (`exposureDir`,
> `pathDirs`,
> `bins`) that `package/ide/app/src/language/toolchain.rs` parses today

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md` identifies the actual invocation:

> `ENTRY_POINT = "package/file-enforcer/core/src/toolchain/bin-paths.ts"`

It runs as `node bin-paths.ts` inside a read-only,
environment-cleared sandbox with a time limit.
Deleting the TypeScript package breaks that invocation regardless of JSON compatibility.
Simply changing it to a PATH lookup is not demonstrated to work in that sandbox.

Smallest fix:
add the IDE launcher and `package/ide/app/bin/inspect-language-toolchain.ts`
to the mandatory cutover consumers.
Resolve an available shadow executable,
preserve the sandbox's read-only behavior,
and test the actual sandbox invocation.
Define the diagnostic when the shadow has not been provisioned.

### 6. Removing CI's diff check is unsafe unless generation is removed from the preceding path

`doc/planning/file-enforcer-rust-rewrite.md`:

> CI's `git diff --exit-code` workaround in `.github/workflows/advisory-core.yml:92-105` can go.

`doc/research/file-enforcer-rust-rewrite-inventory.md` says that workflow currently runs:

> `task-run prepare:pnpm`

The same report identifies `prepare:pnpm` as a generation entry point.

A correct new `check` still cannot detect committed drift after `prepare:pnpm` has repaired it.
Replacing the final diff command with `file-enforcer check`,
while retaining the preceding write,
produces a false-green pipeline.

Smallest fix:
specify a CI path that installs prerequisites without regenerating managed outputs,
then runs file-enforcer's check and pnpr's check against the untouched checkout.
Verify it using an intentionally stale committed output in a disposable fixture.
The job must fail before any preparation step repairs that output.

### 7. The mpm lookup choice has not been shown to preserve accepted update semantics

`doc/planning/file-enforcer-rust-rewrite.md`, “Update”:

> npm and PyPI tools:
> `mpm outdated --output-format json`.

`doc/decision/mpm-toolchain-adoption.md` requires:

> `toolchain:update` follows the registry's `latest` dist-tag,
> so a publisher rollback can move a tool down in the update diff

The supplied record does not demonstrate that an installed-package outdated report
is a complete resolver for that policy.

Concrete cases requiring evidence:

- The lock is old, but the installed package already equals the registry target.
  If no outdated row is returned, absence cannot mean “keep the lock.”
- The registry's `latest` tag moves backwards.
  An upgrade-oriented report might omit the required downgrade.
- A new npm or PyPI declaration has neither an installation nor a resolution.
  The accepted auto-sync recovery needs a locked version to install,
  but update needs the installation before querying it.

Smallest fix:
probe the actual manager-specific invocation in disposable homes,
including those cases, pinned requests, channels, and release-age policy.
Document how omitted rows are interpreted.
Define an initial-resolution procedure for new tools that does not silently turn sync into an unpinned install.
Retain mpm wherever the probes establish sufficient information.

## Medium severity

### 8. Strict JSON is not the lock's complete consumer contract

`doc/planning/file-enforcer-rust-rewrite.md`, “Toolchain plan”:

> `toolchain.lock.json` stays strict JSON for its `jq` readers

The toolchain-split research identifies the relied-on keys:

> `tools.<id>.version`, `.artifacts.<platform>.{url,sha256}`, `.options.androidPackages`

The design preserves parsability but does not explicitly freeze those paths,
tool IDs, platform keys, or value types.
A valid JSON redesign can still break Docker, release workflows,
the test harness, and TypeScript readers.

The declaration list also omits fields the research identifies as operational inputs:

> `manager`, `packageId`, `env`

Those are not interchangeable with tool IDs or executable names.

Smallest fix:
state the lock compatibility contract explicitly.
Include the declaration fields or specify their deterministic derivations.
Add consumer fixtures covering the existing jq selectors and typed readers,
not merely JSON parsing.

### 9. Several inventory outcomes have no explicit rule contract or retirement decision

`doc/planning/file-enforcer-rust-rewrite.md`, “Config model”:

> concatenate,
> mirror a directory,
> set TOML keys where a table exists,
> append a block if absent,
> fan out over packages,
> plus named built-ins

The inventory additionally requires:

> `forbidden-strings.append.local.txt` = comment-only seed created only if absent, mode 0600

It requires the concatenated rules file to be mode 0600,
and package license generation to:

> delete known ids that are no longer required

The selected primitives do not specify create-if-absent behavior,
permission enforcement,
license selection,
or pruning of obsolete license outputs.
Fan-out alone supplies none of those semantics.
The catalog section similarly assigns “parsing and rendering” without explicitly carrying
the required adoption, pruning, internal-version rewriting, and conflict behavior.

Smallest fix:
assign each outcome a concrete built-in or structured operation,
or explicitly retire it.
Add create-if-absent and metadata semantics.
Make check compare planned deletions and required modes as well as bytes.
Reference the complete accepted catalog-sync contract rather than describing it only as rendering.

### 10. The cutover omits fixture migrations already required by an accepted decision

`doc/planning/file-enforcer-rust-rewrite.md`, “Cutover”:

> deletes `package/file-enforcer/core`

`doc/decision/monorepo-manager-all-rust.md`, “Consequences”:

> the `file-enforcer-perf` fixture,
> a `prefer-readonly-parameter-type` fixture read,
> and the `sync:files` tasks need migration.

That decision specifically retires the performance fixture
and moves the other read to a fixture that genuinely depends on `@monochromatic-dev/toml-edit-core`.

The rewrite records vm-builder's replacement,
but not those fixture changes.
Deleting the package while leaving a workspace dependency or fixture file read behind
can break installation or unrelated lint tests.

Smallest fix:
include both fixture changes and their dependency cleanup in the cutover checklist.
Search and exercise remaining package imports and file-path consumers before deletion.

### 11. “No lock” has no replacement concurrency policy for structured edits

`doc/planning/file-enforcer-rust-rewrite.md`, “Adopted defaults”:

> Writes:
> temp file then rename,
> only when content differs;
> no lock and no cache.

The inventory requires Cargo splices to preserve:

> comments, key order, unowned keys

Atomic replacement prevents a reader from seeing a partially written file.
It does not prevent lost updates.
For example, watch can read a Cargo manifest,
another writer can change an unowned key,
and watch can replace the file using its older snapshot.

Smallest fix:
define a concurrent-writer policy before confirming this default.
Serialize competing file-enforcer writers and detect changes to splice inputs before replacement,
or provide an equivalent conflict-detection mechanism.
Test overlapping writers and an intervening handwritten edit.
This does not require restoring the old staleness-manifest machinery.

### 12. The static-path claim is incompatible with discovery-dependent rules as literally written

`doc/planning/file-enforcer-rust-rewrite.md`, “Config model”:

> Every input and output path is known from the config without running anything

“Root mise.toml PATH” says:

> The rule globs the `node_modules/.bin` directories that exist and sorts them

“Adopted defaults” says watch reruns:

> the rules whose declared inputs changed

Concrete matched paths are not known from the config alone.
Package license destinations also depend on manifest contents.
Watching only the files discovered at startup misses a new package,
a newly created `.bin` directory,
or a previously absent optional input.

Smallest fix:
distinguish statically declared selectors from paths discovered through filesystem inspection.
Define discovery roots,
absence dependencies,
and glob-membership invalidation.
Watch must rediscover affected rules on additions and deletions.
This does not require an expression language or reversal of the owner's existence-based PATH choice.

### 13. The confirmed JSON-only contract needs parser-output handling

`doc/planning/file-enforcer-rust-rewrite.md`:

> Every line the tool writes is one JSON object

The crate research describes clap's generated help,
version output,
usage, and errors,
but the design does not specify how those paths enter the JSON event stream.

Using clap's ordinary printing path for `--help`,
`--version`, or an unknown flag would violate the confirmed contract
before the engine starts.

Smallest fix:
route all parser outcomes through the event serializer,
including successful help/version requests and command-line errors.
Specify stdout/stderr placement and exit codes.
Test those paths,
not just successful enforcement and rule failures.

### 14. A published binary cannot by itself prove the schema matches current source types

`doc/planning/file-enforcer-rust-rewrite.md`, “Config schema”:

> The schema is derived from the Rust config types with `schemars`

The schema is also:

> kept current by a file-enforcer rule,
> so `check` fails when it is stale

Distribution defaults to an installed published binary.
If a developer changes the Rust config types but runs the pinned installed binary,
that binary can reproduce the old schema and report it clean.

Separately,
handwritten field decoding and schemars derivation are independent implementations.
Derivation alone does not establish agreement about tagged values,
unknown fields, or the config's `$schema` member.

Smallest fix:
require a source-built schema/check task in development and CI.
Add paired acceptance/rejection cases against the handwritten decoder and generated schema.
Keep editor association as a separate verification gate:
JetBrains JSONC `$schema` support is explicitly unverified in the record,
so test it or document a verified manual mapping.

## Remaining verification findings

### 15. The scanner-cache claim has not been exercised at the consumer boundary

`doc/planning/file-enforcer-rust-rewrite.md`, “No scanner step”:

> forbidden-strings compiles from that text when its cache is missing or stale

The evidence cited is a test-source location,
while the reports state that nothing was executed.
If actual scanner invocation fails to recognize a changed rules file,
dropping the post-step leaves enforcement using stale rules.

Smallest fix:
run the real scanner against a disposable rules file,
first without a compiled cache,
then after changing a rule while retaining the cache.
Prove that the changed rule affects a match.
If it fails, fix runtime cache invalidation rather than silently reinstating the rejected post-step.

### 16. The browser rule's consumer compatibility remains unverified

`doc/planning/file-enforcer-rust-rewrite.md`, “browserslist”:

> writes the same JSON array for `package/config/rolldown/src/browserslist-targets.ts`

The owner accepted different browser data.
That does not establish compatible query handling,
production-section selection,
disabled host discovery,
or output strings accepted by the rolldown consumer.

Smallest fix:
resolve the actual repository configuration with the proposed crate,
write the artifact in a disposable checkout,
and exercise the real rolldown consumer.
Test that unrelated host configuration and stats do not influence it.
Do not require equality with JavaScript browser targets;
that difference is already accepted.

### 17. The TOML editor precedent includes necessary behavior beyond selecting toml_edit

`doc/planning/file-enforcer-rust-rewrite.md` lists `toml_edit` for a future vet.

`doc/research/file-enforcer-rust-rewrite-meow-status.md` records:

> TOML = `toml_edit`
> behind a repository wrapper (21 of 21 cases; bare `toml_edit` 12)

The proposed guarded setters and append operations must preserve comments and unowned structure.
The supplied precedent already shows that the bare editor did not satisfy the complete tested contract.
A dependency vet alone does not close that gap.

Smallest fix:
carry the relevant wrapper semantics and conformance cases into the rewrite's acceptance criteria,
then run them against the actual Cargo transformations.
Formatting changes remain allowed;
comment loss and changes to unowned values do not follow from that allowance.

## Lower severity

### 18. The split section still describes the superseded lock-writing responsibility

`doc/planning/file-enforcer-rust-rewrite.md`, “Toolchain split”:

> `update` (network resolution writing lock versions and artifacts)

“Lock writer” subsequently says:

> `mpm-shadow update` writes only `toolchain.resolution.json`

The later decision is clear,
but the earlier ownership summary still invites an implementation that writes lock fields directly.
That would recreate the two-writer problem the separate resolution file was chosen to remove.

Smallest fix:
replace the split section's wording with “network resolution writing `toolchain.resolution.json`”
and state there that file-enforcer alone renders the lock.
No new owner decision is needed.

## Coverage of the requested sections

1. Contradictions: findings 12 and 18.
2. Required outputs and consumers: findings 4, 5, 8, 9, and 10.
3. Adopted defaults requiring correction or clarification: findings 3, 11, and 14.
4. Ordering and bootstrap: findings 1 through 6.
5. Rule behavior that does not yet establish its intended effect: findings 2, 7, 9, 12, and 13.
6. Unverified claims: findings 7 and 14 through 17.
