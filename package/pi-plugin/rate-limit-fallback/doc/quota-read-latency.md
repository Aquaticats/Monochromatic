# Quota read latency from a live endpoint measurement

## What this record answers

`QUOTA_READ_TIMEOUT_MS` in `src/member-capacity.ts` bounds the optional quota read a member's refusal triggers.
Repository rule `LQM` asks a shipped limit to cite a workload measurement or an owner decision,
and the package README previously asked the installer to take that measurement.
This file is the measurement behind the current value,
taken against these accounts from this machine.
It ran on 2026-11-19,
writing its records from 2026-11-19T18:45:41-05:00 to 2026-11-19T18:47:32-05:00;
commit `076dab6279` (2026-11-19T18:50:55-05:00) first cited it.

## What was measured

- The read under test is the package's own `probeQuota` from `src/quota-endpoints.ts`,
  called with a `ModelRegistry` built over a live `ModelRuntime` in the measuring process.
  Its transport was a timing wrapper around the process's `fetch`,
  passed through `probeQuota`'s `fetchQuota` parameter,
  which is the parameter the dispatcher fills from `options?.fetch ?? globalThis.fetch`.
- Each member's Bearer value came from `ModelRegistry.getApiKeyAndHeaders`,
  the same call the read itself makes.
  The measuring code never read a credential file,
  and no header,
  key or token value was printed or written.
  The run read every file it wrote back and failed if any held a resolved value.
- Every read carried `AbortSignal.timeout(2000)`,
  mirroring `quotaSignal` with no caller timeout,
  so a read that overran the two-second bound would show as `QUOTA_UNAVAILABLE` in the record rather than pass silently.
- Rounds:
  twenty per arm pair for Synthetic's standard lane and NeuralWatt's standard lane,
  six for NeuralWatt's flex lane.
  The arms alternate their order every round.
  Successive arms,
  and the `/v1/rate_limits` poll of each round,
  are at least 1,100 ms apart,
  because NeuralWatt asks clients to poll `/v1/rate_limits` at most once per second ([rate limits][nw-rates],
  "Checking your current limit").
  The two requests inside one NeuralWatt read are issued concurrently,
  so they are not spaced.
- Percentiles are nearest-rank,
  so with twenty samples the 95th percentile is the nineteenth sample and the highest sample stays visible beside it.

## Endpoints and their control arms

Each quota read is paired with a request to the same host and origin that reads no quota,
sent with the same Bearer value:

- Synthetic:
  the read is one `GET https://api.synthetic.new/v2/quotas` plus its JSON body.
  The control is `GET https://api.synthetic.new/openai/v1/models`,
  the authenticated OpenAI-compatible listing the installed synthetic provider documents as its catalog source,
  so it is a request the account already makes.
- NeuralWatt:
  the read is `GET https://api.neuralwatt.com/v1/rate_limits` with the member's model id
  plus `GET https://api.neuralwatt.com/v1/quota`,
  issued concurrently,
  each with its JSON body.
  The control is `GET https://api.neuralwatt.com/v1/models`,
  the listing URL the installed NeuralWatt provider refreshes its catalog from
  (`package/agent-harness-shared/neuralwatt-listing/src/constants.ts`,
  `NEURALWATT_LISTING_URL`).

The timeout the read is bounded by covers credential resolution,
both requests and both body reads,
so the quota arm is timed around the whole `probeQuota` call.
The control arm is timed around one request and its body read.

