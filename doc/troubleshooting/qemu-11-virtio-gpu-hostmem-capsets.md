# QEMU 11.0.0 Venus requires hostmem and a libvirt seccomp workaround

## Metadata

- Status:
  Diagnosed,
  adopted,
  and reboot-verified.
  The domain now uses every optional API capability exposed by the installed
  `virtio-vga-gl` device.
  No comparative frame-time or latency benchmark was run.
- Diagnosed:
  2026-08-02.
- Affected environment:
  virt-manager 5.1.0,
  libvirt 12.4.0,
  QEMU 11.0.0,
  SPICE server 0.16.0,
  a Flathub source manifest using virglrenderer 1.3.0,
  host and guest Linux 7.1,
  host and guest Mesa 26.1,
  and an AMD Radeon RX 7600 host GPU.
- Scope:
  `bazzite-labwc-test`,
  the libvirt domain backed by `$HOME/labwc-vm-test/disk.qcow2`.
- Disposition:
  Keep the enhanced profile because the user explicitly prefers current GPU
  API capability over the narrower stable baseline.
  The user explicitly accepted the session-wide loss of QEMU seccomp after
  reviewing the security tradeoff.
  Preserve the prior persistent XML at
  `$HOME/labwc-vm-test/domain-before-virtio-gpu-sota.xml` for rollback.

## Symptom

The live and persistent domain both contain the expected accelerated local
console configuration:

```xml
<memoryBacking>
  <source type='memfd'/>
  <access mode='shared'/>
</memoryBacking>
<graphics type='spice'>
  <listen type='none'/>
  <image compression='off'/>
  <gl enable='yes'/>
</graphics>
<video>
  <model type='virtio' heads='1' primary='yes' blob='on'>
    <acceleration accel3d='yes'/>
  </model>
</video>
```

The running QEMU command confirms that this becomes SPICE GL on the RX 7600
render node plus a blob-capable `virtio-vga-gl` device:

```text
-spice ... image-compression=off,gl=on,rendernode=/dev/dri/renderD128
-device {"driver":"virtio-vga-gl","id":"video0","max_outputs":1,"blob":true,...}
```

The guest is genuinely hardware accelerated for OpenGL:

```text
direct rendering: Yes
OpenGL renderer string: virgl (AMD Radeon RX 7600 ...)
OpenGL core profile version string: 4.3 (Core Profile) Mesa 26.1.5
```

It is not using the maximum virtual GPU feature set:

```text
[drm] features: +virgl +edid +resource_blob -host_visible
[drm] number of cap sets: 2
Vulkan deviceName = llvmpipe (LLVM 22.1.8, 256 bits)
Vulkan driverName = llvmpipe
```

The visible symptom is therefore a capability ceiling,
not software rendering of the labwc desktop.
OpenGL reaches the virgl default of 4.3,
while Vulkan falls back to the CPU.

The display transport itself works:
the SPICE display,
cursor,
2x guest scale,
and viewer-driven resize path were already verified.
The resize-specific service ordering is recorded in
[`virt-manager-spice-agent-xwayland-resize-race.md`](virt-manager-spice-agent-xwayland-resize-race.md).

## Root cause

### QEMU leaves the additional capability sets off by default

QEMU 11.0.0 declares `venus` and `drm_native_context` as Boolean device
properties whose defaults are false
(`hw/display/virtio-gpu-gl.c:165-173`):

```c
static const Property virtio_gpu_gl_properties[] = {
    DEFINE_PROP_BIT("stats", VirtIOGPU, parent_obj.conf.flags,
                    VIRTIO_GPU_FLAG_STATS_ENABLED, false),
    DEFINE_PROP_BIT("venus", VirtIOGPU, parent_obj.conf.flags,
                    VIRTIO_GPU_FLAG_VENUS_ENABLED, false),
    DEFINE_PROP_BIT("drm_native_context", VirtIOGPU, parent_obj.conf.flags,
                    VIRTIO_GPU_FLAG_DRM_ENABLED, false),
};
```

The local domain enables `blob`,
but it sets neither capability flag and provides no `hostmem` window.

### Venus and DRM native context require both blob and hostmem

