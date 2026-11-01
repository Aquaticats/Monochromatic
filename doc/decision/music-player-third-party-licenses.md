# Ship third-party license notices with every music player release

## Status

Accepted 2026-10-15,
for the parts the agent was asked to decide (the tooling,
the allow-lists and where the
notices go).
Two choices were made by default and stayed open to the owner's veto:
the Slint license this project uses (GPL-3.0-only,
see [Slint](#slint)),
and the source offer for the desktop programs (see [Open decisions](#open-decisions)).
The in-app surface was decided on 2026-10-18 (see [In-app surface](#in-app-surface)).

Amended 2026-10-18 by the owner:
the desktop programs use Slint under its Royalty-free license instead of GPL-3.0-only,
and the desktop Settings gets a Licenses row.
The sections below keep the 2026-10-15 text as history, each marked where it is superseded;
the current arrangement is in [Amendment of 2026-10-18](#amendment-of-2026-10-18).

## Context

`.github/workflows/music-player-release.yml` (decision:
`music-player-releases.md`) released the desktop programs and
the Android APK with no third-party license notices.
Before this change:

- The Linux and Alpine archives held `LICENSE` (the LGPL-3.0 text) and nothing else about licenses;
  the LGPL is a set of additional permissions on top of the GPL version 3,
  and the GPL text was not shipped.
- The Windows executables,
  the disk image and the APK carried no license text at all.
- Every desktop program links about 350 to 440 third-party Rust crates (MIT,
  Apache-2.0,
  BSD,
  MPL-2.0,
  Slint and
  others),
  the Rust standard library,
  the bundled Inter typeface (SIL OFL 1.1) and the Material Symbols icons
  (Apache-2.0).
  The Windows GNU program also links the MinGW-w64 runtime statically.
- The APK contains 121 Android library artifacts (Compose,
  Media3,
  WorkManager,
  Kotlin,
  Guava and their
  dependencies) and a native library with about 194 Rust crates and libopus.

Most of those licenses ask that their copyright notice and license text travel with every binary copy
(MIT,
BSD,
ISC,
Zlib,
Boost,
OFL),
the Apache License 2.0 also asks for the license text and any NOTICE file
(section 4),
and the GPL and the LGPL ask for their own text.

## Decision

### Slint

Superseded on 2026-10-18 (see [Amendment of 2026-10-18](#amendment-of-2026-10-18)):
the desktop programs now use the Royalty-free license.
The text below is the 2026-10-15 decision.

The desktop programs are built with Slint 1.18.1,
whose crates declare
`GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0`
(the `LICENSES/` folder of the `slint` crate).
Nothing in `package/music-player` chose one of the three before this decision.
This decision uses Slint under **GPL-3.0-only**,
the same default the IDE package chose on 2026-10-06
(`package/ide/app/about.toml`),
open to the owner's veto.

What GPL-3.0-only means for the releases:

- The program's own code stays LGPL-3.0-or-later.
  LGPL-3.0-or-later code may be combined with GPL-3.0 code,
  and the combined desktop program is distributed under the
  GPL version 3.
  The release notes and the notice files say so.
- Every other license in a desktop program must be compatible with the GPL version 3.
  That is what the desktop allow-list encodes (see [Allow-lists](#allow-lists)).
- The GPL text must accompany each program:
  the notice file carries it twice,
  once as the text the LGPL supplements and
  once as the license of the Slint crates.
- Section 6 asks for the Corresponding Source.
  The release notes point to the tagged tree of the same repository,
  and each crate's notice names the exact
  crates.io version.
  crates.io keeps published versions,
  but its usage policy lets an owner delete a crate under narrow conditions
  (published less than 72 hours ago,
  or a single owner,
  fewer than 1,000 downloads a month and no reverse dependencies;
  in exceptional cases on request to the crates.io team),
  and the GPL keeps the distributor responsible for the source staying available.
  Whether to attach a vendored source archive to each release is an [open decision](#open-decisions).

The alternatives Slint offers,
and why they are not the default:

- Royalty-free license 2.0,
  attribution option (a):
  show the `AboutSlint` widget in an About screen reachable from the
  top-level menu,
  or in a splash screen.
  The accepted design has neither (see [In-app surface](#in-app-surface)),
  so this option is not available without a
  new design decision.
- Royalty-free license 2.0,
  attribution option (b):
  show the Slint attribution badge on a public web page,
  preferably
  where the binaries are downloaded.
  The release notes could carry it.
  It would keep the desktop programs under the LGPL and drop the GPL obligations above,
  at the price of a badge in
  every release.
  The owner can choose it;
  the change is one line in the notes,
  removing `GPL-3.0-only` from `desktop-app/about.toml`
  (the Slint crates then resolve to `LicenseRef-Slint-Royalty-free-2.0`,
  which has to be added there) and the intro
  text of `desktop-app/bin/third-party-notices.mjs`.
- Slint Software License 3.0 (commercial):
  a paid license;
  not considered.

### Tooling

- Rust crates (both apps):
  **cargo-about 0.9.2**,
  the tool the IDE package already uses
  (`package/ide/app/about.toml`;
  its README records the vetting).
  It runs from the repository's mise tools (`cargo:cargo-about` in `mise.no-env.toml`,
  the version in `mise.lock`),
  so no new crate enters either app.
  `--frozen --fail` reads only local sources and stops on a license outside the accepted list or one it cannot
  determine.
- Android libraries:
  **licensee 1.14.1** (`app.cash.licensee`,
  Apache-2.0).
  It reads the license of every artifact of the release runtime classpath from its Maven metadata,
  offline,
  fails `licenseeAndroidRelease` on a license outside its allow-list,
  and writes `artifacts.json`.
  It was run against this build's AGP 9.4.1 and Gradle 9.8.1 before adoption:
  it listed 121 artifacts,
  exactly the 124 `group:artifact` entries of `:app:dependencies --configuration
  releaseRuntimeClasspath` minus the three BOMs,
  all Apache-2.0,
  and with `allow("MIT")` in place of `allow("Apache-2.0")` it failed with 121 errors.
- NOTICE files of the Android libraries:
  a Gradle task in `app/build.gradle.kts` (`collectAndroidLibraryNotices`)
  reads the Java resources of the release libraries.
  APK packaging drops `META-INF/NOTICE*`,
  so without it the Apache section 4(d) notice of kotlinx.coroutines (shipped
  inside `androidx.concurrent:concurrent-futures-ktx`) would be lost.
  It found that one NOTICE file among 84 library archives;
  a scan of the archives in the Gradle cache found the same.
  That scan covered all 121 libraries licensee lists:
  83 have their own `.aar` or `.jar`,
  and 38 are Kotlin Multiplatform modules whose Android files are the `-android` or `-jvm` variant,
  each opened together with the `classes.jar` and `libs/*.jar` inside every `.aar`.
- One script writes both notice files:
  `package/music-player/desktop-app/bin/third-party-notices.mjs`,
  run by the `notices` mise task of each app.

### What a notice file holds

In order:

1. The program's own license:
   the LGPL-3.0 text (repository `LICENSE`) and the GPL-3.0 text it supplements.
2. Assets:
   the Inter typeface with its OFL 1.1 notice (desktop only) and the Material Symbols icons with the
   Apache-2.0 text.
3. The Rust standard library:
   the notice the pinned toolchain ships (`share/doc/rust/COPYRIGHT-library.html`,
   part of
   the `rustc` component),
   converted to text.
4. Desktop only:
   the MinGW-w64 runtime (Zope Public License 2.1 text,
   `desktop-app/notices/mingw-w64-runtime.txt`,
   from mingw-w64 13.0.0) and winpthreads (`desktop-app/notices/winpthreads.txt`,
   from
   `mingw-w64-libraries/winpthreads/COPYING`),
   which only the Windows GNU program links.
   libgcc is under the GCC Runtime Library Exception,
   which asks for no notice.
5. Android only:
   the Android libraries by license,
   with each `group:artifact:version`,
   the license text,
   and the
   collected NOTICE files.
6. Every Rust crate,
   one entry per distinct license text,
   with the crates it covers by name and version and whether
   the text came from the crate's own file or is the standard SPDX text.

One desktop file covers every desktop target (the `targets` list of `desktop-app/about.toml`,
with the
`alsa-fallback` feature of the Alpine builds),
so a program's notice also names crates only another platform links.
That over-inclusion is harmless;
the reverse cannot happen,
because the script checks every target.

### Allow-lists

Desktop (`package/music-player/desktop-app/about.toml`),
as it stood on 2026-10-15 (since 2026-10-18 `LicenseRef-Slint-Royalty-free-2.0` replaces `GPL-3.0-only`
and every entry must be compatible with the LGPL version 3),
in the order cargo-about prefers for a crate offered under
several licenses:
MIT,
Apache-2.0,
ISC,
BSD-2-Clause,
BSD-3-Clause,
Zlib,
Unicode-3.0,
BSL-1.0,
MPL-2.0,
CC0-1.0,
Unlicense,
GPL-3.0-only.
Each is compatible with the GPL version 3,
which the desktop programs are distributed under:

- The Free Software Foundation's license list calls MIT (Expat),
  ISC,
  the 2- and 3-clause BSD licenses,
  zlib,
  the Unicode License v3,
  the Boost Software License,
  CC0 and the Unlicense lax permissive licenses compatible with
  the GPL.
- Apache-2.0 is compatible with the GPL version 3 (not version 2).
- MPL-2.0 is compatible through its section 3.3 unless a file is marked "Incompatible With Secondary Licenses".
- GPL-3.0-only is Slint's.

MIT is first because the crates' own MIT files carry the copyright lines that license asks to reproduce
(the IDE measured the same ordering against Apache-2.0 first and BSD-3-Clause first).
GPL-2.0-only,
the SSPL,
and licenses with field-of-use or non-commercial limits can never be added.
`cfg_block` (through Turso) declares only `license-file = "LICENSE"`;
the file is the Apache-2.0 short notice,
and a `clarify` entry with the file's SHA-256 records that,
so a changed file stops the task for review.

Android native crates (`package/music-player/android-app/rust/about.toml`):
the same list without GPL-3.0-only.
The APK contains no Slint and stays LGPL-3.0-or-later,
so every entry must be compatible with the LGPL version 3;
a GPL-only crate would turn the APK into a GPL work,
which needs a decision first.

Android libraries (`licensee` block in `android-app/app/build.gradle.kts`):
Apache-2.0 only.
Every library in the release classpath uses it,
and the notices script holds its full text.
Adding a license needs both a compatibility check against LGPL-3.0-or-later and its text in the script (an MIT library,
for example,
needs its own copyright line,
which Maven metadata does not carry).

### Where the notices go

- Every desktop program embeds the desktop file:
  `build.rs` copies the file named by
  `MUSIC_PLAYER_THIRD_PARTY_LICENSES` into the build,
  `src/licenses.rs` embeds it,
  and `music-player --licenses` prints
  it and exits without opening a window.
  A build without the variable embeds a short note that names the task,
  so development builds need no cargo-about;
  with `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true` (the release workflow) a missing file fails the build.
- The Linux and Alpine archives hold it as `THIRD-PARTY-LICENSES.txt` next to `LICENSE`.
- The disk image holds it beside the app and inside the bundle (`Contents/Resources/THIRD-PARTY-LICENSES.txt`,
  sealed by the ad-hoc signature),
  plus `LICENSE.txt`.
- The Windows programs are single files,
  so their notices are the embedded copy and a release file of their own,
  `music-player-<version>-desktop-third-party-licenses.txt`.
  The embedded copy reaches a Windows user only from a terminal:
  a double-click opens no console,
  and PowerShell's `>` leaves an empty file for a GUI-subsystem program
  (`doc/decision/music-player-windows-gui-subsystem.md`),
  so the README and the release notes name `| Out-File`,
  `cmd.exe` redirection and the release file.
  The release workflow's check runs `--licenses` under bash.
- The APK holds the Android file as `assets/THIRD-PARTY-LICENSES.txt`
  (`ThirdPartyLicensesAssetTask` in `app/build.gradle.kts`,
  every variant since 2026-10-18, a short note when the file is absent and the release gate is off),
  and it is the release file `music-player-<version>-android-third-party-licenses.txt`.
- Both release files are attested and listed in `SHA256SUMS` like every other file.

### Release workflow

- The `notices` job runs once on Ubuntu before the desktop jobs:
  it installs Rust,
  node and cargo-about from `mise.lock`,
  runs the desktop `notices` task,
  and uploads the file.
  It caches the cargo registry downloads (nothing is compiled).
- The desktop and Alpine jobs download it,
  build with the two variables set,
  check that `--licenses` prints it byte for
  byte and names `slint`,
  `symphonia`,
  `twox-hash` and `turso_core` (plus `windows` on Windows,
  `cpal` and `alsa` on
  Alpine),
  and package it.
- The Android job installs cargo-about with cargo-ndk,
  runs the Android `notices` task (licensee,
  the NOTICE
  collector,
  cargo-about),
  builds with `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true`,
  checks that the APK's asset
  equals the file and names Media3,
  Material3,
  symphonia,
  twox-hash and opusic-sys,
  and ships the file.

### In-app surface

Superseded on 2026-10-18 for the desktop app:
the owner decided to add a Licenses row to Settings on both platforms without mockups,
and the desktop row exists (see [Amendment of 2026-10-18](#amendment-of-2026-10-18)).
The Android row is a separate change.
The text below is the 2026-10-15 state.

The accepted design (`package/music-player/design/decisions.md`) names every Settings row
(D87 left the page empty;
D97,
D99 and the "Auto-play on select" record define the three rows it has now)
and has no About screen,
no menu entry and no splash screen.
Adding a Licenses row or an About screen would be new UI that no decision accepted,
so this change added none.
The notices were reachable through `--licenses`,
the release files,
the archives and the APK asset.
An in-app surface (a Licenses or About row in Settings that shows the embedded text) needed a design decision;
it is
also what Slint's royalty-free option (a) would need.
The human made that decision on 2026-10-18 (see [Licenses row in Settings](#licenses-row-in-settings-2026-10-18)).

## Amendment of 2026-10-18

### The owner's decision

On 2026-10-18 the owner decided, through the agent's question tool:

1. The desktop releases use Slint under its **Royalty-free Desktop, Mobile, and Web Applications License, version 2.0**
   instead of GPL-3.0-only.
2. Settings gets a **Licenses row** on both platforms.
   It needs no mockups: the owner called it simple and low in design impact.
   The desktop row is built; the Android row is a separate change.

This replaces the default of 2026-10-15 ([Slint](#slint)) and answers two of the [Open decisions](#open-decisions).

### What the license requires

The text is `LicenseRef-Slint-Royalty-free-2.0`, read on 2026-10-18 from the file
`LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` of the Slint repository (branch `master`),
from slint.dev (the Royalty-free License on its terms page), and from the same file in every Slint crate 1.18.1 in the
build.
All of them are byte for byte the same (SHA-256 `5167f5056e850419106ab6265efbdca7cba4d99c849d1445ca0bbf6a1e2315fe`).
It is vendored as `package/music-player/desktop-app/notices/slint-royalty-free-2.0.md`.

Section 1 grants a world-wide, royalty-free, non-exclusive license to "use, reproduce, make available, modify, display,
perform, distribute the Software as part of a Desktop, Mobile, or Web Application".
A Desktop Application is "a computer program that is designed to run on a general-purpose computer (PC or notebook),
typically installed and executed locally on the computer's operating system".

Section 2, the condition for distributing the Software in an Application, in the license's words:

> You may distribute the Software as part of an Application, modified or unmodified, provided that You do either of the
> following:
>
> (a) Display the `AboutSlint` widget in an "About" screen or dialog that is accessible from the top level menu of the
> Application.
> In the absence of such a screen or dialog, display the widget in the "Splash Screen" of the Application.
>
> (b) Display the Slint attribution badge on a public webpage, preferably where the binaries of your Application can be
> downloaded from, in such a way that it can be easily found by any visitor to that page.

Section 3 limits the license:
it does not permit distributing Slint alone, using it within Embedded Systems, distributing an Application that exposes
Slint's APIs, or removing license notices from Slint's source.
The desktop programs are Applications for general-purpose computers, expose no Slint API, and change no Slint source.
slint.dev's licensing summary says the same: the royalty-free license "is free, as long as you disclose that you use
Slint" (for example with the `AboutSlint` widget or the badge), and embedded systems need the commercial license.

### How the desktop programs meet it

Option (a) is used.
The Licenses row of Settings opens the Licenses page (`desktop-app/ui/licenses.slint`),
which shows the `AboutSlint` widget imported from `std-widgets.slint` under its header,
and the notice text under that.
The widget draws Slint's "Made with Slint" logo for the light and the dark scheme and links to slint.dev.
A window test (`ui_licenses_window_tests.rs`) fails when the page does not hold exactly one `AboutSlint` inside the
window and below the header.

One word of the license does not map exactly.
Section 2(a) says the screen is "accessible from the top level menu of the Application".
The desktop app has no menu bar.
Its top level is the player page with a Settings button (Ctrl+Alt+S, Ctrl+Comma on macOS),
and the Licenses page is one row under Settings, below an "About" heading.
That is the nearest thing the app has to an About screen reachable from its top level;
whether it counts as the license's "About screen or dialog" is the owner's call
(see [Decisions left to the owner](#decisions-left-to-the-owner)).
Option (b) is used as well since 2026-10-18 (see [The badge on the release page](#the-badge-on-the-release-page)).
It was added so that the doubt about the word "top level menu" does not decide whether the license is met.

### The badge on the release page

Added on 2026-10-18, with the Licenses page kept.
Section 2(b) asks for the Slint attribution badge "on a public webpage, preferably where the binaries of your
Application can be downloaded from, in such a way that it can be easily found by any visitor to that page".
The binaries are downloaded from the GitHub release page, whose text is the notes that the "Write the release notes"
step of `.github/workflows/music-player-release.yml` writes (one template, `notes.md`, for every release).
The badge is the first element of the notes after the two paragraphs that name the version, the commit and the tag,
above the Downloads
heading, so it is on the screen when the page opens.

What Slint publishes, read on 2026-10-18 and not recalled:

- The license (slint.dev, `https://slint.dev/terms-and-conditions#royalty-free`, section 2(b), and the same sentence in
  the repository's `FAQ.md`) links the words "Slint attribution badge" to
  `https://github.com/slint-ui/slint/tree/master/logo/MadeWithSlint-logo-whitebg.png`.
  Neither says how the badge is embedded or where it links.
- `logo/README.md` of the Slint repository (blob `d7ae82c8cfd8`) says: "Use the #MadeWithSlint logo for attributions to
  Slint (e.g. when using Slint under the Royalty-Free License)",
  "If the background switched between dark and light mode, but only one logo can be shown, use the versions with
  whitebg", and "Do not modify the logo by changing the colors, changing of the aspect ration, or other alterations".
  A GitHub release page switches between light and dark, so the `whitebg` file is the one used.
- The `AboutSlint` widget (the 1.19.0 source, `internal/compiler/widgets/common/about-slint.slint`) draws the
  `MadeWithSlint-logo-light.svg` or `-dark.svg` image with the accessible label "#MadeWithSlint" and opens
  `https://slint.dev` on a click.
  The notes use the same label as the alt text and the same link target.
- `https://slint.dev/madewithslint` answers 404; the page of the community showcase is `https://madewithslint.com`,
  which is not the badge page.
  The widget's documentation page (`docs.slint.dev/latest/docs/slint/reference/std-widgets/misc/aboutslint/`) says only
  "This element displays a Made with Slint badge".
- The license of the badge file: all logos in `logo/` are CC-BY-ND-4.0 (`logo/README.md`; `REUSE.toml` annotates
  `**/MadeWithSlint-**.png` with `CC-BY-ND-4.0` and "Copyright © SixtyFPS GmbH").
  The release notes link the file and do not copy, resize or alter it, which the no-derivatives term allows.
  Nothing of it enters the repository or the release files.

The embed form in the notes (GitHub-flavored Markdown, one line):

```md
[![#MadeWithSlint](https://raw.githubusercontent.com/slint-ui/slint/7577a165d15b7a097789cb7c417691ed3f8c1bcf/logo/MadeWithSlint-logo-whitebg.png)](https://slint.dev)
```

The image URL names the commit of the Slint `master` head of 2026-10-18 (`7577a165d15b7a097789cb7c417691ed3f8c1bcf`)
instead of `master`, so a release page keeps its badge after Slint renames or moves the file.
The file at that commit and at `master` are the same bytes (SHA-256
`6edb9b815f94ff9065edbf062a4d1756d43eb3778f94310315259e413260bab6`, PNG, 424 by 194 pixels).
The same notes sentence under Licenses names the badge,
and the "The app has no screen that shows them yet" sentence for Android was corrected to name the Licenses row, which
exists since 0.1.6.

Checks: the workflow text passes `actionlint` 1.7.12, the notes template rendered with pandoc (`--from gfm`) and with
GitHub's own renderer (`gh api markdown`, mode `gfm`) gives an `<a href="https://slint.dev">` around an `<img>` with
the alt text `#MadeWithSlint`, and a dry run of the release workflow prints the notes with the line.
A new Slint release can change the badge name or the license's section 2(b); the runbook says to reread both
(see `doc/runbook/release-the-music-player.md`).

### The SPDX expression of the desktop programs

- The project's own code, in the repository and in each program: `LGPL-3.0-or-later`.
- Slint (10 crates, 1.18.1): `LicenseRef-Slint-Royalty-free-2.0`, chosen from the crates' own
  `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0`.
- Every other Rust crate: the license cargo-about chose from the allow-list, one of MIT, Apache-2.0, ISC, BSD-2-Clause,
  BSD-3-Clause, Zlib, Unicode-3.0, BSL-1.0, MPL-2.0, CC0-1.0 and Unlicense.
- The Inter typeface `OFL-1.1`, the Material Symbols icons `Apache-2.0`, the Rust standard library with the licenses its
  notice lists, and for the Windows GNU program the MinGW-w64 runtime.

A distributed program therefore carries `LGPL-3.0-or-later AND LicenseRef-Slint-Royalty-free-2.0` together with the
third-party licenses above.
A `LicenseRef-` identifier is local to the document that defines it;
here the definition is the vendored file and the Slint section of the notice file.
No GPL work remains in the program, so the GPL text appears only as the text the LGPL supplements.

### What stays unchanged, and why

- The repository `LICENSE` (the LGPL-3.0 text) and the `license = "LGPL-3.0-or-later"` field of
  `desktop-app/Cargo.toml` and `truepeak-core/Cargo.toml`.
  The field names the license of the crate's own source,
  which the Slint license does not change; the Slint license covers Slint, not this code.
  `file-enforcer.config.ts` manages these manifests, and running `node package/file-enforcer/core/src/cli.ts`
  after this change produced no difference in them.
- Android: the APK contains no Slint, so nothing changes there.

### Compatibility of the remaining licenses

The program is LGPL-3.0-or-later, so every other license in a desktop program must be compatible with the LGPL
version 3.
That is the bar the Android native crates already meet (`android-app/rust/about.toml`),
and the desktop allow-list is now the same list plus the Slint identifier, in the same order.

- MIT, ISC, BSD-2-Clause, BSD-3-Clause, Zlib, Unicode-3.0, BSL-1.0, CC0-1.0 and Unlicense are permissive licenses that
  the Free Software Foundation lists as GPL-compatible,
  and a license that can be combined with the GPL version 3 can be combined with the LGPL version 3,
  which is that license with added permissions.
- Apache-2.0 is compatible with the GPL version 3, and so with the LGPL version 3.
- MPL-2.0 is file-level copyleft and lets the larger work carry other terms,
  so it combines with an LGPL-3.0 program as it did before.
- The cargo-about gate confirms the whole graph: with `GPL-3.0-only` out of the list and the Slint identifier in,
  it accepted all 570 crates, so no crate needs the GPL.

### Tooling changes

- `about.toml` accepts `LicenseRef-Slint-Royalty-free-2.0` in place of `GPL-3.0-only`.
- cargo-about 0.9.2 accepts the Slint crates but writes no text for a `LicenseRef-*` license, so the Slint crates are
  missing from its text list.
  `bin/third-party-notices.mjs` finds them (the crates cargo-about lists whose manifest offers the identifier and that
  have no text), requires each crate's own license file to match the vendored one by SHA-256,
  and writes a Slint section with the text.
  Any other crate without a text stops the script.
  The cause is in `doc/troubleshooting/cargo-about-licenseref-text-is-dropped.md`.
- The notice intro, the release notes of `music-player-release.yml`, the README and the runbook no longer say the
  programs are GPL works.

### The Licenses page and startup

The notice file is about 2 MB and 40,000 lines.
The window does not hold it as a Slint string and the page builds nothing at startup:
opening the page gives it a model (`ui_licenses.rs`, `NoticeLines`) that slices the embedded text one line at a time,
and the ListView builds only the lines in view (a window test feeds 40,000 lines and finds fewer than 150 built).
Closing the page releases the model.

Measured on 2026-10-18 with release builds that embed their own notice file, `main` at `5b6e6d3061` against this branch:

- Opening the page builds the model for 40,039 lines in 833 to 1,098 microseconds (four runs of the real app in the
  nested compositor, from the `opened the Licenses page` log line).
- Startup, taken as the app's own log timestamps from its first line to the first state line with a folder shown
  (`ui_state_log`), five launches per batch.
  Batch one: `main` 90 to 105 ms (fifteen launches, median 95), this branch 90 to 101 ms (ten launches, median 92).
  Batch two, run later: `main` 109 to 126 ms (ten launches, median 114), this branch 102 to 113 ms (ten launches,
  median 106).
  The two batches of the same build differ by about 20 ms, more than any difference between the builds,
  so no startup cost is measurable.
- Binary size: `main` 55,544,040 bytes, this branch 55,816,872 bytes (272,832 more).
  The first build of this branch was 57,790,760 bytes: `NOTICES` was a `const`, which Rust copies into every crate that
  uses it, so the window crate carried a second 2 MB copy beside the library crate's.
  It is a `static` now, and a test compares the address `--licenses` writes from with the address the page reads.
  A build without the AboutSlint widget was 43,648 bytes smaller, so the widget itself costs that much.

### Decisions left to the owner

- Whether the Licenses page counts as the "About screen or dialog accessible from the top level menu" of section 2(a).
  Section 2(b) is met as well by the badge in the release notes, so the answer no longer decides whether the
  license is met.
- Whether the Licenses row should carry the name About, or the page a title that includes it.

## Measurements

Taken on 2026-10-15 at the branch `feat/music-player-license-notices`:

- Desktop:
  cargo-about covered 570 crates in 253 license texts.
  Third-party crates linked per target (`cargo tree --edges normal,no-proc-macro --features alsa-fallback`,
  path crates
  left out):
  438 on x86_64 Linux glibc and musl,
  437 on aarch64 Linux glibc and musl,
  362 on aarch64 macOS,
  350 on
  x86_64 Windows MSVC and GNU,
  348 on aarch64 Windows;
  501 across all targets,
  every one of them in the notices.
  The other 69 crates cargo-about lists are reached only through procedural macros.
  With procedural-macro edges,
  `cargo tree` names 576 crates across the targets;
  the 8 of those that cargo-about
  leaves out (rayon,
  rayon-core,
  crossbeam-deque,
  rspolib,
  natord,
  snafu,
  snafu-derive,
  toml_writer) are compile-time
  dependencies of procedural macros,
  none linked.
  The file was 1,998,225 bytes then and is 2,004,190 bytes on 2026-10-18 after merging `main`;
  the Rust standard library notice is about 1.4 MB of it.
- Android native:
  232 crates in 118 license texts;
  193 linked on aarch64,
  194 on x86_64,
  all covered.
  The APK's file is 1,748,650 bytes.
- Positive controls:
  removing `GPL-3.0-only` from the desktop allow-list made cargo-about exit 1 naming the Slint
  crates;
  `allow("MIT")` alone made `licenseeAndroidRelease` fail;
  a desktop build with
  `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true` and no file failed in `build.rs`.

Re-run on 2026-10-18 at the branch head after merging `main` (each number from a command run that day):

- Desktop: the `notices` task covered 570 crates in 253 license texts.
  `cargo tree --edges normal,no-proc-macro --features alsa-fallback --target x86_64-unknown-linux-gnu` names 440
  lines, 2 of them path crates (`music-player`, `truepeak-core`), so 438 third-party crates, all in the file
  (the task reports 438 for that target).
  The union over the eight targets is 501 and, with procedural-macro edges, 576; the only ones of those 576 that
  the file lacks are the 8 compile-time dependencies of procedural macros named in the 2026-10-15 measurement
  (rayon, rayon-core, crossbeam-deque, rspolib, natord, snafu, snafu-derive, toml_writer).
  `twox-hash 2.1.5`, `slint 1.18.1`, `symphonia 0.6.1` and `turso_core 0.8.2` are entries of the file.
- A debug build with the file embedded prints it byte for byte with `--licenses` (2,004,190 bytes), also into a closed
  pipe with exit status 0.
  Removing `GPL-3.0-only` from a copy of `about.toml` made cargo-about exit 1 with
  `failed to satisfy license requirements` for the Slint crates.
  Building with `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true` and no file failed in `build.rs`.
- Android: `:app:dependencies --configuration releaseRuntimeClasspath` lists 124 `group:artifact` entries and
  `artifacts.json` of `licenseeAndroidRelease` (forced to rerun) 121, all Apache-2.0; the three missing are
  `compose-bom`, `kotlinx-coroutines-bom` and `kotlinx-serialization-bom`.
  With `allow("MIT")` in place of `allow("Apache-2.0")` the task failed.
  The notice file (1,748,650 bytes) names `media3-session`, `material3`, `kotlinx-coroutines-core`, `guava`,
  `work-runtime`, `symphonia`, `twox-hash` and `opusic-sys`; the NOTICE collector read 1 file from 84 libraries.
  The desktop and Android verification tasks passed on that head:
  1,466 nextest tests (1 skipped), clippy and `lint:slint` clean, detekt and `lintDebug` clean,
  1,289 Android unit tests with 0 failures in 127 result files.

## Consequences

- A dependency update that brings in a license outside an allow-list fails the release (and the `notices` tasks) until
  someone reviews it.
- Every desktop binary grows by the size of the notice file (about 2 MB) and the APK by about 1.7 MB.
- A local `mise run //package/music-player/desktop-app:build` embeds the short note,
  not the notices;
  run the
  `notices` task and set `MUSIC_PLAYER_THIRD_PARTY_LICENSES` to get them.
- The desktop release path depends on one more job,
  which runs before the build jobs.

## Licenses row in Settings (2026-10-18)

The human decided through the question tool on 2026-10-18:
add a Licenses row to Settings in both apps,
"doesn't need mockups, just do it, this one is pretty simple and low-impact in terms of design".
No mockup was made, by that decision; the design record is D132 in `package/music-player/design/decisions.md`
(written 2026-10-18, after the rows were built), and this section holds the build facts.

The Android app (branch `feat/music-player-android-licenses-row`) does this:

- Settings ends with a third group under an About heading,
  after Templates and Playback.
  Its one row is Licenses,
  with the supporting line "Open-source libraries used by this app",
  the 72 dp row form, the 40 dp icon circle and the same group rounding as the other rows,
  and the Material Symbols `license` icon (same pinned commit as the other Settings icons;
  it was `description` until 2026-10-18, when it became `license` to match the desktop row).
  The row is a button (role Button), a Tab stop after the Playback switch, and Enter and Space open it.
- The row opens a page of its own on every window width,
  also on the unfolded screen where Settings has two panes:
  a Settings row there shows its content in the right pane (D110),
  but the 1.7 MB notice text is a document, not a setting,
  so it takes the window (the right half on a creased screen, so no text crosses the fold).
  Back (the button or the system gesture) returns to Settings,
  and on the unfolded screen Escape does too.
  Putting the page into the right pane instead is an open choice for the human.
- The page has the Settings header (Back button labelled "Back to Settings", title Licenses) and a lazy list.
  The text is the APK asset `THIRD-PARTY-LICENSES.txt`,
  read on the IO dispatcher and cut into blocks on the default dispatcher
  (`core/LicenseNoticeBlocks.kt`: section titles, subsection titles,
  wrapped prose joined into paragraphs, other lines kept in a monospace face; at most 40 source lines per block).
  The real file becomes 6,649 blocks.
  Titles carry the heading role and differ by size, weight and color;
  a spinner with the words "Loading licenses" shows while it loads, and a message shows if the asset cannot be read.
  The list is one keyboard stop with a ring inside its edge;
  Up, Down, Page Up, Page Down, Home and End scroll it.
- Every build variant now carries the asset,
  not only release.
  A build without the `notices` task's output packs the short note,
  so `build` and the instrumented tests need no cargo-about;
  a build with `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true` (the release workflow) still fails on a missing file.
  Checked on 2026-10-18: `:app:thirdPartyLicensesAsset` with the variable and no file failed,
  and `assembleDebug` without the variable and without the file packed the 258 byte note.
- `desktop-app/bin/third-party-notices.mjs` names the `license` icon in both notices
  (the Android notice named `description` until 2026-10-18).

Measurements on 2026-10-18 (debug APK, Pixel 9 Pro Fold emulator on the host GPU, the real 1,748,364 character file,
5 repeats each, run from logcat and `dumpsys gfxinfo`):

- Open, process cold (force-stop, start, Settings, tap the row), route change to the first frame with the list:
  267, 264, 276, 272 and 265 ms (median 267 ms, 264 to 276).
  Of that, reading the asset took 8 to 9 ms and splitting 181 to 188 ms.
- Open, process warm (Back, tap the row again): 199, 147, 140, 131 and 122 ms (median 140 ms, 122 to 199;
  the splitting falls from 124 to 60 ms as the code warms).
- Scroll, eight swipes down and eight up (250 ms each) per window, five windows:
  901 to 907 frames, 29, 29, 29, 29 and 30 janky frames (3.2 to 3.3 percent),
  50th to 99th percentile of the frame time 16 ms in every window.
  This is a debuggable build on an emulator;
  the 2026-10-15 track list windows (1,000 tracks, top) had 4 to 16 janky frames of 724 to 838
  (`doc/audit/music-player-release-build-measurements-2026-10-15.md`), so the two are not the same build.
- An earlier set on the same code before the indentation fix (one more emulator session):
  cold 267 to 295 ms, warm 133 to 176 ms, janky 13 to 24 of 928 to 938 frames.

The desktop row and the Slint license switch are recorded by the desktop branch.

## Open decisions

As of 2026-10-15; the first and the third were decided on 2026-10-18, and the second no longer applies to the
desktop programs (see [Amendment of 2026-10-18](#amendment-of-2026-10-18)).

- Slint license:
  keep GPL-3.0-only (the desktop programs are GPL-3.0 combined works) or switch to the royalty-free
  license with the attribution badge in the release notes (programs stay LGPL).
- Source offer for GPL-3.0:
  rely on the tagged tree plus crates.io,
  or attach a `cargo vendor` archive of the exact
  crate sources to every release.
- In-app surface:
  decided on 2026-10-18, a Licenses row in Settings of both apps
  (see [Licenses row in Settings](#licenses-row-in-settings-2026-10-18)).

## Alternatives rejected

- cargo-deny `licenses`:
  checks licenses against an allow-list but writes no notice file;
  cargo-about does both,
  and
  the repository already uses it.
- cargo-bundle-licenses:
  writes a license list but has no allow-list failure and is not used in the repository.
- A committed,
  generated notice file checked by a task:
  it would avoid the extra job,
  but every dependency update
  would have to regenerate a 2 MB file in the same commit,
  and the release would fail on a stale one.
- Generating the notices in every desktop job:
  cargo-about would have to run on six runners and inside the Alpine
  container;
  one Linux run covers every target.
- AboutLibraries 15.3.0 (Android):
  run on the same build,
  its strict mode also failed on a disallowed license,
  but it
  merges the per-platform artifacts (86 entries against licensee's 121),
  fetches license texts from the network unless
  `offlineMode` is set (and then has none),
  and adds `res/raw/aboutlibraries.json` for a UI library this app does not
  use.
  licensee is offline by design and lists exactly what the classpath contains.
- Google's oss-licenses-plugin:
  no allow-list failure,
  and showing its output needs the Play services
  `play-services-oss-licenses` library and its activity,
  which the design does not have.
