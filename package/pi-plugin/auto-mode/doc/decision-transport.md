# Decision transport

## Responsibility and callers

`src/assessment-sender.ts` is the transport interface used by the native host.
`native/contract/integration/native-batch/runtime.mjs` binds it to the original judgment.
The implementation converts original questions to the Gateway Decisions contract,
partitions provider-limited question batches,
checks complete response attribution,
and returns complete evidence to the decision consumer.

`src/decision-request.ts` preserves selected state and question names.
`src/decision-response.ts` checks serving-model identity,
vocabulary,
and complete answer membership.
`src/assessment-transport.ts` owns the fixed endpoint,
credential isolation,
response consumption,
and caller cancellation.

## Why this module exists

The provider protocol differs from the native question representation.
Combining conversion with permission decisions would let transport behavior accidentally grant permission.
Duplicating conversion at each caller would spread question-limit handling and attribution checks.
The sender therefore hides provider mechanics while returning evidence for the original decision owner.

The HTTP implementation and credential reader are injected.
Production uses the configured direct fetch route;
tests supply local responses and failures through the same interface.
There is no separate test permission path.

## Invariants

The model remains `openai/gpt-6-luna-decisions`.
The route remains `https://api.llmgateway.io/v1/decisions`.
Every response is validated before it becomes evidence.
Missing answers,
wrong model attribution,
invalid bytes,
credentials in request or response,
and cancellation cannot become approval.

The judgment owns the existing attempt and bundle allowance.
The fifteen-second target remains advisory.
The transport does not create a replacement deadline or independently restart its call budget.

`src/assessment-failure.ts` separates a failed HTTP invocation from preparation and protocol failures.
A fetch `TypeError` while the original caller remains active receives a local sanitized category.
An HTTP 503 response receives that category after bounded response consumption and credential checks.
Other HTTP statuses keep the permanent category.
The native judgment may use its existing single retry for a bundle containing only that category.
No upstream message or borrowed cause is retained.
Cancellation,
credential failures,
malformed responses,
and model uncertainty do not qualify through this category.
Retry failure still withholds an automatic answer.

The configured content-aligned replay on 2026-10-19 received an HTTP 503 during tool 18.
The sender classified it as permanent and the host requested confirmation.
The preserved capture contains 35 HTTP 200 responses and one HTTP 503 response.
This is an observed assessment-service failure, not evidence that page storage caused the failure.
[RFC 9110, section 15.6.4](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.6.4)
defines 503 as temporary overload or maintenance.
Retry eligibility uses the known response status, never borrowed error text.
The existing judgment still owns the sole retry, caller cancellation, and physical-call allowance.
No retry loop or new timer is introduced inside fetch.

## Alternatives

Retrying inside fetch would conceal calls from the judgment's accounting.
Retrying uncertain model answers would spend requests without new evidence.
Treating every exception as retryable would include invalid credentials,
protocol failures,
and cancellation.
The retained implementation classifies actual invocation failures at the HTTP seam
and leaves retry ownership with the existing judgment.

## Verification

- `mise run //package/pi-plugin/auto-mode:test:assessment-transport`
  covers bytes,
  credential isolation,
  cancellation,
  and failure sanitization.
- `mise run //package/pi-plugin/auto-mode:test:decision-wire`
  covers conversion and complete response attribution.
- `mise run //package/pi-plugin/auto-mode:test:assessment-bundles`
  covers provider partitions and shared physical-call accounting.
- `mise run //package/pi-plugin/auto-mode:test:native:decision-fallback`
  includes a failed final HTTP invocation followed by approval or denial.
  The `unavailable` and `unavailable-deny` modes exercise an actual HTTP 503 response
  followed by a successful approval or denial response through the maintained sender.
  The approval fixture executes the original group without confirmation;
  the denial fixture executes no member.

The reported Bash-only group's original transport cause remains unknown:
the installed error discarded its underlying exception.
Three unchanged replays receive successful provider answers with `0.88` approval probability.
The injected invocation-failure regression establishes retry behavior,
not the missing historical cause.
