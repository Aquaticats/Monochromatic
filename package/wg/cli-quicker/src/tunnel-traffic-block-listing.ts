import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for reading the table listing of `nft`.
 */
const l = tagged({ tag: 'tunnel-traffic-block-listing', },);

/**
 What a listing of the nft tables shows about one table.
 */
export type TablePresence = 'present' | 'absent' | 'unknown';

/**
 Reports whether a value is a non-null object.

 @param value - Parsed JSON value.

 @returns Whether it is an object whose fields can be read.

 @example
 ```ts
 isRecord({ table: {} }); // true
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reports whether a listing entry is the table of the given family and name.

 @param entry - One element of the `nftables` array.

 @param family - Address family of the table.

 @param name - Name of the table.

 @returns Whether the entry describes that table.

 @example
 ```ts
 isTable({ entry: { table: { family: 'inet', name: 'x' } }, family: 'inet', name: 'x' }); // true
 ```
 */
function isTable({
  entry,
  family,
  name,
}: {
  readonly entry: unknown;
  readonly family: string;
  readonly name: string
},): boolean {
  return isRecord(entry,) && isRecord(entry.table,)
    && (entry.table
      .family
      === family)
    && (entry.table
      .name
      === name);
}

/**
 Reads the JSON that `nft --json list tables` prints and tells whether one table is in it.
 The answer rests on the structure of the listing,
 not on any message text:
 text such as `No such file or directory` is worded by the system's locale.
 Anything that is not the listing of the tables,
 such as text that is not JSON,
 is `unknown`,
 because it does not show the table gone.

 @param text - Standard output of `nft --json list tables`.

 @param family - Address family of the table.

 @param name - Name of the table.

 @returns Whether the table is listed,
 not listed,
 or the text is not a listing.

 @example
 ```ts
 tablePresence({ text: '{"nftables":[]}', family: 'inet', name: 'wg-quicker-wg0-blocked' }); // 'absent'
 ```
 */
export function tablePresence({
  text,
  family,
  name,
}: {
  readonly text: string;
  readonly family: string;
  readonly name: string
},): TablePresence {
  /**
   Parsed output.
   */
  const parsed = parseJson({ text, },);
  if ((!isRecord(parsed,)) || (!Array.isArray(parsed.nftables,)))
    return 'unknown';
  return parsed.nftables
    .some(function isTheTable(entry: unknown,): boolean {
      return isTable({
        entry,
        family,
        name,
      },);
    },)
    ? 'present'
    : 'absent';
}

/**
 Parses JSON text without throwing.

 @param text - Text to parse.

 @returns The parsed value,
 or `undefined` when the text is not JSON.

 @example
 ```ts
 parseJson({ text: '{"a":1}' }); // { a: 1 }
 ```
 */
function parseJson({ text, }: { readonly text: string; },): unknown {
  try {
    return JSON.parse(text,) as unknown;
  }
  catch (error) {
    l.debug(`nft did not print JSON: ${String(error,)}`,);
    return undefined;
  }
}
