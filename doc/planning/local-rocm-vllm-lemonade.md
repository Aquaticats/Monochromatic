# Local ROCm, vLLM, and Lemonade installation

## Request and settled scope

The user requested installation of ROCm, vLLM, and Lemonade,
with a grilling session before execution.
On 2026-10-30 the user selected:

- Lemonade-managed vLLM rather than an independent vLLM environment.
- Packaged binaries only.
  If GPU compatibility fails, report the blocker instead of attempting a source build.
- Official Lemonade container under Podman, including its browser UI.
  The user explicitly accepted the upstream Ubuntu 24.04 base.

Installation has not started.
Model selection, service behavior, resource limits,
and final shared-understanding confirmation remain open.

In response to the workload question, the user specified `vllm-d3`.
Its project identity, dependencies, packaged distribution,
and fit with Lemonade-managed vLLM must be established before finalizing the installation route.
This clarification does not yet identify model weights or a context length.

## Local evidence

Read-only probes on 2026-10-30 found:

- `/etc/os-release`: Bazzite `44.20261026.0`, Fedora 44.
- `uname --release`: `7.2.11-ogc4.1.fc44.x86_64`.
- `lspci -nnk`: Navi 33, PCI `1002:7480`, using `amdgpu`.
- `/sys/class/kfd/kfd/topology/nodes/1/properties`:
  `gfx_target_version 110002`, render minor 128.
- `/sys/class/drm/card1/device/mem_info_vram_total`: 8,573,157,376 bytes.
- RPM inventory: ROCm runtime, core, HIP, and development components at 7.1.1.
  Installed packages do not establish working GPU inference.
- Podman and Distrobox are installed.
- No vLLM or Lemonade command was found on PATH,
  or declared in the current global toolchain plan and lock.
  Arbitrary virtual environments and containers were not enumerated.

Global tool ownership resides in
`~/.config/monochromatic-dev/toolchain/plan.tools.data.ts`,
the artifact plan, and its generated lockfile.
The plan already declares the ROCm `gfx110X` llama.cpp bundle at `b1358`.
The repository task `toolchain:sync:global` owns convergence.
The older `doc/runbook/llamacpp-rocm-mise-global.md` describes a retired installation owner.

## Upstream evidence

Current upstream files were retrieved through `gh api` on 2026-10-30.
These are documentation and source observations, not runtime validation.

In `lemonade-sdk/lemonade`:

- `docs/guide/configuration/vllm.md` describes a managed bundle with Python,
  PyTorch, ROCm user-space libraries, Triton, and vLLM.
  It says no system Python, PyTorch, or ROCm installation is required.
  It describes `gfx110X` bundles as available but not exercised end to end.
- `src/cpp/resources/backend_versions.json` maps `gfx1102` to `gfx110X`.
- `docs/guide/install/fedora.md` describes an fc44 RPM and `lemond` system service.
  Fedora instructions alone do not validate Bazzite installation.
- `docs/guide/install/docker.md` describes a container with GPU device passthrough.

The `lemonade-sdk/vllm-rocm` release
`vllm0.20.1-rocm7.12.0-gfx110X` exists.
Its archive parts total 2,771,842,804 bytes before extraction.

Standalone `vllm-project/vllm` documentation omits `gfx1102` from its GPU requirements,
while `CMakeLists.txt` accepts that build target.
The default ROCm container architecture list omits it.
None of these observations proves that packaged inference works on this host.
Standalone vLLM is outside the selected scope.

## Verification boundary

The sandbox exposed GPU sysfs entries but did not expose `/dev/kfd` or `/dev/dri`.
Automatic approval review rejected an escalated host probe consisting of device listing,
identity inspection, and `rocminfo`.
The stated reason was a cgroup membership change in the execution mechanism,
which the reviewer considered an unauthorized security-boundary change.
No bypass was attempted.
The user then explicitly approved this probe,
including the tool runner's cgroup placement.
The approved command succeeded:
device nodes were present and `rocminfo` identified AMD Radeon RX 7600 as `gfx1102`.
This verifies host ROCm enumeration, not model inference.

## Packaged deployment frontier

Release `v2026.44.1` offers these Linux routes:

- Official container:
  the release's `Dockerfile` uses Ubuntu 24.04,
  builds the browser UI,
  and includes `build-essential` for vLLM/Triton runtime launcher compilation.
  The published image is `ghcr.io/lemonade-sdk/lemonade-server`.
  Choosing this route accepts the upstream base instead of the repository's CachyOS default.
- Portable archive:
  `lemonade-embeddable-2026.44.1-ubuntu-x64.tar.gz` includes `lemond`,
  the CLI, and resources.
  The release workflow sets `BUILD_WEB_APP=OFF`.
  This supplies CLI/API access without the browser UI.
  Bazzite compatibility needs a runtime check.
- Fedora RPM:
  `lemonade-server-2026.44.1-fc44.x86_64.rpm`.
  Bazzite's package-layering documentation describes `rpm-ostree`,
  a reboot, and manual updates for local RPM installations.

The user selected the official container route.
This route includes normal runtime JIT compilation;
it does not require building Lemonade or vLLM from source.

The planning document's CommonMark rendering was inspected with Pandoc.
Initial planning commit: `b6ac79dd4d`.

## Next decision frontier

Identify the requested `vllm-d3` project and its fit with the selected packaged deployment.
Network reach and server startup questions remain unanswered.
Resolve model weights and context after establishing the requested engine's requirements.
Then settle storage and resource limits using the selected workload.
Present the concrete installation and verification plan for the grilling session's final confirmation.

## Additional inventory

Authorized read-only host inspection found no Lemonade or vLLM rootless containers or images.
Ports 13305 and 8000 had no listeners.
The user has no existing Quadlet directory at `~/.config/containers/systemd`.
The host has 32 online CPUs;
plain `nproc` reports the runner's allocation of 2 rather than the host inventory.
RAM totals 66,211,774,464 bytes,
with 55,843,090,432 bytes available at the probe instant.
These snapshots do not establish future headroom.

The existing Hugging Face cache contains downloaded GGUF weights for
Qwen3.5-4B, Nemotron 3.5 Bolt E4B, EmbeddingGemma 2, and Qwen3-ASR 0.6B.
Snapshot symlinks were followed to verify actual file sizes.
The cache occupies 11,017,674,752 allocated bytes.
No model integrity or Lemonade/vLLM compatibility claim follows from this inventory.

The release's `server_models.json` includes `Qwen3.5-0.8B-FP16-vLLM`,
checkpoint `Qwen/Qwen3.5-0.8B`, with catalog size 1.77 GB.
That size is metadata, not measured transfer size or VRAM demand.
The custom-model guide routes imported GGUF files through llama.cpp;
testing the existing GGUF cache would not verify vLLM.
No model has been selected or downloaded for this task.

The official image persists configuration, backend cache, and Hugging Face models under `/opt/lemonade`.
The backend installer uses the process temporary directory for archive installation.
The concrete container plan must therefore supply temporary storage outside `/tmp`.
Context and GPU memory controls remain to be chosen and exercised through the installed CLI.
