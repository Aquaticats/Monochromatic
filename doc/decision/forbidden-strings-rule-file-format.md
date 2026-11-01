# Forbidden-strings rule-file format: tail-format sections

Adopted 2026-07-20 by explicit maintainer action request ("Adopt it")
following the completed vet at
`doc/audit/tech-forbidden-strings-rule-file-format-vet-2026-07-20.md`
(governing skill SHA-256
`393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`).

## What was adopted

The tail-format sectioned rule file:
 a single hand-edited file whose
syntax is what `tail --verbose -n +1` over per-rule files would produce.
A constrained `==> name <==` header line opens each rule section;
 the
section body is the rule,
 read by the always-verbose engine directly
(multi-line,
 first-column `#` comments).
 Authors "pretend to be tail"
when editing.
 Normative spec:
`doc/planning/forbidden-strings-tail-rule-format.md`.

## Ranking and rejected alternatives

Weights:
 parser auditability 5 (maintainer-chosen governing axis),
 all
other criteria 1.
 Final:
 tail-format 81.8;
 block form 79.5 (rejected:
no rule-naming slot,
 leaving the rule-identity drift problem open);
NestedText subset 75.0 (rejected:
 its boundary-immunity and ergonomics
wins are the axes the maintainer declined to prioritize);
 TOML 54.5
(rejected:
 roughly 16700-line parsing path plus winnow with an open
soundness report,
 toml-rs/toml#1175,
 at a commit gate).
 Full evidence,
hard gates,
 validation,
 and sensitivity live in the vet report.

## Conditions carried from the vet

- The header-collision mitigation is mandatory,
   strengthened by
  maintainer ruling 2026-07-20:
   any non-header line whose trimmed form
  starts with `==>` fails the load closed,
   and genuine arrow-leading
  content uses the `[=]=> ` reshape in regex bodies (bare literals
  cannot begin with `==>`;
   `[=]` in a literal stays literal).
   Nothing
  arrow-leading is ever silently absorbed (spec sections "Near-header
  fail-closed rule" and "Collision mitigation").
- The bare-literal fork resolved by maintainer correction 2026-07-20:
  every rule,
   bare literals included,
   is its own named section
  (single-significant-line bodies classify by the incumbent two-form
  rule).
   A `.literals` list-section form was initially adopted for
  local-appendix compactness and rejected the same day:
   uniform
  per-rule identity is worth one header line per literal.

## Integration boundary and migration

The scanner's rule loader (`package/forbidden-strings/cli/src/rule/`)
gains the sectioned parser with format autodetection:
 a file whose
first significant line is a strict header parses as tail-format;
otherwise the legacy line-based format applies during the transition.
Sequencing is binary-before-data-files:
 (1) parser lands and a release
ships;
 (2) the local gate and CI move to the released binary;
 (3) the
three live rule files convert to tail-format;
 legacy-format removal is
a later decision.
 An old binary reading a tail-format file misparses
headers as bare literals (a lone `/`-like hazard),
 which is why data
files never convert first.

## Exit and rollback

Before step 3 of the migration,
 rollback is deleting the new parser
path;
 the data files are untouched.
 After conversion,
 rollback means
converting the data files back (mechanical,
 since section bodies are
verbatim engine patterns) and re-releasing.

## Revisit triggers

- The header-collision mitigation proves insufficient in practice
  (a real silent split reaches the gate).
- A finding-identity redesign wants structure the section header cannot
  carry.
- The engine's comment or verbose semantics change,
   invalidating the
  raw-body premise.
