# Lemonade 2026.44.1 container installation diagnostics on Bazzite

## Tested environment

These are independent findings from the 2026-10-30 installation,
not one common failure.
The host runs Bazzite 44 with an AMD Radeon RX 7600 (`gfx1102`).
Podman is 5.8.9.
The official Lemonade image is `v2026.44.1`,
digest `sha256:8ae593e309c5f8f348ea7404f20e27db83d0d247cc9ba3b2bf7fb6310e13e73c`.
The packaged ROCm llama.cpp bundle is `b1358`,
reporting build `11997`, commit `0ac1a7fd3`.

The confirmed installation and model provenance are recorded in
[the installation plan](../planning/local-rocm-vllm-lemonade.md).
Inference verification uses disposable containers with 2 CPUs and 8 GiB RAM.
No source rebuild or upstream modification was performed.

## A restricted DRM mapping hid the GPU

### Symptom

`llama-server --list-devices` exited successfully but printed:

```text
# llama-server b1358 output
ggml_cuda_init: failed to initialize ROCm: no ROCm-capable device is detected
Available devices:
  (none)
```

The failing container exposed `/dev/kfd` and `/dev/dri/renderD128`.
It used the same packaged runtime that successfully enumerated the RX 7600 on the host.
Exit status alone did not verify accelerator availability.

### Source boundary and limits

