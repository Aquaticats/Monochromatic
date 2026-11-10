# Vet: tauri-driver (Tauri webview UI testing via WebDriver)

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-07.
 Standard:
 choosing-technology,
 FULL VERIFICATION.
Question:
 is `tauri-driver` + WebdriverIO/Selenium a sound way to UI-test a desktop
Tauri v2 app on Linux?
 Can Playwright drive a Tauri webview as an alternative?

Verdict up front:
 **works on Linux (verified end-to-end),
 recommended as the only
real WebDriver path for a desktop Tauri webview,
 with two material caveats** (native
element click is unsupported on WebKitGTK;
 macOS has no support at all).
 Playwright is
**not** a viable Linux fallback.

## 1. Source and ecosystem audit

Clone:
 `/tmp/agent/tauri-vet` (= `tauri-apps/tauri` monorepo,
 HEAD `66f873d6`,
 2026-06-06).
The tool lives at `crates/tauri-driver` (v2.0.6,
 421 LOC of Rust,
 README still labels it
`pre-alpha`).

Architecture (read in full):
 tauri-driver is a W3C "intermediary node".
 Your WebDriver
client connects to tauri-driver on `--port` (default 4444);
 tauri-driver spawns and proxies
to a platform-native WebDriver on `--native-port` (default 4445).

- `src/main.rs:42-48`:
   spawns the native driver,
   then runs the proxy server.
- `src/webdriver.rs:12-16`:
   native binary is `WebKitWebDriver` on Linux,
  `msedgedriver.exe` on Windows.
   It sets `TAURI_AUTOMATION=true` (v1) and
  `TAURI_WEBVIEW_AUTOMATION=true` (v2) on that process (`webdriver.rs:50-51`),
   which is how
  wry/Tauri enables webview automation.
- `src/server.rs:41-69`:
   on `POST /session` it rewrites the `tauri:options` capability into
  the native shape:
   `webkitgtk:browserOptions {binary,args}` on Linux,
  `ms:edgeOptions` on Windows.
   Only Linux and Windows arms exist.
- `src/main.rs:21-25`:
   **on any other OS it prints "tauri-driver is not supported on this
  platform" and exits 1.
  ** This is the definitive macOS-unsupported proof.

Capability shape (confirmed against official docs
`v2.tauri.app/develop/tests/webdriver/example/webdriverio`):
 no `browserName` required,
just `'tauri:options': { application: <path-to-built-binary> }`,
 host `127.0.0.1`,
 port 4444.

Test/fuzz coverage (source audit):
 the crate has **no unit tests,
 no `tests/` dir,
 no
`fuzz/` dir**,
 and `.github` CI has no tauri-driver/webdriver job.
 For a protocol-rewriting
proxy this is a real gap;
 correctness rests on downstream user reports.

## 2. Maintenance signals

crates.
io (`tauri-driver`):
 first published 2021-06-23;
 latest **2.0.6 on 2026-05-06**;
**186,071 total downloads,
 69,424 recent**.
 Release cadence since the v2 stable line:
2.0.0 (2024-10-02),
 2.0.1 (2024-10),
 2.0.2/2.0.3 (2025-01),
 2.0.4 (2025-04),
2.0.5 (2026-02-04),
 2.0.6 (2026-05-06).
 Roughly quarterly,
 tied to the very active
parent monorepo (same core maintainers:
 lucasfernog,
 amrbashir,
 chippers,
 goosewobbler).

Recent fixes are real maintenance,
 not churn:
 2.0.5 stopped native-driver stdout polluting
