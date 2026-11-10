# Terra OpenSnitch 1.8.0-2 on Fedora 44 stages binaries that cannot activate

## Symptom

`rpm-ostree install opensnitch` stages `opensnitch-1.8.0-2.fc44.x86_64` from Terra.
The staged RPM owns both application binaries:

```text
/usr/bin/opensnitchd
/usr/bin/opensnitch-ui
```

It also owns the desktop entry and `opensnitchd.service`.
The current deployment does not expose those files until reboot.

The staged unit is disabled,
 so explicit service enablement would still be required.
More importantly,
 the staged deployment has functional activation blockers:

- Its `ExecStart` names absent `/usr/local/bin/opensnitchd`,
   not packaged `/usr/bin/opensnitchd`.
- `/etc/opensnitchd/default-config.json` and related daemon data are absent.
- PyQt6 and gRPC are absent,
   so `opensnitch-ui` exits during its first import.
- No OpenSnitch entry exists in `/etc/xdg/autostart`,
   so the UI would not launch at desktop login.

Trying to layer upstream `opensnitch-ui-1.8.0-1.noarch.rpm` over the Terra package fails:

```text
error: Checkout opensnitch-ui-1.8.0-1.noarch:
  Hardlinking 15/63366220d83ef9243213afff922ddaa113dff9a523eed29e9a11523bdc02b4.file
  to opensnitch-ui: File exists
```

That transaction does not add `opensnitch-ui` to the staged RPM database.
The collision is not evidence that a second UI package activated.
Both RPMs own `/usr/bin/opensnitch-ui`.

## Root cause

### Terra combines daemon and UI into one RPM

`terrapkg/packages@462374e:anda/apps/opensnitch/opensnitch.spec:81-85` installs the daemon,
the upstream unit,
 and the Python UI into one build root:

```spec
install -Dm755 opensnitchd -t %buildroot%_bindir
install -Dm644 daemon/data/init/opensnitchd.service %{buildroot}%{_unitdir}/opensnitchd.service

pushd ui
%pyproject_install
```

The same spec lists both binaries at
`terrapkg/packages@462374e:anda/apps/opensnitch/opensnitch.spec:105-106`:

```spec
%_bindir/opensnitch-ui
%_bindir/opensnitchd
```

The separate upstream UI RPM also owns `/usr/bin/opensnitch-ui`.
`rpm-ostree` therefore reaches an existing checkout path
when it tries to combine the local UI RPM with Terra's bundled UI.

### Terra records build dependencies but omits UI runtime dependencies

OpenSnitch v1.8.0 declares required UI modules in
`evilsocket/opensnitch@v1.8.0:ui/requirements.txt:1-5`:

```text
# required
pyqt6>=6.4
protobuf
grpcio-tools>=1.10.1
python-slugify>=7.0.0
```

The launcher imports PyQt6 before argument parsing at
`evilsocket/opensnitch@v1.8.0:ui/bin/opensnitch-ui:43-44`:

```python
from PyQt6 import QtWidgets, QtCore
from PyQt6.QtNetwork import QLocalServer, QLocalSocket
```

`grpc` is imported at `evilsocket/opensnitch@v1.8.0:ui/bin/opensnitch-ui:53`:

```python
import grpc
```

The v1.8.0 `ui/setup.py:11-38` call has no `install_requires` entry:

```python
setup(name='opensnitch-ui',
      version=version,
      # ...
      scripts = [ 'bin/opensnitch-ui' ],
      zip_safe=False)
```

Terra's spec has Python and PyQt build requirements at
`terrapkg/packages@462374e:anda/apps/opensnitch/opensnitch.spec:30-35`,
but no corresponding runtime `Requires`:

```spec
BuildRequires:  python3-devel
BuildRequires:  python-rpm-macros
BuildRequires:  python3dist(pip)
BuildRequires:  python3dist(setuptools) >= 61.0
BuildRequires:  python3dist(wheel) >= 0.37.1
BuildRequires:  python3dist(pyqt5)
```

