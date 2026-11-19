/**
 Refusal texts the prototype's stand-in members return, each with the capture it came from.
 @module
 */

//region Captured member refusals

/**
 Synthetic's subscription refusal, quoted form.

 Capture: `pi-runs/kimi-review-since-alpha8-landing-waits-1/summary.json`, field `modelError`,
 2026-11-10T17:43:34-05:00, 138 characters, recorded in `doc/captures.md` under "Synthetic's
 subscription refusal, quoted form".
 */
export const SYNTHETIC_QUOTA_REFUSAL: string = '429 "You\'ve exceeded your subscription rate limits. '
  + 'Upgrade, or try again later. You can view your usage at https://synthetic.new/billing"';

/**
 NeuralWatt's overage refusal, the 429 a writer member returns when its concurrent budget is full.

 Capture: `pi-runs/worktree-remove-does-not-wait-on-a-read-only-git-2/summary.json`, field
 `modelError`, 2026-11-10T21:11:23-05:00, 795 characters, recorded in `doc/captures.md` under
 "NeuralWatt's overage refusal" and repeated in `src/quota-routing.unit.test.ts`.
 */
export const NEURALWATT_OVERAGE_REFUSAL: string = '429: {"type":"rate_limit_error",'
  + '"code":"concurrent_budget_exceeded","message":"Concurrent limit reached for '
  + 'deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. '
  + 'Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it '
  + 'renews. Turn auto-renew back on to restore your account level\'s limits: '
  + 'https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off",'
  + '"retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,'
  + '"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,"context":{"budget":6,'
  + '"in_flight":6,"model":"deepseek-ai/DeepSeek-V4.1-Flash","limit_type":"concurrent",'
  + '"tpm_window_tokens":0,"tpm_limit":5242880,"floor":5,"burst":6,"burst_shed":false}}';

//endregion Captured member refusals

//region Texts with a source but no capture

/**
 Cloudflare 523 as Cloudflare's own error page heads it.

 Source, not a capture: the page heading "Error 523: origin is unreachable"
 (`https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-523/`,
 read 2026-11-19). A proxied API returns that page to the SDK, and Pi formats a status it has no
 body for as `<status>: <text>` (`@earendil-works/pi-ai` dist/utils/error-body.js:139-145).

 The 522 heading reads "Connection timed out", whose text matches Pi's own `timed? out` retry
 pattern (`dist/utils/retry.js`), so 522 would be retried by Pi and would not show a refusal Pi calls
 non-retryable; 523's heading matches no pattern.
 */
export const CLOUDFLARE_523_REFUSAL: string = '523: Error 523: origin is unreachable';

/**
 The overflow body an OpenAI-compatible member returns when the input exceeds its window.

 Source, not a capture: Pi's own overflow detector lists the shape with example values,
 "OpenAI-compatible: \"Input length (265330) exceeds model's maximum context length (262144).\""
 (`@earendil-works/pi-ai` dist/utils/overflow.js, the `OVERFLOW_PATTERNS` documentation block). The
 counts here come from the request the stand-in refuses, so the text reports the request's own size.
 Pi classifies this text as a context overflow and as neither a transient nor a quota error.

 @param inputChars - characters the stand-in received

 @param limitChars - characters the stand-in accepts

 @returns refusal text in the installed Completions adapter's `<status>: <body>` shape

 @example
 ```ts
 overflowRefusal({ inputChars: 162000, limitChars: 140000 });
 ```
 */
export function overflowRefusal({
  inputChars,
  limitChars,
}: {
  readonly inputChars: number;
  readonly limitChars: number;
},): string {
  return `400: {"error":{"message":"Input length (${inputChars}) exceeds model's maximum context `
    + `length (${limitChars}).","type":"invalid_request_error","code":"context_length_exceeded"}}`;
}

//endregion Texts with a source but no capture
