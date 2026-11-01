# Finite material-owner completion proposal

This proposal closes the remaining authored IDE, terminal, file-manager and design owner gaps against frozen
application `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`. The [machine
proposal](../audit/depot-material-owner-concrete-proposal-2026-10-18.evidence.json) retains exact owner/task
paths, argument vectors, prerequisite producers, source hashes, output readers and resource reservations. No
new allocation is admitted by this document.

## Deciding existing evidence

The [independent retained-evidence search](../audit/depot-rust-legacy-owner-search-2026-10-18.evidence.json)
inventoried 169 files and searched 168, excluding the static assertion catalog explicitly. It validated 94
original/embedded ZIPs, 3,558 member CRCs and 72 exact-alias controls. No qualifying actual outcome or
binary-enumeration candidate joined the selected native owners. The current music-player host cohort is a
standalone Cargo workspace with music-player binary targets. Its outcomes do not execute the IDE, terminal or
file-manager owners. The [Kotlin owner is already
qualified](../audit/depot-kotlin-gradle-owner-join-2026-10-18.evidence.json) and receives no new allocation.

## Exact finite profiles

### `ide-full`

Owner: `package/ide/app`. Task: `//package/ide/app:test`.

Runner: `depot-ubuntu-24.04-4`; 30-minute job and 27-minute inner campaign. Four CPU, 16 GB documented memory
and 130 GB documented disk. List rate USD0.012/minute; maximum nominal compute reservation USD0.36.

- Build the frozen terminal Containerfile locally in the owned VM.
- Retain its original digest and prerequisite probes; add only an explicit
  clang/clang-devel/cmake/make/openssl-devel/zlib-devel tool layer if absent.
- Build the frozen IDE Containerfile from that qualified terminal base; record resolved stable Rust, Nextest,
  Zig0.15.2, cargo-about0.9.2 and native library identities.
- Run the real frozen runtime fetch/build producer for Helix revision
  ba40e547426b0f9896c8bdc699a4ab11f2b37dbc, retain selected grammar inventory and every staged
  grammar/query/notices hash.

Consumer argument vector: `cargo nextest run --no-fail-fast --locked --no-default-features --lib --tests`.

Reader: Cargo metadata package/target identity plus every actual Nextest binary/case/status and summary; no
success from compilation alone.

### `ide-reader-lifecycle`

Owner: `package/ide/app`. Task: `//package/ide/app:test:reader-lifecycle`.

Runner: `depot-ubuntu-24.04-4`; 30-minute job and 27-minute inner campaign. Four CPU, 16 GB documented memory
and 130 GB documented disk. List rate USD0.012/minute; maximum nominal compute reservation USD0.36.

- Prepare the same exact frozen terminal and IDE toolchain Containerfiles and native prerequisite probes.
- Do not prepare the grammar runtime for this independently labelled owning subset/control.

Consumer argument vector: `cargo nextest run --no-fail-fast --locked --no-default-features --lib`.

Reader: Its own Cargo target and Nextest binary/case identities, labelled subset/control; never credit this
row as the full IDE owner.

### `terminal-release`

Owner: `package/terminal/app`. Task: `//package/terminal/app:test:container`.

Runner: `depot-ubuntu-24.04-4`; 30-minute job and 27-minute inner campaign. Four CPU, 16 GB documented memory
and 130 GB documented disk. List rate USD0.012/minute; maximum nominal compute reservation USD0.36.

- Build the real frozen terminal Containerfile with its checksum-qualified Zig0.15.2 and installed
  Fontconfig/FreeType/Noto payloads.
- Probe compiler, Zig, Nextest, native library/header identities and actual font files; qualify an explicit
  missing-tool layer separately.

Consumer argument vector: `cargo nextest run --no-fail-fast --locked --release`.

Reader: Actual release Nextest binary/case/status outcomes and summary, plus native/compiler/font hashes. No
terminal GUI or compositor launch.

### `file-manager-gtk-release`

Owner: `package/file-manager/app`. Task: `//package/file-manager/app:test`.

Runner: `ubuntu-24.04`; 30-minute job and 27-minute inner campaign. Standard public GitHub compute has a
nominal USD0 model. Actual CPU, memory and disk are probed before the owner.

- Install minimal nightly-2026-10-17 and verified Nextest0.9.148.
- Install build-essential,pkg-config,libgtk-4-dev,clang,libclang-dev,cmake,libssl-dev; require actual
  GTK\>=4.12 before build.

Consumer argument vector: `cargo nextest run --no-fail-fast --locked --release`.

