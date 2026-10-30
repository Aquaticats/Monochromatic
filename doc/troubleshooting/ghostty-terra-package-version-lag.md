# Ghostty 1.3.1 on Bazzite 44: Terra layering cannot select 1.4.0 until its Zig 0.17 build failure clears

## Symptom

Ghostty resolves to the system executable and reports `1.3.1`:

```sh
# run on bazzite 44.20261012.1 (booted)
command -v ghostty
ghostty --version
rpm --query ghostty
```

```text
/usr/bin/ghostty
Ghostty 1.3.1
ghostty-1.3.1-3.fc44.x86_64
```

`ujust update` reports no updates for it.
Its last manual run on 2026-10-29 at 20:01 ends the System module with
`No upgrade available.`
and the other modules with `Nothing to update.`:

```text
# journalctl --unit uupd-manual.service (2026-10-29T20:01:44 to 20:04:19)
{"level":"DEBUG","msg":"Resolving dependencies...done"}
{"level":"DEBUG","msg":"No upgrade available."}
{"level":"DEBUG","msg":"Nothing to update."}
{"level":"INFO","msg":"Updates Completed Successfully"}
```

This differs from upstream's latest release,
[Ghostty v1.4.0][ghostty-v140],
tagged on 2026-10-27 at 15:51:48 UTC at commit
`83fb95624b0439ee660c2679b43b38c8b63769cd`.

## Root cause

### Ghostty is a layered Terra RPM

`rpm-ostree status` lists `ghostty` under `LayeredPackages` in both retained
deployments,
and `rpm --query ghostty` reports vendor `Terra`,
packager `Gilver E. <roachy@fyralabs.com>`.
Ghostty is therefore a repository-backed rpm-ostree package layer from the
Terra repository,
not part of the Bazzite base image and not a Flatpak,
Snap,
Homebrew package,
or standalone binary.

The retained deployments establish when the layer was last composed:
the booted `44.20261012.1` deployment was created 2026-10-19 at 04:40:44 UTC,
and the staged `44.20261026` deployment was created 2026-10-27 at 08:25:03 UTC.

### Upstream 1.4.0 exists and Terra bumped its spec the same day

Upstream tagged `v1.4.0` on 2026-10-27 at 15:51:48 UTC.
Terra's f44 spec was bumped to match at 2026-10-27 at 16:10:35 UTC,
commit `f2b0addf9d679cfe661fc0048b283b6dde6e63ce`,
message `bump: ghostty`.
`anda/devs/ghostty/stable/ghostty.spec:6-7` and `:23-24` at that commit read:

```spec
# terrapkg/packages anda/devs/ghostty/stable/ghostty.spec at f2b0addf
Version:        1.4.0
Release:        1%{?dist}
...
BuildRequires:  zig0.15
BuildRequires:  zig0.15-rpm-macros
```

The version bump landed.
The build did not.

### The Terra build of 1.4.0 fails on the Zig 0.17 floor

Ghostty 1.4.0 declares its toolchain floor in `build.zig.zon:6`
at the tag commit:

```zig
// ghostty-org/ghostty build.zig.zon at 83fb956
    .minimum_zig_version = "0.17.0",
```

`build.zig:17` passes that value to `requireZig`,
and `src/build/zig.zig:5-17` enforces it at compile time:

```zig
// ghostty-org/ghostty src/build/zig.zig at 83fb956
pub fn requireZig(comptime required_zig: []const u8) void {
    // Fail compilation if the current Zig version doesn't meet requirements.
    const current_vsn = builtin.zig_version;
    const required_vsn = std.SemanticVersion.parse(required_zig) catch unreachable;
    if (current_vsn.major != required_vsn.major or
        current_vsn.minor != required_vsn.minor or
        current_vsn.patch < required_vsn.patch)
    {
        @compileError(std.fmt.comptimePrint(
            "Your Zig version v{f} does not meet the required build version of v{f}",
            .{ current_vsn, required_vsn },
        ));
    }
}
```

The comparison demands the same major and the same minor,
so the floor is exactly `0.17.x`,
not `0.17 or later`.
A Zig `0.18.x` fails this gate for the same reason `0.16.x` does.

Terra's buildroot supplies `zig0.15` at `0.15.2`,
and the 1.4.0 build fails with that exact message
in [Terra CI run 40538321263][terra-run-40538321263],
jobs `build (anda/devs/ghostty/stable/pkg, x86_64, 44)` and the aarch64 twin,
both `completed/failure` on 2026-10-28:

