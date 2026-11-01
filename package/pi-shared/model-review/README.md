# @monochromatic-dev/pi-shared-model-review

Shared structured model-review transport and availability fallback orchestration for Pi plugins.

The package is infrastructure rather than an extension.
It registers no commands,
tools,
renderers,
or lifecycle hooks.
Callers own model selection,
auth resolution,
prompts,
verdict schemas,
and interpretation of valid verdicts.

## Structured attempt

`runStructuredReviewAttempt()` performs one complete reviewer attempt:

- force a caller-named structured-output tool
- collect and validate its arguments
- retry without tools when the model omits the forced tool
- retry direct JSON once more only when the first JSON response is empty
- extract a balanced JSON object from explanatory text
- enforce one timeout across the complete attempt

The caller supplies a `StructuredReviewContract<TVerdict>` with the tool,
strict parser,
and direct-JSON retry prompt builder.
Unexpected tools,
malformed JSON,
contract failures,
timeouts,
and auth or provider failures reject the attempt.

## Availability fallback

`runReviewWithFallback()` runs the initially selected candidate first.
Only an attempt failure activates fallback selection.
The function resolves up to two distinct candidates before either fallback request starts,
runs them concurrently,
and returns the first valid verdict.
A valid denial is a successful verdict and can win the race.
Fallback is availability recovery,
not consensus voting.

`ReviewUnavailableError` reports every attempted candidate
and normalized failure diagnostic
when no candidate returns a valid verdict.

## Consumer boundary

Auto-mode uses this package for its structured guard verdict transport.
The repository-owned goal extension uses it for independent completion review.
Neither consumer imports the other plugin's entry point or user-facing policy.
