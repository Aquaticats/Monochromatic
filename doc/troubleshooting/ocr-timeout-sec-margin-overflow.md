# OpenCodeReview v1.13.1 accepts `timeout_sec` 9223372036,
then overflows its own +30s margin and fails every HTTP/2 request instantly

Setting the `ocr` provider timeout to the maximum value the tool itself accepts
(`9223372036` seconds) makes every LLM request fail within milliseconds.
The client adds a 30s margin to the timeout as a raw int64 nanosecond addition,
the sum wraps negative,
and Go's HTTP/2 transport treats any nonzero `ResponseHeaderTimeout`
(including a negative one) as an armed,
already expired timer.
The failure text blames the endpoint
(`http2: timeout awaiting response headers`),
which reads as a slow-model problem but is not.

## Symptom

With `timeout_sec` set to `9223372036` on the active provider
(`custom_providers.<name>.timeout_sec` or `providers.<name>.timeout_sec`),
every LLM call against an HTTPS endpoint that negotiates HTTP/2 fails:

```text
[ocr] scan plan failed for package-paused/stylesheet/monochromatic/src/consts/typography.css: Post "https://api.synthetic.new/openai/v1/chat/completions": http2: timeout awaiting response headers (falling back to plan-less)
```

and,
from `ocr llm test`:

```text
Error: llm request failed: Post "https://api.synthetic.new/openai/v1/chat/completions": http2: timeout awaiting response headers
```

Observable properties:

- A full `ocr llm test` fails in about 15s (measured `real 0m15.070s`),
  not after the configured ~292-year timeout.
- `ocr scan` prints the plan-failure line per file and falls back to plan-less,
  then the main task fails the same way.
- Each HTTP attempt dies instantly;
  the ~15s total is the initial attempt plus
  up to five retries with backoff (see "Root cause").
- The symptom reads as "endpoint or model too slow".
  It is not:
  after fixing the value,
  the same deliberately slow model completed `ocr llm test`
  including a tool-call round trip in 20s (see "Harness A").

Any `timeout_sec` in `9223372007..9223372036` triggers this.
`OCR_LLM_TIMEOUT` in the same range goes through the same validator and arithmetic
(`internal/llm/resolver.go:219`).

## Root cause

Walked on v1.13.1,
commit `8eb8334`.

1. The validator accepts up to 9223372036 seconds and advertises that maximum
   in its own rejection message (`internal/llm/resolver.go:230-243`):

   ```go
   // internal/llm/resolver.go:230-243 (v1.13.1)
   func ValidateTimeoutSec(sec int) (time.Duration, error) {
       if sec == 0 {
           return 0, nil
       }
       if sec < 0 {
           return 0, fmt.Errorf("timeout_sec must be non-negative, got %d", sec)
       }
       // Guard against overflow: time.Duration is int64 nanoseconds.
       maxSec := int64(math.MaxInt64 / int64(time.Second))
       if int64(sec) > maxSec {
           return 0, fmt.Errorf("timeout_sec %d overflows time.Duration (max %d)", sec, maxSec)
       }
       return time.Duration(sec) * time.Second, nil
   }
   ```

2. The shared client constructor adds a 30s margin with no overflow guard.
   The margin exists so the per-request context deadline fires first
   (`internal/llm/client.go:48-56`,
   `internal/llm/client.go:67-77`,
   pre-patch):

   ```go
   // internal/llm/client.go:56 (v1.13.1)
   const responseHeaderTimeoutMargin = 30 * time.Second
   ```

   ```go
   // internal/llm/client.go:72-76 (v1.13.1, pre-patch)
   t = t.Clone()
   if timeout > 0 {
       t.ResponseHeaderTimeout = timeout + responseHeaderTimeoutMargin
   }
   return &http.Client{Transport: t}
   ```

3. The sum wraps.
   `9223372036s = 9223372036000000000 ns` fits int64
   (`math.MaxInt64 = 9223372036854775807`),
   but adding `30000000000 ns` yields `9223372066000000000`,
   which wraps to `-9223372007709551616 ns` (about -292.4 years).
   Measured by the source harness ("Harness B"):
   `ResponseHeaderTimeout = -9223372007709551616 ns`.

