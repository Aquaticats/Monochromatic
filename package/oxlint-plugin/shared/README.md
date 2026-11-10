# Oxlint shared runtime helpers

Small runtime primitives shared by the Monochromatic oxlint plugin packages.

The package intentionally stays tiny:
 it owns untyped-record guards
and the `@mutates` contract scanner
that multiple shipped plugins need while linting source code.

Character-class predicates are not its own:
 `isWhitespace` and `isWordChar` come from
 `@monochromatic-dev/agent-harness-shared-text-scan`,
 which owns every ASCII character predicate in this repository.
