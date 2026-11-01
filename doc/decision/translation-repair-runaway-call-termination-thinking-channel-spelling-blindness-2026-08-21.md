# Runaway call termination: the thinking channel that was never read, measured on 2026-08-21

The correction of the claim that both channels are scanned:
three models spell the thinking channel differently,
the scanner read none of it,
and what the fix,
the proof and the invalidated figures were.

Part of [Ending runaway calls on our side of the wire](translation-repair-runaway-call-termination.md),
which lists every file of this record.

## Both channels were scanned and one of them was never read

The section "Both channels are scanned,
and the thinking one matters most" was right about why,
and wrong about what the code did.
`scanStreamDeltas` read `content` and `reasoning_content` off `choices[0].delta` and nothing else.
This provider does not spell the thinking channel the same way for every model.

Measured 2026-08-21 with one streaming call per model,
a cat-themed invented prompt,
no corpus text.
Half the roster uses each spelling.

Carrying `reasoning_content`,
which the scanner already read:

- `zai-org/GLM-5.2` on 328 of its 329 frames.
- `Qwen/Qwen3.6-27B` on 463 of its 511 frames.
- `moonshotai/Kimi-K3` on 79 of its 148 frames.

Carrying `reasoning`,
which it did not:

- `zai-org/GLM-4.7-Flash` on 871 of its 1029 frames.
- `openai/gpt-oss-120b` on 46 of its 206 frames.
- `nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4` on 43 of its 232 frames.

The first reading of this defect put it at two models and it is three.
Counted over every stream record on disk from before the fix,
those three account for 2731 of 5864 calls,
and not one of those 2731 reported a single character of thinking.
So the detector was handed an empty string for every thinking token 47 percent of all recorded traffic
produced,
which is precisely the failure this document already named as the worst case:
a model repeating one sentence forever inside its thinking is never silent,
so the idle guard cannot fire,
and produced no text the degeneration detector could read,
so that could not fire either.
Both models ran to the wall clock instead.

The unreadable-frame tally could not show this and was never going to.
It rises only when a payload fails to parse.
These frames parsed perfectly and were simply not read,
so the number built to make a changed wire format visible sat at zero throughout.
A renamed field is invisible to it,
which is worth remembering before trusting that counter to prove anything about a future provider change.

The fix reads `reasoning_content` first and `reasoning` second,
taking the first non-empty value rather than the first present one,
so a provider that begins sending both spellings as aliases contributes that text once.
Merging them would double every thinking character on such a model
and push the detector toward a verdict on volume the model never produced.

### What proves it

Removing `reasoning` from the precedence list and rebuilding takes the suite from exit 0 to exit 1,
failing the three cases that depend on it,
including a thinking loop spelled `reasoning` that goes from unjudged to `degenerate`.

The control that mattered more ran against the built artifact and a live stream,
because widening the detector's input to a channel its thresholds were never calibrated on
could have produced spurious verdicts on healthy thinking,
which would be worse than the blindness it replaced.
All three models now count their thinking,
`GLM-4.7-Flash` at 5158 characters and Nemotron at 167 where both previously counted zero,
and all three leave both channels `undecided` on an ordinary call.

### What this invalidates

Every generated-character figure recorded for `GLM-4.7-Flash` and Nemotron before this date understates,
by however much thinking those calls did.
Any bound derived from that column has to be re-derived,
which is why the answer-volume bound is blocked on this rather than merely related to it.

Timing figures are unaffected.
The straggler and idle windows re-derived from first production traffic read `firstByteMs` and `maxGapMs`,
which never depended on which field carried the text.

### The size of what was invisible

Run 8 is the first run with the fix in place,
and the before-and-after is not subtle.
Every one of the three affected models went from 0 percent of calls reporting any thinking
to 100 percent of calls reporting it.
The median thinking on those calls is 6236 characters for `gpt-oss-120b`,
12986 for Nemotron,
and 22995 for `GLM-4.7-Flash`,
which is the largest median of any model on the roster and had been recorded as zero throughout.

Anything read off the character columns for those three before 2026-08-21 is wrong by roughly that much,
including the reading that they were the cheap models on the roster.