The pools also list `qwen-token-plan-individual/deepseek-v4.1-flash-1017` and `opencode-go/deepseek-v4.1-flash`.
Neither provider has a quota endpoint (see the README's "Provider sources"),
so `probeQuota` returns empty evidence for both at its provider test,
before it calls `registry.getApiKeyAndHeaders`,
and sends no request:
there is no read of theirs to time.
The measuring code's separate member-discovery step is what resolved their authorization through that registry call.

## Results

Each arm lists median,
95th percentile,
highest,
lowest,
and the run-to-run spread (highest minus lowest).
The ratio is the quota arm divided by the control of the same round.

Synthetic standard lane,
twenty rounds:

- quota read:
  median 50.8 ms,
  95th percentile 56.6 ms,
  highest 69.6 ms,
  lowest 45.0 ms,
  spread 24.6 ms
- control listing:
  median 38.1 ms,
  95th percentile 42.8 ms,
  highest 44.5 ms,
  lowest 36.9 ms,
  spread 7.6 ms
- paired ratio:
  median 1.32,
  lowest 1.01,
  highest 1.80

NeuralWatt standard lane,
twenty rounds:

- quota read:
  median 183.9 ms,
  95th percentile 402.7 ms,
  highest 464.4 ms,
  lowest 84.3 ms,
  spread 380.1 ms
- control listing:
  median 103.6 ms,
  95th percentile 407.8 ms,
  highest 514.2 ms,
  lowest 82.4 ms,
  spread 431.8 ms
- inside the read,
  header time of `/v1/rate_limits`:
  median 122.4 ms,
  95th percentile 402.1 ms,
  highest 463.8 ms;
  of `/v1/quota`:
  median 75.7 ms,
  95th percentile 266.9 ms,
  highest 315.8 ms
- paired ratio:
  median 1.45,
  lowest 0.35,
  highest 5.12

NeuralWatt flex lane,
six rounds:

- quota read:
  median 104.0 ms,
  95th percentile 140.7 ms,
  highest 140.7 ms,
  lowest 83.9 ms,
  spread 56.8 ms
- control listing:
  median 87.8 ms,
  95th percentile 364.1 ms,
  highest 364.1 ms,
  lowest 80.7 ms,
  spread 283.4 ms
- paired ratio:
  median 0.85,
  lowest 0.36,
  highest 1.74

How to read the ratios:
on Synthetic the read is consistently about a third slower than the listing,
and both stay under 70 ms.
On NeuralWatt the quota arm's own spread (380.1 ms) is no larger than its control's (431.8 ms),
and the paired ratio ranges from 0.35 to 5.12,
so that host's run-to-run noise is the same order as the difference between the arms and the ratio cannot separate them.
The absolute readings are what the deadline has to cover,
and the highest one is 464.4 ms.

## Body sizes, measured separately

The controls return model listings,
which are far larger than the quota documents,
so a ratio below one would not mean the quota read is slow:

- Synthetic:
  listing 11,870 bytes,
  `/v2/quotas` 285 bytes
- NeuralWatt:
  listing 38,013 bytes,
  `/v1/rate_limits` 215 bytes,
  `/v1/quota` 1,010 bytes

NeuralWatt serves its listing and its quota documents brotli-encoded,
and Synthetic serves plain JSON.
Every size in this section is a decoded body size.

## Failures and statuses

Every one of the 118 HTTP requests in the latency run answered `200`,
and all 46 reads returned parsed evidence.
No read hit the deadline and none returned `QUOTA_UNAVAILABLE`.
Nothing timed out,
so nothing in this run is censored by the bound it was measured under.

The counts are twenty reads and twenty controls for Synthetic,
twenty reads (forty requests) and twenty controls for NeuralWatt's standard lane,
and six reads (twelve requests) and six controls for its flex lane.

## Why two seconds stays

- The highest read took 464.4 ms (round 16 of NeuralWatt's standard lane),
  so two seconds is 4.3 times the highest observation.
- Adding NeuralWatt's control spread over rounds 2 to 20 (325.4 ms,
  the rounds after the warm-up round,
  whose own control took 514.2 ms) to that highest read gives 789.8 ms,
  which still leaves more than a second of margin for a day twice as noisy as this one.
- Two seconds remains at or below half of any caller-supplied model timeout above four seconds,
  so the existing cap keeps the binding arm whenever a caller sets a shorter timeout.
- What makes the bound safe is that failure mode rather than the arithmetic.
  At the deadline `readQuotaWithinDeadline` (`src/member-capacity.ts`) stops waiting on `probeQuota`
  and resolves `QUOTA_UNAVAILABLE`,
  so the read publishes no new evidence and routing continues on the evidence already held.
  An overrun costs only the reading,
  never the request.
- The paired ratios are not what the value rests on,
  for the reason `## How to read the ratios` gives:
  both control listings return bodies tens of times larger than the quota documents,
  so a ratio charges the read for body size as well as endpoint cost,
  and on NeuralWatt the host's own spread spans the whole ratio range.

## Limits of this measurement

- One machine,
  one network path,
  one set of accounts,
  one day.
- NeuralWatt's read is two concurrent requests while its control is one,
  so that pair differs in request count as well as in what it reads.
- Both arms of one host share one connection pool,
  so neither arm isolates a cold TLS handshake.
  The first round's records show the warm-up as it happened rather than a separate cold-start figure,
  which is why the noise allowance in `## Why two seconds stays` uses the spread from rounds 2 to 20.
- The controls' larger bodies bias the controls slower,
  which makes the reported ratio conservative for the quota arm and does not flatter it.
  The quota arm also carries a credential lookup through the registry that the control arm lacks,
  which biases the quota arm slower.

## Raw records

Every timing,
status,
outcome and body size is a JSON line committed beside this file in `doc/quota-read-latency-records/`:

- `synthetic-standard-quota-read-vs-listing-2026-11-19.jsonl`:
  twenty rounds of the Synthetic standard pair.
- `neuralwatt-standard-quota-read-vs-listing-2026-11-19.jsonl`:
  twenty rounds of the NeuralWatt standard pair.
- `neuralwatt-flex-quota-read-vs-listing-2026-11-19.jsonl`:
  six rounds of the NeuralWatt flex pair.
- `summary-2026-11-19.json` and `analysis-2026-11-19.json`:
  per-arm,
  per-endpoint and paired-ratio statistics derived from those rounds.
- `body-sizes-2026-11-19.jsonl`:
  the separately measured body sizes.
- `positive-control.jsonl`:
  the timing harness's known-delay check.

Each arm line carries the round,
the arm,
the reading in milliseconds,
the status and,
for a read,
its per-request header times.
Every figure in this file follows from those lines by nearest-rank percentiles
and a per-round ratio of the read's milliseconds to that round's control.

Before the run,
the timing harness was shown able to report a delay it was given:
three requests to a local server delayed by 250 ms measured 255.4 ms,
251.1 ms and 252.5 ms.

[nw-rates]: https://docs.neuralwatt.com/guides/rate-limits.md
