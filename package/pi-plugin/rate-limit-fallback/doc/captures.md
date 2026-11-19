# Captures the tests replay

Each capture below is kept on one line, however long, because a wrap would change the text the
tests compare.

Every fixture text this package tests against is recorded here, with the run folder whose
`summary.json` carries it under `${HOME}/temp/agent`, the file’s modification date, and its length
in characters. The tests cite this file instead of a run folder that cleanup removes.

## OpenAI’s policy stop

Two runs carry the same 311 characters, field `modelError`:

- `pi-runs/prewarm-of-a-new-worktree-acts-only-on-the-entry-it-checked-1/summary.json`,
  2026-11-18T21:46:56-05:00.
- `pi-codemode-abort-crash-troubleshooting-2026-11-18/run-1/summary.json`,
  2026-11-18T22:15:17-05:00.

The text alone is recorded, not the wire event that carried it: it begins with the sentence and
carries no status prefix.

```text
This content was flagged for possible cybersecurity risk. If this seems wrong, try rephrasing your request. If you’re doing authorized security work that requires more cyber permissive safeguards, apply for Daybreak access via https://platform.openai.com/settings/organization/status-and-access before retrying.
```

## Synthetic’s subscription refusal, quoted form

- `pi-runs/kimi-review-since-alpha8-landing-waits-1/summary.json`, 2026-11-10T17:43:34-05:00,
  138 characters.

The quoted form `429 "..."` is the other way a recorded refusal begins; ten recorded values carry
it and 201 begin `429: {`.

```text
429 "You've exceeded your subscription rate limits. Upgrade, or try again later. You can view your usage at https://synthetic.new/billing"
```

## NeuralWatt’s overage refusal

- `pi-runs/worktree-remove-does-not-wait-on-a-read-only-git-2/summary.json`,
  2026-11-10T21:11:23-05:00, 795 characters.

It names `retry_strategy.suggested_initial_delay_s`, the provider-given delay the plugin takes as
the member’s hold.

```text
429: {"type":"rate_limit_error","code":"concurrent_budget_exceeded","message":"Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews. Turn auto-renew back on to restore your account level's limits: https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off","retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,"context":{"budget":6,"in_flight":6,"model":"deepseek-ai/DeepSeek-V4.1-Flash","limit_type":"concurrent","tpm_window_tokens":0,"tpm_limit":5242880,"floor":5,"burst":6,"burst_shed":false}}
```