the framework-parsed stdout (#14871);
 2.0.4 fixed driver-process leak on exit (#10108);
2.0.3 fixed `ms:edgeOptions` parsing (#12383).

Open webdriver-scope backlog (via `gh`),
 interpreted:

- #6541 native `.click()`/`.setValue()` error -> **reproduced below**,
   open,
   needs-triage.
- #15415 wdio 9+ rejected because BiDi caps (`webSocketUrl`) aren't stripped -> **hit and worked around below**.
- #15156 tauri-driver accepts connections before it is ready (startup race).
- #10670,
   #9203 broken/outdated WebdriverIO + Selenium doc examples.
- #7068 macOS support:
   **open since 2023,
   "help wanted"**;
   the only attempt,
   draft PR
  #15295 ("add macOS scaffold and design doc"),
   has sat as a draft,
   untouched since
  2026-04-25.
   No WKWebView WebDriver exists upstream either.

Read:
 actively released crate,
 but the WebDriver tooling specifically is thin,
 still
"pre-alpha",
 and carries a multi-year backlog of interaction/doc bugs on Linux.

## 3. FULL VERIFICATION (Linux desktop, headless)

Host is Fedora Atomic/Bazzite (immutable,
 `dnf` blocked),
 so the build and run happen in
an isolated Fedora 43 podman container (also satisfies the resource-isolation rule).
WebKitWebDriver is shipped by `webkitgtk6.0` (GTK4 build) on Fedora;
 the wry app links
`libwebkit2gtk-4.1.so.0` (GTK3).
 Same WebKit core 2.52.3,
 so the GTK4 driver automates the
GTK3 webview fine.

Scaffolded app (`/tmp/agent/tdrv/app`):
 minimal Tauri v2 desktop app,
 plain HTML +
`withGlobalTauri`,
 one button `#increment`,
 a `#counter` span,
 a `#rust` span.
 The click
handler bumps the JS counter and calls a real Rust command
`#[tauri::command] fn increment(value: u32) -> u32 { value + 1 }` over IPC,
 writing the
returned value into `#rust`.

Reproduction (exact commands):

```sh
# container
podman run -d --name tdrv --shm-size=1g --security-opt seccomp=unconfined \
  --memory=8g --cpus=8 -v /tmp/agent/tdrv:/work:Z \
  registry.fedoraproject.org/fedora:43 sleep infinity

# deps (inside)
dnf install -y --setopt=install_weak_deps=False \
  webkit2gtk4.1-devel libsoup3-devel gtk3-devel librsvg2-devel openssl-devel \
  gcc gcc-c++ make pkgconf-pkg-config rust cargo nodejs npm \
  xorg-x11-server-Xvfb webkitgtk6.0 procps-ng

# build app, install the published driver, install client
cd /work/app && cargo build                  # -> target/debug/tauri-vet-app (Tauri 2.11.2, wry 0.55.1)
cargo install tauri-driver --locked          # -> tauri-driver v2.0.6
cd /work/wdio && npm install                 # -> webdriverio 9.27.2

# run headless: Xvfb :99 + dbus + tauri-driver --port 4444 --native-port 4445 + node test.mjs
bash /work/run-test.sh
```

Client is WebdriverIO `remote()` (`/tmp/agent/tdrv/wdio/test.mjs`).
 Two real issues had to
be handled to get a pass:

1. `'wdio:enforceWebDriverClassic': true` in capabilities,
    so wdio 9.27 does not request a
   BiDi session that WebKitWebDriver rejects (issue #15415).
2. Native W3C element click returns `unsupported operation` (issue #6541),
    so the test
   tries native click,
    catches it,
    and falls back to an `execute`-script click.
    Element
   find and `getText` work natively;
    only the pointer-interaction endpoints are broken.

Passing output (`/tmp/agent/tdrv-PASS.log`,
 verbatim,
 noise lines filtered):

```text
===== TOOLCHAIN / VERSIONS =====
OS: Fedora Linux 43 (Container Image) (in podman container)
rustc: rustc 1.95.0
node:  v22.22.2
webdriverio: 9.27.2
tauri-driver: tauri-driver v2.0.6
WebKitWebDriver pkg: webkitgtk6.0-2.52.3-1.fc43.x86_64
app links webkit2gtk: libwebkit2gtk-4.1.so.0
===== RUN =====
tauri-driver: /root/.cargo/bin/tauri-driver
WebKitWebDriver: /usr/bin/WebKitWebDriver
port 4444 open
=== running wdio test (APP_BIN=/work/app/target/debug/tauri-vet-app) ===
SESSION created, id=d61dba73-f959-4a62-92e2-796613295957
BEFORE counter="0"
ERROR webdriver: WebDriverError: unsupported operation when running ".../click" with method "POST"
CLICK via execute-script (native click failed: unsupported operation)
AFTER  counter="1"
RUST   result="1"
ASSERT PASS: #counter === "1" after WebDriver click
ASSERT PASS: Rust #[tauri::command] increment(0) returned 1 over IPC
WDIO_EXIT=0
```

This crosses the full integration boundary:
 WebdriverIO -> tauri-driver -> WebKitWebDriver
-> wry webview -> DOM and the Rust IPC bridge,
 headless,
 exit 0,
 reproduced 3x.

Caveat captured:
 native `click`/`setValue` are unsupported on WebKitGTK,
 so real suites on
Linux must drive interactions through `browser.execute(...)` (or the Actions API where it
works).
 That is a meaningful ergonomic tax,
 not a blocker.

## 4. Alternatives (with concrete rejection reasons)

### Playwright against the Tauri webview (rejected for Linux/macOS)

Playwright drives Chromium/Firefox/WebKit through its own protocols and bundled browser
builds.
 To attach to an **existing** embedded webview you need `connectOverCDP`,
 and the docs
state plainly:
 **"Connecting over the Chrome DevTools Protocol is only supported for
Chromium-based browsers.
"** Tauri on Linux is WebKitGTK and on macOS is WKWebView,
 neither
of which speaks CDP (WebKitGTK exposes the WebKit inspector / WebDriver protocol instead).
Playwright's own WebKit target is its patched WebKit,
 not system WebKitGTK,
 and cannot bind
to the running wry process.
 So Playwright cannot drive the Linux desktop webview at all.
Only on Windows,
 where Tauri uses Chromium-based WebView2,
 is Playwright-over-CDP
conceivable (start WebView2 with `--remote-debugging-port` via
`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`),
 and even that is unofficial and untested here.
 It
does nothing for the stated Linux desktop requirement.
 Rejected as the primary tool;
usable at best as a Windows-only side path.

### Maestro (rejected)

Maestro's supported platforms are iOS,
 Android,
 Flutter,
 and web browsers.
 It has no
desktop-native-application support and no way to attach to an embedded WebKitGTK/WebView2
window inside a Tauri desktop process;
 its "web" mode drives a standalone Chromium browser,
not your app.
 It cannot launch or assert against `target/debug/tauri-vet-app`.
 Rejected:
wrong target class (mobile/web,
 not desktop native).

### Note on the chosen tool's own client options

WebdriverIO is the better-supported client than raw Selenium here:
 `selenium-webdriver`'s
`Builder` constructs a browser-specific driver from `browserName` and throws on an unknown
engine,
 fighting the no-`browserName` Tauri capability shape;
 WebdriverIO `remote()` speaks
raw W3C with whatever caps you pass,
 which is why the Tauri docs use it.

## 5. Decision

For UI-testing a **desktop** Tauri v2 app,
 `tauri-driver` + WebdriverIO is the correct and
only real WebDriver path,
 and it is verified working on Linux.
 Budget for:
 WebKitGTK's
unsupported native click/setValue (drive via `execute`),
 the wdio-9 BiDi flag,
 the
startup race,
 thin upstream tests,
 and **no macOS coverage** (plan Linux+Windows CI,
 manual
on macOS,
 per the maintainers' own workaround in #7068).

---

## Summary (< 300 words)

**Works or not (Linux desktop):
 YES,
 verified end-to-end.
** In a Fedora 43 podman
container,
 a scaffolded Tauri v2 app (Tauri 2.11.2,
 wry 0.55.1,
 a real
`#[tauri::command] increment`) was built,
 then driven headless under Xvfb by
**WebdriverIO 9.27.2 -> tauri-driver v2.0.6 -> WebKitWebDriver (webkitgtk 2.52.3)**.
Command:
 `bash run-test.sh` (Xvfb :
99 + `tauri-driver --port 4444 --native-port 4445` +
`node test.mjs`).
 Output:
 `BEFORE counter="0"` -> click -> `AFTER counter="1"`,
`RUST result="1"`,
 `ASSERT PASS` x2,
 `WDIO_EXIT=0`,
 reproduced 3x.

**Two real frictions handled,
 not hand-waved:
** wdio 9 needs
`'wdio:enforceWebDriverClassic': true` or WebKitWebDriver rejects its BiDi caps (#15415);
and **native element `.click()`/`.setValue()` return "unsupported operation" on WebKitGTK
(#6541)** so interactions must go through `browser.execute(...)`.
 Element find and
`getText` work natively.

**Maturity / platform limits:
** crate is healthy on paper (186k downloads,
 2.0.6 in
May 2026,
 quarterly releases on the active tauri monorepo) but still self-labeled
"pre-alpha",
 has **zero tests/fuzz in-crate**,
 and a years-old Linux interaction-bug
backlog.
 Linux = WebKitWebDriver,
 Windows = msedgedriver.
 **macOS is unsupported**:
`main.rs:21-25` exits with "not supported on this platform";
 #7068 open since 2023,
macOS PR still a stale draft.

**Playwright fallback viability:
 NOT viable on Linux.
** Playwright's `connectOverCDP` is
Chromium-only (per docs);
 WebKitGTK/WKWebView have no CDP,
 so Playwright cannot attach to
the Tauri webview.
 Only Windows WebView2 could,
 unofficially and untested.

**Verdict:
** Adopt `tauri-driver` + WebdriverIO for desktop Tauri UI tests;
 it is the only
working WebDriver path.
 Plan around unsupported native clicks,
 the wdio-9 flag,
 and
Linux+Windows-only coverage (manual macOS).
 Maestro and Playwright are rejected.