QEMU rejects either capability when `blob` or `hostmem` is missing
(`hw/display/virtio-gpu.c:1522-1545`):

```c
if (virtio_gpu_venus_enabled(g->parent_obj.conf)) {
    if (!virtio_gpu_blob_enabled(g->parent_obj.conf) ||
        !virtio_gpu_hostmem_enabled(g->parent_obj.conf)) {
        error_setg(errp, "venus requires enabled blob and hostmem options");
        return;
    }
}

if (virtio_gpu_drm_enabled(g->parent_obj.conf)) {
    if (!virtio_gpu_blob_enabled(g->parent_obj.conf) ||
        !virtio_gpu_hostmem_enabled(g->parent_obj.conf)) {
        error_setg(errp, "drm requires enabled blob and hostmem options");
        return;
    }
}
```

The [QEMU 11.0.0 documentation source][qemu-doc] states that the default
virgl path is limited to OpenGL 4.3,
that `hostmem` plus `blob` enables OpenGL 4.6,
and that [Venus][mesa-venus] and DRM native context need the same
host-memory window
(`docs/system/devices/virtio/virtio-gpu.rst:82-115`):

```rst
By default OpenGL version on guest is limited to 4.3. In order to enable
OpenGL 4.6 support, virtio-gpu host blobs feature (``hostmem`` and ``blob``
fields) should be enabled.

-device virtio-gpu-gl,hostmem=8G,blob=true,venus=true
-device virtio-gpu-gl,hostmem=8G,blob=on,drm_native_context=on
```

### QEMU advertises only explicitly enabled capability sets

Renderer initialization adds Venus and DRM flags only when their device
properties are enabled.
DRM native context additionally requires an EGL display and virglrenderer
newer than 1.1.1
(`hw/display/virtio-gpu-virgl.c:1454-1474`):

```c
if (virtio_gpu_venus_enabled(g->parent_obj.conf)) {
    flags |= VIRGL_RENDERER_VENUS | VIRGL_RENDERER_RENDER_SERVER;
}
if (virtio_gpu_drm_enabled(g->parent_obj.conf)) {
    flags |= VIRGL_RENDERER_DRM;

    if (!(flags & VIRGL_RENDERER_ASYNC_FENCE_CB)) {
        error_report("drm requires egl display and virglrenderer >= 1.2.0");
        return -EINVAL;
    }
}
```

The advertised capability catalog follows the same conditions
(`hw/display/virtio-gpu-virgl.c:1559-1577`):

```c
if (virtio_gpu_venus_enabled(g->parent_obj.conf)) {
    virgl_renderer_get_cap_set(VIRTIO_GPU_CAPSET_VENUS,
                               &capset_max_ver,
                               &capset_max_size);
    if (capset_max_size) {
        virtio_gpu_virgl_add_capset(capset_ids, VIRTIO_GPU_CAPSET_VENUS);
    }
}

if (virtio_gpu_drm_enabled(g->parent_obj.conf)) {
    virgl_renderer_get_cap_set(VIRTIO_GPU_CAPSET_DRM,
                               &capset_max_ver,
                               &capset_max_size);
```

This matches the current guest's two capsets and the enhanced probe's four.

### The installed Flatpak stack contains the optional implementations

The Flathub extension manifest at revision
`9632e695fef825a0d209ce744592013076152334` matches the installed artifact's
QEMU version and available device properties.
It enables Venus and AMD DRM rendering in virglrenderer 1.3.0
(`org.virt_manager.virt_manager.Extension.Qemu.yaml:116-129`):

```yaml
- name: virglrenderer
  buildsystem: meson
  config-opts:
    - -Dvideo=true
    - -Dvenus=true
    - -Dtests=false
    - -Ddrm-renderers=amdgpu-experimental,i915-experimental
  sources:
    - type: archive
      url: https://gitlab.freedesktop.org/virgl/virglrenderer/-/archive/1.3.0/virglrenderer-1.3.0.tar.gz
```

The `amdgpu-experimental` build option is also the reason not to treat DRM
native context as the stability default.

### libvirt's default seccomp policy blocks the Venus render server

