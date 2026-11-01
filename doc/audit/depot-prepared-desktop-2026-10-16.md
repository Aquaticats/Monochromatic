# Prepared Depot desktop runtime experiment

## Scope and current state

This experiment combines installed-package reuse with fresh desktop runtime services
and the existing seven-session interaction suite.
It runs on the isolated `trial/depot-prepared-desktop-20261016` branch,
starting from the verified trial source `d6deca5fa5`.
Production workflows and subscription settings are unchanged.
The experiment passed in both fresh consumers.
[Depot run `9jfk6fsklc`](https://depot.dev/orgs/l4zncjjbj5/workflows/9jfk6fsklc)
completed the native-runtime probe.
Prepared artifact build [run `xxs92vjc25`](https://depot.dev/orgs/l4zncjjbj5/workflows/xxs92vjc25)
and nested-boot verification [run `t7n42wkr5n`](https://depot.dev/orgs/l4zncjjbj5/workflows/t7n42wkr5n)
completed successfully.
Both full consumers in [run `p5rp4p0wvp`](https://depot.dev/orgs/l4zncjjbj5/workflows/p5rp4p0wvp)
passed all 117 injectable checks and their cleanup assertions.
Each separately reports the existing held-modifier case as not injectable.
The [machine-readable evidence](depot-prepared-desktop-2026-10-16.evidence.json)
retains the job intervals,
resource statistics,
artifact hashes,
per-session totals,
and runtime verification events.

The earlier [package snapshot experiment](depot-expanded-cost-2026-10-15.md)
verified the installed versions of 23 packages and ImageMagick in separate fresh consumers,
but the consumers lacked the user-session manager and audio services needed by the interaction harness.
The separate [GitHub runner verification](../troubleshooting/slint-nested-color-scheme-portal.md)
passed 117 interaction checks with no failures and one not-injectable case.
Those separate results did not prove that the prepared Depot CI route could run the complete suite.
The combined consumers now establish that boundary for the recorded source and image.

## Runtime boundary under test

The actual harness launches each compositor and player inside a transient systemd user unit,
identifies the player through that unit's cgroup,
and stops the unit during cleanup.
The relevant source is
`package/music-player/desktop-e2e/src/session.ts`
and `package/music-player/desktop-e2e/src/session-launch.ts`
at trial revision `d6deca5fa5`.
Replacing those operations with unrelated process launches would change the tested isolation boundary.

[Depot's custom-image documentation](https://depot.dev/docs/ci/how-to-guides/custom-images)
states that snapshots retain filesystem contents,
while each consumer starts with a fresh process environment.
It also documents preserving installed packages and pre-pulled container images.
Consequently this experiment checks installed files separately from live service readiness.

## Source audit

Upstream systemd tag `v255`,
commit `db11bab38ccf1ed257f310d29070843d4c58ea01`,
rejects a user-manager startup when `sd_booted()` reports that the host was not booted with systemd.
The consumer reports Ubuntu package version `255.4-1ubuntu8.18`;
its actual direct-start diagnostic matches this upstream guard.
[Systemd `src/core/main.c`](https://github.com/systemd/systemd/blob/v255/src/core/main.c#L2650)
contains:

```c
// systemd v255, src/core/main.c:2651
if (arg_action == ACTION_RUN &&
    sd_booted() <= 0)
        return log_error_errno(SYNTHETIC_ERRNO(EOPNOTSUPP),
                               "Trying to run as user instance, but the system has not been booted with systemd.");
```

The [systemd container interface](https://systemd.io/CONTAINER_INTERFACE/)
describes running systemd as a container's PID 1 with the appropriate private mounts,
namespace boundaries,
and delegated cgroup hierarchy.
The runtime probe checks the actual snapshot's namespace support and available container tools
before attempting that route.
No host-runtime marker is fabricated to bypass `sd_booted()`.

### Native snapshot findings

[Run `9jfk6fsklc`](https://depot.dev/orgs/l4zncjjbj5/workflows/9jfk6fsklc)
reports UID 1001,
PID 1 executable `/init`,
no session-bus environment,
and writable cgroup v2 with `nsdelegate`.
Both an unprivileged user/PID namespace probe and a privileged mount/PID namespace probe exit successfully.
Direct `systemd --user` exits with status 1 and:

```text
# Actual systemd diagnostic from the fresh Depot consumer
Trying to run as user instance, but the system has not been booted with systemd.
```

Podman `5.8.4` and Docker `28.0.4` are already installed.
The successful boot probe uses Podman's documented
[external root filesystem overlay and systemd mode](https://docs.podman.io/en/latest/markdown/podman-run.1.html).
It adds no package installation:
the container uses the snapshot filesystem as its read-only lower layer and discards its writable overlay at removal.
Podman's systemd mode supplies private runtime mounts and a writable container cgroup hierarchy.
The probe sets no network,
a 2-GiB memory limit,
two CPUs,
and a 90-second lifetime.

### Probe corrections

The initial [run `bm8l40jmnp`](https://depot.dev/orgs/l4zncjjbj5/workflows/bm8l40jmnp)
stopped at an optional unprivileged `/proc/1/exe` read with `EACCES`.
That was a probe error,
not evidence that the later namespace or service checks failed.
The corrected probe records a privileged `readlink` result independently;
it returned `/init`.

The first nested-boot [run `nj4x2frgwk`](https://depot.dev/orgs/l4zncjjbj5/workflows/nj4x2frgwk)
created the container successfully,
but immediately querying its systemd manager returned
`Failed to connect to bus: No such file or directory`.
The container was stopped and removed.
The follow-up probe waits for an actual manager response before starting user services
and captures console boot diagnostics.
The first result alone does not distinguish delayed readiness from a failed boot.

### Successful fresh runtime

[Run `t7n42wkr5n`](https://depot.dev/orgs/l4zncjjbj5/workflows/t7n42wkr5n)
waited for the system manager to answer,
started `user@1001.service`,
and ran an actual transient user service with a `MemoryMax=256M` limit.
That process reported its cgroup under
`user@1001.service/app.slice/depot-probe-unit.service`.
The same fresh runtime started PipeWire,
its PulseAudio socket,
and WirePlumber.
`pactl info` returned `PulseAudio (on PipeWire 1.0.5)` with the private `auto_null` default sink.
The probe stopped the audio units and removed the container successfully.

This supplies a measured runtime bridge for the existing systemd-based harness.
It does not change the native Depot sandbox's init system.
The first manager query at `01:44:18.992Z` failed before readiness;
the query at `01:44:19.666Z` returned version `255.4-1ubuntu8.18`.
The transient user unit succeeded at `01:44:26.758Z`.

### Prepared artifact build

[Run `xxs92vjc25`](https://depot.dev/orgs/l4zncjjbj5/workflows/xxs92vjc25)
reused the installed-package snapshot,
installed the build toolchain,
checked the locked workspace dependencies,
and built the application and compositor once.
The 16-CPU,
64-GB job ran from `01:41:57.980Z` to `01:46:59.251Z`,
an elapsed 301.271 seconds in the metrics API.
The complete run took 325.286 seconds from its recorded start,
including orchestration.
The logged snapshot capture and upload took 23.410 seconds and uploaded 4,863,229,952 bytes.

The resulting image is `monochromatic-trial-desktop-ready:20261016-v1`,
digest `sha256:75254e66e4b87526db5fee48ae4c921d27669f3ced6dfbf97b42e3d73aea36b3`.
Its manifest records build source `7b31a6eebec64e193e4165cf7125f781daebf719`,
the exact installed package versions,
and these release artifacts:

- `music-player`:
  53,790,520 bytes;
  SHA-256 `59e365a380f5a92f2057db7f10b8cb027d15cd8857fa8b1975145e353b1e1da3`.
- `monochromatic-nested-wayland-session`:
  9,191,840 bytes;
  SHA-256 `6b112ddb63bde429c9b85ea13eb2d27f839323e1e0a4549f742dbb57995adf23`.

Both consumer jobs use this image with a 2-CPU,
8-GB sandbox.
Their isolated service runtime has a 2-GiB memory limit,
two CPUs,
no network,
and a 660-second maximum lifetime.
The consumer verifies package and artifact identity before booting services.
It runs no package installation or compilation.

The prepared binaries remain valid only for their recorded source inputs.
`git diff 7b31a6eebe..36d48d72ef --name-only`
shows only the experiment workflow and `desktop-app/mise.toml` changes between preparation and consumption.
A revision changing either binary's inputs needs a fresh preparation build.
Cost projections for changing revisions must include preparation plus consumption;
the repeated unchanged-image consumers measure reuse separately.

The initial full consumer run `4pmthztf6p` stopped before the suite because Bash could not find `mise` on `PATH`.
Snapshots preserve the executable but not the preparation step's environment.
The corrected workflow activates the preserved `~/.local/share/mise/bin/mise` executable explicitly
without installing it again.

### First full consumer

Job `n1jkfcsz5c`,
attempt `zfwxb30f4m`,
passed all seven sessions with 117 passed checks,
no failures,
and the existing held-modifier case reported as not injectable.
The per-session results were:

- `wide`:
  75 passed and one not injectable.
- `narrow`:
  10 passed.
- `dialog`:
  2 passed.
- `auto-play`:
  4 passed.
- `quit`:
  1 passed.
- `states`:
  13 passed.
- `visual`:
  12 passed.

The actual suite command took 218.521 seconds.
File verification and fresh service readiness took 24.785 seconds before that command.
The complete task,
including cleanup,
took 243.917 seconds.
The 2-CPU job interval was `01:52:17.353Z` to `01:57:14.348Z`,
or 296.995 seconds including checkout and startup overhead.

Each harness session created and removed its own private audio sink.
The wrapper then verified that only the original `auto_null` sink remained,
no test user units were active,
and no player process remained.
It stopped the audio services,
removed the container,
verified the container no longer existed,
and verified that its overlay marker did not reach the prepared filesystem.

### Repeat consumer and timing spread

Job `612sb7t227`,
attempt `mr8vmn09cj`,
passed the same session totals and every cleanup assertion.
It ran in sandbox `w8j1pt06vlx765bk5kp6`,
distinct from the first consumer's `t65qnpr0s9h8tt931g49`.
Both consumers required an absent consumer marker before writing it,
verified identical prepared binaries and package versions,
and removed their independent runtime overlays.

The second job ran from `01:57:19.672Z` to `02:01:55.777Z`,
or 276.105 seconds.
File verification and service readiness took 17.030 seconds;
the suite command took 209.946 seconds;
the full task through cleanup took 227.671 seconds.
The unchanged-input job spread is 20.890 seconds,
and the suite-command spread is 8.575 seconds.
These repeats establish observed variation,
not a performance improvement from a changed implementation.

The metrics API reports peak sandbox memory utilization of 0.14838 and 0.14656
and peak CPU utilization of 0.25918 and 0.25850 for the respective consumers.
These are sampled whole-sandbox ratios,
not measurements of the application's individual memory or CPU use.
The evidence retains sample counts and observed intervals so they are not mistaken for continuous coverage.

## Evidence collection

The committed probe in
`.depot/workflows/depot-prepared-desktop.yml`
uses the existing `monochromatic-trial-desktop-packages:20261016-v1` image
with a 2-CPU,
8-GB sandbox and a three-minute job limit.
It installs no packages.
Its task records PID 1,
the systemd version,
cgroup mounts,
bounded namespace probes,
and available container or virtual-machine tools.
It prints selected runtime facts rather than the process environment or credentials.

Run it from the pushed experiment branch with:

```sh
# trial/depot-prepared-desktop-20261016
depot ci run --workflow .depot/workflows/depot-prepared-desktop.yml
```

The branch is pushed before each run so that Depot does not inject a local patch
into a repository already preserved in the snapshot.
The [existing patch-collision incident](../troubleshooting/depot-snapshot-mise-cache.md)
records why that distinction matters.

## Unverified integrations and adoption boundary

The interaction result does not claim complete desktop-service integration.
The not-injectable held-modifier case remains tracked by
[issue 1044](https://github.com/Aquaticats/Monochromatic/issues/1044).
The retained dialog log reports these nonfatal boundaries separately:

- The decode-priority request received D-Bus `NoReply` and kept normal priority:
  [issue 1053](https://github.com/Aquaticats/Monochromatic/issues/1053).
- No desktop notification service answered the analysis-progress request:
  [issue 1054](https://github.com/Aquaticats/Monochromatic/issues/1054).
- Smithay emitted `Erroneous EGL call didn't set EGLError` without identifying the wrapped operation:
  [issue 1055](https://github.com/Aquaticats/Monochromatic/issues/1055).
- The existing accessibility-bus diagnostic remains tracked by
  [issue 1041](https://github.com/Aquaticats/Monochromatic/issues/1041).

The new issue search inspected 1,020 repository issue bodies and titles,
including closed issues,
and found no matching runtime boundaries for the three new reports.
The reports retain exact diagnostics and successful-suite context;
none claims these messages caused an interaction failure.

[Issue 997](https://github.com/Aquaticats/Monochromatic/issues/997)
now has a verified experiment for its fresh-session boundary.
[Issue 998](https://github.com/Aquaticats/Monochromatic/issues/998)
still owns production adoption and explicit invalidation for OS packages,
ImageMagick,
toolchains,
workspace dependencies,
and application build inputs.
No production migration or automatic refresh policy is implemented by this experiment.
There is one measured preparation build and two unchanged-image consumers;
a distribution of rebuild cost across changing revisions remains unmeasured.
