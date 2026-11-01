# Dependency footprint

## October 19 refresh

The complete traversal and collection run inside a two-GiB/two-CPU container,
with a 512-MiB Node heap and read-only installed inputs.
The live repair snapshot spans `2026-10-19T04:57:22.737Z` to `2026-10-19T04:57:24.452Z`;
it is not a transactional deployment inventory.

The owned package directory contains 13,349 regular-file paths totaling 106,478,753 bytes.
Deduplicating device/inode pairs gives 24,213,276 unique regular-file bytes
and 29,474,816 allocated regular-file bytes.
Including owned directory and symlink allocation gives 29,691,904 bytes.
Shared dependency symlink targets are excluded from those figures.

`dist` accounts for 99,115,872 apparent regular-file bytes
but 16,850,395 unique bytes because retained immutable releases share files.
The native source tree contributes 2,138,485 bytes,
`src` contributes 2,074,182 bytes,
the package cache contributes 2,115,857 bytes,
and owned files under `node_modules` contribute 805,496 bytes.
Hardlink sharing explains the path/content difference;
it does not justify retaining every release or establish a minimal runtime.
No release was deleted during this refresh.

The independent installed dependency scopes now report:

- `all-installed`: 96 package directories and 255,684,429 unique regular-file bytes.
- `host-required`: 81 directories and 120,136,973 unique regular-file bytes.
- `dependencies-only`: 28 directories and 35,485,785 unique regular-file bytes.
  Required host peers remain excluded from this last scope,
  so it is not runnable deployment evidence.

The receipt owners in the current private scratch are
`bounded-owned-package-footprint.json`,
`bounded-footprint-host-required.json`
and the corresponding named scope receipts.
Optional TypeScript peers and platform binaries still affect the broad installed scope.
Fresh measurement does not resolve the deployment dependency footprint.

## Measurement scope

Package-directory size alone omits shared dependencies behind `node_modules` symlinks.
The 2026-10-18 inventory follows installed package metadata recursively,
records absent edges,
and counts regular files once per device and inode.
It excludes nested `node_modules` from each directory scan,
then visits resolved dependency directories separately.
It excludes development dependency edges.
Workspace source,
tests,
caches and retained builds inside each visited package still count.
Asset symlinks are recorded without following them.