The first real-domain boot advertised three or four capsets,
but capset 0 timed out and labwc remained blocked in uninterruptible sleep.
The QEMU log named the host-side failure:

```text
failed to initialize venus renderer
qemu-system-x86_64: virgl could not be initialized: -1
```

The generated QEMU command contained:

```text
-sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny
```

Venus-only and Venus plus native-context boots failed identically.
A direct QEMU probe without that sandbox had already succeeded.
Setting `seccomp_sandbox = 0`,
restarting the session `virtqemud`,
and retaining Venus made the same current
image start labwc normally.
The generated command then contained `-sandbox off`.
This one-variable differential identifies the seccomp policy as the cause.

The [libvirt security documentation][libvirt-passthrough-security] confirms
that its default QEMU policy denies process spawning,
that it cannot be changed per domain,
and that `seccomp_sandbox = 0` is the available global driver setting.
The original libvirt policy discussion identifies `spawn=deny` as blocking
`fork` and `execve` [explicitly][libvirt-seccomp-mail].
QEMU [issue 3156][qemu-3156] already tracks a server-file-descriptor design
that would let Venus work without disabling the sandbox.

### The SPICE display choice is already correct

The current domain uses local SPICE OpenGL with `listen=none`.
The [libvirt domain documentation][libvirt-domain] says native SPICE OpenGL
works locally through a Unix socket and has stronger performance than the
SPICE plus `egl-headless` alternative.
The same documentation says `blob=on` can reduce or eliminate copies in the
guest-to-host display path.

Replacing SPICE GL with `egl-headless`,
QXL,
or two-dimensional virtio would therefore move the local display path in
the wrong direction.
The missing features are on the virtual GPU device,
not the SPICE transport.

## Verification

### Versions and host capability

The observed versions were:

```text
virt-manager Flatpak 5.1.0
libvirt 12.4.0
QEMU 11.0.0
SPICE server 0.16.0
host Mesa 26.1.5
host GPU AMD Radeon RX 7600, radeonsi and RADV
host Linux 7.1.3
```

The host has one PCI display controller,
the RX 7600 used by the host Wayland session.
Its sysfs device exposes neither `sriov_totalvfs` nor
`mdev_supported_types`.
PCI assignment is therefore not a shareable upgrade on this host.

The installed QEMU device property probe listed all required knobs:

```text
blob=<bool>                 default: off
hostmem=<size>              default: 0
venus=<bool>                default: off
drm_native_context=<bool>   default: off
```

The installed device catalog did not contain a rutabaga device.
It contained `virtio-vga-gl`,
`virtio-gpu-gl`,
and vhost-user variants.

### Current configuration catalog

The current domain was inspected with:

```bash
flatpak run --command=virsh org.virt_manager.virt-manager \
  --connect qemu:///session dumpxml --inactive bazzite-labwc-test

flatpak run --command=virsh org.virt_manager.virt-manager \
  --connect qemu:///session qemu-monitor-command \
  bazzite-labwc-test --hmp 'info spice'
```

Guest capability checks used:

```bash
ssh -p 2222 -i "$HOME/labwc-vm-test/id_ed25519" \
  user@127.0.0.1 \
  'sudo dmesg | grep -E "features:|number of cap sets|cap set";
   DISPLAY=:12 glxinfo -B;
   vulkaninfo --summary'
```

Paths that work cleanly:

- KVM and host CPU passthrough are active.
- QEMU has multiple open file descriptors to host `/dev/dri/renderD128`.
- SPICE reports connected main,
  display,
  input,
  and cursor channels.
- Guest OpenGL is direct and rendered by virgl on the RX 7600.
- `resource_blob` and `context_init` are enabled.
- The guest display is current at 1800x1810 with scale 2.
- `spice-vdagent.service` and `xwayland-satellite.service` are active.

Paths that remain below the installed stack's capability:

- `host_visible` is disabled.
- Only virgl capsets 1 and 2 are advertised.
- OpenGL stops at 4.3.
- Vulkan exposes only llvmpipe.

### Disposable enhanced probe

