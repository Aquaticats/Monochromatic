# Kitesurf as a replacement for `agent-browser`

## Status

Screening record,
not final and not approved by the owner.
A Claude Code session (Opus 5.5) wrote it at 2026-11-13T16:49:35-05:00,
answering "Should we replace agent-browser with kitesurf.dev?".
It used vendor pages,
the installed `agent-browser --help`,
and searches of this repository.
It did not clone,
build,
or run Kitesurf.
The same session corrected it at 2026-11-13T16:56:35-05:00,
after a second pass by the advisor model (same vendor, so not an independent review),
in the Kitesurf build requirements,
the source for the Boa fallback,
the file that holds the stay-on-this-machine rule,
and the reading of the EAGAIN troubleshooting doc.

Kitesurf exits at screening,
so there is no vet report under `doc/audit/`:
no serious alternative was promoted,
and no broad discovery ran,
because the only candidates were the one the question named and the incumbent.

## Verdict

Keep `agent-browser`.
Do not replace it with Kitesurf in any of the forms Kitesurf comes in.

## What each tool is

### `agent-browser`

Version 0.43.0 is installed through the repository's global toolchain exposure.
It is the command-line driver agents call:
`open`,
`eval`,
`click`,
`screenshot`,
`errors`,
`set viewport`,
`set media`,
named `--session`s.
By default it launches a local Chromium.
Its `--help` also lists `lightpanda` and `obscura` as `--engine` values besides the default `chrome`,
and `--cdp <port|url>` with `--cdp-headers <json>` to attach to any Chrome DevTools Protocol endpoint.

### Kitesurf

Kitesurf is a browser engine,
not a driver.
It runs on Cloudflare Workers and exposes a Chrome DevTools Protocol endpoint
([landing page][kitesurf-landing]).
Layout and paint come from Blitz and CSS from Stylo ([repository README][kitesurf-readme-engine]);
page JavaScript runs in Dynamic Workers,
with Boa in WebAssembly "[as the last resort][kitesurf-boa-fallback]" ([announcement][kitesurf-open-source]).
It comes in these forms:

- the public playground at kitesurf.dev,
  which agents can try "[without an account][kitesurf-playground-no-account]";
- managed Browser Run,
  opted into with `browser=kitesurf` on a Cloudflare account with an API token;
- a self-run engine,
  possible since Cloudflare published the source under Apache 2.0 on 2026-11-12
  ([announcement][kitesurf-open-source]),
  started with `wrangler dev` on `127.0.0.1:8787`
  and built with Bazel ([README][kitesurf-readme-bazel]).

Because Kitesurf is an engine,
the only way it fits the existing workflow is behind `agent-browser --cdp`.
The stack the landing page advertises swaps the driver too,
for `chrome-devtools-mcp`.

## Why not

### Rendered verification would lose its owner

This holds for every form.
The repository uses `agent-browser` to confirm what a real browser shows:
`AGENTS.md` requires loading agent-authored pages,
checking console errors and rendered state,
and checking light and dark color schemes.
Recorded uses include:

- `doc/runbook/css-refactor-computed-style-regression.md`:
  computed-style diffs across viewport widths,
  with hover and focus,
  against two local servers;
- `doc/troubleshooting/axe-modal-dialog-contrast.md`:
  contrast checks after `set media light`;
- `set viewport` across several `agent-browser` troubleshooting docs,
  and `--allow-file-access` for local files.

Cloudflare's own Kitesurf page says it trades away
"[pixel-perfect rendering][kitesurf-pixel-perfect]",
cannot "[play video or render WebGL][kitesurf-video-webgl]",
and cannot start a "[long-running, authenticated session][kitesurf-auth-session]" with persistent state.
Its layout and style engines are not Chromium's,
so a check that passes on Kitesurf does not show how Chromium-based browsers render the page.
Kitesurf cannot take over this job,
and that alone rules out the replacement.

### Hosted forms cannot reach the pages being checked

The playground and Browser Run run on Cloudflare's network.
The rendering checks this repository records run against `file://` paths or loopback servers,
which a remote engine cannot open without a tunnel exposing them to that host.
`CLAUDE.md` keeps agent-authored pages on this machine.
This record did not check whether Browser Run keeps copies of page content.

### The self-run form is new and has a heavy build

The source was published one day before this record.
The README calls the engine "[experimental][kitesurf-readme-engine]"
and the announcement says it is "[under heavy development][kitesurf-heavy-development]".
A local engine needs Bazelisk (Bazel then downloads its own pinned Rust toolchains),
Node 22 or newer,
and pnpm 9.15.4,
and its first build compiles the Rust core.
This repository's `toolchain.plan.jsonc` has `pnpm` and `wrangler` entries but no Bazel or Bazelisk entry.

