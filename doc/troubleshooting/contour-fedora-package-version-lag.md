# Contour 0.6.3 on Bazzite 44: Fedora layering cannot select 0.7 before its RPM update lands

## Symptom

On Bazzite 44,
Contour resolves to the system executable and reports `0.6.3`:

```sh
command -v contour
contour --version
rpm --query --file /usr/bin/contour
```

```text
/usr/bin/contour
Contour Terminal Emulator 0.6.3
contour-terminal-0.6.3.8249-1.fc44.x86_64
```

This differs from upstream's latest release,
[Contour 0.7.0.8982][contour-070],
published on 2026-08-17 at 18:29:18 UTC.

## Root cause

### Contour is a layered Fedora RPM

`rpm-ostree status` lists `contour-terminal` under `LayeredPackages` in both retained deployments.
The JSON form lists it in `requested-packages`.
The [rpm-ostree administrator handbook][rpm-ostree-handbook] defines that field as yum repository packages
requested by the user for overlay.
It documents `rpm-ostree install <pkg>` as the underlying package-layering operation.

The evidence establishes that Contour was installed as a repository-backed rpm-ostree package layer,
not as a Flatpak,
Snap,
Homebrew package,
Nix profile entry,
or standalone executable.
It does not distinguish a direct `rpm-ostree install contour-terminal` invocation from a wrapper that invoked the same
operation.
No shell-history evidence was used.

The retained deployment dated 2026-07-31 already contains the request,
so the current RPM database's 2026-08-17 `Install Date` is deployment composition time,
not the original request time.

### Fedora 44 still publishes 0.6.3.8249

The Fedora 44 package branch at commit `c285f661efad4be41dd8957d0c68e7da26a5a8dc` pins version `0.6.3.8249`.
`contour-terminal.spec:1-7` contains:

```spec
Name:           contour-terminal
Version:        0.6.3.8249
Release:        %autorelease
Summary:        Modern C++ Terminal Emulator
License:        Apache-2.0
URL:            https://github.com/contour-terminal/contour
Source:         %{url}/archive/v%{version}/contour-%{version}.tar.gz
```

A Fedora-only DNF query returns `0.6.2.8008-3.fc44` and `0.6.3.8249-1.fc44`,
with no upgrade for the installed package.
Even [Fedora Rawhide][fedora-rawhide] still publishes `0.6.3.8249`.
The package manager therefore has no `0.7` RPM it can select.

### The release arrived after the current deployment

The booted layered deployment's commit timestamp is 2026-08-17 at 08:11:08 UTC.
The human-readable `Version` date,
2026-08-02,
belongs to its base Bazzite image rather than the locally layered commit.
Contour `0.7.0.8982` was published later on 2026-08-17 at 18:29:18 UTC.
That deployment could not contain a release that did not yet exist.

Fedora's Packit automation opened
[Fedora 44 pull request 17][fedora-pr-17]
at 19:00:09 UTC,
shortly after the upstream release.
The pull request changes the package version from `0.6.3.8249` to `0.7.0.8982`,
It was observed open with no comments on 2026-08-18 at 19:40:18 UTC;
the API reports its last update at 19:32:33 UTC.
Until that change is merged,
built,
and published to Fedora 44 updates,
`rpm-ostree upgrade` can only retain `0.6.3.8249`.

### The executable intentionally omits the release build serial

Fedora builds from the GitHub source archive.
The verified `0.6.3.8249` archive contains `metainfo.xml` but no `.git` directory or `version.txt`.
At upstream tag `v0.6.3.8249`,
commit `3aef6f7243f571fd4fda2642da190a244bb87a57`,
`cmake/Version.cmake:54-67` extracts only the three-component release value from `metainfo.xml`:

```cmake
if(("${THE_VERSION}" STREQUAL "" OR "${THE_VERSION_STRING}" STREQUAL "") AND (EXISTS "${CMAKE_SOURCE_DIR}/metainfo.xml"))
    file(READ "${CMAKE_SOURCE_DIR}/metainfo.xml" changelog_contents)
    string(REGEX MATCH "<release version=\"([0-9]*\\.[0-9]+\\.[0-9]+)\".*$" _ "${changelog_contents}")
    set(THE_VERSION ${CMAKE_MATCH_1})

    if (NOT("$ENV{RUN_ID}" STREQUAL ""))
        string(CONCAT THE_VERSION "${THE_VERSION}." $ENV{RUN_ID})
    endif()

    set(THE_VERSION_STRING "${THE_VERSION}")
endif()
```

`src/contour/ContourApp.cpp:489-499` embeds that value in the CLI description:

```cpp
return CLI::command {
    "contour",
    "Contour Terminal Emulator " CONTOUR_VERSION_STRING
    " - https://github.com/contour-terminal/contour/ ;-)",
    CLI::option_list {},
    CLI::command_list {
        CLI::command { "help", "Shows this help and exits." },
        CLI::command { "version", "Shows the version and exits." },
```

Consequently,
`contour --version` prints `0.6.3` while the Fedora RPM version is `0.6.3.8249`.
Those strings identify the same build line.

## Verification

The installed and available package probes were run on Bazzite 44 on 2026-08-18.
The source checks used:

