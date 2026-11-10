# agent-harness-shared-text-scan

Regex-free text-scanning helpers shared by agent harness integrations.

This package lives under `package/agent-harness-shared/` because current
consumers are Pi and Claude Code plugin code paths.
It is a holding package for the DRY extraction in
`doc/planning/dry-pi-plugins-claude-code-plugin.md`.
Long-term generic module splits are tracked in GitHub issue #276.

## Pi dependency trust

PJG:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Exports

The package exports:

- character predicates:
  `isDigit`,
  `isLowerAlpha`,
  `isUpperAlpha`,
  `isAlphaNum`,
  `isWordChar`,
  and `isWhitespace`
- token splitting:
  `splitWhitespace`
- word-boundary phrase lookup:
  `containsWordBoundedPhrase`,
  `containsAnyOfWordBounded`,
  and `PHRASE_NOT_FOUND`
- delimiter and line stripping:
  `stripBetweenDelims` and `stripLinesStartingWith`
- delimiter validation:
  `EmptyDelimiterError`,
  thrown by `stripBetweenDelims` when a delimiter literal is empty

## Usage

```ts
import {
  splitWhitespace,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

splitWhitespace('  a b  ');
// ['a', 'b']
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/agent-harness-shared/text-scan:test:unit
mise run //package/agent-harness-shared/text-scan:lint
```
