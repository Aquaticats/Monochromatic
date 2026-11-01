# Assertion message matchers

This note records one boundary distinction for the native test catalogs.

## Test seam versus input gating

A caught-error predicate passed to `node:assert` `throws` or `rejects` is a test
seam:
 it matches a value the code under test already threw,
 after the observed behavior happened.
Replacing a legacy error-message matcher with such a predicate is a lint
cleanup,
 not new validation.

Incoming SDK message inspection is a different boundary:
 parsing or categorizing live steering, prompt, or request text to decide what
 the fixture does next invents behavior the SDK never promised
 (for example,
 treating queued steering text as immediate cancellation).
That class of message-category validation stays forbidden in these catalogs.

## Fixture values

`src/native-producer-lifetime-values.ts` authors the model, context, response,
and stream fixtures as valid SDK values using `normalizeContext` and
`createAssistantMessageEventStream`.
Test-only response correlation names live in fixture-owned weak tables,
never asserted onto SDK input.
The prepared-group capability is deliberately unused and throws if invoked.

## Literal expectations

`messageIncludes` in `src/native-producer-lifetime-fixture.mjs`
builds one caught-error predicate:
`Error.isError(error) && error.message.includes(substring)`.
No regex language,
sentinel,
gap scanner,
or pattern list is kept.

The three former `completed.*superseded` sites now match the stable literal
diagnostic substring `completed and was superseded`
from the actual owner message
(`Response judgment completed and was superseded by a later original judgment`
in `native/contract/collector/request-producer-judgment-consumers/owner-judgments.ts`).
This deliberately tightens those assertions to the expected diagnostic.

It is not claimed to be equivalent to the former regex.
A private discriminating check on `completed\ncompleted superseded`
returned `{"regexTest":true,"firstOccurrence":false}`:
 the regex matches at the second `completed`,
 while a first-occurrence gap emulation rejected.
The emulation was removed rather than extended,
and the assertions now name the expected diagnostic literally.

Expected thrown-value identity checks
(`error === failure`,
cause identity)
stay separate predicates so identity semantics are preserved.