```text
# terrapkg/packages actions run 40538321263 job 123169011317 log
rpmbuild │ src/build/zig.zig:13:9: error: Your Zig version v0.15.2
           does not meet the required build version of v0.17.0
rpmbuild │ error: Bad exit status from /var/tmp/rpm-tmp.sjw2pN (%install)
rpmbuild │ RPM build errors:
```

Related `@fromBackingInt` and `b.root` compile errors in the same log are
consequences of the same version mismatch,
not separate defects.
The published toolchains cannot satisfy the floor:
on 2026-10-29 the enabled repositories offer `zig 0.15.2-3.fc44` (fedora),
`zig 0.16.0-1.fc44` (updates),
and `zig0.15 0.15.2-2.fc44~bootstrap` (terra).
Upstream Zig 0.17.0 exists,
released 2026-10-01 per [ziglang.org/download/index.json][zig-index],
but no Fedora or Terra package carries it yet.

### The published repo therefore still serves only 1.3.1-3

Terra's repodata was fetched fresh on 2026-10-29,
checksum-matched against its signed `repomd.xml`
(`sha256:3afe48c1f8c430843b58ba5bbf31ca325648dcdd4711305efc319cd128c62555`),
and the signature verified `Good signature from "Terra 44 <security@fyralabs.com>"`.
Its `ghostty` entries are:

```text
# parsed from terra44 repodata primary.xml, generated 2026-10-29T23:55:13Z
ghostty 0:1.3.1-3.fc44 x86_64
ghostty 0:1.3.1-3.fc44 aarch64
```

No `ghostty-1.4.0` NEVRA exists in any enabled repository:

```sh
# run on bazzite 44.20261012.1
dnf --assumeyes repoquery --available --qf '%{name} %{evr} %{repoid}\n' ghostty
```

```text
ghostty 1.3.1-3.fc44 terra
ghostty 1.3.1-3.fc44 terra-source
```

### ujust update correctly reports no upgrade

`ujust update` runs `/usr/libexec/uupd-update`,
which starts `uupd-manual.service`,
whose `ExecStart` is `/usr/bin/uupd --hw-check=false --json --log-level=debug`.
Its System module executes `rpm-ostree upgrade`
(`/usr/share/ublue-os/just/10-update.just:5-9` pipes the service output to
`jq` for display).
The driver constructs that invocation at `drv/rpmostree/rpmostree.go:143`:

```go
// ublue-os/uupd drv/rpmostree/rpmostree.go at 9c3148f
	cli := []string{binaryPath, "upgrade"}
```

rpm-ostree resolves the layered packages against the enabled rpm-md
repositories and,
when nothing newer is published,
prints its own verdict at `src/daemon/rpmostreed-transaction-types.cxx:365-370`:

```c
// coreos/rpm-ostree src/daemon/rpmostreed-transaction-types.cxx at e5f67f1
  if (!changed)
    {
      if (upgrading)
        rpmostree_output_message ("No upgrade available.");
      else
        rpmostree_output_message ("No change.");
    }
```

`No upgrade available.` is rpm-ostree speaking for the whole transaction.
`Nothing to update.` is the Flatpak module reporting an empty update set.
Neither message is ghostty-specific;
both are correct while Terra publishes no newer build.

### The staged base update does not touch ghostty

A newer base deployment,
`44.20261026`,
is staged and awaiting reboot.
`rpm-ostree db diff` between the booted and staged deployments
reports 260 changed package lines and no `ghostty` line:
the staged update does not alter the ghostty layer.
Rebooting applies those upgrades but keeps ghostty at `1.3.1-3.fc44`.

## Verification

The installed and available package probes were run on Bazzite 44
on 2026-10-29.
The source checks used:

- Ghostty tag commit `83fb95624b0439ee660c2679b43b38c8b63769cd` at `v1.4.0`.
- Terra spec commit `f2b0addf9d679cfe661fc0048b283b6dde6e63ce` on branch `f44`.
- uupd commit `9c3148fd989df9baf1444e6245c373847a724f88`.
- rpm-ostree commit `e5f67f1be7dfb50bc64bf98b37b0a70f08568c7f`.
- Terra repodata `primary.xml` with
  sha256 `3afe48c1f8c430843b58ba5bbf31ca325648dcdd4711305efc319cd128c62555`,
  generated 2026-10-29T23:55:13Z,
  signature verified against `/etc/pki/rpm-gpg/RPM-GPG-KEY-terra44`.

