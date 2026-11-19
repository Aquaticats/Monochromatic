# Rootless podman traffic reaches OpenSnitch as `passt.avx2`

Rootless podman sends every container's network traffic through `pasta`, so OpenSnitch asks about `passt.avx2`
rather than about the program inside the container, and a rule for it covers every rootless container at once.
Since 2026-11-18 a permanent rule allows it (section "The firewall rule: `passt` is allowed permanently").

## Symptom

On 2026-11-18 at about 18:28 (America/Toronto), a `podman run` of `docker.io/cachyos/cachyos-v4:latest` that ran
`pacman -Sy` hung at ":: Synchronizing package databases..." while OpenSnitch 1.9.0 showed:

```text
# OpenSnitch v1.9.0 prompt, as the owner saw it
passt.avx2 (/usr/bin/passt.avx2)
/usr/bin/pasta --config-net --dns-forward 169.254.1.1 -t none -u none -T none -U none --no-map-gw --quiet
  --map-guest-addr 169.254.1.2 --netns /run/user/1000/netns/netns-4df75fd5-...
passt.avx2 is connecting to fastly.mirror.pkgbuild.com on TCP6 port 443
User ID 1000
```

The container was stopped with `podman stop`; it needed a SIGKILL after five seconds.

## Root cause

Rootless podman's default network here is `pasta` (`pasta 2026_10_02.cba3570`). The container's network namespace
has no route of its own; `pasta` opens the matching sockets on the host for it, so the host firewall sees the
connection as `pasta`'s (its binary is `passt.avx2`). The prompt's command line names the container's network
namespace, not the program in it.

Podman is not in the data path at all: it sets the container up and hands it to `conmon`. `pasta`'s manual describes
itself as "a translation layer between a Layer-2 virtual interface and native Layer-4 sockets": the container sees a
tap device, and `pasta`, running in the host's namespace, reads its packets and opens ordinary host sockets to the
destination. OpenSnitch names the process that owns the socket, which is `pasta`.
It shows `/usr/bin/passt.avx2` because `/usr/bin/pasta` is a link to `passt` (one program, which behaves as `pasta`
when called by that name) and `/usr/bin/pasta.avx2` a link to `passt.avx2`, the build for AVX2 CPUs; the prompt shows
the executable `passt.avx2` running with the command line `/usr/bin/pasta ...` (`ls -l /usr/bin/pasta*
/usr/bin/passt*`, 2026-11-18).

Who else runs `pasta` here, read 2026-11-18: `pacman --query --info passt` lists it as required by `podman` only and
optional for nothing. Everything that drives rootless podman uses it, including `podman build` steps that download
packages and this repository's tooling that runs containers. libvirt 12.9.0 is installed and can use `passt` as a
VM's user-mode network backend when a domain asks for it (`<backend type='passt'/>`); no package in this repository
names `passt`. Any program can also start `pasta` for a namespace of its own, since it needs no privilege.

`pasta` has no destination permission controls. Its manual lists, for outbound traffic, only `--no-tcp`,
`--no-udp` and `--no-icmp`, which disable a whole protocol, and `-o`/`--outbound`, `--outbound-if4` and
`--outbound-if6`, which pick the source address or interface (`man pasta`, read 2026-11-18). There is no allow or deny
list and no prompt. Whoever starts the container picks those options, and podman's defaults give every rootless
container full outbound access. The one per-container control is podman's `--network none`, chosen by the caller.

## The firewall rule: `passt` is allowed permanently

The owner's policy, 2026-11-18: "My policy is allow things that have builtin net permission controls."
The session first recommended Drop, reading the policy against `pasta` alone. The owner then decided:
"Permanently allow passt/pasta . mvm is our too so its own builtin disabling net is possible and should be added as a
gh issue feature request on our repo." The control that counts is the caller's: podman's `--network none` turns a
container's network off, so a container that should not reach the network is started with it.

The rule is `/etc/opensnitchd/rules/allow-passt.json` (root-owned, mode 600 like its neighbours), action `allow`,
duration `always`, operand `process.path`, regular expression `^/usr/bin/passt(?:\.avx2)?$`, case-sensitive.
It was written in place with `sudo tee` so the rules watcher sees a `Write` event
(`doc/troubleshooting/opensnitch-1-7-2-system-firewall-quirks.md`, section "Update rule files in place");
`/var/log/opensnitchd.log` shows `Ruleset changed due to allow-passt.json, reloading ...` at 23:38:20 UTC
(18:38:20-05:00). Before it, the log shows four in-memory deny rules for `/usr/bin/passt.avx2`, from 23:31:33 to 23:33:04 UTC,
spaced exactly 30 seconds apart. The UI's prompt timeout here is `default_timeout=30`
(`~/.config/opensnitch/settings.conf`), so these may be prompts that timed out to their default action as much as
Drop answers; which it was is not established. No rule file under `/etc/opensnitchd/rules/` other than
`allow-passt.json` names `passt`.

Checked afterwards: `podman run --rm --network pasta docker.io/cachyos/cachyos-v4:latest curl --head
https://geo.mirror.pkgbuild.com/` printed `http 200` and exited 0, with no prompt.

The same switch for this repository's VM tool is issue 6312 ("mvm: a built-in switch to start a VM with no network"):
every `mvm` VM gets QEMU user-mode networking today (`commonDevices` in `package/mvm/cli/src/domain-xml-builders.ts`)
with no way to leave it out.

## Verified workarounds

### Build without container networking

Fetch what the build needs on the host (crate sources with `cargo vendor` or `cargo fetch`, packages with the host
tools), mount them read-only, and run the container with `--network none`. Nothing then reaches `pasta`.
Use it for any build that does not need the network, whatever the firewall allows. Not yet run here.

## What does not work

- Allowing `passt.avx2` once per prompt: each new connection prompts again, so unattended builds stall at the prompt.
- Restricting destinations inside `pasta`: it has no option for it.