The current VM and disk were not modified.
A qcow2 overlay used the offline
`disk-clean-labwc-uwsm.qcow2` image as its read-only backing file.
The probe used 4 GiB guest RAM,
4 vCPUs,
a 2 GiB host-memory window,
and this GPU device:

```text
-device virtio-vga-gl,hostmem=2G,blob=true,venus=true,drm_native_context=true
```

The probe was run once with `egl-headless` and once with SPICE GL.
A real `spicy` client connected to the SPICE Unix socket and opened main,
display,
input,
and cursor channels.
QEMU emitted no renderer error in either run.

Inside the disposable guest:

```text
[drm] Host memory window: 0x380000000000 +0x80000000
[drm] features: +virgl +edid +resource_blob +host_visible
[drm] features: +context_init
[drm] number of cap sets: 4
```

OpenGL selected the AMD native context:

```text
direct rendering: Yes
OpenGL vendor string: AMD
OpenGL renderer string: AMD Radeon RX 7600 (radeonsi, navi33, ACO, ...)
OpenGL core profile version string: 4.6 (Core Profile) Mesa 26.1.4
```

Vulkan exposed the RX 7600 through Venus:

```text
deviceName = Virtio-GPU Venus (AMD Radeon RX 7600 (RADV NAVI33))
driverName = venus
driverInfo = Mesa 26.1.4
```

A finite Vulkan rendering run exercised the selected Venus device rather than
only enumerating it:

```text
Selected GPU 0: Virtio-GPU Venus (AMD Radeon RX 7600 (RADV NAVI33)), type: DiscreteGpu
VKCUBE_EXIT=0
```

The enhanced probe therefore verifies capability exposure,
Vulkan rendering,
and a real SPICE client connection.
It does not provide a frame-rate benchmark,
a long-duration stability result,
or a regression test for the existing cursor and resize workarounds.
The disposable baseline had Mesa 26.1.4 and an earlier 7.1 guest kernel build,
while the current domain has Mesa 26.1.5 and the later host-matched kernel
build.
The probe verifies feature compatibility,
not the complete deployed-domain upgrade.

### libvirt override generation

libvirt 12.4.0 does not have stable domain XML attributes for `hostmem`,
`venus`,
or `drm_native_context`.
Its documented [`qemu:override` mechanism][libvirt-qemu] can add frontend
properties to the device generated by libvirt.

A minimal XML fixture with the same video,
memfd,
SPICE GL,
and override structure was passed through `virsh domxml-to-native`.
libvirt generated:

```text
{"driver":"virtio-vga-gl","id":"ua-video0","max_outputs":1,
 "blob":true,"hostmem":2147483648,"venus":true,
 "drm_native_context":true}
```

This proves the override reaches the intended device without adding a second
GPU.
The persistent domain initially had no user-authored video alias,
while the live XML contained libvirt's generated `video0` alias.
Adoption added one `ua-video0` alias to the persistent video element.

### Real-domain adoption

The persistent `bazzite-labwc-test` domain now uses:

```text
hostmem=2147483648
blob=true
venus=true
drm_native_context=true
sandbox=off
```

QMP returned the same four device-property values from the running
`ua-video0` object.
The current guest reported `+host_visible`,
four complete capsets with IDs 1,
2,
4,
and 6,
and no GPU warning after the seccomp workaround.

OpenGL moved to native AMD rendering:

```text
OpenGL renderer string: AMD Radeon RX 7600 (radeonsi, navi33, ACO, ...)
OpenGL core profile version string: 4.6 (Core Profile) Mesa 26.1.5
```

Vulkan exposed both Venus and native RADV paths.
Finite Wayland rendering runs succeeded on each selected physical-GPU path:

```text
Selected GPU 0: Virtio-GPU Venus (AMD Radeon RX 7600 (RADV NAVI33))
VENUS_EXIT=0
Selected GPU 2: AMD Radeon RX 7600 (RADV NAVI33)
RADV_EXIT=0
```

The SPICE client opened main,
display,
cursor,
input,
audio,
and USB redirection channels.
Opening the console changed the guest from 1280x800 to 1800x1810,
while scale remained 2.
That verifies viewer-driven resize and preserves the established scaling path.
A guest reboot changed the boot ID and reproduced all capsets,
OpenGL 4.6,
both Vulkan rendering runs,
active labwc integration services,
and the 1800x1810 scaled display.