Reproduce the package selection without consulting unrelated repositories:

```sh
# run on bazzite 44.20261012.1
rpm --query ghostty
rpm-ostree status
dnf --assumeyes repoquery --available --qf '%{name} %{evr} %{repoid}\n' ghostty
rpm-ostree db diff
journalctl --unit uupd-manual.service --since '2026-10-29 20:00' \
  --until '2026-10-29 20:05' --no-pager --output=cat
```

Working catalog:

- `/usr/bin/ghostty` is owned by `ghostty-1.3.1-3.fc44.x86_64` and reports
  `Ghostty 1.3.1`.
- `rpm-ostree status` identifies `ghostty` as a layered package in both
  retained deployments.
- The Terra 44 repositories return `1.3.1-3.fc44` as the latest available
  build for both x86_64 and aarch64.
- The staged `44.20261026` deployment composes and boots normally.

Unavailable catalog:

- The Terra 44 repositories return no `ghostty` NEVRA above `1.3.1-3.fc44`.
- Terra CI run 40538321263 fails to build the bumped `1.4.0` spec on both
  x86_64 and aarch64.
- No enabled repository provides Zig `0.17.x`.
- `ghostty-tip` in the same repository is a `1:202610150459` snapshot,
  older than the 1.4.0 tag,
  and its latest CI bump also fails
  (patch hunk reject on `test/cases.zig` in run 40946667781).

## Verified workarounds

No immediate Bazzite 44 workaround was applied or represented as verified
because no 1.4.0 RPM is published anywhere reachable.
The verified distribution path is to wait until the Terra-only upgrade query
returns a `ghostty-1.4.0` NEVRA,
then compose and boot the new deployment:

```sh
# run on bazzite 44.20261012.1
dnf --assumeyes repoquery --available --qf '%{name} %{evr}\n' ghostty
ujust update
systemctl reboot
ghostty --version
```

The tradeoff is waiting on Terra's packaging pipeline,
tracked as [terrapkg/packages issue 19566][terra-issue-19566].
`ujust update` also composes every available system update,
not only Ghostty,
and activation requires a reboot.

## What does not work

- Re-running `ujust update` resolves the same `1.3.1-3.fc44` NEVRA and
  repeats `No upgrade available.`
- Rebooting into the staged `44.20261026` deployment applies 234 base and
  layered upgrades but `rpm-ostree db diff` shows ghostty is not among them.
- Reinstalling `ghostty` from the enabled Terra 44 repositories selects the
  same `1.3.1-3.fc44` RPM.
- Switching to `ghostty-tip` does not reach 1.4.0 content:
  the published snapshot is `1:202610150459`,
  which predates the 2026-10-27 tag,
  and its CI build is failing separately.
- Building the 1.4.0 source with the packaged toolchains cannot work:
  `zig0.15` is 0.15.2 and Fedora `zig` is 0.16.0,
  while `requireZig` demands exactly 0.17.x.
- Flatpak and Homebrew channels carry no Ghostty on this host
  (`flatpak remote-ls flathub` lists none,
  `brew info --formula ghostty` reports no such formula).

A separate transient incident occurred during the nightly `uupd.service` run
at 2026-10-29 04:09:
`error: Updating rpm-md repo 'terra': cannot update repo 'terra':
repomd.xml GPG signature verification error: Bad PGP signature`,
ending that run with `Updates finished with errors!`.
The manual runs at 04:33 and 20:01 refreshed the same repository metadata
successfully,
so that failure neither caused nor masks the version lag.

## Upstream filing decision

`.out-of-scope/` was checked for ghostty,
Terra,
packaging,
uupd,
and Bazzite exemptions.
The only hit,
`claude-code-upstream-bugs.md`,
covers Claude Code cursor overrides and does not apply.
No matching exemption exists.

Terra's tracker already carries the exact report:
[issue 19566, `[FAILING] ghostty`][terra-issue-19566],
opened 2026-10-29 at 00:58:19 UTC,
assigned to `GildedRoach`,
linking the same failing job.
[Pull request 19567, `fix(ghostty): Update build requirements`][terra-pr-19567]
proposed switching `BuildRequires` from `zig0.15` to `zig`
and was closed unmerged on 2026-10-29 at 09:46:52 UTC.
Its thread records the maintainer position:

