# Can LLMs detect their own inference degradation from the inside?

Short answer: **yes, with caveats**.
The claim "there's no way for me to distinguish degraded inference from unlucky sampling from the inside" is too strong.
Several practical techniques exist, ranging from fully internal (no API needed) to API-based self-probing.

## What "from the inside" means

Two distinct scopes:

- **Hidden-state access** (model internals): requires custom inference infrastructure;
not available to agents running via API
- **Text-output self-probing** (behavioral): the model or agent runs known tasks and checks its own answers;
works with any API client

This package implements the second approach.

## Research backing

### Hidden-state approaches (not applicable to agents, but proves the concept)

- **Gnosis** (arxiv 2512.20578): lightweight circuits trained on internal hidden states achieve 0.96 AUROC predicting
model failures.
  Proves models carry detectable signals about their own reliability.
- **Internal Flow Signatures** (arxiv 2602.01897): audits token-level decision trajectories to localize where errors
originate inside the model.
- **Chain-of-Embedding** (arxiv 2410.13640): uses latent space representations to detect incorrect responses without
generating output.

### Text-output approaches (applicable to agents)

- **Self-consistency sampling** (AAAI 2025, arxiv 2402.13904): running the same prompt N times and measuring agreement.
  Higher disagreement = lower reliability = possible degradation.
  Works even when the model is degraded, because different error types are often uncorrelated.
- **Confidence-Informed Self-Consistency** (arxiv 2502.06233): weighting responses by self-assessed confidence.
  Achieves same accuracy with 40% fewer samples.
- **Self-Evaluation via Token-Level Prediction** (arxiv 2312.09300):
reformulating open-ended tasks as token-level predictions enables self-scoring.
  Correlates better with quality than likelihood-based metrics.
- **Calibrated Reflection** (ACL TrustNLP 2025): prompting the model to reflect on its outputs improves calibration.

### Introspective awareness (not applicable to agents)

- **Emergent Introspective Awareness** (Anthropic, October 2025):
concept injection experiments show Claude Opus 4/4.1 can sometimes detect
and identify concepts injected directly into their neural activations.
  Succeeds only ~20% of the time, requires activation-level access (injecting known vectors into model internals),
  and fails silently or hallucinates at wrong injection strengths.
  Without activation-level ground truth, asking a model "are you degraded?" falls into confabulation territory;
  models can act introspective without being introspective.
  Reinforces the design choice to use objective behavioral probes rather than self-reported model state.

### Real-world degradation incidents

- **August 2025**: Anthropic confirmed 56.5 hours of degraded inference from a faulty infrastructure upgrade
(status.anthropic.com).
  The status page was updated only after the fact; users detected it first via behavioral changes.
- **January 2026** (GitHub #21046): community-reported "shadow downgrade": laziness, context loss,
constraint violations.
- **February 2026** (GitHub #27574): planning failures, debugging loops, introducing new bugs.
  Matches the exact behavioral pattern that prompted this package.

## Why self-probing works

Degraded inference typically affects the model in broad, systematic ways:

- Lower effective temperature (more repetitive, less creative)
- Reduced instruction compliance (ignores constraints, adds unrequested content)
- Behavioral drift toward passivity (describing instead of acting)
- Impaired reasoning on even simple tasks

These are testable.
A healthy model scores 1.0 on "what is 7 * 8?" every time.
A healthy model outputs `<<CANARY_OK>>` exactly when asked.
A healthy model writes a function when asked to write a function, not a description of what a function would look like.

If any of these fail, or if self-consistency drops (different answers on repeated runs), something is wrong:
either with the model weights being served, the quantization level, or the routing infrastructure.

## Limitations

- **Self-grading under degradation**: a sufficiently degraded model might misjudge its own outputs.
  Mitigated by using objective scoring (exact string match, JSON parse) rather than asking the model to evaluate itself.
- **Sampling vs systemic**: a single probe failure could be sampling noise.
  Mitigated by running each probe multiple times (self-consistency)
  and requiring consistent failure to flag degradation.
- **Probe coverage**: the probes test specific capabilities; degradation could affect untested dimensions.
  Mitigated by covering three orthogonal axes (factual, instruction-following, behavioral).
- **API cost**: running 9 probes x 3 consistency runs = 27 API calls per check.
  At temperature 0 with short max_tokens, this is negligible cost.

## How this package uses the research

The `inference-canary` package implements a text-output self-probing approach:

1. **Canary probes**: 9 probes across 3 categories (factual, instruction, behavioral) with objective scoring functions
2. **Self-consistency**: each probe runs N times (default 3); consistency is tracked alongside scores
3. **Degradation detection**: overall score below threshold (default 0.8) flags likely degradation
4. **Categorical diagnosis**: per-category scores pinpoint which capabilities are affected
5. **Actionable output**: report includes specific weak probes and inconsistencies