- Fedora 44 package commit `c285f661efad4be41dd8957d0c68e7da26a5a8dc`.
- Installed-version upstream commit `3aef6f7243f571fd4fda2642da190a244bb87a57` at tag `v0.6.3.8249`.
- Latest upstream commit `374755def5cea3c5bb4e9be08c26d0c8cbed4e2c` at tag `v0.7.0.8982`.
- Fedora source archive SHA-512
  `a889e6e73882968c5f13c2c106fe0220eede6f6595060ed54f574997946c54d799f2ce00290549ce5b2735ffb4e80818eee66dac3590b27351145eec7405d46b`.

Reproduce the package selection without consulting unrelated repositories:

```sh
rpm --query --file /usr/bin/contour
rpm-ostree status
dnf --cacheonly \
  --disable-repo='*' \
  --enable-repo=fedora,updates,updates-archive \
  repoquery --available --latest-limit=10 contour-terminal
dnf --cacheonly \
  --disable-repo='*' \
  --enable-repo=fedora,updates,updates-archive \
  repoquery --upgrades contour-terminal
```

Working catalog:

- `/usr/bin/contour` is owned by `contour-terminal-0.6.3.8249-1.fc44.x86_64`.
- `rpm-ostree status` identifies `contour-terminal` as a layered package.
- The Fedora 44 repositories return `0.6.3.8249-1.fc44` as the latest available build.
- `contour --version` returns the corresponding three-component application version `0.6.3`.

Unavailable catalog:

- The Fedora 44 repositories return no `contour-terminal` upgrade.
- Fedora Rawhide has no published `0.7` build yet.
- The upstream `0.7.0.8982` release has Ubuntu 26.04 DEB,
  macOS DMG,
  and Windows assets,
  but no Fedora RPM or AppImage.
- Flatpak,
  Snap,
  Homebrew,
  and Nix do not provide the executable currently resolved on this host.

## Verified workarounds

No immediate Fedora 44 workaround was applied or represented as verified because no `0.7` RPM has been published.
The verified distribution path is to wait until the Fedora-only upgrade query returns `0.7.0.8982`,
then compose and boot the new deployment:

```sh
dnf --cacheonly \
  --disable-repo='*' \
  --enable-repo=fedora,updates,updates-archive \
  repoquery --upgrades contour-terminal
rpm-ostree upgrade
systemctl reboot
contour --version
```

The tradeoff is waiting for Fedora review and publication.
`rpm-ostree upgrade` also composes every available system update,
not only Contour,
and activation requires a reboot.

## What does not work

- Reinstalling `contour-terminal` from the enabled Fedora 44 repositories selects the same `0.6.3.8249` RPM.
- Running `rpm-ostree upgrade` before Fedora publishes the update cannot obtain upstream's GitHub release directly.
- Treating `contour --version` as the complete RPM version loses the `.8249` release build serial by design.
- Installing the upstream Ubuntu DEB as though it were a Fedora RPM uses the wrong package format and dependency model.
- The upstream release assets do not contain a Fedora RPM or AppImage to layer directly.
- The current RPM `Install Date` does not establish the original request date
  because each deployment has its own RPM database.

## Upstream filing decision

`.out-of-scope/` was checked for Contour,
Fedora packaging,
and rpm-ostree exemptions.
No matching exemption exists.
Contour's open and closed GitHub issues and pull requests were searched for Fedora or RPM reports involving `0.7`;
none matched.
Fedora's package pull requests contain an exact existing update:
[Fedora 44 pull request 17][fedora-pr-17].
There is nothing additive to post because it already changes the correct version and checksum.

1. Is it really upstream's fault?
   No.
   Contour released `0.7.0.8982`,
   Fedora correctly serves its currently published `0.6.3.8249` package,
   and rpm-ostree correctly installs the newest package available from that repository.
2. Can upstream fix it?
   Fedora's package maintainers can merge the existing update,
   build it,
   and publish it.
3. Are they supporting this use case?
   Yes.
   Fedora packages Contour for Fedora 44,
   and Packit generated a Fedora 44 update pull request for this release.
4. Would the repository welcome our contribution?
   The Fedora package repository exposes a pull-request workflow and contains the Packit contribution configuration.
   No repository-local prohibition on external or AI-assisted contributions was found.
5. Will they likely fix it?
   The generated update and the package's history of version bumps are positive signals,
   but no merge or published build exists yet.
6. Have we prototyped a minimal fix compatible with their architecture?
   No separate prototype is needed.
   The existing Packit pull request is already the minimal package metadata update,
   and constraint 1 fails because this is publication timing rather than a defect.

No new issue or comment draft is retained.
Repeating that `0.7.0.8982` exists would not advance the existing Fedora 44 update pull request.

[contour-070]: https://github.com/contour-terminal/contour/releases/tag/v0.7.0.8982
[fedora-pr-17]: https://src.fedoraproject.org/rpms/contour-terminal/pull-request/17
[fedora-rawhide]: https://packages.fedoraproject.org/pkgs/contour-terminal/contour-terminal/fedora-rawhide.html
[rpm-ostree-handbook]: https://coreos.github.io/rpm-ostree/administrator-handbook/
