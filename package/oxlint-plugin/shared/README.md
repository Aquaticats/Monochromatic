# Oxlint shared runtime helpers

Small runtime primitives shared by the Monochromatic oxlint plugin packages.

The package intentionally stays tiny:
 it owns the character-class predicates,
the whitespace-delimited token scan,
the TSDoc block-tag list
and untyped-record guards
that multiple shipped plugins need while linting source code.
