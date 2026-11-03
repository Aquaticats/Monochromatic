# Chromium live histogram upstream contribution

## Reproduction verified at 2026-11-03T10:03:12+00:00

The user requested:
"Lets do the 'Chromium's live histogram map ping' upstream contribution now.
Flag any blockers."
They subsequently required collecting missing fresh upstream reproduction evidence now,
rather than treating its absence as a reason to stop.
The user filed [Chromium issue 581090722](https://issues.chromium.org/issues/581090722).
Its public page was verified to contain the report,
the `Internals>Metrics` component,
and the sanitized JSON attachment.

## Work items

- Fresh Chromium reproduction completed:
  official Linux snapshot `1733707`,
  Chromium `159.0.8141.0`,
  source revision `986947eb4afc18471e51b7cb473f7c2f779f62bf`.
  A headed browser with a disposable Btrfs profile produced matching browser-main write faults
  in its live `BrowserMetrics` mapping.
  Both faulting instructions symbolize to `base::SampleVectorBase::Accumulate(int, int)`,
  specifically `lock xadd %r12d,(%rax)`.
  The current build recorded `60` ordinary and `60` sparse samples through
  `chrome.metricsPrivate.recordBoolean` and `recordSparseValue` from `chrome://settings/`.
  Rendered counts matched with subprocess imports disabled.
  Sampled blocked spans were `104 ms` and `1484 ms`;
  a browser-window command took `1159.963 ms` within the latter span.
  These are not exact fault durations or input-to-presentation latency.
- Upstream message:
  [draft](../planning/browser-disk-responsiveness/chromium-metrics-ping.md)
  and [sanitized historical evidence](../planning/browser-disk-responsiveness/chromium-metrics-ping-evidence.json)
  were committed in `364e8a8bb9`.
  Commit `58f62c8c92` replaced the missing-reproduction wording with fresh-build evidence.
  The source-checked route is `Internals>Metrics`,
  component `1456863`,
  through <https://issues.chromium.org/new>.
  Related issue `580929788` denies anonymous access;
  its discussion remains unknown.
- Submission access:
  the user explicitly declined signing into a testing browser,
  citing concern for their Google Account.
  The isolated sign-in window was closed.
  Do not ask them to authenticate in a testing browser again.
  Prepare the verified report for their normal browser.
  A human maintainer reply must receive a human reply under Chromium's policy.
- Agent behavior:
  the user approved tightened `PX1` and `CB1` wording.
  `AGENTS.md` and generated `CLAUDE.md` were committed in `93a9b867d5`.
  The user also requested disabling the bundled Browser plugin.
  User Codex configuration now sets `browser@openai-bundled.enabled = false`
  and `mcp_servers.node_repl.enabled = false`.
  `codex mcp get node_repl --json` confirms the latter.
  Continue using `agent-browser`.

## Evidence and limits

The historical Helium capture directly correlates its main-thread write fault with a live `.pma` mapping.
The observed `6622 ms` interval is sampled blocked time,
not exact fault latency or input-to-presentation latency.
Ordinary Chromium's historical filesystem waits have no mapping attribution;
keep that observation separate until fresh evidence establishes one.
The [route research](../planning/browser-disk-responsiveness/chromium-metrics-ping-route.md)
distinguishes current mapped-read and sparse-lock changes from the live-write case.

The downloaded snapshot lives under `${HOME}/temp/agent/chromium-live-repro-1733707`.
The bounded collector is `${HOME}/temp/agent/chromium-live-fault-trace.ts`.
The [retained reproduction clients](../troubleshooting/chromium-disk-responsiveness/live-histogram-reproduction.md)
preserve the executed source and its timing/isolation limits.
Only disposable state may be mutated for verification.
Synthetic host load remains prohibited.
Source research uses a same-vendor subagent,
not independent review.

## Completion and handoff

The fresh capture,
symbolization,
native histogram counts,
browser rendering of those counts,
trace-loss checks,
and sanitization are complete.
Commit `51da9d59db` retains the reproduction clients and verification tasks.
The source-selection pass verified `22` citations with `agent-browser`.
Scoped Markdown lint and Pandoc rendering passed;
the final verification record records the delivered source hashes.

All owned browser processes and the tracefs instance are gone.
The primary browser exited normally.
A separate simultaneous-scope teardown emitted a zygote fatal error;
the corrected `Browser.close` lifecycle check exited normally without that diagnostic.
The existing user-authorized Helium recorder was not changed.

The bundled Browser plugin and its browser-control server are disabled in user configuration.
A fresh uncached local app-server catalog confirmed no Browser skills.
The approved agent rules and generated copy are committed.

The initial upstream report is posted.
For the live form,
the advised role is `Chromium Contributor`
and the advised template is `Defect on Linux`;
the user supplied those exact available choices.
Do not request sign-in through a testing browser.
Human maintainer feedback requires a human reply.

## Active follow-up work

- The user requested a script to watch the issue for at least the next `24` hours.
  Anonymous read-only HTTP requests used by the public page have passed live checks.
  The watcher is running under `${HOME}/.local/state/chromium-issue-watch/581090722`.
  Its saved deadline is `2026-11-04T12:07:36.851Z`,
  with checks every `5` minutes.
  The [watch record and retained sources](chromium-issue-581090722-watch.md)
  document the active service,
  successful live reads,
  resource bounds,
  and functional/type checks.
- The user then asked why no patch had been prepared and attached.
  Prepare and test a candidate patch.
  The [source assessment](../planning/browser-disk-responsiveness/chromium-metrics-patch.md)
  recommends initial local-memory allocation for reporting-ineligible unbranded processes,
  preserving forced-reporting and test overrides.
  The C++ candidate and native policy/startup tests now exist in the isolated Chromium worktree.
  Its commits are `512d4157b2` and `e18dcd1e5f`,
  based on upstream `360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
  The [retained candidate](../troubleshooting/chromium-disk-responsiveness/reporting-ineligible-storage.patch)
  is not yet validated for attachment.
  Native build and browser verification remain required.
- The user approved the `HQP` readiness-check addition to the troubleshooting skill.
  It requires an attempted authorized fix prototype before a filing handoff,
  with explicit report-only scope or a concrete prototype blocker as the alternatives.
  The source rule is edited and generated copies are synchronized.
  Commit `27626b0968` saves the rule,
  watcher record,
  and completed reproduction verification record.
  Markdown lint passed.
  A corrected skill-lint invocation checked all `10` skills with no problems or warnings;
  the first narrowly targeted invocation checked no files and does not count.

## Native validation preparation at 2026-11-03T09:29:22-05:00

The isolated build inputs live under `${HOME}/temp/agent/chromium-metrics-build-context`.
The build reuses Chromium's own DEPS-pinned tools as checkout-local inputs inside the container.
It does not install them into Monochromatic's toolchain or the user's PATH.
The host wrapper uses a Node archive whose version and SHA-256 came from `toolchain.lock.json`.
The selected CachyOS base uses `latest` and matches the host's verified x86-64-v4 support.

The build container uses `28` CPUs,
`44 GiB` memory,
`4096` processes,
and I/O weight `10`.
It has no host repository,
home,
credentials,
or desktop socket mounted.
A dedicated build volume retains the checkout and logs after container exit.
Native compilation,
expected test-name verification,
the startup regression's failing control,
and actual browser mapping/sample checks remain queued.

The image built successfully as
`bc571a2fcbb237228b60f0d6beec4a51ec9be900add7a90eaffd6ee5b0c0e7a8`.
The orchestration TypeScript check passed.
The `chromium-metrics-native-build` container is starting dependency setup through the local `build` task.
Its volume is `chromium-metrics-native-build-20261103`;
logs are under `/work/logs` in that volume.
The container has a `12h` ceiling and removes itself on exit.

The Chromium clone has an intentionally disabled push URL.
Its local branch now uses push remote `.` so the auto-push service can skip it.
Do not enable upstream pushes from this prototype.

## Network pre-allowance at 2026-11-03T15:35:59+00:00

The user instructed:
"Make sure you pre-allow all relevant processes network in opensnitch."

The installed OpenSnitch daemon is `1.8.0`,
running as `opensnitch.service`.
The authorized precedence allowance is
`/etc/opensnitchd/rules/000-allow-chromium-contribution-network.json`.
It covers the rootless container network helpers,
Git HTTP helpers,
Python,
curl,
the package manager,
and the build's Node/CIPD paths.
The existing managed toolchain-root allowances remain in place.
The new rule is root-owned with mode `0600`;
its SHA-256 is `16ed85d6d526bd613ca3f9e44c596248073f827302124e02d70700b0f969be89`.
The daemon logged its ruleset reload at `2026-11-03T15:35:59Z`.
The [existing OpenSnitch record](../troubleshooting/opensnitch-1-8-auto-reject-blocks-toolchain-updates.md)
documents the precedence and direct-write reload behavior used here.

This does not establish OpenSnitch as the cause of the fetch stall.
The file named `reject-always-simple-usr-bin-pasta-avx2.json` actually had `action: allow`.
The original `gclient` attempt emitted
`STALL DETECTED: gclient has been silent for 5 minutes.`
Its Git HTTPS process was still present without any pack files at `2026-11-03T15:10:37Z`.
The cause remained unknown.

The owned build container was stopped before retrying.
Podman escalated from `SIGTERM` to `SIGKILL` after its `10` second stop window;
that intentional stop produced exit `137`,
not a native C++ test result.
The checkout and logs remain in the named build volume.
The installed-dependency image is
`0585ba53e6d9ce0898bf7aaeb1b3bc3116a8f25f58698f9cb78cd08a2e58b567`.
The retry uses `--init` and retains the same resource bounds.

The local build configuration now has `network:probe`,
`sync:runtime`,
and `build:retry-acquisition` tasks.
Verify live access with Node,
Git,
the hermetic Python launcher,
curl,
and CIPD before restarting acquisition.
The retry archives the interrupted checkout and logs,
keeps the exact Chromium revision,
and enables verbose `gclient` output.
Native build,
test-name checks,
the failing startup control,
and live browser mapping/sample checks remain outstanding.
Forced-reporting browser checks use `--network=none`.

The network checks completed at `2026-11-03T16:13:29Z`.
Git,
curl,
and CIPD passed their initial reads.
Node and hermetic Python received HTTP `503` from the Google source endpoint,
then passed against the pinned GitHub mirror file;
Git's mirror read passed too.
The initial probe is retained under `/work/logs/network-probe-Ro830U`,
and the mirror probe under `/work/logs/network-probe-dqcOVV`.
The [OpenSnitch verification record](../troubleshooting/opensnitch-1-8-auto-reject-blocks-toolchain-updates.md)
distinguishes permission changes from endpoint availability.

Source acquisition has restarted from `https://github.com/chromium/chromium.git`
with the same pinned revision.
The `build:retry-acquisition` task still owns the running container.
The updated test drivers are copied through a network-disabled helper container,
with no host source or home mounted into it.

## Cached source recovery at 2026-11-03T17:31:34+00:00

The mirrored full fetch failed with Git/libcurl's
`curl 92 HTTP/2 stream 3 reset by server (error 0x8 CANCEL)`,
followed by `early EOF` and `invalid index-pack output`.
The retry also failed before another attempt stalled.
At the process probe,
the pack decoder was waiting in `anon_pipe_read`;
the container's memory-event counters showed no OOM events.
The acquisition was explicitly interrupted and exited `130` at `2026-11-03T16:53:36Z`.
No C++ test had run.

The installed Git `2.56.0` manuals were read for `http.version`,
the idle-transfer controls,
and header-only tracing.
The next attempt uses `HTTP/1.1` in the disposable builder's Git configuration,
an idle limit of less than `1` byte per second for `120` seconds,
and a private header-only trace.
The host user's Git configuration is not part of this experiment.

An existing audit cache held `514440` packed objects,
with `1416134 KiB` of packed storage,
at Chromium tag `156.0.8078.73` and commit
`2f9a11097447f226133bc1a31c787291a5eeab98`.
Chromium's public commit API confirmed that identity.
A native local clone exported only the selected public tag's reachable objects,
preserving its shallow history.
The export's connectivity check passed.
Its fetch URL is public and its push URL is disabled.

The `image:cache` task bakes that export into `localhost/chromium-metrics-build:cached`.
The `build:seeded` task copies the cache into its own checkout,
fetches and checks out the original target revision
`360d943d418b3d6645fea86766f5dcc4a7cc3b2e`,
and verifies that revision again after dependency synchronization.
The cached older snapshot is input reuse,
not the version on which the patch will be validated.
The updated orchestration type check passed.

## Native compilation reached at 2026-11-03T17:40:12+00:00

The seeded source fetch completed in `88387 ms`.
Checkout,
dependency synchronization,
dependency revision capture,
patch applicability,
patch application,
dependency hooks,
and GN generation all exited successfully.
The verified source remains `360d943d418b3d6645fea86766f5dcc4a7cc3b2e`.
The build started compiling `chrome`,
`chrome/test:unit_tests`,
and `chrome/test:browser_tests` at `2026-11-03T17:40:12.725Z`.
This resolves the source-acquisition obstruction for this attempt;
it does not establish the cause of the previous transfer failures.

The next work is to finish compilation,
run the native policy and startup tests,
exercise the compiled browser with the mapped positive control first,
then prove that reverting only the startup storage choice makes the regression fail.
The candidate must be restored and rebuilt after that control.
The test driver must check the intended failed assertion,
not merely a nonzero exit.
No native test pass or patch efficacy is yet claimed.
