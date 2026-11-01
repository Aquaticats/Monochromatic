# Xiaomi-OCR-0 on the CPU without Python we write

Research for issue 2576,
answering the owner's question of 2026-11-01:
"What is going on?
Why are we writing any Python?"
All web sources were fetched on 2026-11-01.
Nothing was installed,
built,
run or downloaded beyond JSON,
config,
Markdown and source text;
no model weights were fetched,
so no route in this note was run end to end.
Statements that begin with "Inference" follow from the cited text but were not executed.

## Answer

The wrapper was proposed because the Xiaomi repository ships no installable Python package.
That is true,
but it only matters if the Transformers route is the only route.
It is not.
The checkpoint's architecture is the Qwen3.5 family,
which llama.cpp converts and runs,
community GGUF files of this exact model are published,
and the repository's own toolchain plan already installs a CPU llama.cpp build through the `archive` route.
Inference:
a global CPU-only command with no Python written by us is reachable with a plan entry and no mpm code.
The costs are community-converted weights,
an unmeasured CPU speed,
and no `xiaomi-ocr` verb
(the command is `llama-cli` with flags).

## Why the Python wrapper was proposed

- The upstream tree has 143 entries at commit `b1a9bb4c07060c453785e9151715ccda6b96f0b1`
  ([tree][xm-tree]).
  A search of the entry list for `pyproject`,
  `setup.py`,
  `setup.cfg`,
  any `.toml`,
  `__main__` or `console_scripts`
  found nothing,
  and the same search for `requirements` found three entries,
  so the search can match.
- None of seven guessed PyPI names exists (HTTP 404):
  `xiaomi-ocr`,
  `xiaomi-ocr-0`,
  `xiaomi_ocr`,
  `xiaomi-ocr0`,
  `xiaomiocr`,
  `seerray`,
  `xiaomi-ocr-mcp`.
  PyPI offers no name-search API,
  so unguessed names were not checked.
- uv cannot build a directory like that:
  its build frontend fails with
  "does not appear to be a Python project,
  as neither `pyproject.toml` nor `setup.py` are present in the directory"
  ([uv error][uv-notproject]).
- The troubleshooting note `doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md` left the alternatives open:
  its section "Limits and next verification" lists "the Ollama/llama.cpp alternatives" among what "were not measured".
  The design proposal for the issue then took the Transformers route as given.

## Facts about the model and the toolchain

- The checkpoint declares `Qwen3_5ForConditionalGeneration` ([config][hf-config]);
  its `model_type` is `qwen3_5`,
  with `qwen3_5_text` and `qwen3_5_vision` sub-configs
  (read from the same file at revision `bbd8cf8d21f256b3644b44cb860f3a7cecb93c30`).
- The weights file has the same Git LFS object id at the revision the community conversions started from
  (`e4d1c4a6804bd9ef342b93d705a73af003e2ef4e`) and at the revision the CPU recipe verified:
  `d4551e9f6127ac9e35cff7b4f7a72a90410e03e6c8676a36f0e2b7eaeb5d5348`,
  1746937856 bytes
  ([tree at e4d1c4a6][hf-tree-old],
  [tree at bbd8cf8d][hf-tree-new]).
  The titles of the commits after `e4d1c4a6` concern only the model card and evaluation results ([commit list][hf-commits]).
- The repository plan pins `llama.cpp` to `github:ggml-org/llama.cpp@b9829` on the `archive` route
  (`toolchain.plan.jsonc`,
  entry `llama.cpp`),
  and its lock resolves `linux-x64` to `llama-b9829-bin-ubuntu-x64.tar.gz`
  (`toolchain.lock.json`).
  The same release publishes separate `vulkan`,
  `rocm`,
  `sycl` and `openvino` Linux assets ([b9829 assets][ll-b9829]),
  so the plain `ubuntu-x64` asset is the CPU build (Inference from the asset names).
