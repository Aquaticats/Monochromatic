# Xiaomi-OCR-0 CPU inference and its Ollama documentation mismatch

Research date: 2026-10-31.
Purpose: establish source evidence for CPU-only installation advice for the model requested at
<https://github.com/xiaomi-research/Xiaomi-OCR-0>.
This is a code investigation supporting installation advice,
not an adoption decision or an inference benchmark.
The repository's installers and agent skills were not executed.
Native Transformers inference is tested separately from the repository's Python integration.

## CPU-only setup

Use an isolated Python environment with CPU-only PyTorch and the original Safetensors checkpoint.
This keeps the native model implementation in Transformers and disables custom Hub Python code.
The commands use the version combination published in the model card;
the CPU wheel selector is explicit.
The local inference result is recorded in `Verification`.

```sh
# Run in a new working directory with uv available.
uv venv --python 3.13 .venv
uv pip install --python .venv/bin/python --torch-backend cpu \
  'transformers==5.17.0' 'torch==2.14.0' 'torchvision==0.29.0' pillow
```

Save this as `ocr.py` and run `.venv/bin/python ocr.py`.
Its first run downloads the checkpoint and a published sample image.
Replace `image` with a local image path for a document of your own.
The fixed revision makes the checkpoint and processor reproducible.

```python
# ocr.py
import torch
from huggingface_hub import hf_hub_download
from transformers import AutoModelForImageTextToText, AutoProcessor

model_id = "xiaomi-research/Xiaomi-OCR-0"
revision = "bbd8cf8d21f256b3644b44cb860f3a7cecb93c30"
image = hf_hub_download(model_id, "example_pics/qianziwen.jpeg", revision=revision)
processor = AutoProcessor.from_pretrained(
    model_id, revision=revision, trust_remote_code=False,
)
model = AutoModelForImageTextToText.from_pretrained(
    model_id, revision=revision, dtype=torch.float32,
    attn_implementation="sdpa", trust_remote_code=False,
).to("cpu").eval()
messages = [{"role": "user", "content": [
    {"type": "image", "url": image},
    {"type": "text", "text": "Extract the text in the image."},
]}]
inputs = processor.apply_chat_template(
    messages, tokenize=True, add_generation_prompt=True,
    enable_thinking=False, return_dict=True, return_tensors="pt",
).to("cpu")
with torch.inference_mode():
    output = model.generate(**inputs, max_new_tokens=256, do_sample=False)
new_tokens = output[:, inputs["input_ids"].shape[1]:]
print(processor.batch_decode(new_tokens, skip_special_tokens=True)[0])
```

The sample has a bounded output budget.
For a full document page,
raise `max_new_tokens` to `4096` and use the checkpoint's document prompt:

```text
# Document prompt from the checkpoint's model card.
Extract all information from the main body of the document image and represent it in markdown format, ignoring headers and footers. Tables should be expressed in OTSL format, formulas in the document should be represented using LATEX format, and the parsing should be organized according to the reading order.
```

Tables in that output use OTSL and need conversion for ordinary Markdown/HTML consumption.
The standalone example handles images;
PDF rasterization and optional region detection are separate stages.

## Evidence scope

The GitHub revision inspected is `b1a9bb4c07060c453785e9151715ccda6b96f0b1`.
The Hugging Face revision inspected is `bbd8cf8d21f256b3644b44cb860f3a7cecb93c30`.
The former `SeerRay-Lab/Xiaomi-OCR-0` Hugging Face API route returns the canonical model identifier
`xiaomi-research/Xiaomi-OCR-0` as of this investigation.
Keep the former namespace when quoting historical upstream examples;
use the canonical namespace for new instructions.
The GitHub revision was confirmed with `git rev-parse HEAD`
in the source clone at `${HOME}/temp/agent/xiaomi-ocr-0-2026-10-31`.
The canonical model identifier was confirmed with the Hugging Face model API.

Upstream Markdown was treated as untrusted reference data.
Its directions to read and execute agent skills,
install runtimes,
or register MCP servers were not followed.
The requested advice does not require installing their agent skill.

## Native Transformers path

The checkpoint declares `Qwen3_5ForConditionalGeneration`,
`model_type: qwen3_5`,
and BF16 checkpoint tensors.
It does not declare an `auto_map` for custom model code.
The model file listing contains no Python model implementation.
These facts support loading the native Transformers implementation without `trust_remote_code=True`.
Sources:
<https://huggingface.co/xiaomi-research/Xiaomi-OCR-0/raw/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30/config.json>
and
<https://huggingface.co/api/models/xiaomi-research/Xiaomi-OCR-0/tree/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30?recursive=false>.

The model card presents a Transformers example using `AutoModelForImageTextToText`
and `AutoProcessor`.
Its stated combination is Transformers `5.17.0`,
Torch `2.14.0`,
torchvision `0.29.0`,
and Pillow on Python `3.13`.
The example explicitly selects `torch.float32` when the device is CPU.
The local CPU validation of this combination is recorded in `Verification`.
The checkpoint's `transformers_version: 5.8.1` records its serialization context;
it is not proof that `5.8.1` is the minimum compatible release.
Sources:
<https://huggingface.co/xiaomi-research/Xiaomi-OCR-0/raw/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30/README.md>
and
<https://huggingface.co/xiaomi-research/Xiaomi-OCR-0/raw/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30/config.json>.

