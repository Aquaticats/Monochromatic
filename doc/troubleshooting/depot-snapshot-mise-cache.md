# A Depot CI snapshot consumer waits on an additional mise cache restore

## Mise 2026.10.9 offline snapshot controls need the runner's home context

### Symptom and source boundary

Research allocation `wh76h0pjpx` booted the prepared image and verified its tool hashes,
but the network-isolated availability control failed:

```text
# Mise 2026.10.9 inside the research network/user namespace
[WARN] migrate: failed create_dir_all: ~/.local/state/mise/incomplete-installs
mise ERROR failed create_dir_all: ~/.local/state/mise/trusted-configs
mise ERROR Permission denied (os error 13)
```

The research wrapper switched user/group through `sudo unshare` without explicitly passing the runner's home context.
This was a harness failure;
it was not an observed missing-tool download or snapshot corruption.
Mise source revision `ac91d06fa6100935b7a07b08bebf03dd6dd9a0dd`
derives its trust directory from its state directory in `crates/mise-util/src/dirs.rs:27`:

```rust
// jdx/mise, crates/mise-util/src/dirs.rs:27
pub static TRUSTED_CONFIGS: Lazy<PathBuf> = Lazy::new(|| STATE.join("trusted-configs"));
```

The default state path follows the resolved home context in `crates/mise-util/src/env.rs:102` and `:160`:

```rust
// jdx/mise, crates/mise-util/src/env.rs:102 and :160
pub static XDG_STATE_HOME: Lazy<PathBuf> =
    Lazy::new(|| var_path("XDG_STATE_HOME").unwrap_or_else(|| HOME.join(".local").join("state")));
pub static MISE_STATE_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_STATE_DIR").unwrap_or_else(|| XDG_STATE_HOME.join("mise")));
```

### Verified correction and limits

Pushed harness `997db20b3bd5f1cdf3413c645d964c34dac63518`
passes the original runner's `HOME`,
`USER` and `LOGNAME` explicitly to the dropped-user namespace.
Actual allocation `x1psl3qf94` then passes `mise install` for the installed,
locked tool subset with networking disabled.
Its empty isolated Mise tool home fails in the same offline namespace;
online installation into that empty home recovers the expected locked Node binary hash.
The positive and negative controls distinguish installed-tool availability from an accidental download.
They do not benchmark complete snapshot boot or every installed tool.

The successful consumers also use the prepared Mise executable directly,
avoiding an additional setup-action cache restore.
The image's tool manifest and frozen source/lock/policy hashes are checked before actual consumers run.
Changing the checkout does not update the prepared tools automatically.

## Depot native snapshot tags and block metadata are separate boundaries

### Symptom and deciding-source limit

Allocation `6wt5gm6t75` failed before creating a sandbox with the full registry digest reference:

```text
# Depot native CI dispatch, 2026-10-18
Dispatch failed: [not_found] image "9fb974fd32185c8afd5711370869d03e0f28e5b560c928eae324c20151ddeee8" not found
```

The unique documented registry tag boots successfully in `x1psl3qf94`,
`s9w4jkv0hc` and `rrlzfmp2qv`.
Depot CLI revision `67a618f86b05d925f317e3d3a4f656f2fc8ff447`
serializes workflow content and sends it to the service in `pkg/cmd/ci/run.go:225`:

```go
// depot/cli, pkg/cmd/ci/run.go:225
yamlBytes, err := yaml.Marshal(workflow)
```

The deciding server-side image resolver is not present in that public source.
This particular digest-form failure does not establish that Depot universally prohibits digest references.
The verified research bridge uses a unique tag and checks the resolved manifest digest before and after execution.
This detects a changed resolution within the observed run;
it does not make the tag immutable or exclude every resolution race.

### Native block-image metadata correction

