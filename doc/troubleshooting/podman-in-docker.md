# Podman inside Docker on Hetzner-style cloud volumes: six independent issues found while shipping inference-canary-viewer (2026-03-07)

Each bug below stands alone with its own symptom,
 root cause,
verification,
 workarounds with tradeoffs,
 what does not work,
 and a
5-constraint upstream audit.

---

## Bug 1: overlay storage driver fails inside Docker volumes

### Symptom

Pulling any image inside a podman-in-docker container fails with:

```text
mount /var/lib/containers/storage/overlay, flags: 0x1000: permission denied
```

Even with `SYS_ADMIN` capability granted,
 the overlay mount cannot be
established.

### Root cause

Docker's volume layer presents the storage directory through a
mountpoint that does not support overlayfs's `set_attr` semantics on
the lower or upper directory.
 Overlay needs to set extended attributes
on the underlying filesystem and to bind-mount over its own staging
area;
 both fail when the parent is a bind-mounted Docker volume on an
already-overlayed host filesystem.

This is a kernel-level interaction,
 not a podman bug;
 podman attempts
the canonical mount sequence and the kernel refuses.

### Verification

Reproduce inside the outer container:

```bash
mkdir -p /tmp/overlay-test/{lower,upper,work,merged}
mount -t overlay overlay \
  -o lowerdir=/tmp/overlay-test/lower,upperdir=/tmp/overlay-test/upper,workdir=/tmp/overlay-test/work \
  /tmp/overlay-test/merged
# Expected: permission denied
```

If the bare overlay mount fails,
 podman's overlay-backed storage will
fail the same way.

### Verified workaround

Switch podman's storage driver to `vfs` in
`/etc/containers/storage.conf`:

```toml
[storage]
driver = "vfs"
```

VFS does full file copies between layers instead of using overlay
mounts;
 performance is bounded by disk throughput rather than mount
syscalls.

Tradeoff:
 VFS multiplies disk usage by the layer count (full copy per
layer) and slows image pulls proportionally.
 For the inference-canary
deployment,
 pulls are infrequent and image sizes are small,
 so the
slowdown is acceptable.
 For workloads that pull large images on every
deploy,
 evaluate `fuse-overlayfs` instead (requires `/dev/fuse` and
adds another moving part).

### What does not work

- `SYS_ADMIN` alone:
   the capability is necessary but not sufficient;
  the underlying kernel mount path still refuses.
- `--cap-add=ALL`:
   same as `SYS_ADMIN` in terms of the overlay-on-
  bind-mount restriction;
   not a kernel-level workaround.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. The Linux kernel refuses
    overlay-on-bind-mount by design;
     podman behaves correctly.