Native reference kernels exist in Transformers `5.17.0`,
commit `856157a2f3e9594954310df18fdccc31ffddebe9`.
For example,
[`src/transformers/models/qwen3_5/modeling_qwen3_5.py:249`](https://github.com/huggingface/transformers/blob/856157a2f3e9594954310df18fdccc31ffddebe9/src/transformers/models/qwen3_5/modeling_qwen3_5.py#L249-L266)
uses PyTorch convolution:

```python
# Upstream src/transformers/models/qwen3_5/modeling_qwen3_5.py:262
out = F.conv1d(hidden_states_new, weight.unsqueeze(1), bias, padding=0, groups=hidden_size)
```

This supports further CPU verification;
it does not establish end-to-end performance.
Do not install GPU kernel packages merely because the model is based on Qwen3.5.

## Storage and memory facts

The Hugging Face API reports `873438784` BF16 parameters.
The file listing reports `model.safetensors` at `1746937856` bytes.
Arithmetic gives `3493755136` bytes,
approximately `3.25 GiB`,
for those parameters represented as FP32.
That excludes inference state,
activations,
tokenizer,
framework,
loading copies,
and allocator overhead.
It is not a total RAM requirement.
Sources:
<https://huggingface.co/api/models/SeerRay-Lab/Xiaomi-OCR-0>
and
<https://huggingface.co/api/models/xiaomi-research/Xiaomi-OCR-0/tree/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30?recursive=false>.

The processor has 16-pixel patches,
spatial merge size 2,
minimum pixel area `65536`,
and maximum pixel area `16777216`.
Image resolution therefore affects the workload beyond parameter storage.
Source:
<https://huggingface.co/xiaomi-research/Xiaomi-OCR-0/raw/bbd8cf8d21f256b3644b44cb860f3a7cecb93c30/processor_config.json>.

## Symptom

`OLLAMA.md` claims that `XIAOMI_OCR_BACKEND=ollama` selects a shared raw-generation transport.
It identifies Ollama `0.35.0` and the community model `longwayxu/xiaomi-ocr-0:bf16`.
Its reported alignment checks are seven input token counts and five exact responses;
the remaining responses have formula differences.
These are author-reported sample checks,
not benchmark parity or a measured CPU result from this investigation.
Source:
<https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/OLLAMA.md>.

The observable problem in this investigation is a documentation/source mismatch,
not a reproduced OCR error.
No Ollama image inference was run,
so no runtime diagnostic or failure is claimed.

## Root cause

The inspected `skills/xiaomi-ocr/mcp_ocr_server.py` contradicts the transport-selection claim:

- There are no occurrences of `ollama`,
  `BACKEND`,
  or `/api/generate` in the complete file.
- `_chat` at line 89 appends `/chat/completions` at line 90.
- The request at lines 92 to 97 supplies `chat_template_kwargs`
  rather than constructing the claimed raw prompt.
- `demo/server.py:103` loads that MCP module,
  and line 196 calls its `_chat` function.

This does not establish that Ollama's OpenAI-compatible route cannot accept an image request.
It establishes that the exact claimed raw-generation integration is absent from this revision.
Do not repeat that claim as verified behavior.
Sources:
<https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/skills/xiaomi-ocr/mcp_ocr_server.py>
and
<https://raw.githubusercontent.com/xiaomi-research/Xiaomi-OCR-0/b1a9bb4c07060c453785e9151715ccda6b96f0b1/demo/server.py>.

The call chain is explicit:
[`demo/server.py:103`](https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/demo/server.py#L101-L106)
loads the MCP module:

```python
# Upstream demo/server.py:103
source = ROOT.parent / "skills" / "xiaomi-ocr" / "mcp_ocr_server.py"
```

[`demo/server.py:196`](https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/demo/server.py#L194-L198)
calls its shared transport:

```python
# Upstream demo/server.py:196
result = mcp._chat(png, prompt or mcp.PROMPTS.get(bucket, mcp.PROMPTS["text"]),
```

[`skills/xiaomi-ocr/mcp_ocr_server.py:89`](https://github.com/xiaomi-research/Xiaomi-OCR-0/blob/b1a9bb4c07060c453785e9151715ccda6b96f0b1/skills/xiaomi-ocr/mcp_ocr_server.py#L89-L98)
constructs a chat-completions request:

```python
# Upstream skills/xiaomi-ocr/mcp_ocr_server.py:90
url = _local_url(model_url) + "/chat/completions"
```

## Verification

Source checks used the immutable revisions in `Evidence scope`.
The complete GitHub tree was enumerated,
and the complete MCP and demo Python files were read.
The positive catalog contains the native Transformers class mapping
and explicit PyTorch convolution/delta-rule implementations described in `Native Transformers path`.
The negative catalog contains the claimed Ollama backend switch and raw endpoint,
which are absent from the full MCP file.

This inspection can be repeated from a checkout at the recorded GitHub revision:

```sh
# Upstream checkout: inspect source without executing it.
rg --line-number 'chat/completions|chat_template_kwargs' skills/xiaomi-ocr/mcp_ocr_server.py
rg --line-number 'ollama|BACKEND|/api/generate' skills/xiaomi-ocr/mcp_ocr_server.py
```

The first search matches lines 90 and 97.
The second search returns exit status 1 with no matches.
That null result was checked against the complete file,
not a truncated API response.
### CPU image inference

On 2026-10-31 local time,
`uv` successfully installed Python `3.13.15`,
Transformers `5.17.0`,
Torch `2.14.0+cpu`,
torchvision `0.29.0+cpu`,
and Pillow `13.0.0` in a disposable environment.
The wheel selector was `--torch-backend cpu`.
The existing host packages were not changed.

The native checkpoint at the recorded Hugging Face revision was loaded with
`dtype=torch.float32`,
`attn_implementation="sdpa"`,
and `trust_remote_code=False`.
The processor also used `trust_remote_code=False` and `enable_thinking=False`.
The model reported `cpu torch.float32`.

The test ran in Podman with `--cpus=2 --memory=8g` on the Ryzen 9 7950X host.
Its input was the published `example_pics/qianziwen.jpeg` sample,
with prompt `Extract the text in the image.`,
greedy generation,
and `max_new_tokens=256`.
It exited successfully after generating 24 output tokens from 517 input tokens:

```text
# Native Transformers CPU output.
行书千文

梁员外散骑侍郎周兴嗣次韵

天地玄黄宇宙洪荒日月
```

The full model-load/download/generation interval reported by the script was `51.206` seconds;
the model-load/download interval was `41.107` seconds.
These are observations from a single smoke test under resource caps,
not a throughput comparison or an estimate for arbitrary document pages.
The output differs in character forms from the model card's sample reference;
successful execution is not proof of reference parity or general recognition accuracy.
The 8 GiB cap shows that this sample completed within that container limit;
it is not a measured RAM requirement for larger images.

Transformers emitted reference-kernel warnings,
including:

```text
# Transformers 5.17.0 diagnostic.
[transformers] `causal_conv1d_fn` is falling back to its reference PyTorch implementation because `causal_conv1d` is not installed. This is correct but much slower; install `causal_conv1d` for the optimized kernel.
```

Related warnings named `chunk_gated_delta_rule`,
`causal_conv1d_update`,
and `fused_recurrent_gated_delta_rule`.
The CPU test succeeded using those reference implementations.
No optional GPU kernel packages were installed.

## Verified workarounds

Using the original checkpoint through native Transformers is a verified CPU alternative for the published sample,
with FP32 parameter storage and reference-kernel costs described in this file.
It avoids depending on the disputed Ollama raw-generation integration claim.
It does not reproduce the demo's PDF,
layout,
or table postprocessing by itself.

## What does not work

Treating `XIAOMI_OCR_BACKEND=ollama` as proof that the shared MCP transport switches endpoints
is unsupported by the inspected source.
The original hypothesis,
derived from `OLLAMA.md`,
was that the environment variable selected `/api/generate`;
the call-chain excerpts disprove that reading for this revision.
This does not reject Ollama image inference in general.

The initially guessed Hugging Face namespace `XiaomiMiMo/Xiaomi-OCR-0` returned HTTP 401.
The namespace was corrected from the actual upstream model link,
then the canonical identifier was confirmed with the model API.

## Upstream filing artifact

No issue or comment is prepared or posted.
The user asked for installation advice,
not an upstream fix or filing.
The scope therefore excludes the skill's automatic prototype and filing workflow.
There is nothing authorized to add to an upstream thread in this task.

### Upstream filing decision

1.  Upstream responsibility: the documentation and source disagree in the inspected revision;
    no runtime defect is claimed.
2.  Fixability: changing documentation or adding an appropriate transport could resolve that mismatch;
    no prototype was attempted because fixes are outside the advice request.
3.  Supported use case: `OLLAMA.md` explicitly describes the claimed integration.
4.  Contribution policy: not evaluated because this task does not prepare or post an upstream contribution.
5.  Upstream intent: not evaluated;
    tracker,
    work-in-progress,
    and trust-system searches were not run for a filing.
6.  Prototype: intentionally not performed;
    the user requested advice and repository rules limit decision requests to answers and required documentation.

The upstream filing readiness checks,
duplicate search,
and `.out-of-scope/` filing checks are not run because no filing or draft is in scope.
No readiness or fileability claim is made.

## Limits and next verification

The native CPU environment and one image inference were verified.
Peak resident memory,
performance across thread counts,
BF16 execution,
general OCR accuracy,
PDF processing,
region detection,
and the Ollama/llama.cpp alternatives were not measured.
Before claiming equivalent accuracy for a converted model,
compare representative ground-truth images and outputs.