## Verified workarounds

### Keep the current regression-covered display profile

No change is needed for labwc,
panel,
input,
cursor,
resize,
or general desktop testing.
The current configuration already uses libvirt's documented preferred local
SPICE GL topology and hardware-accelerated OpenGL.
This is a topology claim,
not a measured comparison of latency,
frame pacing,
CPU use,
or `image-compression=off`.

Tradeoff:
GPU software that requires OpenGL newer than 4.3 or hardware Vulkan does not
match the physical host.
Vulkan work runs on the guest CPU.

## Adopted enhancement

### Enable every available virtual GPU capability through qemu:override

The maximum-capability device is now end-to-end verified in the real
libvirt-managed domain:

```xml
<!-- Add xmlns:qemu="http://libvirt.org/schemas/domain/qemu/1.0" to <domain>
     if another qemu namespace element has not already done so. -->
<video>
  <model type="virtio" heads="1" primary="yes" blob="on">
    <acceleration accel3d="yes"/>
  </model>
  <alias name="ua-video0"/>
</video>

<qemu:override>
  <qemu:device alias="ua-video0">
    <qemu:frontend>
      <qemu:property name="hostmem" type="unsigned" value="2147483648"/>
      <qemu:property name="venus" type="bool" value="true"/>
      <qemu:property name="drm_native_context" type="bool" value="true"/>
    </qemu:frontend>
  </qemu:device>
</qemu:override>
```

Keep the existing memfd shared memory backing and SPICE GL configuration.
The override affects device creation,
so the domain must be off before adopting it.
If the persistent video element already contains an alias,
replace that alias with `ua-video0` rather than adding another.

The optional features are independent.
QEMU documents `hostmem` plus `blob` as sufficient for OpenGL 4.6 and adds
`venus` for Vulkan.
`drm_native_context` is a separate AMD-native driver path and is not required
for either capability.
Venus-only and the combined profile were both exercised during diagnosis.
The former exposed three capsets;
the latter exposes four and is the adopted configuration.
Both require the seccomp workaround in this libvirt stack.

The Flatpak session driver configuration is:

```ini
# $HOME/.var/app/org.virt_manager.virt-manager/config/libvirt/qemu.conf
seccomp_sandbox = 0
```

Restart the session `virtqemud` after changing this setting.
It applies to every current and future VM managed by this Flatpak
`qemu:///session` connection,
not only `bazzite-labwc-test`.
The connection currently contains only this domain.
The Flatpak sandbox remains,
but it does not replace the QEMU process-level seccomp defense that this
setting removes.

Tradeoffs:

- libvirt explicitly describes `qemu:override` as a development and testing
  interface with no compatibility guarantee;
  the domain is tainted while it is present.
- virglrenderer labels its AMD native-context backend experimental.
- The virtual GPU ABI becomes more dependent on the host GPU,
  Mesa,
  QEMU,
  and virglrenderer versions.
- Disabling QEMU seccomp removes a host-defense layer from every current and
  future domain in this Flatpak session connection.
  The connection currently contains only this domain,
  and the user explicitly accepted this tradeoff.
- The renderer enumerates duplicate Venus and native Vulkan devices.
  An unqualified `vkcube` selected the hardware-backed Venus GPU and exited 0,
  but other applications may need explicit device selection.
- The existing cursor,
  resize,
  2x-scale,
  reboot,
  OpenGL,
  and Vulkan paths passed the adoption checks.
- The 2 GiB host-memory window was verified here,
  but no workload-specific sizing study was performed.

## What does not work

### Venus does not initialize under libvirt's default seccomp policy

With `seccomp_sandbox` at its default,
libvirt emits `spawn=deny` and virglrenderer cannot start the Venus render
server.
The guest still sees the larger capset count,
but capset 0 times out and labwc blocks before publishing `WAYLAND_DISPLAY`.
Removing only DRM native context does not fix it.
Disabling the session QEMU seccomp sandbox does.