2.  **Can upstream fix it?
    ** Either podman could surface a clearer error
    message ("storage driver overlay unavailable:
     try vfs or
    fuse-overlayfs"),
     but the underlying mount failure is not theirs to
    fix.
3.  **Are they supporting this use case?
    ** Podman supports VFS for
    exactly this case.
4.  **Will they likely fix it?
    ** Diagnostic wording could improve;
    functional behaviour cannot.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 The VFS escape hatch is documented and
works.

---

## Bug 2: fine-grained capabilities are insufficient for podman inside Docker

### Symptom

Running podman with a carefully curated capability set
(`SYS_ADMIN`,
 `MKNOD`,
 `/dev/fuse`,
 `seccomp=unconfined`,
`label=disable`) fails during image layer extraction:

```text
remount /, flags: 0x44000: permission denied
```

### Root cause

Image extraction performs a `remount` on the root mount point to apply
new flags.
 The remount path requires capabilities beyond the curated
set above;
 specifically it interacts with mount namespace setup that
Docker's non-privileged-container sandbox blocks.

Capability lists below `privileged: true` are insufficient for
podman's layer-extraction code path in the in-Docker scenario.

### Verification

Run podman without `privileged: true` and the curated capability list:

```bash
docker run --rm \
  --cap-add SYS_ADMIN --cap-add MKNOD \
  --device /dev/fuse \
  --security-opt seccomp=unconfined --security-opt label=disable \
  -it quay.io/podman/stable podman pull alpine
# Fails at the remount step.
```

Repeat with `--privileged`;
 the pull completes.

### Verified workaround

Set `privileged: true` on the outer container.
 Then lock down inner
podman containers explicitly:

```yaml
# compose snippet
services:
  podman-host:
    image: quay.io/podman/stable
    privileged: true
    # Inner containers launched by podman use:
    #   --cap-drop=ALL --read-only --network=none
```

Tradeoff:
 the outer container runs with full capabilities,
 but the
inner containers (the ones actually executing untrusted workloads)
drop everything.
 The blast-radius surface area is the outer host,
 not
the application;
 the application sits inside two layers of
containerisation with stricter inner limits.

### What does not work

- `--cap-add=ALL --privileged=false`:
   privileged is more than the union
  of caps;
   it also affects seccomp,
   apparmor/selinux,
   and device cgroup
  scope.
   Adding individual caps does not replicate it.
- Replacing podman with rootless podman:
   rootless podman has the same
  issue inside Docker;
   it just fails at user-namespace setup instead
  of remount.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. Docker's non-privileged
    sandbox is doing what it says on the tin.
2.  **Can upstream fix it?
    ** Docker could relax the remount restriction
    for containers with `SYS_ADMIN`;
     doing so would weaken the sandbox.
3.  **Are they supporting this use case?
    ** "Run podman inside Docker"
    is an unusual deployment;
     not a documented Docker use case.
4.  **Will they likely fix it?
    ** No.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 Privileged outer + restricted inner is
the canonical pattern.

---

## Bug 3: cgroup v2 controllers unavailable

### Symptom

Podman inside Docker fails to apply cgroup limits on inner containers:

```text
controller 'pids' is not available under /sys/fs/cgroup/libpod_parent/...
```

The error appears during inner-container creation,
 not at podman
startup.

### Root cause

Docker creates a per-container cgroup namespace that exposes only
the slice for that container.
 Podman expects to see the host's cgroup
tree to create its own sub-slices;
 the per-container namespace hides
the rest.

### Verification

```bash
docker run --rm -it quay.io/podman/stable cat /sys/fs/cgroup/cgroup.controllers
# Output is truncated -- pids/memory/cpu missing or shows only inherited subset.
```

Repeat with `--cgroupns=host`:

```bash
docker run --rm --cgroupns=host -it quay.io/podman/stable cat /sys/fs/cgroup/cgroup.controllers
# Full controller set.
```

### Verified workaround

Pass `cgroupns_mode: host` (Compose) or `--cgroupns=host` (CLI) so
podman sees the host's cgroup tree:

```yaml
services:
  podman-host:
    cgroupns_mode: host
```

Tradeoff:
 the outer container can read the host cgroup tree,
 which
slightly weakens isolation.
 For a deployment where the outer container
runs trusted application code (the case here),
 the trade is acceptable.
For multi-tenant scenarios it would not be.

### What does not work

- `--cgroup-parent=/` on the outer container:
   changes the slice
  podman runs under but does not unhide the controller subtree.
- `--privileged` alone:
   necessary for bug 2 but does not change
  cgroup-namespace visibility.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. Docker's per-container
    cgroup namespace is a deliberate isolation feature.
2.  **Can upstream fix it?
    ** They could;
     it would weaken isolation by
    default.
3.  **Are they supporting this use case?
    ** `cgroupns_mode` exists
    precisely for this case.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.
 Documented setting works.

---

## Bug 4: missing ca-certificates breaks TLS pulls

### Symptom

Podman pull from Docker Hub fails inside the container:

```text
x509: certificate signed by unknown authority
```

Even though the host can pull the same image without issue.

### Root cause

The host's Docker daemon performs TLS validation against its own CA
store;
 the daemon then writes the image bits into the container's
filesystem.
 Podman inside the container runs the TLS handshake
itself,
 using the container's CA store,
 which on minimal base images
(plain `debian:slim`) ships with zero root CAs.

### Verification

```bash
docker run --rm -it debian:slim ls /etc/ssl/certs/
# Empty or minimal.
```

After installing ca-certificates:

```bash
docker run --rm -it debian:slim sh -c "apt-get update && apt-get install -y ca-certificates && ls /etc/ssl/certs | head"
# Multiple PEM files visible.
```

### Verified workaround

Install ca-certificates into the outer container image:

```dockerfile
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
```

For the chicken-and-egg case where apt itself cannot reach repos over
HTTPS (because no CAs are present),
 see
[`TROUBLESHOOTING.hetzner-firewall.md`](hetzner-firewall.md)
which documents the two-step install for Ubuntu base images on a
Hetzner firewall.

Tradeoff:
 image size grows by ~150KB (the CA bundle).
 Negligible.

### What does not work

- Skipping TLS validation (`--tls-verify=false`):
   works but disables
  certificate verification for every pull,
   which is worse than the
  problem.
- Mounting the host's `/etc/ssl/certs` into the container:
   works for
  one host but breaks portability;
   the container image now depends on
  host filesystem layout.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. Minimal base images
    intentionally omit CAs to keep size down;
     users are expected to
    install them.
2.  **Can upstream fix it?
    ** Bundling CAs by default would bloat every
    minimal image.
3.  **Are they supporting this use case?
    ** Yes;
     `apt-get install
    ca-certificates` is the documented path.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.

---

## Bug 5: Bun crashes with OOM on CPUs without AVX

### Symptom

Inside the container,
 any `bun -e "..."` command crashes with exit
code 137 (OOM kill) and reports massive memory allocation (13GB+ RSS)
before the kernel reaps it.
 `bun --version` works because the version
print path does not invoke JavaScriptCore's JIT.

On Alpine images the crash is clearer:

```text
CPU lacks AVX support
```

On Debian images the failure is silent until OOMKilled.

### Root cause

Bun's JavaScriptCore expects AVX SIMD instructions for its JIT
optimisation path.
 QEMU's default CPU model (the default for
virtualised hosts including some Hetzner-style VMs) does not expose
AVX even when the underlying physical CPU supports it.
 JSC takes a
