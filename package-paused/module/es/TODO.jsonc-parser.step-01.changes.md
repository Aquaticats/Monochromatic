# JSONC parser: step 01 changes

Date: 2025-10-24

Files changed

- `packages/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.ts`

Summary

- Added `scanQuotedString()` local helper: iterative, escape-aware double-quote scanner
  that preserves the original string slice and returns the remaining tail.
- Replaced the string branch in `customParserForArray` primitive parsing
  to use `scanQuotedString` instead of the fragile regex-based approach.

Notes

- Only string parsing within the array primitive path was modified;
  no array/object looping or return contracts changed in this step.
- Strings remain preserved as their original quoted slice in `parsed.value`.