4. No competing deadline can fire.
   The per-request deadline is `WithRequestTimeout(cfg.Timeout)`
   (`internal/llm/client.go:613`),
   about 292 years out,
   and `time.Time.Add` degrades to wall-only instead of wrapping when the
   monotonic addition leaves the range
   (Go 1.27.2 `src/time/time.go:1182-1189`):

   ```go
   // Go src/time/time.go:1182-1189 (1.27.2)
   te := t.ext + int64(d)
   if d < 0 && te > t.ext || d > 0 && te < t.ext {
       // Monotonic clock reading now out of range; degrade to wall-only.
       t.stripMono()
   } else {
       t.ext = te
   }
   ```

   The header timer is therefore the only deadline that can fire,
   and it is
   broken.
   (An unset or zero `timeout_sec` clamps to a 5-minute default at
   `internal/llm/client.go:595-596`,
   so the bug needs an explicitly huge value.)

5. Go's HTTP/2 transport arms the header timer on any nonzero duration,
   negative included
   (Go 1.27.2 `src/net/http/internal/http2/transport.go:1370-1384`;
   the `ocr` binary embeds the go1.26.9 stdlib per `strings`,
   and "Harness A" demonstrates identical behavior in the shipped binary):

   ```go
   // Go src/net/http/internal/http2/transport.go:1370-1376 (1.27.2)
   if d := cc.responseHeaderTimeout(); d != 0 {
       timer := time.NewTimer(d)
       defer timer.Stop()
       respHeaderTimer = timer.C
       respHeaderRecv = cs.respHeaderRecv
   }
   ```

   `time.NewTimer(negative)` is already expired,
   the `<-respHeaderTimer` branch returns `errTimeout`
   (`transport.go:1384`),
   whose text is `"http2: timeout awaiting response headers"`
   (`src/net/http/internal/http2/http2.go:374`).
   The HTTP/1 transport arms only on a positive duration
   (`src/net/http/transport.go:3101`:
   `if d := pc.t.ResponseHeaderTimeout; d > 0 {`),
   so an HTTP/1-only endpoint instead silently loses the header cap and hangs.

6. Retries amplify one instant failure into the observed ~15s.
   The margin's own comment records that a header-timeout win
   "surfaces as a nil-response transport error that shouldRetry treats as
   retryable,
   so the request would be retried up to 5 more times"
   (`internal/llm/client.go:48-56`).
   Six instant attempts plus exponential backoff matches the measured total.

7. Blast radius is every protocol client and command.
   The helper is shared by the OpenAI,
   OpenAI Responses and Anthropic
   constructors (`internal/llm/client.go:613-614`,
   `internal/llm/client.go:1191`,
   `internal/llm/responses_client.go:52`),
   and the scan plan phase calls through the same shared client
   (`internal/scan/agent.go:887`,
   error print at `internal/scan/agent.go:894`).

## Verification

Version under test:
`ocr --version` reports
`open-code-review v1.13.1 (8eb8334) linux/amd64`,
installed via npm `@alibaba-group/open-code-review`
(platform package `@alibaba-group/ocr-linux-x64`,
native binary `bin/opencodereview`,
`strings` shows `go1.26.9`).
Upstream source checked at tag `v1.13.1`,
commit `8eb8334`.

### Harness A: shipped binary against a real slow provider

Endpoint `https://api.synthetic.new/openai/v1` (HTTPS,
HTTP/2 via ALPN),
model deliberately very slow,
provider configured with `extra_body.stream: true`,
so the post-fix success also exercises the streaming branch.

Failing run (measured 2026-10-17):

```sh
ocr config set custom_providers.Synthetic.timeout_sec 9223372036
time ocr llm test
```

```text
Error: llm request failed: Post "https://api.synthetic.new/openai/v1/chat/completions": http2: timeout awaiting response headers
real    0m15.070s
```

Passing run after the workaround:

```sh
ocr config set custom_providers.Synthetic.timeout_sec 2147483647
time ocr llm test
```

```text
✓ Connection test successful
✓ Tool-call round trip verified
real    0m20.076s
```

Same endpoint,
same slow model:
before the fix the probe fails in 15s
(six instant transport deaths plus backoff);
after it,
a full conversation plus tool round trip succeeds in 20s.

### Harness B: source-level overflow probe

In a disposable clone
(`gh repo clone alibaba/open-code-review <dir> -- --depth 1 --branch v1.13.1`),
add this test to package `llm`:

```go
// internal/llm/overflow_harness_test.go (package llm)
func TestHeaderTimeoutMarginOverflowHarness(t *testing.T) {
    const maxAcceptedSec = 9223372036 // ValidateTimeoutSec's documented maximum
    d, err := ValidateTimeoutSec(maxAcceptedSec)
    if err != nil {
        t.Fatalf("ValidateTimeoutSec(%d) returned error: %v", maxAcceptedSec, err)
    }
    c := httpClientWithHeaderTimeout(d)
    tr, ok := c.Transport.(*http.Transport)
    if !ok {
        t.Fatalf("transport is %T, want *http.Transport", c.Transport)
    }
    if tr.ResponseHeaderTimeout <= 0 {
        t.Fatalf("ResponseHeaderTimeout = %d ns; want positive: timeout+margin overflowed int64", tr.ResponseHeaderTimeout)
    }
}
```

Run `go test ./internal/llm/ -run TestHeaderTimeoutMarginOverflowHarness -count=1 -v`.

Pre-patch output (Go 1.27.2 toolchain building the go1.26.9 module):

```text
overflow_harness_test.go:29: ResponseHeaderTimeout = -9223372007709551616 ns; want positive: timeout+margin overflowed int64
--- FAIL: TestHeaderTimeoutMarginOverflowHarness (0.00s)
```

Post-patch output,
after applying
[ocr-timeout-sec-margin-overflow.patch](ocr-timeout-sec-margin-overflow.patch),
run together with the pre-existing header-timeout test
(`internal/llm/retry_boundary_test.go:735`):

```text
--- PASS: TestHeaderTimeoutMarginOverflowHarness (0.00s)
--- PASS: TestHTTPClientWithHeaderTimeout (0.00s)
ok      github.com/alibaba/open-code-review/internal/llm        0.007s
```

### Value catalogs

Values that behave cleanly:

- unset or `0`:
  5-minute default (`internal/llm/client.go:595-596`).
- `900` (the configuration-docs example):
  header timeout 930s.
- `2147483647` (about 68 years,
  int32 max seconds):
  live-verified by the passing "Harness A" run;
  header timeout `2147483677s`,
  about 7.1e17 ns under `MaxInt64`.
- `9223372006` (largest safe value):
  the sum `9223372036000000000 ns` still fits `MaxInt64`.
  Computed boundary,
  not live-run.

Values that fail:

- `9223372007..9223372036` on an HTTP/2 endpoint:
  instant
  `http2: timeout awaiting response headers` per attempt.
  `9223372036` live-reproduced ("Harness A");
  the range boundary is computed from `MaxInt64`.
- The same range on an HTTP/1-only endpoint (not live-run;
  derived from the `d > 0` arming condition,
  `src/net/http/transport.go:3101`):
  no header cap at all,
  so requests hang until interrupted.
- `9223372037` and above:
  rejected at set time
  (live-verified under a disposable `HOME`):

  ```text
  Error: invalid timeout_sec for custom_providers.stub.timeout_sec: timeout_sec 9223372037 overflows time.Duration (max 9223372036)
  ```

- Negative values:
  rejected by `ValidateTimeoutSec`
  (`internal/llm/resolver.go:234-235`).

## Verified workarounds

