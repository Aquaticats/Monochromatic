# Laya 0.3.20 notebook preprocessing does not establish full-policy training or state-disjoint calibration

## Symptom and scope

The auto-mode evaluation requires complete current `AGENTS.md` and independently qualified axiom probabilities.
The published Laya fine-tuning notebook is not a verified drop-in recipe for that workload.
Its preparation stage uses checkpoint token budgets,
and its calibration split operates on question items rather than entire states.

This investigation performed no training,
optimizer step,
model forward,
benchmark-data download,
or Hub publication.
The measured behavior concerns preparation and split code,
not a trained model's accuracy or an observed production failure.

## Source identity

Read-only clone:
`~/temp/agent/laya-auto-mode-2026-09-26`.
Repository:
<https://github.com/NandhaKishorM/laya>.
Revision:
`4066d5d5fbf08b66c6757ddeedbd797bd7655bc0`.
Version:
0.3.20.
Paths in the source trace are relative to that clone.
The recipe is `notebooks/laya_finetune_typed_decisions_2xT4_kaggle.ipynb`.

The probe uses only its inspected preprocessing function and index-splitting statements.
No upstream file was modified.
The private preparation manifest retains the notebook hash and extraction scope.

## Root cause trace

### Encoding happens before worker configuration changes

The preprocessing cell at notebook lines 98-125 loads checkpoint configuration
and passes its token limits into `build_sequence`:

```python
# notebooks/laya_finetune_typed_decisions_2xT4_kaggle.ipynb, preprocessing cell excerpts
with open(os.path.join(model_dir, "rl_agent_config.json")) as f:
    cfg = json.load(f)

seq, markers = build_sequence(
    tok, state, {"t": t, "ins": q["instructions"], "crit": crit},
    cfg["max_len"], cfg["head_max_len"]
)
```

The caller saves already tokenized items before launching the worker.
The worker cell at notebook lines 232-251 later loads configuration,
changes its limits,
and loads those items:

```python
# notebooks/laya_finetune_typed_decisions_2xT4_kaggle.ipynb, worker cell excerpts
cfg["max_len"] = 1024
cfg["head_max_len"] = 256
all_items = torch.load("/kaggle/working/train_items.pt", weights_only=False)
```

Those assignments do not retokenize an existing item.
With the pinned English checkpoint's 512-token preparation budget,
the complete-policy control becomes a 512-token item with both option markers still present.
The notebook's marker-count test therefore does not establish complete-state retention.

This is an application-fit finding.
It does not prove the notebook promises full-policy or arbitrary-length training.
Do not execute its unpinned install/download cells or unsafe-pickle load against unreviewed artifacts.
The probe did neither.

### Calibration item separation is not state separation

The preprocessing loop at notebook lines 136-146 appends an item for each available question in each state.
The worker at lines 258-264 then splits the flattened item indices:

```python
# notebooks/laya_finetune_typed_decisions_2xT4_kaggle.ipynb, split statements
CALIB_MAX = 400
order = list(range(len(all_items)))
random.Random(20260922).shuffle(order)
n_calib = min(CALIB_MAX, len(all_items) // 10)
calib_items = [all_items[i] for i in sorted(order[:n_calib])]
train_items = [all_items[i] for i in sorted(order[n_calib:])]
```

The current source excludes those exact calibration item indices from optimization.
Do not describe it as still fitting on the same optimized items merely because the README calls them training items.
Their origin in the dataset's train split is different from membership in the optimizer's subset.

However,
a state's other questions can enter training while one of its questions is held out.
The measured synthetic layout demonstrates that this split permits shared states.
It does not quantify leakage or calibration inflation in the real benchmark.
The guard's future qualification requires its own state/family separation
rather than adopting this split unit implicitly.

### Fit and serving temperature intervals differ

The inspected notebook's `fit_one_temp` at lines 202-218 ends with:

```python
# notebooks/laya_finetune_typed_decisions_2xT4_kaggle.ipynb
return float(torch.clamp(log_t.exp(), 0.1, 10.0).item())
```

The serving helper uses different bounds at `laya/common.py:376-390`:

```python
# laya/common.py
TEMP_MIN = 0.5
TEMP_MAX = 5.0

return min(hi, max(lo, t))
```

`laya/agent.py:390-407` normalizes loaded per-type and option-count temperatures through that helper.
A future exported fit therefore needs validation after loading,
not merely inspection of its stored number.
This source-range observation does not establish that an unrun fit would leave the serving interval.
No fit,
optimizer step,
or synthetic trained checkpoint was created to make that claim.
The measured Noul baseline selected valid temperatures;
its observed `choice:11+` warning concerns a different bucket.

### Published checkpoint calibration is a separate artifact

The pinned typed-decisions configuration at model revision
`1a793eb568e6718f15941d08f85432581df534e3`
contains per-type Noul temperature `1.0575125217437744`
and inherited `noul:2` temperature `1.983399510383606`.
The runtime selects the option-count bucket first at `laya/agent.py:768-774`:

```python
# laya/agent.py, selected temperature lookup
qt = QTYPES[q["t"]]
t_scale = self.temperature_by_options.get(temp_bucket(qt, k), self.temperature[qt])
```

