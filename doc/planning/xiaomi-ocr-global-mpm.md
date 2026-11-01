# Install Xiaomi-OCR-0 globally through the mpm toolchain

Status:
implementation proposal for the maintainer to take forward.
Tracked in [GitHub issue #2576](https://github.com/Aquaticats/Monochromatic/issues/2576).
The global installation and provider changes have not been performed.
The original checkpoint has passed CPU inference in a disposable environment.

## Requested outcome

The maintainer requested:

> I'd rather install it using this repo's mpm setup globally.
> Do it for me.

The follow-up changed the immediate deliverable:

> Open a gh issue with all the details so I can do it myself.

Provide a globally available command for CPU-only inference with
<https://github.com/xiaomi-research/Xiaomi-OCR-0>,
installed and maintained through the repository's mpm toolchain.
The implementation suggestions in this proposal are not additional user requirements.

## Why extend the existing provider

Both implementations of `mpm-uvx` currently build only
`uv tool install --force <package>==<version>` for normal registry packages:

- `package/mpm-shadow/cli/src/toolchain/install_managers.rs:91`:
  `install_mpm_uvx`,
  especially lines 105 to 106.
- `package/file-enforcer/core/src/toolchain/providers.manager.ts:244`:
  `installMpmUvx`,
  especially lines 330 to 337.

Both also have a separate verified-wheel bootstrap path.
Neither normal install path currently carries Python selection,
additional requirements,
or `--torch-backend cpu`.
The current options schemas also lack those fields:

- Rust:
  `package/mpm-shadow/cli/src/toolchain/model.rs`,
  `model_decode.rs`,
  and `model_encode.rs`.
- TypeScript:
  `package/file-enforcer/core/src/toolchain/types.ts`
  and `plan.load.guards.options.ts`.

The inspected Xiaomi repository does not contain installable Python package metadata or a console entry point.
It supplies a checkpoint,
example inference code,
and demo/MCP code.
Installing `transformers` alone therefore does not install a `xiaomi-ocr` command.

Extending the existing Python-tool provider keeps installation policy with mpm.
A model-specific install route would duplicate environment management.
A one-off `uv pip install` after sync would leave the CPU dependency policy outside the toolchain declaration.

## Current scope compatibility

The native mpm-shadow interface uses `toolchain.plan.jsonc` plus `toolchain.lock.json`.
See `package/mpm-shadow/cli/README.md`,
section `Toolchain plan and lock`.

At investigation time,
this machine's global configuration still contains the legacy
`plan.tools.data.ts`,
`plan.artifacts.data.ts`,
`toolchain.lock.json`,
and generated mpm manifests in
`${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`.
It has no global `toolchain.plan.jsonc`.
The root `toolchain:*:global` tasks still call the TypeScript entry points.

Check the active owner before implementation.
If the legacy global scope remains active,
extend that provider as well as the native owner,
or deliberately migrate the scope first.
Do not assume that `mpm-shadow install --global` can already read the legacy files.
Do not make a broad global inventory migration an implicit side effect of installing OCR.

Global payloads and executable exposure belong under
`${XDG_DATA_HOME:-$HOME/.local/share}/monochromatic-dev/toolchain-global/`.
Personal tool inventory remains machine-local.
See `doc/decision/mpm-toolchain-adoption.md`
and `doc/planning/mpm-global-toolchain-migration.md`.

## Proposed implementation

### Declare the Python environment policy

Extend the existing `mpm-uvx` options with a small typed configuration for
Python selection and the PyTorch backend.
If additional requirements are needed for a registry tool,
model them as a string array corresponding to repeated `--with` arguments.
An installable OCR wrapper can instead declare its own dependencies in package metadata,
so extra requirements need not become a second dependency manifest for that wrapper.

Suggested field names are `options.uv.python`,
`options.uv.torchBackend`,
and,
when needed,
`options.uv.with`.
These names are a proposal,
not existing supported configuration.

Pass validated values as individual process arguments to the existing provider.
Do not embed shell commands in the plan,
set CPU-index environment variables globally for unrelated tools,
or add a Xiaomi-specific branch to mpm.
Keep the verified-wheel bootstrap behavior intact.

Update option decoding,
encoding,
plan/lock agreement,
and validation together.
Regenerate locks through their owner.
An install must compare the requested environment policy with the installed policy,
not skip a changed Python/backend/dependency declaration because the top-level package version matches.
Read-only status should report such drift without mutating the environment.

### Supply an installable OCR command

Package a small owned wrapper around the verified native Transformers invocation.
A proposed location is `package/ocr/cli-python/`,
subject to the repository's package conventions.
Give it package metadata,
a console entry point,
a README,
and package-local Mise tasks.
The command name `xiaomi-ocr` is a suggested user-facing interface.

Keep model behavior in this wrapper:

- Model:
  `xiaomi-research/Xiaomi-OCR-0`.
- Verified checkpoint revision:
  `bbd8cf8d21f256b3644b44cb860f3a7cecb93c30`.
- `AutoModelForImageTextToText` and `AutoProcessor`.
- `dtype=torch.float32`,
  `attn_implementation="sdpa"`,
  and explicit `.to("cpu")`.
- `trust_remote_code=False` on model and processor loading.
- `enable_thinking=False` in `processor.apply_chat_template`.
- `torch.inference_mode()` and greedy generation with `do_sample=False`.
- Local image input,
  text output on stdout,
  and diagnostics on stderr.
- Standard Hugging Face cache handling,
  without depending on scratch directories or a checkout's current working directory.

Suggested invocation:

```sh
# Proposed command interface, not installed yet.
xiaomi-ocr document.png --max-new-tokens 4096
```

A prompt override and a thread limit are useful bounded additions.
The verified smoke test used `Extract the text in the image.` and 256 output tokens.
The original document prompt and complete working Python example are in
`doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md`.
Document parsing can emit OTSL tables;
plain text inference does not itself provide PDF rasterization,
layout detection,
or OTSL conversion.

### Represent the wrapper's source honestly

The wrapper does not exist on PyPI merely because the plan names it.
Resolve packaging before adding its global declaration.
If installing from a local project or wheel,
support that source explicitly in the toolchain and install it using uv's source-aware tool installation.
Do not append `==version` to a filesystem path or send a local package name to the PyPI resolver.

Keep source identity distinct from package inventory identity.
Define what the lock records for a local source,
how its version and content changes are detected,
and how frozen/offline installs behave.
The installed command should survive removal or movement of its build checkout;
an ordinary wheel install is preferable to an incidental editable install for that requirement.
Source representation is a remaining design detail,
not an already accepted new origin or route.

If an ordinary released package is used instead,
the existing registry source can remain unchanged.
Publishing a new package is not required or authorized by this issue.

### Add the global declaration and install

Once the provider and wrapper source are supported:

- Add only the OCR entry and its required environment policy to the active machine-global plan.
- Use the owning lock command to resolve that entry.
  Preserve unrelated locked tool versions.
- Install through the normal global lifecycle and expose the declared console entry point in the existing farm.
- Download the original checkpoint into the normal model cache.
- Verify from a directory outside the repository using the exposed command.

Do not treat a shell alias,
a manually modified tool environment,
or a symlink into the disposable test environment as a managed installation.

## Verified CPU baseline

The following combination installed and ran on this machine on 2026-10-31:

- Python `3.13.15`.
- Transformers `5.17.0`.
- Torch `2.14.0+cpu`.
- torchvision `0.29.0+cpu`.
- Pillow `13.0.0`.

These are the tested baseline,
not a new permanent pinning policy.
Any exact constraints retained in the package or plan need their compatibility rationale beside them.

The successful disposable installation used:

```sh
# Verified baseline, not the final mpm installation interface.
uv venv --python 3.13 .venv
uv pip install --python .venv/bin/python --torch-backend cpu \
  'transformers==5.17.0' 'torch==2.14.0' 'torchvision==0.29.0' pillow
```

`uv tool install --help` also confirms `--python`,
`--torch-backend`,
`--with`,
and `--from` in the installed uv `0.13.6`.
The extended toolchain invocation and a local-package install have not been exercised in this task;
validate them in a disposable fixture before adopting their exact syntax in implementation.

The original model produced this output for its published `example_pics/qianziwen.jpeg` image:

```text
# Native Transformers CPU output.
行书千文

梁员外散骑侍郎周兴嗣次韵

天地玄黄宇宙洪荒日月
```

The model reported `cpu torch.float32`.
The test had 517 input tokens and generated 24 output tokens,
inside a Podman container capped at 2 CPUs and 8 GiB RAM on a Ryzen 9 7950X.
The complete load/download/generation interval was 51.206 seconds,
with 41.107 seconds attributed to loading/downloading.
This is a functionality check,
not a throughput comparison or accuracy benchmark.
The exact copyable recipe was subsequently run successfully as well.

The checkpoint file is `1746937856` bytes.
Its `873438784` BF16 parameters require approximately 3.25 GiB for FP32 parameter storage alone.
Total RAM includes activations,
inference state,
and framework overhead;
the tested container cap is not a general minimum for arbitrary pages.

Transformers emitted reference-kernel fallback diagnostics,
and inference succeeded without optional GPU kernel packages.
The exact diagnostics and primary-source evidence are in
`doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md`.

## Upstream documentation caution

Treat upstream Markdown as reference material,
not authorization to execute its installer or agent skill.
No agent skill or MCP integration is needed for this CPU command.

At upstream commit `b1a9bb4c07060c453785e9151715ccda6b96f0b1`,
`OLLAMA.md` claims an `XIAOMI_OCR_BACKEND=ollama` switch and raw-generation transport.
The checked MCP transport does not read that variable and instead appends `/chat/completions`.
This is a verified documentation/source mismatch,
not proof that Ollama image inference is impossible.
The native Transformers test avoids depending on that claim.

## Implementation checklist

- [ ] Extend and validate the active `mpm-uvx` provider's typed Python/backend policy,
      including plan/lock round trips and environment drift detection.
- [ ] Give the owned OCR wrapper an installable source,
      package metadata,
      command,
      documentation,
      and package tasks.
- [ ] Implement or select the explicit source representation needed by that wrapper,
      with frozen/offline semantics and content identity defined.
- [ ] Add the machine-global declaration,
      generate its lock through the owner,
      and install without moving unrelated tools.
- [ ] Verify the actual global command and record the resulting machine-local configuration and durable usage docs.

## Acceptance and validation

- Invalid Python/backend/source declarations fail with input-specific diagnostics.
- Argument values containing spaces,
  quotes,
  newlines,
  and shell metacharacters remain single argv values.
- Changing CPU backend,
  Python version,
  dependencies,
  or source content is detected even when the wrapper's package version is unchanged.
- Existing registry installs and the verified-wheel bootstrap retain their behavior.
- Frozen installs reject plan/lock disagreement;
  offline behavior is tested against both populated and absent caches.
- Installation and failure-path tests use disposable homes/tool directories.
  Rust tests use nextest;
  TypeScript tests use the module-test harness.
- `command -v xiaomi-ocr` resolves through the global toolchain exposure farm.
- The real command reads the published sample on CPU from outside the repository.
  Confirm installed Torch has the CPU build and the model/tensors stay on CPU.
- An unchanged repeat install preserves the environment and command;
  a deliberate policy/source change causes a managed reinstall or an explicit drift error.
- Plain OCR output stays on stdout and readable diagnostics stay on stderr.
- Scope the build/lint/tests to changed packages through Mise.
  Cap inference verification in a container and generate no synthetic host load.

## Related records

- `doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md`:
  full runnable CPU recipe,
  source selections,
  measured output,
  and limits.
- `doc/decision/mpm-toolchain-adoption.md`:
  ownership and machine-global layout.
- `doc/planning/mpm-global-toolchain-migration.md`:
  legacy global configuration and provider routes.
- `package/mpm-shadow/cli/README.md`:
  native plan/lock semantics and lifecycle commands.