Reader: Cargo package/lib/bin enumeration plus actual release Nextest case identities; GTK startup is not
inferred from pure model/filesystem tests.

### `file-manager-sticky-release`

Owner: `package/file-manager/variant-gtk-sticky`. Task:
`//package/file-manager/variant-gtk-sticky:test`.

Runner: `ubuntu-24.04`; 30-minute job and 27-minute inner campaign. Standard public GitHub compute has a
nominal USD0 model. Actual CPU, memory and disk are probed before the owner.

- Same pinned minimal Rust/Nextest and actual GTK/native prerequisites as the GTK owner.
- Keep the authored ignored Wayland boundary ignored; never run test:wayland or launch compositor.

Consumer argument vector: `cargo nextest run --no-fail-fast --locked --release`.

Reader: Separate owning package/binary/case identities; retain the authored Wayland skip and do not count
path-dependency tests as this owner.

### `file-manager-qt-release`

Owner: `package/file-manager/variant-qt`. Task: `//package/file-manager/variant-qt:test`.

Runner: `ubuntu-24.04`; 30-minute job and 27-minute inner campaign. Standard public GitHub compute has a
nominal USD0 model. Actual CPU, memory and disk are probed before the owner.

- Same minimal nightly-2026-10-17 and Nextest0.9.148.
- Install
  build-essential, pkg-config, cmake, clang, libclang-dev,
  qt6-base-dev, qt6-declarative-dev, qt6-tools-dev, qt6-tools-dev-tools.
- Probe qmake6 and Qt6Core/Gui/Qml/Network, preserve build.rs real QML/cxx-qt producer. The authored
  default-state QString assertion requires no QGuiApplication launch.

Consumer argument vector: `cargo nextest run --no-fail-fast --locked --release`.

Reader: Actual cxx-qt/QML build identities plus executed release Nextest outcomes; no GUI interaction claim.

### `design-authored-node-imagemagick`

Owner: `package/music-player/design`. Tasks: `//package/music-player/design:test:unit`,
`//package/music-player/design:test:cover-picker:revisit`,
`//package/music-player/design:test:feedback:overlay`, `//package/music-player/design:test:first-run:access`,
`//package/music-player/design:test:keyboard-map`, `//package/music-player/design:test:scan-indicator`,
`//package/music-player/design:test:search:filename-comparison`,
`//package/music-player/design:test:settings-pane`, `//package/music-player/design:test:template-editor`,
`//package/music-player/design:test:template-reference`, `//package/music-player/design:test:track-menu`.

Runner: `ubuntu-24.04`; 30-minute job and 27-minute inner campaign. Standard public GitHub compute has a
nominal USD0 model. Actual CPU, memory and disk are probed before the owner.

- Materialize exact frozen design and referenced music-player assets; verify every admitted pointer
  OID/size/hash and actual PNG dimensions before consumers.
- Use locked Node26.12 and a private Fedora41 tooling container with real ImageMagick7 and libstdc++; probe
  actual Node/ImageMagick identities first.
- Run each authored Node owner unchanged, sequentially. All image mutations/builds stay in owned disposable
  fixtures.

Reader: Retain each real owning process status and its actual custom guard output, with generated image
identities only. There is no authored Playwright/browser owner in these tasks.

## Admission and publication boundary

The campaign permits at most two simultaneous jobs, seven primary jobs and two correction jobs. Each job has a
30-minute cap. The three Depot primary profiles reserve USD1.08. Two conditional 16-CPU/64-GB Depot correction
profiles reserve USD2.88, only for an observed named prerequisite, resource or harness failure. The remaining
USD1.04 is explicit storage/LFS/other contingency. The total nominal guard is USD5; these are reservations,
not invoices. Public GitHub storage and any LFS charge remain unobserved.

The local input control qualified 1,582 asset/HTML files, 1,428 LFS materializations and 350,992,617 actual
bytes with no rejected payload. The cloud consumer must repeat the exact admitted owner/shared-payload hashes
and format probes. Raw pointer bytes, raw Git blobs and materialized working bytes remain separate.

Evidence uploads contain at most 64 MiB per job and have one-day retention. Tool images and original/generated
media stay private; public receipts contain metadata and hashes. No Android, emulator, compositor-launch or
desktop application relaunch is included. No source assertions, subscriptions, production routes, model calls
or broad credentials change. The older prepared Rust images are not inputs to these profiles.

Conditional resource corrections also increase the corresponding research container cap to at most 16 CPU, 48
GiB and 4,096 PIDs. An observed producer resource failure may receive the same explicitly recorded bound
adapter. Frozen Containerfile bytes, source test selections and assertions stay unchanged.
