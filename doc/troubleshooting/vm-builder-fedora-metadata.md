# VM image build spends minutes retrieving Fedora metadata from its selected mirror

## Symptom

During the OdyTTY migration on 2026-11-02,
`rpm-ostree install` in the development image remained at repository metadata retrieval.
The last completed progress message was `Updating metadata for 'librewolf'...done`.
The process was receiving data;
it had not failed or deadlocked.

## Observed boundary and limits

The base image reported Fedora CoreOS `44.20261012.3.1`.
Its Fedora repository configuration selected mirrors through:

```ini
# /etc/yum.repos.d/fedora.repo in the disposable build container
metalink=https://mirrors.fedoraproject.org/metalink?repo=fedora-$releasever&arch=$basearch
gpgcheck=1
```

`ss --tcp --info --numeric --processes` in the container's network namespace
showed the active `rpm-ostree` connection to `195.220.108.108:443`.
One observation recorded `36,790,730` received bytes
and `550.930` seconds since the last request bytes were sent.
The receive timestamp was current,
so log silence did not mean that the transfer had stopped.
The cause of the observed transfer rate was not established.
No defect is attributed to `rpm-ostree`,
Podman,
the mirror operator,
or the host network.

## Verification and workaround

A direct fetch from Fedora's `download-ib01.fedoraproject.org` endpoint returned
the `15,678,154`-byte Fedora 44 primary metadata file in `3.460687` seconds.
Its SHA-256 matched the repository metadata:
`c48e47563bbf65b996c95caf4a608223f982c314cab637e6ab87dd1df67b9d26`.
The release and updates `repomd.xml` requests also returned HTTP `200`.
These observations use different requests and are not a controlled bandwidth benchmark.

The first build was stopped by sending SIGTERM to its `rpm-ostree` PID.
The build exited `143`.
Only after it ended was the optional mirror selection added to
`package/vm-builder/cli/Containerfile`.
The default remains `FEDORA_DIRECT_MIRROR=false`.
When enabled,
the Containerfile replaces only the Fedora release and updates metalink lines with direct base URLs,
retaining the literal `$releasever` and `$basearch` substitutions and `gpgcheck=1`.
The updates archive and LibreWolf repositories are unchanged.

The verified build invocation was:

```sh
# package/vm-builder/cli
mise run //package/vm-builder/cli:image:build -- --build-arg FEDORA_DIRECT_MIRROR=true
```

The retry completed repository metadata retrieval,
resolved the package transaction,
downloaded the packages,
and completed the `rpm-ostree install` layer.
That layer includes OdyTTY `0.17.5`.
This evidence establishes the package-installation workaround;
the [migration record](../handover/odytty-interim-switch.md) records final image validation separately.

The tradeoff is selecting a specific download endpoint instead of metalink failover.
The resulting image retains that selection for the affected repositories.
Future builds without the argument retain the base image's ordinary mirror selection.

## What did not establish a fix

Waiting for another progress line did not identify the active transfer;
the socket counters established that data was arriving.
The kernel.org metadata index also returned HTTP `200`,
but no image build was validated against that mirror in this incident.
Neither an index fetch nor a checksum check alone proves that the package transaction completes.

## Upstream filing decision

No upstream defect was established,
so the upstream-fault condition is unmet.
Fixability,
support scope,
contribution policy,
maintainer intent,
and an upstream prototype are consequently not applicable.
No issue or comment was drafted or filed.
This is a local build-recovery record,
separate from the historical Fedora download observation in
[the Windows music-player build notes](music-player-windows-build-and-test.md).
