# Span 02 live shadow

## Purpose and authority

The user requested running `respan/span-02` as a production shadow on 2026-10-19,
including the supervised Pi MiMo sessions.
The shadow observes their actual Decisions requests and sends the same inputs through platform `fetch`.
It records model scores and comparisons;
it cannot authorize tools, deny tools, answer approval dialogs, or replace Luna responses.

The runner is a separate Podman process with read-only mounts for both live worker fixtures
and the repository.
Only its private result directory is writable.
No `pi-ai` or `openai` package implements its API calls.
This separate process is justified because the existing sessions are already running:
changing their loaded transport would couple the experiment to the primary decision path.

## Captured input and correlation

Existing capture files contain the actual serialized Gateway Decisions requests.
The shadow preserves the complete `input` string and question instruction strings.
It maps choice descriptions to OpenRouter categorical criteria without changing their wording.
Old captures are baseline inventory, not new shadow traffic.

The diagnostic reader starts at the current end of each existing log.
It reads only subsequently appended records,
preserving partial UTF-8 and JSON lines across writes.
State and question hashes correlate native assessment diagnostics with captured HTTP requests.
Worker identity remains part of the correlation key.
Categorical disagreements are model-answer differences,
not proof that either model or the executed tool verdict is correct.

The comparison applies the user's current uncertainty-dependent admission rule to both distributions.
The running primary artifact retains its own installed operating policy.
The shadow does not retroactively change that policy.

## Execution and retained evidence

The initial deployment uses private fixture
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/span02-shadow-live-FJysy5`.
`manifest.json` records container identity,
source digests,
read-only inputs and resource bounds.
`output/state.json` records readiness and counts.
`output/events.jsonl` records responses,
correlations,
disagreements and failures.
Individual shadow response bodies remain in that same private output directory.
Credentials are forwarded only through the named environment variable and are not recorded.

The runner uses two GiB memory,
two CPUs,
a 256 MiB Node heap,
at most two concurrent HTTP calls,
a ninety-second request signal and no retries.
It selects Respan only,
disables provider fallback and requests `data_collection: deny`.
A pending queue of 256 captured requests stops the observer rather than exhausting host memory.
The original input captures remain available if shadow processing stops.

The launch task is `start-span02-shadow` in the persistent preparation-failure scratch `mise.toml`.
The runner scripts and their immutable deployment copies remain under that scratch directory.
This is an operational deployment for the running sessions;
shipping a maintained package command remains follow-up work.

## Verification state

Node syntax checks pass for the runner,
wire conversion and diagnostic reader.
Podman reports the deployed container running with memory limit `2147483648`
and CPU quota `2000000000` nanocpus.
Both request watchers and both diagnostic tails report ready.
Their initial baselines contain 166 source-worker requests and 130 native-worker requests.
Those baseline captures are not counted as newly shadowed calls.
The first four newly captured live requests returned HTTP `200`
with model `respan/span-02-20261015` and matched their Luna diagnostic records.
Native captures `130` and `131`,
and source captures `166` and `167`,
are the verified initial production sample.
Both one-head whole-operation decisions agree.
Seven categorical heads differ across the two 27-head clause assessments.
These differences include selected meaning or admission under the user's current rule;
they are not seven demonstrated model mistakes.
Observed shadow call intervals range from `640.827` to `4259.027` milliseconds.
No shadow request failed in that sample.

The live runner is an immutable copy of its launch sources.
Subsequent private source edits add boolean-question handling and more isolated input-error handling;
those edits have not replaced the running copy or been qualified by its production results.
Keep deployment source digests and results together.

The [initial trial](span-02-trial.md) remains a separate replay experiment.