The working `tools-r1` manifest resolves to
`sha256:9fb974fd32185c8afd5711370869d03e0f28e5b560c928eae324c20151ddeee8`.
Its configuration descriptor is 6,502,729 bytes and has media type
`application/vnd.depot.thin.snapshot.config.v2+json`.
The verified configuration keys are `chunk_size_bytes`,
`disk_size_bytes`,
`metadata_chunks`,
`layers` and `chunks`.
It is block-map metadata,
not a conventional Docker image configuration.
An absent Docker `config.Env` field cannot be interpreted as evidence that no environment was captured.
Searching this JSON for synthetic plaintext also does not inspect referenced filesystem blocks.
Fresh-consumer filesystem and runtime-environment checks provide separate,
bounded observations.

The snapshot helper's uploaded sparse-layer digest is also distinct from the final manifest digest.
For `tools-r2-git256`,
the uploaded layer is `sha256:14702a111207f4051bdc1767a8646cfbc74385570f6554e903e7230d6fe13afc`,
while the resolved manifest is `sha256:fb197837831d438592d9455b3a31d808a20ad8bdc2ae76b05d8b36f6656419cd`.
Use the manifest when binding a fresh-image consumer.
Neither the helper's `max-age: 6h` input nor a successful push proves registry deletion after that interval.

### Research verification and upstream filing decision

The [reuse campaign](../planning/depot-reuse-strategy-research.md)
and [digest-bound receipts](../audit/depot-reuse-strategy-2026-10-18.evidence.json)
retain the exact workflows,
provider identities,
uploaded artifact digests and successful/failed controls.
Only disposable research workflows changed.

The `.out-of-scope/` filename inventory contains no Depot or Mise exemption.
Current open/closed issue and pull-request searches for `"trusted-configs" "Permission denied"` in `jdx/mise`
and `snapshot image digest` in `depot/cli` returned no results.
Those narrow searches are not exhaustive duplicate checks.

1.  The home-context failure is at our harness boundary;
    an upstream defect is not established.
2.  Fixing our environment is verified;
    server-side digest-resolution fixability is not established without its deciding source.
3.  Prepared tools and native image tags are supported use cases;
    this particular digest-form contract remains unverified.
4.  Contribution policies were not exhaustively audited because no upstream defect or patch is proposed.
5.  Maintainer intent about this specific digest reference was not established.
6.  No upstream patch was prototyped;
    the tested corrections are consumer configuration and evidence interpretation.

There is no justified upstream filing to add.
No external issue or comment was sent.

## Symptom

In trial run `qrjbnf2gbv` on 2026-10-15,
the prepared-image consumer logged `Restoring mise cache` at `22:04:47.258Z`
and `mise cache not found` at `22:08:59.691Z`.
The elapsed interval was 252.433 seconds.
Logger build,
type checks,
and unit tests subsequently passed.

The image already contained the installed Node tools and workspace dependencies.
The additional Actions cache was unnecessary for the intended image-reuse experiment.

## Root cause boundary