- The machine-global plan does not contain that entry.
  Its only llama.cpp entry is `github:lemonade-sdk/llamacpp-rocm`,
  locked at `b1360`
  as `llama-b1360-ubuntu-rocm-gfx110X-x64.zip`,
  a ROCm GPU build for AMD gfx110X.
  The global plan also holds `pipx:huggingface_hub`,
  which exposes `hf` (checked:
  `hf download --help` lists `--revision`).
- The plan comment on `llama.cpp` says it is pinned because "llama.cpp tags before the Linux build finishes uploading".
  A second hazard for `@latest`:
  the release GitHub marks latest is `v0.8.0` (published 2026-10-28) with one asset,
  `nightly-tag.txt`,
  while the twelve newest `bNNNNN` builds listed are marked prerelease ([release list][ll-releases]).

## Route 1: llama.cpp with a GGUF

- At tag `b9829` the text converter registers `Qwen3_5ForConditionalGeneration` ([text][ll-conv-text]),
  and the vision-projector converter registers it too ([mmproj][ll-conv-vision]).
  The runtime has the architecture name `qwen35` ([arch table][ll-arch]).
  The Xiaomi guide says the same:
  "Use llama.cpp's `Qwen3_5ForConditionalGeneration` converter" ([OLLAMA.md][xm-ollama-conv]).
- GGUF files of this exact model are published.
  `prithivMLmods/Xiaomi-OCR-0-GGUF` holds BF16 (1516744960 bytes),
  Q3_K_L,
  Q3_K_M,
  Q4_K_M (529297664 bytes),
  Q4_K_S,
  Q5_K_M,
  Q5_K_S and Q6_K text models,
  plus `Xiaomi-OCR-0.mmproj-bf16.gguf` (207346624 bytes) ([file list][hf-prithiv-tree]).
  The Hugging Face API reports its GGUF architecture as `qwen35` and the license as Apache 2.0 ([model API][hf-prithiv-api]).
  Other GGUF repositories exist (`richardyoung`,
  `holooo`,
  `argus-ai` on Hugging Face,
  `xzl01` on GitHub) ([search][hf-search]).
- Xiaomi's own README names two of them and says
  "We have not evaluated these community GGUF conversions" ([README][xm-readme-unevaluated]).
  No GGUF is published under the `xiaomi-research` namespace ([search][hf-search]).
- The prithivMLmods model card names no converter version or source revision ([card][hf-prithiv-card]).
  The `xzl01` repository does:
  llama.cpp `b10568`,
  source revision `e4d1c4a6` ([README][xz-readme-build]).
- The converter is not in the release archive.
  `convert_hf_to_gguf.py` at `b9829` imports the `conversion` package from the source tree,
  and `xzl01` states that "a shallow clone of the matching tag is the reliable route"
  because fetching the single script file is not enough,
  and lists `torch`,
  `transformers` and `gguf` as requirements ([README][xz-readme-clone]).
  The `gguf` PyPI package installs only `gguf-convert-endian`,
  `gguf-dump`,
  `gguf-set-metadata`,
  `gguf-new-metadata` and
  `gguf-editor-gui` ([pyproject][ll-gguf-scripts]).
  The plan's llama.cpp bins include `llama-quantize` and `llama-gguf-split`,
  which act on GGUF files,
  not on checkpoints.
  Own conversion is therefore a one-off use of llama.cpp's Python converter,
  not code we maintain.
- The command line already exists in the tools the plan installs.
  At `b9829`,
  `llama-cli` accepts `--image` ([arg.cpp][ll-arg-image]),
  `--mmproj` ([arg.cpp][ll-arg-mmproj]),
  `--single-turn` ([arg.cpp][ll-arg-single]),
  `-hf <user>/<model>[:quant]` with "mmproj is also downloaded automatically if available" ([arg.cpp][ll-arg-hf]),
  and `--reasoning`,
  whose option list is server,
  completion and cli,
  not mtmd ([arg.cpp][ll-arg-reasoning]).
  `llama-mtmd-cli` prints "Usage:
  %s [options] -m <model> --mmproj <mmproj> --image <image> --audio <audio> -p <prompt>"
  and runs single-turn when both a prompt and an image are given ([mtmd-cli.cpp][ll-mtmd-usage]).
  The `-hf` mmproj pick chooses a sibling GGUF whose file name contains `mmproj` ([download.cpp][ll-download-mmproj]).
