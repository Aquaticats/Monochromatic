# Rootless podman traffic reaches OpenSnitch as `passt.avx2`

Rootless podman sends every container's network traffic through `pasta`, so OpenSnitch asks about `passt.avx2`
rather than about the program inside the container, and a rule for it covers every rootless container at once.

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

## What follows for the firewall rule

The owner's policy, 2026-11-18: "My policy is allow things that have builtin net permission controls."
`pasta` has none, so by that policy it gets no permanent allow; a permanent rule for `passt.avx2` would let any
container any agent or test starts reach any host. The recommendation given was Drop.

## Verified workarounds

### Build without container networking

Fetch what the build needs on the host (crate sources with `cargo vendor` or `cargo fetch`, packages with the host
tools), mount them read-only, and run the container with `--network none`. Nothing then reaches `pasta`.
This was the plan for the `zfsdev` build in `doc/audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md` before the
owner's decision made that build unnecessary; it has not been run yet.

### A narrow rule, if builds need the network

An OpenSnitch rule for `passt.avx2` limited to the hosts a build needs (the CachyOS mirrors, `index.crates.io`,
`static.crates.io`) keeps the rule per program and per destination. Not created or tested yet.

## What does not work

- Allowing `passt.avx2` once per prompt: each new connection prompts again, so unattended builds stall at the prompt.
- Restricting destinations inside `pasta`: it has no option for it.
