// The podman arguments of a container that renders on the hardware GPU.
// `--device /dev/dri` passes the host's render nodes so that wgpu finds the hardware Vulkan driver (RADV) of the image's
// mesa-vulkan-drivers; `VK_DRIVER_FILES` names that driver only, so the CPU rasterizer (lavapipe) that package also
// installs is not even listed. The native tests render on the GPU through the harness in src/native/gpu_snapshot.rs,
// so every container that runs them takes these arguments; no other task needs them. The podman bound caps no GPU
// memory. Plan: doc/planning/ide-gpu-snapshot-harness.md.
export const gpuArguments = [
  '--device',
  '/dev/dri',
  '--env',
  'VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json'
];