- The checkpoint's chat template emits an empty think block unless `enable_thinking` is defined and true
  ([template][hf-template]),
  so non-thinking is the template default.
  Xiaomi's MCP server sends `temperature` 0 and `chat_template_kwargs` with `enable_thinking` false ([server][xm-mcp-chat]).
- Quality evidence is thin.
  Xiaomi reports its own BF16 GGUF import matched 7 of 7 input token counts and 5 of 7 raw responses,
  with formula differences in the other two,
  on Apple silicon with Ollama 0.35.0 ([OLLAMA.md][xm-ollama-quality]).
  `xzl01` reports character similarity of 96.6% to 100% between f16 and Q4_K_M on four cases,
  measured against its own f16 output,
  not against the Transformers output ([README][xz-readme-accuracy]).
  Nobody published a comparison of a GGUF against the native Transformers FP32 output.
  Xiaomi also says pulling a model alone "does not configure" its image preprocessing,
  prompt and decoding settings
  ([README][xm-readme-pull]).
- Size and speed.
  Q4_K_M plus projector is about 736 MB of weights,
  against a 1746937856-byte checkpoint
  that needs about 3.25 GiB as FP32 parameters (`doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md`).
  The only llama.cpp timings found (4.6 s to 16.5 s per case,
  1507 MB resident for Q4_K_M)
  were taken on an Intel Arc iGPU through Vulkan with all layers offloaded ([hardware][xz-readme-hw]),
  so they are not CPU figures.
  No CPU timing of this model under llama.cpp exists in the sources found.
  Indirect evidence only:
  a llama.cpp issue about a different Qwen3.5-VL model reports correct vision output on the CPU backend
  ([issue 31104][ll-issue-cpu]).

Pros:
no Python written or run;
an existing mpm route and an existing plan entry;
small weights;
the runtime is already pinned and locked.
Cons:
community-converted weights of unknown fidelity;
no measured CPU speed;
pinned runtime `b9829` is older than the converters the community used,
so GGUF compatibility is unverified;
no fixed-flag `xiaomi-ocr` verb.

Inference:
a one-shot invocation would look like this.
It was not run.

```sh
# Proposed one-shot OCR with the CPU build; not run. The two GGUF files come from `hf download --revision`.
llama-cpu-cli --model Xiaomi-OCR-0.Q4_K_M.gguf --mmproj Xiaomi-OCR-0.mmproj-bf16.gguf \
  --image page.png --reasoning off --temp 0 --single-turn --prompt "Extract the text in the image."
```

## Route 2: Transformers' own command line

- The current release is 5.22.1 (uploaded 2026-10-30);
  its commands are `add-new-model-like`,
  `chat`,
  `download`,
  `env`,
  `serve` and `version` ([entry point][tf-cli-main]).
  There is no `run` command and no OCR or pipeline command.
- `transformers chat` is a text client:
  it takes a model id and the base URL of a running server ([chat.py][tf-chat-args]).
  The file contains no mention of images,
  vision or pixels (searched for `image`,
  `vision`,
  `multimodal`,
  `vlm`,
  `pixel`;
  no match).