The [installed revision's initialization path][ggml-init],
`ggml/src/ggml-cuda/ggml-cuda.cu:250`,
returns an empty device record when device enumeration fails:

```cpp
// ggml/src/ggml-cuda/ggml-cuda.cu:253
cudaError_t err = cudaGetDeviceCount(&info.physical_device_count);
if (err != cudaSuccess) {
    GGML_LOG_ERROR("%s: failed to initialize " GGML_CUDA_NAME ": %s\n", __func__, cudaGetErrorString(err));
    return info;
}
```

This identifies the emitting call and empty result.
It does not identify which additional DRM node the ROCm runtime needed.
No claim about a general driver defect follows from this case.

### Verification and workaround

The working probe was:

```sh
# Disposable device-enumeration probe; no model is loaded.
podman run --rm --memory=2g --cpus=2 --network=none \
  --no-healthcheck --timeout=60 \
  --userns=keep-id:uid=10001,gid=10001 \
  --device=/dev/kfd --device=/dev/dri \
  --security-opt=label=disable --env ROCR_VISIBLE_DEVICES=0 \
  --volume "${HOME}/.local/share/monochromatic-dev/toolchain-global/github-lemonade-sdk-llamacpp-rocm/b1358:/runtime:ro" \
  ghcr.io/lemonade-sdk/lemonade-server:v2026.44.1 \
  /runtime/llama-server --list-devices
```

Working cases:

- The host executable listed the RX 7600 and integrated GPU.
- The container with all of `/dev/dri` listed both GPUs.
- The same full mapping with `ROCR_VISIBLE_DEVICES=0` listed only the RX 7600.

Failing cases:

- Mapping only `renderD128` produced the empty-device diagnostic.
- Removing `ROCR_VISIBLE_DEVICES` from that restricted mapping produced the same diagnostic.

The verified workaround is the documented full DRM mapping,
with the visibility variable selecting the discrete GPU.
This exposes more DRM device nodes to the container;
the visibility variable is a runtime selection, not a device-access security boundary.
SELinux separation was disabled for these specific rootless containers,
without changing host-wide SELinux settings or relabeling the existing runtime bundle.

The diagnostic alone did not justify replacing ROCm or rebuilding llama.cpp.
Those approaches were not attempted.

## A backend configuration change outlasted the CLI connection

### Symptom

The installation command was:

```sh
# Command recorded against the disposable installation server.
podman exec lemonade-install-beqwl2iw lemonade config set \
  rocm_channel=nightly llamacpp.rocm_bin=b1358 llamacpp.backend=rocm \
  extra_models_dir=/storage/models/LFM3-1.7B
```

The CLI returned:

```text
# lemonade config set output
Error setting config: Could not connect to Lemonade server (Failed to read connection).
Make sure the server is running and try again.
```

The server was still running.
Its configuration file contained the requested values,
and its log showed a `b1358` download continuing after the client disconnected.
The backend subsequently appeared as installed.

### Root cause

The [CLI default][client-timeout],
`src/cpp/cli/lemonade_client.cpp:17`, is:

```cpp
// src/cpp/cli/lemonade_client.cpp:17
static const int DEFAULT_READ_TIMEOUT_MS = 30000;
```

The [configuration command][config-client],
`src/cpp/cli/main.cpp:1387`, uses the request overload without a longer timeout:

```cpp
// src/cpp/cli/main.cpp:1387
std::string response = client.make_request(
    "/internal/set", "POST", updates.dump(), "application/json");
```

The [server configuration handler][config-effects],
`src/cpp/server/server.cpp:7294`, applies side effects before producing its response:

```cpp
// src/cpp/server/server.cpp:7294
auto result = config_->set(body, [this](const json& applied) {
    apply_config_side_effects(applied);
});
```

A changed backend binary setting reaches [synchronous installation][bin-install]
in `src/cpp/server/server.cpp:7415`:

```cpp
// src/cpp/server/server.cpp:7415
backend_manager_->install_backend(recipe, backend);
```

The client deadline and the server-side operation therefore have different lifetimes.
The reported connection failure does not establish that the requested configuration was rejected.

### Verification and recovery

The configuration-setting request was the failing case.
The following readbacks were successful after the server completed installation:

```sh
# Readbacks used before any retry of the mutation.
podman exec lemonade-install-beqwl2iw lemonade config
podman exec lemonade-install-beqwl2iw lemonade backends
podman exec lemonade-install-beqwl2iw \
  /storage/cache/bin/llamacpp/rocm-nightly/llama-server --version
```

They reported `rocm_channel=nightly`, `llamacpp.rocm_bin=b1358`,
an installed `llamacpp:rocm` backend,
and runtime build `11997`.
The configuration mutation was not repeated.
The tradeoff is that a caller must inspect server state after this CLI error;
the command's exit status cannot be used as an atomic success/failure signal for this operation.

Upstream pull request `lemonade-sdk/lemonade#2222` was read.
It lengthened timeouts for the streaming pull/install requests,
not the configuration request used here.
That change is not evidence that this configuration path has a long read timeout.

## Consumer verification limits

### Text and decision API checks

The first non-streaming text request used default reasoning,
a 64-token limit, and a 120-second HTTP deadline.
It reached the backend and was cancelled cleanly when the client deadline expired.
The backend slot subsequently reported `is_processing=false`.

The installed model's chat template defaults `enable_thinking` to true.
The same request with `chat_template_kwargs.enable_thinking=false`
returned HTTP 200 and `Liquid text runtime ready.`.
This is a functional check, not a latency benchmark or general reasoning-performance result.

The d1 request returned HTTP 200 with yes/no, single-choice, and scoring answers.
It reported zero generated tokens.
The startup log recorded `offloaded 25/25 layers to GPU`,
and the container exited with code 0 after the test.

### Browser request controls

The browser listed both installed backends and the loaded text model without uncaught page errors.
Turning off its thinking checkbox did not change the LFM3 request's observed `<think>` generation prefix.
That request was cancelled using the browser's Stop control.

The [UI setting helper][ui-thinking],
`src/app/src/renderer/utils/appSettings.ts:329`, builds a top-level override:

```ts
// src/app/src/renderer/utils/appSettings.ts:329
if (!settings.enableThinking.useDefault) {
  overrides.enable_thinking = settings.enableThinking.value;
}
```

This source finding does not establish where the setting ceased to affect this request,
or how other model templates behave.
The verified workaround for this browser test was the explicit per-load llama.cpp argument
`--chat-template-kwargs '{"enable_thinking":false}'`.
The template-only endpoint then returned a prompt ending in `<think></think>`,
and a fresh browser chat displayed `OK.`.
The override was not saved as a model default or startup configuration.

An initial verification assertion incorrectly treated the `/props` display as the CLI prediction limit.
It reported `-1` despite the running process having `--predict 64`.
The [props implementation][props-display],
`tools/server/server-context.cpp:5661`, constructs a fresh task parameter record:

```cpp
// tools/server/server-context.cpp:5661
task_params tparams;
tparams.sampling = params.sampling;
```

The [effective request limit][prediction-limit],
`tools/server/server-context.cpp:2096`, uses the per-request value or the CLI fallback:

```cpp
// tools/server/server-context.cpp:2096
slot.n_predict_max = task.params.n_predict != -1 ? task.params.n_predict : params_base.n_predict;
```

The running browser request subsequently reported `n_predict=64`,
the closed thinking prefix, and an idle slot after completion.
No change to the CLI prediction bound was needed.

### Managed vLLM CLI

The managed vLLM server's `--help` command exited with code 0.
It emitted this warning from `tvm_ffi/_optional_torch_c_dlpack.py:181`:

```text
# Managed vLLM CLI warning
Failed to JIT torch c dlpack extension, EnvTensorAllocator will not be enabled.
We recommend installing via `pip install torch-c-dlpack-ext`
```

No additional package was installed to address that optional extension.
Full vLLM model inference was not exercised;
the selected Liquid models use llama.cpp.
The projector is installed, but image/video decision inputs were not tested.

## Upstream filing decision

Nothing was posted upstream and no fileable issue or patch is proposed.

1.  Attribution:
    the restricted-device case has a verified container-configuration workaround;
    the configuration-request timeout has a source-backed client/server lifetime mismatch.
2.  Fixability:
    device requirements below the observed enumeration call remain untraced;
    a longer-lived configuration operation could address the second case,
    but no patch is asserted.
3.  Supported use:
    the full DRM mapping and backend-version configuration are documented upstream;
    the restricted mapping was not established as a supported configuration.
4.  Contribution policy:
    a contribution-readiness review was not performed because this task installs packaged software.
5.  Upstream direction:
    no conclusion about willingness to accept a change was drawn.
6.  Prototype:
    none was built under the user's packaged-binaries-only constraint.

On 2026-10-30,
the exact-diagnostic GitHub search for `repo:ggml-org/llama.cpp "no ROCm-capable device" in:title,body`
returned issue `20142`.
Its body describes build `8204` on Debian with `gfx1151`,
so it does not establish the cause of this build `11997` container case.
The search `repo:lemonade-sdk/lemonade config timeout in:title,body`
returned 45 results, including pull request `2222`.
The request type in that pull request was checked against the installed source.
No tracker result was treated as proof that a different failure was fixed here.

The source clones were read at the cited commits without modifying upstream code.
The cited source selections were checked in `agent-browser`,
including GitHub's code-editor text for virtualized off-screen lines.

[ggml-init]: https://github.com/ggml-org/llama.cpp/blob/0ac1a7fd35c127a18c0cbc6c57d12e256a6a385b/ggml/src/ggml-cuda/ggml-cuda.cu#L250-L257
[client-timeout]: https://github.com/lemonade-sdk/lemonade/blob/69948a7d40afbf91f6ad990d8b863c83ce5cc71d/src/cpp/cli/lemonade_client.cpp#L15-L18
[config-client]: https://github.com/lemonade-sdk/lemonade/blob/69948a7d40afbf91f6ad990d8b863c83ce5cc71d/src/cpp/cli/main.cpp#L1387-L1389
[config-effects]: https://github.com/lemonade-sdk/lemonade/blob/69948a7d40afbf91f6ad990d8b863c83ce5cc71d/src/cpp/server/server.cpp#L7291-L7304
[bin-install]: https://github.com/lemonade-sdk/lemonade/blob/69948a7d40afbf91f6ad990d8b863c83ce5cc71d/src/cpp/server/server.cpp#L7411-L7418
[ui-thinking]: https://github.com/lemonade-sdk/lemonade/blob/69948a7d40afbf91f6ad990d8b863c83ce5cc71d/src/app/src/renderer/utils/appSettings.ts#L329-L331
[props-display]: https://github.com/ggml-org/llama.cpp/blob/0ac1a7fd35c127a18c0cbc6c57d12e256a6a385b/tools/server/server-context.cpp#L5661-L5666
[prediction-limit]: https://github.com/ggml-org/llama.cpp/blob/0ac1a7fd35c127a18c0cbc6c57d12e256a6a385b/tools/server/server-context.cpp#L2095-L2098
