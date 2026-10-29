/**
 Shared sample shape for the fuzz value generators.
 
 Every scalar arbitrary yields a `ValueSample`: the TOML source fragment plus,
 where the projected value is predictable, the native value the parser should
 produce for `key = <text>`. Datetimes omit `value` (the parser's `Date`
 projection is host-zone-shifted, so only parse-success and round-trip are
 asserted for them).
 
 @module
 */

import type { SemanticValue, } from './equality.ts';

/**
 One generated TOML value fragment with its optionally-predicted projection.
 */
export type ValueSample = {
  /**
   TOML source text for the value, suitable on the right of `key = `.
   */
  readonly text: string;
  /**
   Native value the parser is expected to project, when predictable.
   */
  readonly value?: SemanticValue;
};
