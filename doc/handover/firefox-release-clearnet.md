# Firefox release installation and clearnet routing

## Requested behavior at 2026-11-03T03:19:26-05:00

The user's instruction was:

> Install Firefox like how we installed Firefox ESR.
> Mark it go through clearnet in our wg set of apps.

Regular Firefox follows the existing Mozilla archive installation layout.
The application exception belongs to the WG packages.
Firefox ESR keeps its existing tunnel policy.

## Installed application

Mozilla Firefox `159.0` is installed at `~/.local/opt/firefox`.
`~/.local/bin/firefox` links to its executable.
`~/.local/share/applications/firefox.desktop` registers the application as `Firefox`,
uses the bundled Firefox icon,
and matches `RemotingName=firefox` with `StartupWMClass=firefox`.
The desktop entry follows the existing ESR entry's fields.
The default browser selection and existing browser profiles were not changed.

The download resolved from Mozilla's `firefox-latest-ssl` product endpoint to
`firefox/releases/159.0/linux-x86_64/en-US/firefox-159.0.tar.xz` on its installer CDN.
`gpgv` verified the detached signature as Mozilla Software Releases.
The primary key fingerprint was `14F26682D0916CDD81E37B6D61B7B526D98F0353`.
`firefox --version` reported `Mozilla Firefox 159.0`.
`desktop-file-validate` accepted the new entry,
`update-desktop-database` registered it,
and `kbuildsycoca6 --noincremental` rebuilt KDE's application cache.

The scratch directory and application directory cross a mount boundary.
The initial scratch installer's `renameSync` returned `EXDEV` before creating the installation.
The completed installer uses `mv`,
matching the existing ESR installation procedure.
Its receipt is `~/temp/agent/firefox-release-install/receipt.json`.

## WG policy

Commit `df2198ee7b` adds release browser discovery and updates the application notices.
Commit `50694b2b12` applies the required documentation wrapping.
The release matcher in `package/wg/cli-quicker-exempt/src/application_targets.rs` accepts:

- Desktop services named `app-firefox@*.service`.
- Executables ending in the complete path components `firefox/firefox` or `firefox/firefox-bin`.

The exact directory requirement excludes ESR,
backup directory names,
and unrelated executables such as `updater`.
Existing Nightly matching is retained.
The established watcher still marks an entire matched cgroup;
process discovery retains its existing rescan interval and initial-socket race.

The rebuilt release companion restarted the application watcher for `fr-par-fr1`,
mark `100`,
desktop UID `1000`.
The WireGuard interface and bypass route watcher were not restarted.
`ip route get 1.1.1.1 mark 100` selected physical interface `wlp9s0`,
table `52000`.
The application watcher reported that its separate sentinel started and confirmed readiness.
Its root session scope was capped at `CPUQuota=200%`,
`MemoryMax=1G`,
`TasksMax=128`,
and `IOWeight=10`.

The startup log also reported pre-existing toolchain processes that could not be moved
because each was its service's main process.
Those process-watch notices are separate from Firefox's socket marking.
The full log is `~/temp/agent/firefox-release-install/activate.log`.

## Verification

The Rust package's nextest run passed `266` tests,
including `firefox_release_service_name_stays_channel_specific`
and `scan_combines_named_and_process_targets`.
Cargo check and release Clippy passed.
The Rust and Markdown lint passed after wrapping the new comments;
an existing `rust/avoidable-closure` warning remains in `src/update_routing.rs`.

The privileged nextest test
`pin_tests::application_watcher_covers_named_targets_and_future_ghostty`
passed against disposable cgroups.
It verified the release and Nightly marks for TCP and UDP over IPv4 and IPv6,
ESR and unrelated cgroups staying unmarked,
and removal of the marks after watcher shutdown.
The first attempt could not find its debug CLI because the host separates Cargo's build and target directories.
The passing invocation set `CARGO_BUILD_BUILD_DIR` to the package's `target` directory for that invocation only,
and used nextest's target runner `sudo --non-interactive --`.

The TypeScript package build,
unit tests,
type check,
and endpoint-routing integration test passed.
Package-wide Oxlint reports existing errors in unchanged files:

- `eslint(max-lines)` in `src/tunnel-bypass-command-fixture.ts`.
- `no-restricted-syntax(no-regex)` and `no-restricted-syntax(no-switch)` in `src/tunnel-bypass-route-shape.ts`.

Those files have no diff against the parent of the Firefox policy commit.

## Consumer verification at 2026-11-03T03:26:30-05:00

The installed browser loaded the HTTPS check page with a disposable profile,
and its real process cgroup appeared in the companion's target listing.
WebDriver BiDi reported browser version `159.0`,
the page title `Example Domain`,
and the page's rendered body text.
The captured screenshot displayed that text.

`ss --tcp --numeric --processes --extended state established` showed an actual Firefox HTTPS socket
with `fwmark:0x64`,
the configured mark `100`.
A route lookup for that socket's IPv6 peer with mark `100`
selected `wlp9s0` and table `52000`.
The IPv4 route lookup with the same mark selected that interface and table too.
This verifies the running browser's clearnet exemption while the tunnel remains active.

The test ended its BiDi session and stopped its disposable browser scope.
`systemctl --user list-units --all 'firefox-release-verification*'` reported no loaded units afterward.
The verification receipts,
socket evidence,
and screenshot live in `~/temp/agent/firefox-release-install/`.
No browser verification process remains.
The application watcher and its sentinel remain active as the installed routing policy requires.

Pandoc's CommonMark renderer accepted this handover and the updated package READMEs.
The rendered output was inspected for the application entries and verification prose.
There is no remaining installation or routing action.

## Package lint at 2026-11-09T14:06:50-05:00

Branch `style/wg-cli-quicker-oxlint-findings` fixes the findings recorded under "Verification",
and `mise run //package/wg/cli-quicker:lint:oxlint` reports `0 warnings and 0 errors` on it.
No rule was loosened and no finding carries a suppression.

- `package/wg/cli-quicker/src/tunnel-bypass-command-fixture.ts` split into sibling modules:
  `tunnel-bypass-command-fixture-types.ts` for the shared types,
  `tunnel-bypass-command-fixture-staging.ts` for the staged copy of the built CLI,
  `tunnel-bypass-command-fixture-sudo.ts` for the privileged runners,
  and the original file for the commands that run inside the fixture namespace and the state reader.
  The staging test is renamed `tunnel-bypass-command-fixture-staging.unit.test.ts` after the module it exercises.
- `package/wg/cli-quicker/src/tunnel-bypass-route-shape.ts` reads a decimal field with a cursor
  and judges each kept keyword with its own branch,
  so the route grammar carries neither a regex nor a switch.
- The host tests import the hosts module once,
  and the stylistic findings in `tunnel-clearnet.ts`,
  `tunnel-traffic-block-rules.ts`,
  `tunnel-traffic-block.ts`,
  and `tunnel-bypass-route-physical.ts` are rewritten in the shapes the rules ask for.

The package build,
unit tests,
the type check,
and the package's Oxlint run pass on that branch.
