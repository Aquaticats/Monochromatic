# ONNX Script 0.7.2 compatibility checker does not validate exported models

## Status and symptom

The compatibility-facade finding remains source-only.
A separate bounded frontend canary has now imported the dependencies,
exported a toy graph,
and exercised the actual ONNX checker directly.
A later English Laya producer also passed explicit structural checking,
as recorded in the [producer result][laya-producer-result].
No compatibility-facade runtime failure is claimed.
The risk is treating PyTorch's successful checker-status branch as proof that a graph passed structural validation.
Do not confuse this with the separately measured Laya CPU latency or attention-memory results.

## Source identities

PyTorch 2.10 is pinned at `449b1768410104d3ed79d3bcfe4ba1d65c7f22c0`.
ONNX Script 0.7.2 is pinned at `082bfa28e959a4c0639d91b62663c6c59c4dfc42`.

Private evidence lives at `~/temp/agent/laya-onnx-source-audit-2026-09-27`.
Its source ledgers are `torch-exporter-ledger.json`,
`runtime-boundary-source-ledger.json`,
`release-followup-source-ledger.json`,
and `build-details-source-ledger.json`.

Third-party checkouts were not edited.
Git objects were materialized into the owned evidence directory for reading.

## Root cause boundary

PyTorch `torch/onnx/_internal/_lazy_import.py` names the
`onnxscript._framework_apis.torch_2_9` facade.
Its `check_model` import delegates through the `torch_2_8` and `torch_2_6` facades
into `torch_2_5`.
The selected function in `onnxscript/_framework_apis/torch_2_5.py:61-68` explicitly does no validation:

```python
# onnxscript/_framework_apis/torch_2_5.py:61-68
def check_model(model: ir.Model) -> None:
    """No-op retained for API compatibility.

    This intentionally performs no validation. Running the ONNX checker here was
    dropped because it can report false positives and adds overhead to export.
    """

    del model  # Intentionally unused: this function performs no validation.
```

No replacement implementation was installed or executed.

When verification is enabled,
PyTorch `torch/onnx/_internal/exporter/_core.py:1576-1581` calls that facade:

```python
# torch/onnx/_internal/exporter/_core.py:1576-1581, selected statements
onnxscript_apis.check_model(onnx_program.model)
export_status.onnx_checker = True
```

Under this source combination,
normal return from that function does not establish an ONNX structural-checker result.
This does not imply that every verification stage is disabled.
The same PyTorch source separately invokes ONNX Runtime output verification at line 1619.
Its error/status/return handling also requires separate review;
a flag named `verify` is not by itself a strict artifact-publication condition.

## Verification and limits

### Working evidence

- The delegated source paths were resolved from pinned Git objects.
- The actual runtime image's missing ONNX dependency records were measured without candidate imports.
- Wheel hashes,
  target-tag controls,
  and the separate ONNX wheel's signed source attribution passed their recorded checks.
  None is a graph-correctness test.

### Actual checker catalog

Private `export-canary/canary.py` ran in `proc_dfe2` with ONNX 1.23.0,
ONNX Script 0.7.2,
ONNX IR 1.0.0,
ML dtypes 0.6.0,
and Python-backend protobuf 7.36.2.
Its mise task was `probe:export-canary` in the private evidence repository.

The valid `Identity` graph passed `onnx.checker.check_model`.
The otherwise comparable graph with an undeclared input raised `onnx.checker.ValidationError`:

```text
Nodes in a graph must be topologically sorted, however input 'undeclared' of node:
name:  OpType: Identity
 is not output of any previous nodes.
```

A separately exported arithmetic graph passed explicit file-based checking and ONNX IR reload.
Host-side recheck `proc_c941` checked the actual exited image,
baked sources,
resource evidence,
and quarantined artifact hashes.
Graph inspection `proc_8bd0` found one `Add` node,
no model-local functions,
and a 512-byte external initializer.
No numerical inference was performed on that graph.
This follows the inspected deserialization-only program,
not its literal `inferenceExecuted: false` scope declaration.
The host-side recheck is not an independently authored code review.

### Unverified execution cases

- An invalid graph returning through the actual imported compatibility facade.
- Numerical parity of an exported graph against its source computation.
- Numerical or semantic full-policy behavior of the structurally exported Laya artifact.
- Failed verification leaving artifacts that a qualified consumer correctly refuses to promote.

The direct-checker catalog does not close these cases.
See the [qualification record](../planning/pi-auto-mode-laya-qualification.md#python-onnx).

## Verified workarounds

Explicit structural checking passed the valid/invalid controls and exported toy graph.
The exercised consumer-side call was:

```python
# export-canary/canary.py, inside the contained experiment
onnx.checker.check_model('/out/toy.onnx')
```

This avoids relying on the compatibility facade's status flag,
but establishes structure only,
not output equivalence or guard quality.
The planned consumer must still validate numerical parity independently,
identify every graph/external-data artifact,
and promote only after all required checks pass.
The canary always retains `promoted: false`;
no production implementation or complete publication workaround is qualified.

## What does not work as evidence

- The facade's function name does not establish that it calls an ONNX checker.
- `export_status.onnx_checker = True` alone does not establish structural validation with this dependency version.
- A successful artifact download,
  compatible wheel tag,
  or verified build attribution does not establish model correctness.
- No evidence here establishes that all PyTorch or ONNX Script versions behave this way.

## Separate retained exporter diagnostics

PyTorch 2.10 `torch/onnx/_internal/exporter/_registration.py:108-111` handles absent optional vision operators:

```python
# torch/onnx/_internal/exporter/_registration.py:108-111
if namespace == "torchvision":
    if importlib.util.find_spec("torchvision") is None:
        logger.warning("torchvision is not installed. Skipping %s", qualified_name)
        return None
```

The canary retained this warning for `torchvision::nms`,
`torchvision::roi_align`,
`torchvision::roi_pool`,
and `torchvision::deform_conv2d`.
Its parsed `Add` graph consumes none of those registrations.
This is not evidence that a consumer needing those operators can omit them.
No extra dependency was installed to silence the warnings.

A `FutureWarning` displayed at Python `copyreg.py:99` has its text defined in
PyTorch `torch/utils/_pytree.py:1328-1332`:

```python
# torch/utils/_pytree.py:1328-1332
@deprecated(
    "`isinstance(treespec, LeafSpec)` is deprecated, "
    "use `isinstance(treespec, TreeSpec) and treespec.is_leaf()` instead.",
    category=FutureWarning,
)
```

Both diagnostic classes remain unchanged in `export-canary/run-initial.json`.
Neither establishes the cause of the compatibility-facade behavior,
and neither prevented the observed toy result.
No upstream fault or future-version compatibility conclusion follows.

## Upstream filing artifact

Nothing to file from this source-only finding.
No new upstream issue or comment was sent.

### Upstream filing decision

#### Upstream fault

Not established.
The compatibility function explicitly documents its deliberate no-op behavior.

#### Fixability

Relevant source is available,
but no defect requiring a change has been reproduced.

#### Supported use

Export compatibility is supported;
mandatory structural validation is not promised by this function's current body/docstring.

#### Contribution policy

Not assessed for a filing because no reportable defect has been established.

#### Maintainer intent

The source explicitly says the checker was removed for false positives and overhead.
Do not infer agreement to restore it.

#### Prototype

None.
The conditions requiring an upstream-fix prototype are not satisfied.

[laya-producer-result]: ../planning/pi-auto-mode-laya-qualification.md#laya-producer-preparation-and-result