- `transformers serve` can run it.
  It has `--device` ("e.g. 'auto',
  'cuda:0',
  'cpu'"),
  `--dtype` and `--reasoning` ([serve.py][tf-serve-opts]),
  converts `image_url` message parts into processor images for vision-language models ([utils.py][tf-serve-image]),
  and loads the class named in the checkpoint's `architectures` ([model_manager.py][tf-serve-load]).
  It is a long-running server;
  a one-shot OCR needs a separate HTTP client.
- `serve` refuses to start without the serving extra:
  "Missing dependencies for serving.
  Install with `pip install transformers[serving]`" ([serve.py][tf-serve-missing]).
  Torch is not a base requirement;
  it belongs to the `torch` and `serving` extras ([PyPI metadata][pypi-tf]).
- uv's guide says the default Linux torch wheel targets CUDA ("GPU-accelerated wheels on Linux"),
  and that "`--torch-backend` is only available in the `uv pip` interface"
  ([guide][uv-torch-default],
  [guide][uv-torch-backend]).
  The design research for this issue found that the uv 0.13.6 source applies the flag to tool installs.
- Today's `mpm-uvx` cannot express any of that.
  `install_mpm_uvx` runs `uv tool install --force <package>==<version>` and nothing else
  (`install_managers.rs` lines 105 to 106,
  [file][mpm-uvx]).
  A package name may contain only letters,
  digits and `.-_@/`,
  so `transformers[serving]` is refused
  (`update_registry.rs` lines 33 to 35,
  [file][mpm-name]).
  The plan has no field for `--with`,
  `--python` or `--torch-backend`.
  Inference:
  `pipx:transformers` would install without torch and `serve` would stop on the "Missing dependencies" error.

Pros:
runs the original checkpoint;
the CPU recipe was measured (51.206 s for 24 output tokens from 517 input tokens at two CPUs,
of which 41.107 s was loading);
no conversion.
Cons:
needs the full provider work from the issue (extras or `--with`,
CPU torch backend,
drift detection) minus the wrapper;
the result is a server plus a client,
not a command;
torch stack and FP32 memory.

## Route 3: Xiaomi's repository as it is

- No packaging metadata (section "Why the Python wrapper was proposed"),
  so `uv tool install` or `uvx` of the
  repository cannot build it.
- The MCP server `skills/xiaomi-ocr/mcp_ocr_server.py` imports `postprocess.*` from the clone root (`XIAOMI_OCR_REPO` or two
  directories up),
  talks only to an OpenAI-compatible server at `XIAOMI_OCR_LOCAL_URL` (default `http://127.0.0.1:8000/v1`),
  and refuses a non-loopback address ([server][xm-mcp-env]).
  It is a stdio tool server for agents,
  not a shell command.
- The evaluation script `evaluation_pipeline/e2e_img2md.py` requires `--input-jsonl` and `--output_dir`,
  starts `vllm serve` with GPU device options,
  and imports `evaluation_pipeline` and `postprocess` from the clone ([script][xm-e2e]).
- `requirements.txt` lists `openai`,
  `Pillow`,
  `pypdfium2`,
  `numpy`,
  `tqdm` and `wordfreq` ([requirements][xm-requirements]).
  The browser demo needs a running inference server too ([README][xm-readme-demo]).

None of these is a CPU inference command.
Each needs a separate inference server,
which Route 1 can supply through `llama-server`,
an OpenAI-compatible endpoint.
Inference:
that pairing would add OTSL conversion and PDF handling later,
at the cost of running Xiaomi's Python from a clone.

## Route 4: other runtimes

### Ollama

- A community BF16 model exists:
  `longwayxu/xiaomi-ocr-0:bf16`,
  1.8 GB (1.6 GB model,
  207 MB projector),
  architecture `qwen35` ([page][ollama-model]).
- Ollama documents image input on the command line as `ollama run gemma4 ./image.png what is in this image?` ([docs][ollama-vision]).
  Xiaomi requires "a Qwen3.5-compatible Ollama (tested with 0.35.0)" ([OLLAMA.md][xm-ollama-version]);
  the latest release is v0.40.13.
- Linux assets are `.tar.zst` files,
  1.5 GB for `ollama-linux-amd64.tar.zst` ([release][ollama-release]).
  The `archive` route unpacks only `.tar.gz`,
  `.tgz`,
  `.tar.xz`,
  `.tar.bz2`,
  `.tar` and `.zip`
  (`install_support.rs` line 44,
  [file][mpm-suffixes]);
  any other name is copied as one bare file (`install_archive.rs` lines 146 and 161 to 171,
  [file][mpm-bare]).
  mpm-shadow already runs `tar --zstd` for `.conda` payloads (`install_support.rs` line 217,
  [file][mpm-zstd]),
  so the gap is small but real.
- It needs a running daemon,
  which mpm does not manage.
  Xiaomi says the Ollama path gets its preprocessing and prompt from the repository's Python MCP or demo.
  `doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md` records that the claimed `XIAOMI_OCR_BACKEND=ollama` switch is absent
  from the inspected source.

### vLLM on the CPU

- Documented for x86 and Arm:
  "Pre-built vLLM wheels for x86 with AVX512/AVX2 are available since version 0.17.0",
  installed from GitHub release URLs or the `wheels.vllm.ai` index,
  with a `vllm/vllm-openai-cpu` container image ([docs][vllm-cpu]).
- That is not a `pipx:` request,
  `vllm serve` is a server,
  and no source found says this checkpoint runs on vLLM's CPU backend.
  Xiaomi's README points to a vLLM GPU guide ([README][xm-readme-vllm]).

### LM Studio

- `lms` "ships with LM Studio" and needs LM Studio to be run "at least once" ([docs][lms-cli]).
  It is bundled with a desktop application,
  not a release archive,
  so no mpm route installs it.

### Optimum and ONNX

- `optimum-intel` can export `qwen3_5` to OpenVINO (it is in `MULTI_MODAL_TEXT_GENERATION_MODELS`,
  [utils.py][opt-intel-list]),
  but `optimum-cli` has only `export`,
  `env` and `register` command groups ([commands][opt-intel-cmds]);
  running the result is Python API use.
- `optimum-onnx` lists Qwen3 and Qwen3-MoE as supported and no Qwen3.5 entry ([overview][opt-onnx-list];
  a search of that file
  for `qwen` finds only the three Qwen lines).
  A `gh search code` query for `qwen3_5` in the repository returned no result,
  while the control query `qwen3` returned its
  exporter configs;
  code search can lag,
  so treat this as absence of evidence.
- No ONNX export of this model appears in the Hugging Face search ([search][hf-search]).
  MLX and NInfer conversions exist there;
  neither was evaluated.

## Ranking

Ranking by how well each meets "a global command,
CPU-only,
through mpm,
with no Python we maintain":
Route 1 > Route 2 > Ollama > vLLM CPU > Route 3,
LM Studio and Optimum,
because of the reasons for each adjacent pair that follow.
The last three are not ordered among themselves,
since each fails on its own deciding fact.

- Route 1 over Route 2:
  Route 1 needs a plan entry and no mpm code,
  and yields a one-shot command.
  Route 2 needs the provider work and yields a server.
  Route 2 is ahead on one axis only:
  it runs the original weights,
  with a measured CPU result,
  so Route 1 wins only if community-converted weights are acceptable.
- Route 2 over Ollama:
  both need mpm changes,
  but Route 2's are the ones the issue already scoped,
  and it runs the unconverted checkpoint.
  Ollama also needs a daemon,
  a 1.5 GB archive format the route cannot unpack,
  and an unmeasured CPU run of a community conversion.
- Ollama over vLLM CPU:
  Ollama has a documented one-line image command and a published model;
  vLLM CPU has no cited support for this checkpoint and installs from off-PyPI wheels.
- vLLM CPU over the last group:
  it at least has an install route and a CPU backend in the documentation.
  Xiaomi's repository has no package,
  the MCP server is not a command,
  `lms` ships with a desktop app,
  and Optimum only exports.

## What mpm needs for Route 1

- A plan entry in the machine-global plan,
  copied from the repository's `llama.cpp` entry,
  under a bin name that does not collide.
  The global plan already declares `llama-cli` and the other `llama-*` names for the ROCm build,
  and `bin_paths_output` refuses a name declared twice (`bin_paths.rs` lines 85 to 92,
  [file][mpm-dup]).
  `binAliases` maps an exposed name to a real executable (`exposure.rs` lines 104 to 122,
  [file][mpm-alias]),
  the same shape as the `pnpm` entry in the test fixtures (`fixtures.rs` lines 44 to 45,
  [file][mpm-fixture]).
  Inference,
  untested:

  ```jsonc
  // Machine-global toolchain.plan.jsonc; proposal, not applied.
  "llama.cpp-cpu": {
    "request": "github:ggml-org/llama.cpp@b9829",
    "route": "archive",
    "bins": ["llama-cpu-cli"],
    "options": { "binAliases": { "llama-cpu-cli": "llama-cli" } }
  }
  ```

- Then the standard lifecycle:
  `update` for that id,
  `install --global`.
  No change to mpm-shadow code,
  no uv options,
  no source origin and no wrapper package.
- Weights are not an mpm tool on any route.
  `llama-cli -hf` fetches unpinned from the repository's default branch.
  The pinned alternative is `hf download <repo> <files> --revision <sha> --local-dir <dir>` through the already-global `hf`.
  For `prithivMLmods/Xiaomi-OCR-0-GGUF` the revision read on 2026-11-01 was `7bbfa072e0f3d70ac722e26fbccef5b9f64b7121`.
  Whether `-hf repo:BF16` matches `Xiaomi-OCR-0.BF16.gguf` was not checked.
- The issue's rule that an alias is not a managed install applies to a `xiaomi-ocr` verb.
  mpm links binaries as they are (`exposure.rs`,
  [file][mpm-alias]) and the repository bans shell wrapper scripts,
  so a fixed-flag verb needs an owner decision:
  a documented command line,
  or a wrapper we would write.

## Unverified, and how to settle it

- Whether `b9829` loads a GGUF converted by a newer llama.cpp (`xzl01` used `b10568`).
  Settle it by a capped container run (`--cpus 2`) of `llama-cli` on the sample image,
  or by choosing a newer pinned tag with an `ubuntu-x64` asset.
- The CPU speed and memory of Route 1,
  and its output against the Transformers output on the sample image.
- That the plain `ubuntu-x64` asset has no GPU backend inside (inferred from asset names only).
- Whether `llama-mtmd-cli` would honour the template's non-thinking default (depends on the GGUF embedding the template).
- Whether the global exposure farm accepts the alias entry as written.

## Sources

[xm-tree]: https://api.github.com/repos/xiaomi-research/Xiaomi-OCR-0/git/trees/b1a9bb4c07060c453785e9151715ccda6b96f0b1?recursive=1
[uv-notproject]: https://github.com/astral-sh/uv/blob/0.13.6/crates/uv-build-frontend/src/error.rs#L25
[hf-config]: https://huggingface.co/xiaomi-research/Xiaomi-OCR-0/raw/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30/config.json#:~:text=Qwen3_5ForConditionalGeneration
[hf-tree-old]: https://huggingface.co/api/models/xiaomi-research/Xiaomi-OCR-0/tree/e4d1c4a6804bd9ef342b93d705a73af003e2ef4e
[hf-tree-new]: https://huggingface.co/api/models/xiaomi-research/Xiaomi-OCR-0/tree/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30
[hf-commits]: https://huggingface.co/api/models/xiaomi-research/Xiaomi-OCR-0/commits/main
[ll-b9829]: https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/b9829
[ll-releases]: https://api.github.com/repos/ggml-org/llama.cpp/releases?per_page=12
[ll-conv-text]: https://github.com/ggml-org/llama.cpp/blob/b9829/conversion/qwen.py#L620
[ll-conv-vision]: https://github.com/ggml-org/llama.cpp/blob/b9829/conversion/qwen3vl.py#L16
[ll-arch]: https://github.com/ggml-org/llama.cpp/blob/b9829/src/llama-arch.cpp#L41
[xm-ollama-conv]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/OLLAMA.md#:~:text=Qwen3_5ForConditionalGeneration
[hf-prithiv-tree]: https://huggingface.co/api/models/prithivMLmods/Xiaomi-OCR-0-GGUF/tree/main
[hf-prithiv-api]: https://huggingface.co/api/models/prithivMLmods/Xiaomi-OCR-0-GGUF
[hf-prithiv-card]: https://huggingface.co/prithivMLmods/Xiaomi-OCR-0-GGUF/raw/main/README.md
[hf-search]: https://huggingface.co/api/models?search=Xiaomi-OCR-0&limit=100
[xm-readme-unevaluated]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/README.md#:~:text=We%20have%20not%20evaluated%20these%20community%20GGUF%20conversions
[xz-readme-build]: https://raw.githubusercontent.com/xzl01/Xiaomi-OCR-0-GGUF/3737ac44c6036f2e66f6e515c2632f8995770285/README.md#:~:text=Converted%20with%20llama.cpp
[xz-readme-clone]: https://raw.githubusercontent.com/xzl01/Xiaomi-OCR-0-GGUF/3737ac44c6036f2e66f6e515c2632f8995770285/README.md#:~:text=clone%20of%20the%20matching%20tag%20is%20the%20reliable%20route
[xz-readme-accuracy]: https://raw.githubusercontent.com/xzl01/Xiaomi-OCR-0-GGUF/3737ac44c6036f2e66f6e515c2632f8995770285/README.md#:~:text=character%2Dlevel%20similarity%20vs%20f16
[xz-readme-hw]: https://raw.githubusercontent.com/xzl01/Xiaomi-OCR-0-GGUF/3737ac44c6036f2e66f6e515c2632f8995770285/README.md#:~:text=Intel%20Arc%20iGPU%20via%20Vulkan
[ll-gguf-scripts]: https://github.com/ggml-org/llama.cpp/blob/b9829/gguf-py/pyproject.toml#L21-L26
[ll-arg-image]: https://github.com/ggml-org/llama.cpp/blob/b9829/common/arg.cpp#L2344-L2351
[ll-arg-single]: https://github.com/ggml-org/llama.cpp/blob/b9829/common/arg.cpp#L1661-L1669
[ll-arg-mmproj]: https://github.com/ggml-org/llama.cpp/blob/b9829/common/arg.cpp#L2313-L2319
[ll-arg-hf]: https://github.com/ggml-org/llama.cpp/blob/b9829/common/arg.cpp#L2750-L2759
[ll-arg-reasoning]: https://github.com/ggml-org/llama.cpp/blob/b9829/common/arg.cpp#L3266-L3283
[ll-mtmd-usage]: https://github.com/ggml-org/llama.cpp/blob/b9829/tools/mtmd/mtmd-cli.cpp#L43-L46
[ll-download-mmproj]: https://github.com/ggml-org/llama.cpp/blob/b9829/common/download.cpp#L570-L616
[hf-template]: https://huggingface.co/xiaomi-research/Xiaomi-OCR-0/raw/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30/chat_template.jinja#:~:text=enable_thinking%20is%20defined
[xm-mcp-chat]: https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/skills/xiaomi-ocr/mcp_ocr_server.py#L96-L97
[xm-ollama-quality]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/OLLAMA.md#:~:text=matched%20HF%20BF16%2FMPS%20on%20all%207%20input%20token%20counts
[xm-readme-pull]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/README.md#:~:text=pulling%20the%20model%20alone%20does%20not%20configure%20these%20steps
[ll-issue-cpu]: https://github.com/ggml-org/llama.cpp/issues/31104
[tf-cli-main]: https://github.com/huggingface/transformers/blob/v5.22.1/src/transformers/cli/transformers.py#L25-L32
[tf-chat-args]: https://github.com/huggingface/transformers/blob/v5.22.1/src/transformers/cli/chat.py#L314-L319
[tf-serve-opts]: https://github.com/huggingface/transformers/blob/v5.22.1/src/transformers/cli/serve.py#L65-L89
[tf-serve-image]: https://github.com/huggingface/transformers/blob/v5.22.1/src/transformers/cli/serving/utils.py#L1243-L1248
[tf-serve-load]: https://github.com/huggingface/transformers/blob/v5.22.1/src/transformers/cli/serving/model_manager.py#L292-L293
[tf-serve-missing]: https://github.com/huggingface/transformers/blob/v5.22.1/src/transformers/cli/serve.py#L120-L121
[pypi-tf]: https://pypi.org/pypi/transformers/5.22.1/json
[uv-torch-default]: https://docs.astral.sh/uv/guides/integration/pytorch/#:~:text=GPU%2Daccelerated%20wheels%20on%20Linux
[uv-torch-backend]: https://docs.astral.sh/uv/guides/integration/pytorch/#:~:text=only%20available%20in%20the%20uv%20pip%20interface
[mpm-uvx]: ../../package/mpm-shadow/cli/src/toolchain/install_managers.rs
[mpm-name]: ../../package/mpm-shadow/cli/src/toolchain/update_registry.rs
[xm-mcp-env]: https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/skills/xiaomi-ocr/mcp_ocr_server.py#L26-L36
[xm-e2e]: https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/evaluation_pipeline/e2e_img2md.py#L424-L447
[xm-requirements]: https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/requirements.txt
[xm-readme-demo]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/README.md#:~:text=The%20demo%20sends%20requests%20to%20a%20local%20SGLang%20or%20vLLM%20inference%20server
[ollama-model]: https://ollama.com/longwayxu/xiaomi-ocr-0:bf16
[ollama-vision]: https://docs.ollama.com/capabilities/vision#:~:text=ollama%20run%20gemma4
[xm-ollama-version]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/OLLAMA.md#:~:text=Install%20a%20Qwen3.5%2Dcompatible%20Ollama
[ollama-release]: https://api.github.com/repos/ollama/ollama/releases/latest
[mpm-suffixes]: ../../package/mpm-shadow/cli/src/toolchain/install_support.rs
[mpm-bare]: ../../package/mpm-shadow/cli/src/toolchain/install_archive.rs
[mpm-zstd]: ../../package/mpm-shadow/cli/src/toolchain/install_support.rs
[vllm-cpu]: https://docs.vllm.ai/en/latest/getting_started/installation/cpu/#:~:text=Pre%2Dbuilt%20vLLM%20wheels%20for%20x86%20with%20AVX512%2FAVX2%20are%20available%20since%20version%200.17.0
[xm-readme-vllm]: https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/README.md#:~:text=vLLM%20GPU%20installation%20guide
[lms-cli]: https://lmstudio.ai/docs/cli#:~:text=ships%20with%20LM%20Studio
[opt-intel-list]: https://github.com/huggingface/optimum-intel/blob/0100bb36ea09fe8613008ee74cb796dc5e1c379d/optimum/exporters/openvino/utils.py#L623-L638
[opt-intel-cmds]: https://github.com/huggingface/optimum-intel/tree/0100bb36ea09fe8613008ee74cb796dc5e1c379d/optimum/commands
[opt-onnx-list]: https://github.com/huggingface/optimum-onnx/blob/46533fb66570021f52a39d77112b703762fdaada/docs/source/onnx/overview.mdx#L107-L109
[mpm-dup]: ../../package/mpm-shadow/cli/src/toolchain/bin_paths.rs
[mpm-alias]: ../../package/mpm-shadow/cli/src/toolchain/exposure.rs
[mpm-fixture]: ../../package/mpm-shadow/cli/src/toolchain/fixtures.rs