The private receipt owner is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/dependency-directory-inventory.ts`.
Its scopes have deliberately different meanings:

- `all-installed` counts installed production dependencies,
  peers and optional packages for every installed platform:
  96 directories and 252,839,286 unique regular-file bytes.
- `host-required` excludes optional peers and native packages incompatible with the measured
  Linux,
  x64,
  glibc host:
  81 directories and 117,291,830 unique regular-file bytes.
- `dependencies-only` also excludes every peer:
  28 directories and 32,640,783 unique regular-file bytes.
  Required Pi peers are absent from this scope,
  so it is not a runnable deployment closure.

These are named installed-directory snapshots during active repair,
not final package sizes or an executed import graph.
The package itself contributes 80,566,621 apparent regular-file bytes in this snapshot,
including repeated immutable release paths.
That apparent count does not deduplicate shared inodes.
The earlier 22,933,936-byte cleanup snapshot predates subsequent builds.

## Executed module-load checkpoint

The original reported MiMo interaction replays on `native-release-kFkBoY` with the configured extensions,
skills and tool catalogs.
It produces 52 successful tool results,
zero approval dialogs and 62 HTTP 200 Luna responses.
The request audit parses every saved request,
checks 1,025 questions,
passes required-data and excluded-data positive controls,
and finds no excluded structured thinking or raw command fields.
This is a replay checkpoint before the subsequent logger diagnostic repair and MiMo source changes,
not the final post-repair consumer pass.

Node load hooks observe 2,509 unique regular files totaling 14,229,802 bytes.
The collector records 2,691 load events in the actual Pi process;
module aliases and repeated events do not multiply regular-file bytes.
The collector's separate positive controls cover ESM,
CommonJS,
JSON and exclusion of an unimported file.
The measurement excludes loader bootstrap before registration,
native addon binaries not reported by the hooks,
non-module assets,
unexercised interaction paths and system runtime libraries.
It is an executed module set rather than a runnable deployment closure or a package-directory total.

The largest observed source owners are:

- Auto-mode: 332 files and 2,219,293 bytes.
- Pi coding agent: 224 files and 1,983,231 bytes.
- Jiti: three files and 1,717,397 bytes.
- Pi AI: 206 files and 1,409,467 bytes.
- TypeBox: 1,039 files and 1,364,953 bytes.
- Undici: 108 files and 1,187,480 bytes.
- OpenAI: 199 files and 982,350 bytes.
- Pi TUI: 45 files and 653,011 bytes.

These owners belong to the complete configured host,
including its user extensions.
The load set does not establish that each import is necessary or that an unexercised dependency can be deleted.
The first analysis failed because four fixture-home npm paths actually name read-only mounts of the real agent tree.
Resolving those explicit mount aliases accounts for every observed file URL;
the corrected report has no unresolved paths.
Anonymous nested `package.json` files do not replace their nearest named package owner.
The retained receipt is `reported-replay-module-loads.json` under the session's private verification directory.

## Platform variants and optional peers

`valibot` declares TypeScript as an optional peer.
Following every installed peer therefore reaches TypeScript `7.0.2`
and its installed Darwin/Linux,
x64/arm64 native binaries.
Those binaries contribute 110,976,970 regular-file bytes before counting the TypeScript wrapper.
They are excluded from the host-required scope because that scope excludes the optional peer itself.
This is a measurement distinction,
not deletion or a demonstrated runtime improvement.

The repository's `supportedArchitectures` selection intentionally retains these platform families
and both glibc and musl variants.
Its portability rationale is recorded in
the [architecture selection decision](../../../../doc/decision/pnpm-supported-architectures.md).
Changing that shared selection would affect unrelated packages and cross-platform installation.

## Retained parser responsibilities

The parsers have different callers and input grammars:

- Satteri `0.10.5` supplies Markdown trees and original source offsets to policy indexing.
  The installed wrapper contributes 370,691 regular-file bytes;
  the host-compatible GNU/Linux x64 binding contributes 4,146,762 bytes.
  `native/contract/native-index/native-parser.ts` qualifies manifest identity,
  resolver identity,
  supported Markdown options and returned node shape.
- Yuku `0.19.0` parses JavaScript and TypeScript used by the qualified SDK transformations
  and supported program preparation.
  Its host-compatible native binding contributes 692,567 regular-file bytes.
  [Build lifecycle](build-lifecycle.md) and [operation preparation](operation-preparation.md)
  identify the consumers and their execution invariants.
- Unbash `5.1.0` supplies shell parsing through
  `@monochromatic-dev/agent-harness-shared-shell-command-analyzer`.
  Its installed directory contributes 330,535 regular-file bytes.
  The shared analyzer owns that grammar;
  auto-mode retains the parsed operation projection used in actual provider requests.

These package-directory measurements include metadata and declarations,
not only loaded native code.
They do not establish that every internal parser feature is exercised.

## Pi and provider libraries

The host-required inventory includes Pi's installed package directories,
separately from this package's owned transformed SDK artifact.
The installed coding-agent directory contributes 23,199,269 regular-file bytes.
The OpenAI and Anthropic SDK directories contribute 18,477,473 and 13,297,508 bytes respectively.
Those counts include source,
declarations and other installed files;
they are not measured imported-code sizes.
Configured model execution and original host behavior remain required consumer checks.

The resolver inventory records absent declared Bedrock and Google SDK edges.
`pnpm-workspace.yaml` explicitly removes those edges.
The [existing SDK staging diagnosis](../../../../doc/troubleshooting/pi-sdk-staging.md)
explains why published declarations are not the effective workspace selection.
The [dependency audit](../../../../doc/troubleshooting/dependencies.md)
records the lazy-provider and optional-fallback rationale.
Their absence is not a newly demonstrated installation failure.
This inventory does not establish that the removed providers work.

## Remaining completion evidence

Final reporting still needs a fresh package inode/allocation measurement,
reviewed inactive-release cleanup,
source/runtime/build/verification inventories,
and a deployment-oriented import or packaging inventory.
The installed-directory scopes remain useful evidence,
but none substitutes for exercising the configured consumer.