The staged RPM's generated requirements consequently include Python 3.14 and native daemon libraries,
but not PyQt6,
 gRPC,
 protobuf,
 slugify,
 packaging,
 or requests.
The staged deployment has none of PyQt6 or gRPC.

### Terra omits desktop-session autostart

The freedesktop [Desktop Application Autostart Specification][xdg-autostart]
requires an application desktop file in an XDG autostart directory.
The default system directory is `/etc/xdg/autostart`.
A menu entry under `/usr/share/applications` does not register desktop-session autostart.

OpenSnitch's v1.8.0 UI RPM creates that registration at
`evilsocket/opensnitch@v1.8.0:utils/packaging/ui/rpm/opensnitch-ui.spec:45-47`:

```spec
deskfile=/etc/xdg/autostart/opensnitch_ui.desktop
if [ -d /etc/xdg/autostart -a ! -h  $deskfile -a ! -f $deskfile ]; then
    ln -s /usr/share/applications/opensnitch_ui.desktop /etc/xdg/autostart/
fi
```

Terra's combined package runs only systemd macros in its scriptlets.
The staged deployment has no `/etc/xdg/autostart/opensnitch_ui.desktop`.
Even after dependency repair,
users would need to launch the UI manually before it could display interactive connection prompts.

### Terra installs an upstream unit with the wrong prefix

The upstream source unit uses its source-build default at
`evilsocket/opensnitch@v1.8.0:daemon/data/init/opensnitchd.service:5-8`:

```ini
[Service]
Type=simple
ExecStart=/usr/local/bin/opensnitchd
Restart=always
```

Terra installs that file unchanged while placing the executable in `%_bindir`,
 which is `/usr/bin`.

OpenSnitch's own v1.8.0 RPM recipe performs the missing prefix correction at
`evilsocket/opensnitch@v1.8.0:utils/packaging/daemon/rpm/opensnitch.spec:45-47`:

```spec
sed -i 's/\/usr\/local/\/usr/' daemon/data/init/opensnitchd.service
install -m 755 daemon/opensnitchd %{buildroot}/usr/bin/opensnitchd
install -m 644 daemon/data/init/opensnitchd.service %{buildroot}/usr/lib/systemd/system/opensnitch.service
```

No unit override or `/usr/local/bin/opensnitchd` entry exists on the measured host.
Enabling Terra's unit without correcting `ExecStart` therefore cannot launch its packaged daemon.

### Terra omits daemon configuration that startup requires

Terra's `%install` section packages the binary and unit,
but contains no install operation for `daemon/data/default-config.json`,
`system-fw.json`,
 `network_aliases.json`,
 default rules,
 or tasks.
The staged RPM file list confirms that no `/etc/opensnitchd` path is owned.

OpenSnitch defaults to the missing configuration path at
`evilsocket/opensnitch@v1.8.0:daemon/main.go:67`:

```go
configFile = "/etc/opensnitchd/default-config.json"
```

Startup treats a missing file as fatal at
`evilsocket/opensnitch@v1.8.0:daemon/main.go:133-141`:

```go
func loadDiskConfiguration() (*config.Config, error) {
    if configFile == "" {
        return nil, fmt.Errorf("Configuration file cannot be empty")
    }

    raw, err := config.Load(configFile)
    if err != nil || len(raw) == 0 {
        return nil, fmt.Errorf("Error loading configuration %s: %s", configFile, err)
    }
```

OpenSnitch's own RPM recipe installs the omitted configuration at
`evilsocket/opensnitch@v1.8.0:utils/packaging/daemon/rpm/opensnitch.spec:51-66`.
It also packages eBPF objects at lines 68 to 70.
Terra's RPM omits those optional objects too;
the daemon can fall back to `/proc` monitoring after configuration is repaired.

## Verification

Verified on 2026-08-22 against:

- staged `opensnitch-1.8.0-2.fc44.x86_64` from Terra;
- staged rpm-ostree commit `8e4066c92cc77be5209f09398e4c547e0a1c42d759b5ef810250c80db5498f98`;
- `terrapkg/packages` commit `462374e056f69facd2357797326c679d0f3b827c`;
- OpenSnitch tag `v1.8.0`,
   commit `b404c4c6316760fa7bc415509d3f8d747f7dc9cc`.

Set the measured deployment root:

```bash
staged=/sysroot/ostree/deploy/default/deploy/8e4066c92cc77be5209f09398e4c547e0a1c42d759b5ef810250c80db5498f98.0
```

### Checks that succeed

The staged RPM database contains Terra's combined package but no separate UI package:

```console
$ rpm --root "$staged" --query opensnitch opensnitch-ui
opensnitch-1.8.0-2.fc44.x86_64
package opensnitch-ui is not installed
```

Terra's package owns both binaries:

```console
$ rpm --root "$staged" --query --file /usr/bin/opensnitch-ui
opensnitch-1.8.0-2.fc44.x86_64
$ rpm --root "$staged" --query --list opensnitch | grep -E '/usr/bin/opensnitch(d|-ui)$'
/usr/bin/opensnitch-ui
/usr/bin/opensnitchd
```

The daemon binary links and reports its version when staged libraries are supplied:

```console
$ LD_LIBRARY_PATH="$staged/usr/lib64" "$staged/usr/bin/opensnitchd" --version
1.8.0
```

Fedora 44 repositories provide the runtime dependencies named by the prototype:

```console
$ dnf --cacheonly repoquery --available --latest-limit=1 \
    python3-pyqt6 python3-grpcio python3-protobuf python3-slugify python3-packaging python3-requests
python3-grpcio-1.48.4-57.fc44.x86_64
python3-packaging-25.0-8.fc44.noarch
python3-protobuf-3.19.6-20.fc44.noarch
python3-pyqt6-6.11.0-4.fc44.x86_64
python3-requests-2.33.1-1.fc44.noarch
python3-slugify-8.0.4-4.fc44.noarch
```

### Checks that fail

The UI fails at its first import:

```console
$ PYTHONPATH="$staged/usr/lib/python3.14/site-packages" \
    /usr/bin/python3 -sP "$staged/usr/bin/opensnitch-ui" --help
Traceback (most recent call last):
  File ".../usr/bin/opensnitch-ui", line 43, in <module>
    from PyQt6 import QtWidgets, QtCore
ModuleNotFoundError: No module named 'PyQt6'
```

The daemon reaches its mandatory configuration load and exits:

```console
$ LD_LIBRARY_PATH="$staged/usr/lib64" "$staged/usr/bin/opensnitchd" -process-monitor-method proc
WAR  Error loading network aliases:
     open /etc/opensnitchd/network_aliases.json: no such file or directory
!!!  Error loading configuration /etc/opensnitchd/default-config.json:
     open /etc/opensnitchd/default-config.json: no such file or directory
```

The staged unit is disabled and names the absent executable:

```console
$ systemctl --root="$staged" is-enabled opensnitchd.service
disabled
$ grep '^ExecStart=' "$staged/usr/lib/systemd/system/opensnitchd.service"
ExecStart=/usr/local/bin/opensnitchd
```

The desktop-session autostart entry is absent:

```console
$ test -e "$staged/etc/xdg/autostart/opensnitch_ui.desktop"
$ echo $?
1
```

The upstream local UI RPM and Terra RPM both own the path that reported `File exists`:

```console
$ rpm --query --package --list ~/Downloads/opensnitch-ui-1.8.0-1.noarch.rpm \
    | grep --line-regexp '/usr/bin/opensnitch-ui'
/usr/bin/opensnitch-ui
$ rpm --root "$staged" --query --file /usr/bin/opensnitch-ui
opensnitch-1.8.0-2.fc44.x86_64
```

## Verified workarounds