> Looks like we'll have to wait until Fedora releases version 0.17 or higher.
> Or add a new package with zig 0.17 to terra like the zig0.15 one.

Nothing new is worth a second issue.

Work in progress check,
run 2026-10-29:

- `gh issue list --repo terrapkg/packages --state all --search ghostty`
  returns issue 19566 open and assigned.
- `gh pr list --repo terrapkg/packages --state all --search 'ghostty zig'`
  returns pull request 19567 closed unmerged and its merged `zig0.15`
  macro predecessors.
- No merged change adds a Zig 0.17 package,
  and the enabled repositories confirm none is published.
- The maintainer named both fix paths on the closed pull request,
  so the plan is theirs and in motion.

1. Is it really upstream's fault?
   No.
   Ghostty 1.4.0 declares and enforces a Zig 0.17.0 build floor,
   Terra correctly serves its currently published `1.3.1-3.fc44`,
   and rpm-ostree correctly installs the newest package available
   from that repository.
   This is toolchain availability and publication lag rather than a defect.
2. Can upstream fix it?
   Terra can:
   package Zig 0.17 as its own `zig0.17` build dependency,
   matching the existing `zig0.15` approach,
   then point `BuildRequires` at it.
3. Are they supporting this use case?
   Yes.
   Terra packages Ghostty for Fedora 44 and Bazzite,
   and its tracker has a dedicated `Failing` issue type that this fills.
4. Would the repository welcome our contribution?
   The `failing.yml` issue template requires an LLM disclosure field
   and instructs agents to write
   `I did use an LLM to write this issue`.
   AI-assisted filings are accepted with truthful disclosure,
   so constraint 4 holds only for a draft that discloses assistance
   and names which checks a human verified.
5. Will they likely fix it?
   Yes.
   Issue 19566 is open and assigned,
   and the maintainer comment names both concrete fix paths.
6. Have we prototyped a minimal fix compatible with their architecture?
   No.
   The minimal fix is a new Zig 0.17 package plus a `BuildRequires` bump,
   and constraint 1 fails because this is publication timing
   rather than a defect,
   so no prototype was required or built.

One additive detail is absent from the thread and sharpens its plan:
`requireZig` matches major and minor exactly,
so the wait is for precisely `0.17.x`.
A Fedora `zig` landing first at 0.18 would fail the gate just like 0.16,
which matters for choosing between waiting and packaging `zig0.17` now.
The drafted comment is kept below and is not posted;
posting to an external repository is the poster's decision,
and any filing puts a real person's account in front of Terra maintainers,
where a maintainer can denounce a poster and some repositories run trust
or blocklist systems.

~~~md
**Disclosure:** I used an LLM agent to draft this comment. I ran the
reproduction commands and verified the cited source lines myself.

One detail on the two options in this thread: `requireZig` pins the exact
major.minor, so the wait is for precisely `0.17.x`, not `0.17 or higher`.
`src/build/zig.zig:9-11` at `v1.4.0` reads:

```zig
    if (current_vsn.major != required_vsn.major or
        current_vsn.minor != required_vsn.minor or
        current_vsn.patch < required_vsn.patch)
```

with `build.zig.zon:6` setting `.minimum_zig_version = "0.17.0",`.
A Fedora `zig` 0.18.x would fail this gate exactly like 0.16.0 does,
so waiting on the unversioned Fedora `zig` package can overshoot.

Fedora 44 inventory on 2026-10-29: `zig 0.15.2-3.fc44` (fedora),
`zig 0.16.0-1.fc44` (updates), `zig0.15 0.15.2-2.fc44~bootstrap` (terra).
No 0.17 is available in any enabled repository, while upstream Zig 0.17.0
released 2026-10-01. Packaging `zig0.17` alongside `zig0.15`, then pointing
`BuildRequires` at it, builds `ghostty-1.4.0-1.fc44` today.
~~~

[ghostty-v140]: https://github.com/ghostty-org/ghostty/releases/tag/v1.4.0
[terra-issue-19566]: https://github.com/terrapkg/packages/issues/19566
[terra-pr-19567]: https://github.com/terrapkg/packages/pull/19567
[terra-run-40538321263]: https://github.com/terrapkg/packages/actions/runs/40538321263/job/123169011317
[zig-index]: https://ziglang.org/download/index.json
