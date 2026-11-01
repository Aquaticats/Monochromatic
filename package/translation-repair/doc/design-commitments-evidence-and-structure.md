# Design commitments: evidence and structure

The commitments on what counts as evidence,
how a document's structure is detected,
how refusals are handled,
and how text entering a prompt is fenced.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **No single model output is a decision point.**
  Every decision is either deterministic code
  or an aggregate over independent model calls from different vendor families.
- **Issues carry verifiable evidence.**
  Spans and insertion anchors reference stable node IDs and offsets against a hashed base document;
  claims failing deterministic validation are discarded.
- **The source is not ground truth.**
  Suspected source transcription errors,
  interpretive ambiguity,
  and alignment failures are first-class issue states that can block correction and preserve safer translations.
- **Structure is detected per document,
  never assumed per class.**
  Footnote handling activates on detected markers (open convention set:
  `〔1〕`,
  `[^1]`,
  `[1]`);
  unrecognized conventions become findings for human confirmation,
  not silent misparses.
- **Refusals are handled reactively,
  never predicted.**
  Content is never pre-classified for sensitivity;
  refusals reroute across model families and feed a measured scorecard.
- **Text entering a prompt is fenced against its own content.**
  `selectFence` chooses a delimiter strictly longer than any run inside every string a prompt encloses.
  A fixed delimiter would let a translation containing a setext heading
  underline close its own block and have the rest of its text read as instructions.