### Blob without hostmem does not unlock the later API paths

The current domain is the counterexample.
It has `blob=on`,
but the guest reports `-host_visible`,
only two capsets,
OpenGL 4.3,
and llvmpipe Vulkan.

### Scale Display does not add GPU capability

Viewer scaling changes the sampled presentation size.
It cannot add a host-memory window or capability set.
The virt-manager maintainers also record blur as the material downside of
scaled presentation in
[virt-manager issue 747][virt-manager-scaling].
The current resize-follow path preserves native pixels and should remain the
presentation choice.

### egl-headless is not a local performance upgrade

libvirt documents SPICE plus `egl-headless` as the remote-capable fallback
with weaker performance than native local SPICE OpenGL.
The earlier labwc investigation also observed pathological readback cost when
software cursors,
raw image transfer,
and `egl-headless` were combined.

### QXL and two-dimensional virtio move rendering back toward software

QEMU's source documentation says the two-dimensional backend requires a
software renderer for 3D.
QXL does not provide the virgl,
Venus,
or DRM native capsets used by the successful probe.

### Rutabaga is unavailable in this installed QEMU build

QEMU 11 documents rutabaga as another accelerated backend,
but the installed `-device help` catalog contains no
`virtio-gpu-rutabaga` device or rutabaga library.
It is not a selectable improvement without changing the Flatpak extension.

### PCI passthrough is not a drop-in improvement on this host

The RX 7600 is the only measured display controller and is actively used by
the host Wayland session.
It exposes no SR-IOV or mediated-device interface.
Assigning it whole would remove the host's renderer and abandon the current
local SPICE console unless the host gained another display GPU or moved to a
headless passthrough design.

## Recommendation and ranking

### Enhanced hostmem, Venus, and AMD native-context profile

Pros:
The real domain now exposes OpenGL 4.6 through native radeonsi,
Venus Vulkan,
and native RADV on the RX 7600.
It is the maximum feature set in the installed QEMU device.
Cursor,
scale,
viewer-driven resize,
reboot,
and finite Vulkan rendering checks pass.

Cons:
The AMD native-context backend is packaged as experimental.
libvirt needs an unsupported override,
and Venus requires disabling QEMU seccomp for every VM in this Flatpak
session connection.
Vulkan enumeration contains duplicate devices.
No comparative performance or long-duration stability run was performed.

### Baseline SPICE GL plus virgl profile

Pros:
The display path is hardware accelerated,
local,
configured with blob resources,
and uses only stable libvirt XML.
It retains libvirt's default QEMU seccomp defense.

Cons:
OpenGL is limited to 4.3 and Vulkan is software-rendered.
It does not meet the user's preference to exercise the installed stack's
current GPU API capabilities.

### Whole-GPU passthrough

Pros:
It removes the virtual API translation layer and can provide physical-GPU
behavior.

Cons:
The current host has no second display GPU,
SR-IOV virtual function,
or mediated device.
Whole-device assignment conflicts with the local desktop and SPICE workflow.

Ranking:
enhanced virtual GPU profile > baseline SPICE GL profile > whole-GPU
passthrough.
The enhanced profile beats the baseline because the user's explicit priority
is hands-on access to the current GPU API feature set,
and the adoption checks pass.
The baseline beats passthrough because it preserves the host renderer and
working local SPICE workflow without removing another security boundary.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers QEMU,
libvirt,
virt-manager,
virglrenderer,
or this capability class.

Duplicate searches found:

- [virt-manager issue 362][virt-manager-venus] tracks UI and CLI support for
  Venus and links the libvirt gap.
- [libvirt issue 638][libvirt-638] tracks stable XML for `hostmem`,
  Venus,
  and native-context options.
- [QEMU issue 3156][qemu-3156] tracks a sandbox-compatible Venus render-server
  file descriptor.
  Its goal cites the same libvirt `spawn=deny` conflict measured here.

The six filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
    Yes for the missing stable XML and sandbox-safe Venus launch path.
    The current builds expose the features,
    but the default libvirt security policy cannot start Venus.
2.  **Can upstream fix it?**
    Yes.
    QEMU can accept a pre-opened render-server descriptor,
    and libvirt can model both the descriptor and GPU properties.