path that allocates aggressively before hitting the unsupported
instruction;
 on Linux that path manifests as runaway memory growth
until OOMKilled.

### Verification

Check the CPU flags visible inside the VM/container:

```bash
grep -m1 ^flags /proc/cpuinfo | tr ' ' '\n' | grep -E 'avx|sse4'
# If avx is absent, Bun will crash.
```

Reproduce:

```bash
docker run --rm debian:slim sh -c "apt-get update -qq && apt-get install -y curl unzip -qq && curl -fsSL https://bun.sh/install | bash && /root/.bun/bin/bun --version"
# OK on AVX-capable host CPU exposed to the VM.
# Fails with exit 137 on AVX-hidden VM.
```

### Verified workaround

Tell QEMU/libvirt to pass through the host CPU model so AVX is exposed:

```bash
# QEMU CLI
-cpu host

# libvirt XML
<cpu mode='host-passthrough'/>
```

Tradeoff:
 passthrough ties the VM to the specific CPU family;
 live
migration to a host with a different CPU model can fail.
 For static
deployments (the case here),
 the tie is acceptable.
 For migratable
fleets,
 use `host-model` instead,
 which selects the closest matching
canonical CPU model and still exposes AVX when the physical CPU has it.

### What does not work

- Forcing Bun to use the interpreter (`BUN_JSC_useJIT=0`):
   no such env
  var;
   JSC's JIT is enabled at build time.
- Switching to Node:
   works (Node's V8 has AVX-conditional code paths)
  but is a different runtime change.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline.
     Bun could detect
    the missing AVX and produce a clean error instead of OOM-spiral;
    the JIT path is the offender,
     not Bun's design choice to use a JIT.
2.  **Can upstream fix it?
    ** A `cpuid` check at startup with a friendly
    error message is a small change.
3.  **Are they supporting this use case?
    ** Bun ships for x86_64
    generally;
     the QEMU-without-AVX case is unusual but real.
4.  **Will they likely fix it?
    ** Unknown;
     not a blocker for the modal
    Bun user.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report unless this recurs in a context where
host-passthrough is unavailable.

---

## Bug 6: `bun --port 3000 file.html` binds to IPv6 loopback only

### Symptom

`bun --port 3000 /var/www/index.html` exits the parent shell with the
server seemingly running,
 but Docker's `-p 3000:3000` port forwarding
returns connection refused to external clients.
 `curl localhost:3000`
from inside the container works.

### Root cause

Bun's static-file-server mode (the no-script-just-html form) binds to
`::1` (IPv6 loopback) only.
 Docker's port-publish flag maps the host
IPv4 address `0.0.0.0:3000` to the container's `0.0.0.0:3000`,
 but no
process is listening there.

The `--hostname` flag is not recognised by the static-server mode;
 bun
treats it as a positional file path:

```text
error: File not found "0.0.0.0"
```

### Verification

Inside the container with the file-mode server running:

```bash
ss -ltnp | grep 3000
# tcp LISTEN 0 511 [::1]:3000  *:* users:(("bun",pid=...))
# Only the IPv6 loopback bind, no 0.0.0.0.
```

### Verified workaround

Use `Bun.serve()` inline with an explicit `hostname`:

```bash
bun -e "Bun.serve({port:3000,hostname:'0.0.0.0',fetch(r){ return new Response(Bun.file('/var/www/index.html')) }})"
```

Tradeoff:
 longer command,
 manual file routing.
 The static-server
shortcut is unusable for containerised serving;
 the inline `Bun.serve`
shape is required.

### What does not work

- `bun --port 3000 --hostname 0.0.0.0 file.html`:
   the static-server
  mode parses `--hostname 0.0.0.0` as a filename and errors.
- Publishing the container port to `::` (IPv6 wildcard):
   Docker on
  most Linux distributions still binds the IPv4 side,
   and external
  clients arriving on IPv4 cannot reach the container's IPv6-only
  listener.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Yes,
     partly.
     The static-server
    shortcut should accept `--hostname` or default to `0.0.0.0` when
    port-forwarded contexts (Docker) are common.
2.  **Can upstream fix it?
    ** Likely.
     Either accept the flag in
    static-server mode or default the bind to dual-stack.
3.  **Are they supporting this use case?
    ** Static serving for quick
    testing is documented;
     the IPv6-only bind is not.
4.  **Will they likely fix it?
    ** Unknown.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 The `Bun.serve()` workaround is enough
for this deployment.

---

## Debugging methodology lessons

These three lessons emerged from the bug-hunting on the day of the
deployment:

- **Measure before accusing.
  ** We initially guessed VFS was slow;
  timing showed the pull took 7 seconds.
   The real issue (AVX) was
  completely unrelated.
- **Read the actual error,
   not the symptom.
  ** Exit code 137 (OOM)
  looked like a memory issue.
   The Alpine crash report revealed a CPU
  instruction set problem causing the memory blowup.
- **Test the simplest case first.
  ** `bun --version` vs `bun -e "..."`
  isolated the issue to JIT compilation,
   narrowing the search space.