The [typed-decisions model card](https://huggingface.co/convaiinnovations/laya-typed-decisions)
explicitly says the published fit used already-trained items
and its inherited buckets override the newer per-type fit.
The current notebook's export removes inherited buckets,
but that source change does not rewrite an existing published checkpoint.
No temperature fit or override was performed in this investigation.

## Verification

Private harness:
`~/temp/agent/laya-training-input-audit-2026-09-26`.
Frozen harness commit:
`9f921f1`.
Result commit:
`c20421f`.
Process `proc_610b` exited 0.
Image:
`9887ea1741517968a5caa5f0a479205d419669ae7fd9b2f81d1ad9b7db6252b6`.
Result SHA-256:
`b1895a4b37b3907e9a9cbec8037a2d4ce40f2ca0a1a41fc32f02d77495fcfa04`.

Verified commands from the private directory:

```sh
# ~/temp/agent/laya-training-input-audit-2026-09-26
mise --no-env --no-hooks run build
mise --no-env --no-hooks run probe
```

The bounded container imports the existing audited tokenizer and sequence builder,
not an Agent or checkpoint parameters.
Runtime has 2 GiB RAM,
2 CPUs,
no added swap,
64 PIDs,
256 descriptors,
a 60-second limit,
no network,
no credentials,
no host mounts,
and a read-only filesystem with a 64 MiB temporary volume.
A Python audit hook rejects subprocess and network activity.
The full policy is supplied only to tokenization;
no clipped input is evaluated by a model.

### Working controls

- The short-state encoding produced 47 tokens.
- Explicit re-encoding after increasing the limit produced 1,024 tokens.
- A first-party whole-state split produced zero overlapping state IDs.
- No model forward or optimizer operation exists in the admitted call path.

### Boundaries reproduced

- Initial encoding used `max_len: 512` and `head_max_len: 192`.
- The saved item contained 512 tokens and remained at 512 after changing configuration.
- Complete input at the original head budget required 12,553 tokens.
- The actual index splitter received 6,000 synthetic items representing 1,200 states with five questions each.
- It assigned 5,600 items to training and 400 to calibration.
- The calibration items covered 345 states;
  every one also had another question in training.

The last finding is conditional on this explicit synthetic layout,
not a count from downloaded benchmark rows.
The whole-state control proves the overlap measurement can distinguish the grouping unit.
The tokenizer emitted its generic `12518 > 8192` warning;
no model was run,
so no indexing exception was tested or inferred from that warning.

## Verified boundary and future requirements

The probe distinguishes budget changes from re-encoding
and exact-item holdout from whole-state holdout.
It verifies those observations,
not a complete replacement trainer.
No trained or recalibrated checkpoint was produced.

Any future authorized guard-training recipe would need:

- Final input encoding and complete-policy assertions before materializing examples.
- Explicit treatment and qualification of context beyond the declared encoder range.
- Independent references with known data rights,
  not Jev outputs or unexamined teacher distributions.
- Disjoint training,
  calibration,
  and final qualification groups at the appropriate state/family level.
- Temperature fitting and serving in matching precision,
  with effective bucket precedence and runtime clamps checked.
- Resource measurement within separately authorized training limits.

These are candidate-feasibility requirements,
not permission to start training.
The [source-supported feasibility note](../planning/pi-auto-mode-laya-finetune-feasibility.md)
distinguishes whole-model adaptation,
head-training boundaries,
community examples,
and the unmeasured resource/quality questions.

## What does not work as evidence

- Increasing the worker's `max_len` after token items are saved.
- Treating retained option markers as proof of retained policy.
- Calling item-index separation state/family separation.
- Claiming the current notebook still has the historical same-item fit solely from README wording.
- Treating a notebook fix as a rewrite of published checkpoint temperatures.
- Importing published specialist benchmark accuracy as auto-mode guard accuracy.

The [dataset card](https://huggingface.co/datasets/LocalLLaMA/typed-decisions)
states that its gold distributions measure teacher agreement,
not independent correctness.
Its license declaration does not identify every teacher-service right.
No such gold was used here.

## Upstream filing decision

No upstream issue,
comment,
patch,
or vendor contact is proposed.
Existing [issue 186](https://github.com/NandhaKishorM/laya/issues/186)
was read with its comments.
It concerns the older same-item calibration fit;
the inspected notebook has changed that path.
Re-reporting that historical issue as unfixed would be incorrect.
The `.out-of-scope/` inventory was previously checked in this evaluation;
no Laya-specific exemption was identified.

1.  Upstream fault:
    not established for the complete-policy guard workload,
    which exceeds the published context examples.
2.  Fixability:
    preprocessing and grouping can be designed differently;
    no impossibility claim is made.
3.  Supported use:
    the notebook demonstrates its named benchmark,
    not this guard's complete-policy or scenario-family qualification contract.
4.  Contribution policy:
    no contribution is proposed;
    policy acceptance is not inferred from an open tracker.
5.  Maintainer direction:
    the historical calibration issue was addressed in source;
    this record does not claim a new request was accepted or rejected.
6.  Fix prototype:
    the preparation/split control is a factual probe,
    not a validated trainer patch.

There is nothing to file as a new upstream bug on the present evidence.
