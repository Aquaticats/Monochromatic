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
- Defer startup configuration.
  The user said, "Startup preference: Don't set it up yet."

Installation has not started.
Model selection, service behavior, resource limits,
and final shared-understanding confirmation remain open.

In response to the workload question, the user specified `vllm-d3`,
then confirmed the intended model is Hugging Face `vllm-sr/d3`,
from vLLM Semantic Router's Decision 3.0 family.
The previously selected Lemonade-managed vLLM route must be reassessed against this exact model.
Upstream describes a separate `vllm-srun` runtime;
generic vLLM installation does not establish D3 execution support.
The full model was the target at that point in the discussion.

The user subsequently requested Liquid AI's new models,
specifically the smallest text and smallest decision models released "this week".
The working date window is October 26 to 30, 2026.
The full D3 model is therefore no longer a fixed deployment target.
Identify the new releases by parameter count and verify their original release dates,
distinguishing release dates from model-card updates.
The user explicitly confirmed that both models must be Liquid AI's own releases.

The user's network answer was "fine", without naming a network scope.
The assistant stated that temporary verification servers would use localhost only.
No persistent startup or network configuration is authorized by that interpretation.

## Selected Liquid models

Liquid AI's [October 27 launch][liquid-launch] identifies the requested models:

- [LFM3-1.7B][liquid-text]: 1.73 billion parameters, post-trained text generation.
- [d1-2B][liquid-decision]: 1.92 billion parameters, structured decisions.

These are the smallest respective models in that week's release.
The older `d1-omni-600M` announcement is dated October 7 and falls outside the requested week.
Repository creation and model-card modification dates were not substituted for the public launch date.
The launch date and both parameter counts were verified in the rendered model pages.

Both official GGUF cards require llama.cpp `b11960` (`v0.8.0`) or newer.
The installed global `b1358` bundle was probed directly:
`llama-server --version` reported `0.8.0-dev`, build `11997`, commit `0ac1a7fd3`.
Both version and help probes exited successfully without loading a model or starting a server.
This establishes the installed version, not inference success.

The [decision GGUF card][liquid-decision-gguf] documents the `/v1/systemone` endpoint.
Its packaged llama.cpp path supports yes/no decisions, single-choice selection, and scoring.
Multiple-choice selection, extraction, estimates, and bounding boxes are not supported there yet.
The card's runtime requirement and supported output types were checked in the rendered page.

Official artifact metadata reports these Q8_0 file sizes:

- Text weights: 1,846,238,496 bytes.
- Decision weights: 1,846,240,576 bytes.
- Optional decision vision projector: 203,031,584 bytes in Q8_0,
  or 378,995,744 bytes in F16.

File sizes do not establish VRAM requirements.
Both model cards use LFM Open License v1.0.

## Concrete arrangement awaiting confirmation

Retain the working host ROCm installation.
Install the selected official Lemonade container and its managed vLLM backend,
preserving the original software request.
Select packaged llama.cpp ROCm build `b1358` for the Liquid text model.
Lemonade's released configuration guide documents selecting that build through
`rocm_channel=nightly` and `llamacpp.rocm_bin=b1358`.
Confirm these settings against the installed CLI before applying them.
Its default backend pins predate the Liquid models' minimum version.

Use the existing `b1358` runtime directly for d1's decision API.
Lemonade `v2026.44.1` does not register `/v1/systemone`,
and its classifier interface is not implemented by the llama.cpp backend.
Do not add a custom proxy or claim that Lemonade's chat API serves d1 decisions.

Proposed initial verification uses Q8_0 weights and text inputs,
one model loaded at a time,
with a 2,048-token context and at most 64 generated text tokens.
Use disposable containers capped at 2 CPUs and 8 GiB RAM for inference checks.
These are verification limits, not a claimed performance optimum.
Keep downloaded models and managed backends in dedicated Lemonade storage under the user's home,
shared by the temporary containers without duplicating model weights.
Use dedicated disk-backed temporary storage outside `/tmp`.
Bind temporary servers to localhost and stop them after verification.
Do not configure startup services or broader network access.
The user must confirm this arrangement and the decision-output limitation before installation begins.

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
Standalone generic vLLM was outside the originally selected scope.
The later D3 clarification reopens the runtime dependency question.

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

Confirm the concrete Liquid-model arrangement and the packaged decision-output scope.
Startup configuration is deferred.
Use localhost only for transient verification;
broader network exposure remains unselected.
Verify the proposed context, storage, and resource settings against installed CLI capabilities before execution.
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

## Full D3 investigation

Before the user requested the smallest new models,
upstream Semantic Router was inspected at commit `6aeaf5d415532771ec77d26c25ebe0a7b64da0ed`.
Its `src/model-runtime/README.md` specifies a separate `vllm-srun` runtime,
distributed in Semantic Router images.
Its model registry records 26,086,635,760 loaded parameters for full `vllm-sr/d3`.

The [upstream memory record][d3-memory]
reports 48.59 GiB of BF16 weights and 50.28 GiB after loading and model-card requests.
These are upstream MI325X measurements, not measurements on this host.
The citation and its quoted numeric values were verified in an isolated browser session,
which was then closed.

The inspected runtime has a BF16 CPU execution path.
Its placement calculation estimates 59,538,082,320 bytes including reserve for full D3,
exceeding the earlier available-RAM snapshot of 55,843,090,432 bytes.
Local CPU fit and performance were not tested.
The published ROCm image's architecture lists omit `gfx1102`;
the inspected native engine places model stacks on one device rather than splitting weights across CPU and GPU.
No model weights were downloaded and no D3 inference was attempted.

[d3-memory]: https://github.com/vllm-project/semantic-router/blob/6aeaf5d415532771ec77d26c25ebe0a7b64da0ed/src/model-runtime/docs/records/decision3-device-memory.md#L24-L39
[liquid-launch]: https://huggingface.co/blog/LiquidAI/lfm3#:~:text=Published%20October%2027%2C%202026
[liquid-text]: https://huggingface.co/LiquidAI/LFM3-1.7B#:~:text=1.73B
[liquid-decision]: https://huggingface.co/LiquidAI/d1-2B#:~:text=1.92B
[liquid-decision-gguf]: https://huggingface.co/LiquidAI/d1-2B-GGUF#:~:text=llama.cpp