## What Kitesurf would offer

Cloudflare reports 3 to 7 times less CPU and memory than warm-pool Chromium,
and 1.7 to 1.8 times longer wall time,
as "[medians of five Browser Run Quick Action runs][kitesurf-benchmark]" over its own corpus of URLs.
That is a hosted benchmark,
not a measurement on this machine.

No repository record ties an `agent-browser` failure to Chromium's CPU or memory.
`doc/troubleshooting/agent-browser-eagain.md` records `Resource temporarily unavailable`
during heavy parallel use,
but its root cause section names the per-user task ceiling and a read timeout,
so it is no evidence that a lighter engine would help.
The `agent-browser` troubleshooting docs cover versions 0.36.0 to 0.39.1,
not the installed 0.43.0,
so nobody has shown they are current defects.

## Ranking

Best first:
`agent-browser` launching local Chromium,
then `agent-browser` driving Kitesurf over `--cdp`,
then `chrome-devtools-mcp` driving Kitesurf.

### `agent-browser` launching local Chromium

In its favor,
Chromium is the engine the verification rules need to observe,
local pages stay on this machine,
and every runbook and troubleshooting doc already targets it.
Against it,
Chromium costs CPU and memory under parallel agent use.
It ranks above Kitesurf behind `--cdp` because both keep the same driver,
but only Chromium renders the way the checked pages will be seen.

### `agent-browser` driving Kitesurf over `--cdp`

In its favor,
it keeps every existing command and session,
and the engine may cost less if the hosted benchmark carries over to a self-run engine.
Against it,
Cloudflare documents its rendering as not pixel-perfect,
it has no video,
WebGL,
or persistent authenticated state,
a hosted engine cannot reach local pages,
and a self-run one needs a new Bazel build.
Whether `agent-browser` 0.43.0 works against Kitesurf's partial Chrome DevTools Protocol was not tested.
It ranks above `chrome-devtools-mcp` with Kitesurf because it keeps the driver the repository's rules,
runbooks,
and troubleshooting docs name,
while swapping the driver too adds a migration and gains nothing in rendering.

### `chrome-devtools-mcp` driving Kitesurf

In its favor,
it is the integration Cloudflare documents and advertises.
Against it,
it carries every drawback of Kitesurf behind `--cdp`,
and it also replaces the command-line driver the repository's rules and runbooks are written for.

## Gates not run, and why

- Broad discovery:
  not run,
  because the candidates were the named one and the incumbent.
- Clone,
  source audit,
  maintenance audit,
  and execution:
  not run,
  because Kitesurf exits on limits its vendor documents.
  No third-party code ran.
- Unverified:
  whether Kitesurf loads `file://` pages,
  honors `#:~:text=` fragments,
  or emulates `prefers-color-scheme`,
  and whether Browser Run retains page content.

## Possible follow-up

A narrower role,
such as a cheap engine for reading public pages during research,
would need its own vet.
`agent-browser` 0.43.0 already accepts `--engine lightpanda` and `--engine obscura`,
so such a vet would compare those with Kitesurf behind `--cdp`.

Sources accessed 2026-11-13.

[kitesurf-landing]: https://kitesurf.dev/#:~:text=There%20is%20no%20Chromium%20process
[kitesurf-readme-engine]: https://github.com/cloudflare/kitesurf#:~:text=experimental%20browser%20engine
[kitesurf-readme-bazel]: https://github.com/cloudflare/kitesurf#:~:text=Bazel%20builds%20everything
[kitesurf-open-source]: https://blog.cloudflare.com/kitesurf-open-source/#:~:text=open%20source%20on%20GitHub%20under%20the%20Apache%202.0%20license
[kitesurf-heavy-development]: https://blog.cloudflare.com/kitesurf-open-source/#:~:text=under%20heavy%20development
[kitesurf-boa-fallback]: https://blog.cloudflare.com/kitesurf-open-source/#:~:text=as%20the%20last%20resort
[kitesurf-playground-no-account]: https://kitesurf.dev/#:~:text=without%20an%20account
[kitesurf-pixel-perfect]: https://developers.cloudflare.com/browser-run/kitesurf/#:~:text=pixel%2Dperfect%20rendering
[kitesurf-video-webgl]: https://developers.cloudflare.com/browser-run/kitesurf/#:~:text=Play%20video%20or%20render%20WebGL
[kitesurf-auth-session]: https://developers.cloudflare.com/browser-run/kitesurf/#:~:text=long%2Drunning%2C%20authenticated%20session
[kitesurf-benchmark]: https://developers.cloudflare.com/browser-run/kitesurf/#:~:text=medians%20of%20five%20Browser%20Run%20Quick%20Action%20runs
