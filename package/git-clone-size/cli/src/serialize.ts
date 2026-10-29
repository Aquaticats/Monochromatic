import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { colorizeJson, } from './color.ts';
import type { EstimateSnapshot, } from './types.ts';

/**
 Serializes one snapshot to a single line. With color off the line is verbatim
 `JSON.stringify` output (valid JSONL for `jq`/parsers); with color on the same
 line is ANSI-highlighted, and stripping the escapes yields the identical JSON.
 Exactly one line per snapshot either way; no redraw, no multi-line form.
 
 @param snapshot - the snapshot to serialize
 
 @param colorOn - whether to ANSI-highlight the JSON tokens
 
 @returns one line of (optionally colored) JSON, without a trailing newline
 
 @mutates snapshot - `JSON.stringify` may invoke getters, proxy traps, or nested `toJSON` methods
 
 @example
 ```ts
 const line = serializeSnapshot({ snapshot, colorOn: false });
 process.stdout.write(`${line}\n`);
 ```
 */
export function serializeSnapshot(
  {
    snapshot,
    colorOn,
  }: ForeignBorrowed<Readonly<{
    snapshot: EstimateSnapshot;
    colorOn: boolean;
  }>>,
): string {
  /**
   Verbatim JSON line, the machine-readable form.
   */
  const json = JSON.stringify(snapshot,);
  return colorOn ? colorizeJson({ json, },) : json;
}
