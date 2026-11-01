# Decision-model trial priorities

## Adopted direction

The user subsequently excluded Perplexity and selected Luna on October 14,
2026.
The user then accepted a `0.80` operating cutoff to reduce manual judgment burden.
The ranking retained in this document is historical;
it does not authorize further Perplexity trials.
The [current admission decision](../decision/pi-auto-mode-semantic-admission.md#current-luna-decision)
governs implementation and Luna-specific verification.

## Purpose

The user asked which other decision models to try on October 14,
2026.
This ranks experiments,
not production technologies.
No candidate is qualified to replace the incumbent under the unchanged semantic admission rule.
The user's follow-up asking for the most promising model retained this ordering.
The October 14 catalog recheck still lists Perplexity's 128-question and 262,144-token limits.
Perplexity leads the next-trial recommendation;
Luna has the strongest measured complete-input serving result among the tested new candidates.
Provider-specific compatibility records retain the actual requests,
authorizations,
and results.

## Candidate coverage

The initial discovery considered Perplexity Decider,
OpenAI Decisions,
Microsoft Decision,
Cloudflare Clef,
and additional catalog leads alongside Jev,
Drex,
SubQ,
and Liquid.
The initial new-model trial priority was Perplexity,
then OpenAI,
then Microsoft.
The user subsequently enabled the latter two on the incumbent gateway,
and their authorized serving and complete-input probes are now complete.

### Perplexity Decider V1.1 27B

The [current serving catalog][perplexity] documents structured state,
typed questions,
a 128-question ceiling,
and a 262,144-token context.
Its request vocabulary resembles the incumbent format.
Those published limits justify checking this 84-question assessment.
They do not prove that its actual rendered input fits,
that a response arrives within five seconds,
or that the estimates meet 0.99.
Access,
private-input authorization,
operational terms,
and semantic qualification remain unverified for this route.

### GPT-6 Luna Decisions

The [Gateway probe record](pi-auto-mode-gateway-decisions-current-contract.md)
now establishes a complete valid 84-answer response within five seconds on one attempt.
The conversion preserves original state and question content exactly as JSON text.
The limitation is semantic admission:
none of those answers passed the existing 0.99 unique-maximum rule.
A further study would need fixed hypotheses and labeled controls,
not retries until confidence rises.

### Microsoft Decision 1

The same [Gateway probe record](pi-auto-mode-gateway-decisions-current-contract.md)
establishes successful access and literal-label classification.
Its complete request failed with a TypeSafe-named 64,000-token boundary.
No answer identifies the underlying model for that rejected request.
Investigating the conversion boundary precedes interpreting it as a model-quality failure.

### Other candidates and incumbents

Cloudflare Clef remains a discovery candidate;
the [inspected serving catalog][clef] included a text-truncation warning.
Its exact current route must demonstrate complete-input retention before a policy experiment.
Additional catalog leads have not completed primary contract verification and are not ranked as ready trials.

Jev remains the incumbent but rejects the tested complete lossless representations.
Drex accepts complete input in the retained studies but has not met the adopted semantic and deadline gates together.
SubQ's measured 16-question cap cannot carry this assessment under three calls.
Liquid serves the control,
rejects the original complete input,
and timed out on the shared-content attempt.
These results remain separate from general claims about model quality.

## Current ranking and limits

For the next new-model experiment,
Perplexity remains first:
its published capacity and familiar request contract offer an untested path through the current compatibility problem.
For additional studies of already exercised models,
Luna precedes Microsoft because Luna returned the complete assessment inside the deadline,
while Microsoft's tested route still rejects it.
Microsoft precedes an unverified Clef route because Microsoft has demonstrated authenticated typed serving
and a concrete rejection to investigate;
complete-input retention for the proposed Clef route remains unverified.

This experiment order does not waive any adoption gate,
authorize additional private-input exports,
or change the current production route.

[perplexity]: https://openrouter.ai/perplexity/pplx-decider-v1.1-27b
[clef]: https://openrouter.ai/cloudflare/clef