3.  **Are they supporting this use case?**
    Yes.
    Dedicated open QEMU,
    libvirt,
    and virt-manager issues cover each management layer.
4.  **Would the repositories welcome a contribution?**
    Yes.
    The issue text invites patches,
    and the checked contribution files do not exclude this feature class.
5.  **Will they likely fix it?**
    Plausible,
    but not promised.
    The issues remain open and include a prior server-descriptor patch design.
6.  **Have we prototyped a minimal upstream fix?**
    No.
    This work verifies the failure,
    driver-wide workaround,
    and end-user behavior,
    but it does not implement the server-descriptor or stable-schema changes.

No new issue or comment was posted.
QEMU issue 3156 already describes the exact missing sandbox mechanism,
and no source patch was produced to advance that tracker.

### Additive comment draft, do not post as-is

~~~md
Tested QEMU 11.0.0 and virglrenderer 1.3.0 on a Radeon RX 7600 through
libvirt 12.4.0.

A real Linux domain using
`virtio-vga-gl,hostmem=2G,blob=true,venus=true,drm_native_context=true`
failed under libvirt's default
`-sandbox on,...,spawn=deny,...` policy.
QEMU logged `failed to initialize venus renderer`,
the guest timed out reading capset 0,
and labwc blocked before publishing `WAYLAND_DISPLAY`.
Venus-only failed the same way.

Setting `seccomp_sandbox = 0` for the session QEMU driver changed the generated
argument to `-sandbox off`.
The same domain then exposed four capsets,
OpenGL 4.6 on radeonsi,
Venus Vulkan,
and native RADV.
Finite `vkcube` runs succeeded on both Vulkan paths,
and SPICE cursor,
2x scale,
viewer resize,
and reboot checks passed.

This confirms the runtime gap tracked by QEMU issue 3156.
It does not implement the sandbox-compatible render-server descriptor.
~~~

## Source and environment record

Source was inspected in private,
read-only clones at these revisions:

- QEMU `v11.0.0`,
  commit `98b060da3a4f92b2a994ead5b16a87e783baf77c`.
- The virt-manager QEMU Flatpak extension,
  commit `9632e695fef825a0d209ce744592013076152334`.

The QEMU release tag matches the installed `qemu-system-x86_64 --version`.
The extension manifest revision matches the installed runtime's QEMU version,
device properties,
and renderer feature set.

Runtime evidence came from the persistent and live libvirt XML,
the running QEMU command and QMP properties,
QEMU renderer diagnostics,
SPICE monitor state,
guest kernel DRM logs,
`glxinfo -B`,
`vulkaninfo --summary`,
finite `vkcube` runs,
a disposable qcow2 overlay,
and the rebooted real domain.
The pre-adoption persistent XML is stored at
`$HOME/labwc-vm-test/domain-before-virtio-gpu-sota.xml`.
The adopted XML is stored at
`$HOME/labwc-vm-test/domain-virtio-gpu-sota.xml`.

[qemu-doc]: https://github.com/qemu/qemu/blob/v11.0.0/docs/system/devices/virtio/virtio-gpu.rst#L82-L115
[libvirt-domain]: https://libvirt.org/formatdomain.html#graphical-framebuffers
[libvirt-qemu]: https://libvirt.org/drvqemu.html#overriding-properties-of-qemu-devices
[libvirt-passthrough-security]: https://www.libvirt.org/kbase/qemu-passthrough-security.html
[libvirt-seccomp-mail]:
  https://lists.libvirt.org/archives/list/devel@lists.libvirt.org/message/XH36M6E3XG2FN6BZCGHXKZFUV3IHVOZX/
[mesa-venus]: https://docs.mesa3d.org/drivers/venus.html
[virt-manager-scaling]: https://github.com/virt-manager/virt-manager/issues/747
[virt-manager-venus]: https://github.com/virt-manager/virt-manager/issues/362
[libvirt-638]: https://gitlab.com/libvirt/libvirt/-/work_items/638
[qemu-3156]: https://gitlab.com/qemu-project/qemu/-/work_items/3156