The official replacement is staged but has not been booted or activated.
No OpenSnitch process was started during staging;
the booted deployment remains active.
The booted system already has an `inet opensnitch` nftables table.
Its output and forward chains use `policy accept`,
and its queue rules use `flags bypass` while no daemon process is running.
No OpenSnitch unit journal entries appeared during the staging window,
so the table was left unchanged rather than attributing it to the offline transaction.

### Replace Terra's package with the official release pair

OpenSnitch v1.8.0 publishes separate daemon and UI RPMs.
The daemon SHA-256 is
`e06e9119daf764e56455b61c319e496274c0274bb53bb94a0ff1ab72967fea7d`;
the UI SHA-256 is
`e5527b6b0040f771cd5345d4917269f0fe98b6d06064bae15f4ab937e45b4a08`.
Both match GitHub's release asset digests.
Both RPM signatures verify against release key fingerprint
`F34016AC014BAAF8C90AC730141D0D4E9FF44A67`.

The two official RPM file lists do not overlap.
The daemon RPM includes `/etc/opensnitchd` configuration,
eBPF objects,
 `/usr/bin/opensnitchd`,
 and `opensnitch.service`.
The UI RPM declares PyQt6 as a hard dependency and gRPC,
protobuf,
 slugify,
 packaging,
 notifications,
 and Qt SQL support as recommendations.

This transaction was verified with `--dry-run` on the affected host:

```bash
rpm-ostree install --dry-run \
  --uninstall=opensnitch \
  ~/temp/agent/opensnitch-official-2026-08-21/opensnitch-1.8.0-1.x86_64.rpm \
  ~/Downloads/opensnitch-ui-1.8.0-1.noarch.rpm
```

It resolved successfully,
planned removal of the Terra package request,
and selected both official RPMs plus the missing Fedora 44 UI dependencies.
The same command then succeeded without `--dry-run`.
`rpm-ostree status --json` reports both RPMs under `requested-local-packages`
in the staged deployment and no Terra package request.

Inspection of the staged deployment confirms:

- the official RPM versions are `opensnitch-1.8.0-1.x86_64`
  and `opensnitch-ui-1.8.0-1.noarch`;
- configuration,
   rules,
   tasks,
   daemon and UI executables,
  eBPF objects,
   and `opensnitch.service` are present;
- `opensnitch.service` executes `/usr/bin/opensnitchd`;
- the required Python packages are installed;
- the UI autostart link targets `/usr/share/applications/opensnitch_ui.desktop`.

The offline RPM scriptlets did not enable `opensnitch.service`.
`systemctl --root=<staged-deployment> is-enabled opensnitch.service` reports `disabled`.
Enabling through `--root` is not possible because the staged deployment is read-only.
After booting it,
activate the firewall deliberately with:

```bash
sudo systemctl enable --now opensnitch.service
```

Tradeoff:
local RPM requests persist across rpm-ostree upgrades,
but they do not follow new OpenSnitch releases from a package repository.
Each future upstream release requires a deliberate local RPM replacement.

### Correct Terra's package source

A package-spec correction prototype is recorded as
[`terra-opensnitch-1-8-packaging.patch`](terra-opensnitch-1-8-packaging.patch).
Its zero-context hunks apply to a clean `terrapkg/packages@462374e` checkout with:

```bash
git apply --check --unidiff-zero terra-opensnitch-1-8-packaging.patch
git apply --unidiff-zero terra-opensnitch-1-8-packaging.patch
```

It:

- bumps Terra's RPM release;
- corrects the service executable path before installation;
- packages default configuration,
   rules,
   and tasks as `%config(noreplace)`;
- adds hard runtime requirements for modules needed by the base UI;
- adds optional requests,
   monitoring,
   and notification integrations as recommendations;
- installs the desktop entry in the system XDG autostart directory.

A structural positive-control harness reported the original spec as `INCOMPLETE`,
listed every absent correction,
and reported the patched spec as `READY`.
`git diff --check` also passed.

Tradeoff:
the prototype has not been built into an RPM or exercised through an installed Fedora 44 package.
It is source evidence for Terra maintainers,
 not a package the user should install.