The [mise action source](https://github.com/jdx/mise-action),
revision `c2a87611a18de5b3828c5652fe268e992400cb5c`,
calls cache restoration before setup when the `cache` input is true in `src/index.ts:67`:

```typescript
// src/index.ts, run()
if (core.getBooleanInput('cache')) {
  cacheKey = await restoreMiseCache()
} else {
  core.setOutput('cache-hit', false)
}
```

`restoreMiseCache()` starts the log group in `src/index.ts:374`,
awaits `cache.restoreCache([cachePath], primaryKey)` at line 385,
and emits the miss message at line 389.
The observed wait falls within that operation.
The underlying service delay and retry behavior were not traced;
do not attribute it to snapshot boot time or conclude that every cache miss has this cost.

## Verification and workaround

`action.yml:51` defines `cache` with default `true` and documents disabling cache reads and writes.
Run `jp9w8gw62r` reused the existing snapshot tag and passed the same logger checks with:

```yaml
# .depot/workflows/depot-trial.yml, typescript-snapshot-use
- uses: jdx/mise-action@c2a87611a18de5b3828c5652fe268e992400cb5c
  with:
    minimum_release_age: 1d
    install: false
    cache: false
```

Its log contains no `Restoring mise cache` group.
The combined validation run completed in 118 seconds,
including independent Android and prerequisite-probe jobs.
That combined interval is not a controlled estimate of the cache flag's performance effect.

The tradeoff is that this action no longer restores or saves the tool directory through Actions cache.
The tested consumer obtains its tools from the prepared Depot image instead.
Changing a source checkout does not refresh tools baked into an existing image tag;
image invalidation must remain explicit.

## What does not establish a fix

A successful eventual cache miss does not explain the delay.
No retry-policy change,
cache service fix,
or production workflow modification was made.

## Upstream filing decision

No matching exemption appeared in `.out-of-scope/`.
`gh search issues 'cache slow' --repo jdx/mise-action` returned no results;
this is not an exhaustive duplicate search.

1.  An action or Depot defect is unproven;
    the unnecessary extra cache was a trial configuration choice.
2.  Service-side fixability is unknown without a service trace.
3.  Disabling the action cache is explicitly supported.
4.  Contribution policy was not assessed because no upstream patch is proposed.
5.  Maintainer intent on this delay is unknown.
6.  The consumer configuration change was exercised successfully;
    no upstream implementation patch is justified by the available evidence.

There is no actionable upstream filing to add.
No external report was sent.

## Systemd 255.4 rejects direct user-manager startup in a fresh Depot snapshot

### Symptom and source boundary

The installed-package image contains the systemd and audio executables,
but its native PID 1 is `/init`.
In [run `9jfk6fsklc`](https://depot.dev/orgs/l4zncjjbj5/workflows/9jfk6fsklc),
`systemd --user` returned status 1 with:

```text
# systemd 255.4-1ubuntu8.18 in the fresh Depot consumer
Trying to run as user instance, but the system has not been booted with systemd.
```

Upstream systemd `v255`,
commit `db11bab38ccf1ed257f310d29070843d4c58ea01`,
contains the matching guard in `src/core/main.c:2651`:

```c
// systemd v255, src/core/main.c:2651
if (arg_action == ACTION_RUN &&
    sd_booted() <= 0)
        return log_error_errno(SYNTHETIC_ERRNO(EOPNOTSUPP),
                               "Trying to run as user instance, but the system has not been booted with systemd.");
```

`src/libsystemd/sd-daemon/sd-daemon.c:717` implements `sd_booted()` by testing the early-boot runtime directory:

```c
// systemd v255, src/libsystemd/sd-daemon/sd-daemon.c:722
if (laccess("/run/systemd/system/", F_OK) >= 0)
        return true;
```

This explains the installed executable's direct-start rejection.
It does not establish that an isolated systemd runtime cannot run on the sandbox.
The [systemd container interface](https://systemd.io/CONTAINER_INTERFACE/)
describes the required PID 1 and mount boundaries.

### Verified runtime bridge and readiness

The existing Podman `5.8.4` executable successfully boots systemd against a disposable overlay
of the installed filesystem.
[Run `t7n42wkr5n`](https://depot.dev/orgs/l4zncjjbj5/workflows/t7n42wkr5n)
then starts a real user manager,
executes a transient user service,
starts private PipeWire services,
queries them with `pactl info`,
and stops and removes the runtime.
The reproducible task is on `trial/depot-prepared-desktop-20261016`,
commit `4880b47696`:

```sh
# Pushed experiment checkout, no desktop package installation
depot ci run --workflow .depot/workflows/depot-prepared-desktop.yml --job session-probe
```

The task calls Podman with `--rootfs /:O`,
`--systemd=always`,
`--cgroupns=private`,
`--network=none`,
`--memory=2g`,
`--cpus=2`,
and `--timeout=90`.
Podman's [root filesystem documentation](https://docs.podman.io/en/latest/markdown/podman-run.1.html)
defines the `:O` overlay behavior.
Podman `v5.8.4`,
commit `5431df23c742e5edea35bef34eed696f4db0106b`,
implements it in `libpod/container_internal.go:1777`:

```go
// Podman v5.8.4, libpod/container_internal.go:1777
if c.config.RootfsOverlay {
    overlayDest := c.runtime.GraphRoot()
    contentDir, err := overlay.GenerateStructure(overlayDest, c.ID(), "rootfs", c.RootUID(), c.RootGID())
```

The tradeoff is a privileged,
bounded nested container with private runtime state and an additional boot step.
It provides the systemd boundary required by the existing test harness;
it does not make the native sandbox itself systemd-booted.
The complete desktop consumer experiment is tracked in the
[prepared-runtime audit](../audit/depot-prepared-desktop-2026-10-16.md).

The first boot probe,
run `nj4x2frgwk`,
queried the manager immediately and received `Failed to connect to bus: No such file or directory`.
The follow-up waits for a successful `systemctl show --property=Version --value` response before starting user units.
Its first query failed at `01:44:18.992Z`,
then version `255.4-1ubuntu8.18` arrived at `01:44:19.666Z`.
The immediate-query failure therefore did not prove a failed boot.
Fabricating `/run/systemd/system` to bypass the guard was not attempted and is not a verified workaround.

### Upstream filing decision for the runtime boundary

The `.out-of-scope/` search found no exclusion for Depot or systemd startup.
The repository already tracks the missing-session boundary in
[issue 997](https://github.com/Aquaticats/Monochromatic/issues/997).

1.  No upstream defect is established:
    the native image process model does not satisfy the direct systemd user-manager precondition.
2.  A runtime integration can supply that precondition;
    the isolated boot probe verifies one such integration.
3.  Systemd documents its container interface and Podman documents systemd containers with filesystem overlays.
4.  Upstream contribution policies were not evaluated because no upstream implementation change is proposed.
5.  No upstream maintainer decision on this repository's harness integration was established.
6.  The repository-side boot bridge was exercised;
    no upstream patch was needed or prototyped.

There is no upstream filing artifact to add.
The remaining work belongs to the repository's runtime integration and consumer verification.

### Fresh consumer command paths

The first full-suite consumer,
run `4pmthztf6p`,
stopped before its task because Bash emitted `mise: command not found` with exit status 127.
The preparation action had installed `mise` at `~/.local/share/mise/bin/mise` and added its directory to `PATH`.
The [snapshot contract](https://depot.dev/docs/ci/how-to-guides/custom-images)
preserves filesystem contents without preserving step environment changes.
The executable therefore needs activation in each fresh consumer.

Commit `36d48d72ef` invokes that preserved executable explicitly:

```sh
# .depot/workflows/depot-prepared-desktop.yml, prepared consumer
PATH="${HOME}/.local/share/mise/bin:${PATH}" "${HOME}/.local/share/mise/bin/mise" run //package/music-player/desktop-app:trial:depot:prepared:consume
```

Run `p5rp4p0wvp` reached artifact verification,
service startup,
and application interaction through this invocation.
The consumer did not run a mise installation or cache restore.
This path belongs to the tested preparation action configuration;
a different installation location must update the activation path as well.
No upstream defect or upstream patch is established.

## Depot CLI 2.102.22 local patches collide with files retained by snapshot checkout

### Symptom

Run `341kqr5sw7` created the desktop-package snapshot successfully,
then failed its first consumer during `Apply local patch from Depot Cache`:

```text
# git apply diagnostic in the first snapshot consumer
error: .depot/workflows/depot-package-snapshot.yml: already exists in working directory
```

The same diagnostic named other files introduced on the unpublished trial branch.
No package verification task had run yet.
This incident is separate from the mise-cache delay.
[Failed consumer](https://depot.dev/orgs/l4zncjjbj5/workflows/341kqr5sw7)

### Source trace and scope

Depot CLI `v2.102.22`,
`pkg/cmd/ci/run.go:601`,
sets an injected checkout's `ref` to the local patch's merge base.
```go
// depot/cli, pkg/cmd/ci/run.go:601, v2.102.22
withMap["ref"] = mergeBase
```

The generated shell step at line 612 applies the patch:

```sh
# depot/cli, pkg/cmd/ci/run.go:612, inside the generated shell string
git apply --allow-empty /tmp/local.patch
```

Lines 622 to 624 insert that step immediately after checkout:

```go
// depot/cli, pkg/cmd/ci/run.go:622, v2.102.22
newSteps = append(newSteps, steps[:checkoutIndex+1]...)
newSteps = append(newSteps, patchStep)
newSteps = append(newSteps, steps[checkoutIndex+1:]...)
```

[Tagged CLI source](https://github.com/depot/cli/blob/v2.102.22/pkg/cmd/ci/run.go#L549)

The snapshot was captured after the preparation job applied that local patch.
Its newly added files remained on disk.
The consumer intentionally used `clean: false` to retain installed workspace dependencies,
then checked out the merge base and attempted the same add-file patch.
Git rejected the paths already present.
This identifies the checkout/patch interaction;
it does not establish damaged snapshot contents or failed package installation.
[Snapshot checkout guidance](https://depot.dev/docs/ci/how-to-guides/custom-images#best-practices)

### Verification and workaround

The reproduction lives on `trial/depot-package-snapshot-20261016`,
commit `0577b002b2`,
in `.depot/workflows/depot-package-snapshot.yml`.
Its tasks are in `package/music-player/desktop-app/mise.toml`.
Before that branch was pushed,
the following invocation uploaded a local patch relative to `origin/main`:

```sh
# Isolated trial worktree with the committed branch still unpublished
depot ci run --workflow .depot/workflows/depot-package-snapshot.yml
```

The failure catalog is a snapshot captured from locally patched checkout files,
followed by a `clean: false` consumer with an injected add-file patch.
The working catalog uses the same snapshot and files with a pushed commit,
so the CLI has no local patch to inject.
The tested boundary change was:

```sh
# Same isolated trial worktree, publishing only the experiment branch
git push --set-upstream origin trial/depot-package-snapshot-20261016
depot ci run --workflow .depot/workflows/depot-package-snapshot.yml
```

Repeat run `6w30pv5zds` checked out `0577b002b2` without a local-patch step.
Both consumers verified all 23 installed package versions and executed ImageMagick without reinstalling them.
Their workspace dependency checks reported `Already up to date`.
Attempts `x8b3b4xhkb` and `kpwm36f319` had distinct sandbox IDs;
both also required a prior-consumer marker to be absent before writing their own.
The completed run therefore verifies fresh consumers of the same prepared filesystem.
[Patch-free repeat](https://depot.dev/orgs/l4zncjjbj5/workflows/6w30pv5zds)

The tradeoff is publishing the experiment commit to the repository before this form of snapshot consumption.
`clean: false` also preserves other untracked files,
so dependency reuse must include explicit lockfile and tool-version checks.
The image tag still needs refresh when its preparation inputs change.

### What does not establish a workaround

Reusing the same image with the same locally injected add-file patch already failed.
Enabling checkout cleaning was not tested here;
it would remove the untracked workspace dependencies whose reuse this trial measures.
Skipping an arbitrary failed patch is not verified and could leave the wrong source revision.

The separate fresh-session probe reported `Failed to connect to bus: No medium found`
from `systemctl --user` and `systemd-run --user`.
`pactl info` reported `Connection failure: Connection refused`.
Those commands ran after package verification passed;
installed files do not establish a running user manager or audio service.
This probe did not attempt to create a replacement session manager.

### Upstream filing decision for the patch interaction

No `.out-of-scope/` entry names Depot.
Issue and pull-request searches for `"snapshot" "patch"` in `depot/cli` returned no matches.
That search is bounded evidence,
not a claim that no related report exists.

1.  An upstream defect is not established:
    the trial deliberately retained untracked files and then reapplied an add-file patch.
2.  CLI handling can be changed in principle,
    but the correct ownership and cleanup contract has not been established.
3.  Local patches and preserving snapshot dependencies are separately documented;
    this exact repeated-patch combination has not been shown to have a supported contract.
4.  Contribution policy was not evaluated because no upstream filing or patch is proposed.
5.  The searches found no maintainer decision on this combination.
6.  The repository-side pushed-commit workaround was exercised;
    no upstream fix was prototyped or claimed.

There is no upstream filing artifact to add at this stage.
No external report was sent.
