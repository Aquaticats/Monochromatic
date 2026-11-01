# Literal replacement text in generated harnesses

The Windows trial harness initially failed dprint with `invalid indentation`.
A JavaScript replacement string contained literal PowerShell `$` followed by a quote.
Replacement-string expansion inserted the matched string's suffix into the generated YAML.
A callback replacer preserved the original PowerShell commands,
and the corrected workflow passed formatting and command-preservation checks.
The [diagnosis](../troubleshooting/depot-native-nextest-completion.md)
and [route evidence](../audit/depot-coherent-native-routes-2026-10-17.evidence.json)
retain the controls.

## Proposed clarification

Tighten the existing `AGENTS.md` rule `SYB` without adding a new rule:

```text
# Proposed AGENTS.md replacement; not applied
SYB: Cross-syntax text obeys destination grammar.
Encode at final interpolation; literal JS replacement uses callbacks.
Never invent comment-string DSLs for relations types or ASTs can express.
```

Keeping the current wording avoids adding a language-specific example,
but leaves replacement-string grammar implicit.
The proposed wording makes the observed boundary explicit,
but requires readers to distinguish literal text from intentional replacement patterns.
Ranking:
clarification > unchanged wording,
because the explicit literal-text condition covers this demonstrated failure while retaining deliberate substitutions.
The proposal does not change application code or the current rule.