It also leaves Terra's omitted eBPF objects unchanged;
OpenSnitch can fall back to `/proc`,
 but that differs from upstream's default eBPF path.

## What does not work

- **Reboot alone.**
  Reboot exposes the staged files,
   but does not fix missing UI imports,
   configuration,
   or the unit's executable path.
- **Enable the unit alone.**
  `opensnitchd.service` would still execute absent `/usr/local/bin/opensnitchd`.
- **Start `/usr/bin/opensnitchd` directly.**
  It exits because `/etc/opensnitchd/default-config.json` is absent.
- **Retry the separate upstream UI RPM over Terra's RPM.**
  Both packages own `/usr/bin/opensnitch-ui`,
   and the observed checkout abort leaves no separate UI package staged.
- **Install only PyQt6.**
  This moves the UI past its first import,
  but does not supply other base UI modules and does not repair the daemon.
- **Treat `systemctl is-enabled` as the packaging defect.**
  Disabled state can be an expected activation choice.
  The wrong `ExecStart`,
   missing configuration,
   and missing runtime dependencies are the functional packaging defects.

## Upstream filing artifact: no permissible AI-authored comment

The exact duplicate is [terrapkg/packages issue 15904][terra-15904].
It reported the missing unit and missing configuration.
[Pull request 15995][terra-15995] added the unit,
but installed its upstream `/usr/local` path unchanged and did not add configuration.
[A comment posted on 2026-08-22][terra-15904-comment]
already identifies the path mismatch and repeated configuration omission.
The missing UI runtime requirements and absent XDG autostart entry are additional evidence not present in that thread.

No additive comment draft is provided.
Terra's current [AI policy][terra-policy] says LLMs must not write issue descriptions or further comments.
Copying an agent-written draft into the issue would violate that policy.

### Upstream filing decision

1.  **Is it really upstream's fault?**
    Yes,
     where upstream means Terra packaging.
    The OpenSnitch source contains the required data.
    Its own RPM recipe demonstrates both prefix correction and configuration installation.
2.  **Can upstream fix it?**
    Yes.
    The package spec controls runtime requirements,
     installed data,
     and unit rewriting.
3.  **Are they supporting this use case?**
    Yes.
    Terra ships the package for Fedora 44,
     and issue 15904 requested an out-of-box working service.
4.  **Would the repo welcome our contribution?**
    Not as agent-authored external text.
    Terra accepts package fixes,
    but its AI policy explicitly prohibits LLM-written issue comments and pull request descriptions.
5.  **Will they likely fix it?**
    There is positive maintenance evidence:
    PR 15995 was merged for the first report,
     and a maintainer backported it to Fedora branches.
    This is evidence of active maintenance,
     not a prediction of timing.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Partially.
    The linked spec patch covers the measured activation blockers and passes its structural positive-control harness.
    It has not passed a Terra package build or installed-runtime test,
     so it does not satisfy the full prototype gate.

Constraints 4 and 6 fail.
Nothing should be posted or filed from this document.
No `.out-of-scope/` entry covers Terra or OpenSnitch packaging.

## Sources

- [Terra packages repository][terra-packages]
- [Terra package issue 15904][terra-15904]
- [Terra package pull request 15995][terra-15995]
- [Terra AI and packaging policy][terra-policy]
- [OpenSnitch v1.8.0 source release][opensnitch-release]
- [Desktop Application Autostart Specification][xdg-autostart]

[terra-packages]: https://github.com/terrapkg/packages
[terra-15904]: https://github.com/terrapkg/packages/issues/15904
[terra-15904-comment]: https://github.com/terrapkg/packages/issues/15904#issuecomment-5377198692
[terra-15995]: https://github.com/terrapkg/packages/pull/15995
[terra-policy]: https://docs.terrapkg.com/contributing/policies/#ai-policy
[opensnitch-release]: https://github.com/evilsocket/opensnitch/releases/tag/v1.8.0
[xdg-autostart]: https://specifications.freedesktop.org/autostart-spec/latest/