1. Keep `timeout_sec` at or below `9223372006` so the margin sum fits int64.
   This machine runs `2147483647` on both configured providers:

   ```sh
   ocr config set custom_providers.Synthetic.timeout_sec 2147483647
   ocr config set custom_providers.hyper.timeout_sec 2147483647
   ```

   Tradeoffs:
   not literally infinite;
   a genuinely stalled connection is now bounded only by manual interruption
   or a caller context (compare upstream issue [#1717][] about stalled requests),
   and about 68 years is far past any real review run.
   The ~4.3x headroom under `MaxInt64` also survives any plausible future
   margin growth in ocr.

2. `9223372006` (about 292 years),
   the tightest safe value.
   Tradeoff:
   it sits exactly one margin below the wrap boundary,
   so a future ocr release that enlarges `responseHeaderTimeoutMargin`
   silently re-breaks the same config.

## What does not work

- The tool's own advertised maximum `9223372036`:
  that is this bug
  (the validator's rejection message names it as `max`).
- `timeout_sec: 0` or unset as "no timeout":
  callers clamp to the 5-minute
  default before the constructor sees the value
  (`internal/llm/client.go:595-596`),
  so the helper's documented "no cap" semantics for nonpositive input are
  unreachable from config.
- Negative values:
  rejected at set time
  (`internal/llm/resolver.go:234-235`).
- Legacy `llm.timeout_sec` while a provider entry resolves:
  the provider path reads `entry.TimeoutSec`
  (`internal/llm/resolver.go:586`),
  and the legacy `llm` block is only a fallback endpoint source.
- Waiting longer,
  or switching to a faster model:
  each attempt dies in milliseconds regardless of endpoint speed,
  and the same slow model succeeds once the value is fixed ("Harness A").

## Upstream filing decision

- Out-of-scope check:
  `.out-of-scope/` holds no OpenCodeReview or `ocr`
  exemption (listing checked 2026-10-17).
- Duplicate search (`gh search issues --repo alibaba/open-code-review --include-prs`,
  queries `timeout awaiting response headers`,
  `timeout_sec overflow`,
  `ResponseHeaderTimeout`):
  hits are [#1161][] (closed:
  `timeout_sec` did not override the SDKs' hardcoded
  10-minute `ResponseHeaderTimeout`;
  fixed by PR [#1162][],
  which introduced the
  margin),
  [#1179][] (merged:
  `config set` support for `timeout_sec`),
  [#1717][] (open:
  stalled requests bypass timeouts),
  and [#1202][] (closed:
  no idle timeout on the response body).
  None mentions the margin overflow or the negative wrap.
  The [#1162][] review thread was read in full:
  it discusses margin sizing and retry semantics,
  never overflow.
  Conclusion:
  a new issue referencing [#1161][] and [#1162][],
  not a comment on either.
- Candidate-fix applicability gate:
  upstream `main`'s `internal/llm/client.go`
  (fetched 2026-10-17 through the GitHub contents API)
  still contains the unguarded sum,
  so neither the newest release nor `main` carries a fix.
- Six-constraint check:
  1. Really upstream's fault:
     yes.
     Their validator advertises a maximum
     (`internal/llm/resolver.go:240`)
     that their own constructor corrupts 30s below.
  2. Can upstream fix it:
     yes.
     Saturating addition,
     prototyped in
     [ocr-timeout-sec-margin-overflow.patch](ocr-timeout-sec-margin-overflow.patch).
  3. Supporting this use case:
     yes.
     [#1161][] exists precisely so `timeout_sec` is honored on slow endpoints,
     and the configuration docs ship a `timeout_sec` example.
  4. Would welcome the contribution:
     yes,
     with disclosure.
     `CONTRIBUTING.md` section "AI-Assisted Development" requires disclosing
     AI/LLM use and the tools/models used;
     `.github/ISSUE_TEMPLATE/bug_report.yml` carries an "AI/LLM Disclosure"
     field;
     bug reports are explicitly invited.
     No ban found in the files checked.
  5. Likely to fix:
     yes.
     [#1161][] received a same-day PR ([#1162][]) with active maintainer review;
     this is an edge case of that merged code;
     no won't-fix signal found.
  6. Prototyped minimal fix:
     yes.
     Disposable clone at tag `v1.13.1` (`8eb8334`);
     "Harness B" fails pre-patch,
     passes post-patch,
     and the pre-existing `TestHTTPClientWithHeaderTimeout` passes post-patch.
- Decision:
  all six constraints hold,
  so the draft issue in "Draft issue" is fileable as-is.
  Not filed:
  the default policy is to not file,
  filing is an external action awaiting the user's decision,
  and the filer must complete the AI/LLM disclosure truthfully.

## Draft issue

Fileable as-is once the filer completes the disclosure field.

~~~md
**Title:** `timeout_sec` near the documented maximum overflows the +30s
`ResponseHeaderTimeout` margin, failing every HTTP/2 request instantly

**Labels:** bug

**Version:** v1.13.1 (`8eb8334`); `main` as of 2026-10-17 carries the same code.

**LLM Provider:** custom provider, protocol `openai`,
HTTPS endpoint negotiating HTTP/2.

**Description:**

`ValidateTimeoutSec` accepts up to 9223372036 seconds, and its rejection
message advertises that maximum (`internal/llm/resolver.go:230-243`).
`httpClientWithHeaderTimeout` then computes
`ResponseHeaderTimeout = timeout + responseHeaderTimeoutMargin` as a plain
int64 nanosecond addition (`internal/llm/client.go:56` and `:74`).
For every `timeout_sec` in `9223372007..9223372036` the sum wraps negative
(measured: `9223372036` gives `-9223372007709551616 ns`).
Go's HTTP/2 transport arms its header timer on `d != 0`, so the negative
duration fires immediately: every request fails with
`http2: timeout awaiting response headers`, `shouldRetry` treats the
nil-response transport error as retryable, and one logical call burns six
instant attempts (about 15s total).
The per-request `WithRequestTimeout` deadline (about 292 years) can never fire
first, so nothing masks the broken timer.
HTTP/1-only endpoints fail the other way: the `d > 0` arming check silently
drops the cap and requests hang.

**Reproduction:**

```sh
ocr config set provider myprovider   # any openai-protocol provider on HTTPS/h2
ocr config set custom_providers.myprovider.timeout_sec 9223372036
time ocr llm test
# Error: llm request failed: Post ".../chat/completions": http2: timeout awaiting response headers
# real ~15s (six instant attempts plus backoff), regardless of endpoint speed
```

Source-level probe (fails pre-fix, passes with the suggested patch):

```go
d, _ := ValidateTimeoutSec(9223372036)   // accepted: "max 9223372036"
c := httpClientWithHeaderTimeout(d)      // package llm
// c.Transport.(*http.Transport).ResponseHeaderTimeout == -9223372007709551616
```

**Suggested fix:** saturate the sum in `httpClientWithHeaderTimeout`
(`internal/llm/client.go:74`):

```go
headerTimeout := timeout + responseHeaderTimeoutMargin
if headerTimeout < timeout {
    // int64 nanosecond overflow: saturate instead of wrapping negative,
    // which would arm Go's HTTP/2 header timer in the already-expired state.
    headerTimeout = time.Duration(math.MaxInt64)
}
t.ResponseHeaderTimeout = headerTimeout
```

Alternative: tighten the `ValidateTimeoutSec` cap to
`(math.MaxInt64 - int64(responseHeaderTimeoutMargin)) / int64(time.Second)`
(9223372006) and update the error text.
The saturating fix additionally protects the `OCR_LLM_TIMEOUT` env path and any
future caller that skips validation.

**AI/LLM Disclosure:** AI-assisted.
Diagnosis, source trace, live reproduction, and the prototype patch were
produced by a DeepSeek Harness agent at the request of the person running this
configuration; a human observed the original symptom and reviewed the recorded
commands and outputs.
~~~

[#1161]: https://github.com/alibaba/open-code-review/issues/1161
[#1162]: https://github.com/alibaba/open-code-review/pull/1162
[#1179]: https://github.com/alibaba/open-code-review/pull/1179
[#1717]: https://github.com/alibaba/open-code-review/issues/1717
[#1202]: https://github.com/alibaba/open-code-review/issues/1202
