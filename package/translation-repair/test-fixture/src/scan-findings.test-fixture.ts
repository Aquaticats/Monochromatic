import { expect, } from '@monochromatic-dev/test-harness-core/ts';

//region Scan findings
// How a scan over the package's own files fails: with every finding it made,
// one per line. The assertion library's message cuts a list it cannot show in
// a few dozen characters down to its length (`expected [ …(31) ] to deeply
// equal []`), so a reader of a failing scan had to run the scan's walk again
// in a probe to learn what it found. The cut is the default of chai, which
// `module-test` neither raises nor exports (issue 610), so each helper here
// writes the difference itself, every line whole, and then makes the
// comparison the scan made, so a scan passes and fails on the inputs it did.

/**
 One entry of a list a scan compares: a finding's text, a number it counted,
 or a group of texts that belong together.
 */
type ListedEntry = string | number | readonly string[];

/**
 One value of a record a scan compares: a count, a flag, or a list of entries.
 */
type RecordValue = number | boolean | readonly ListedEntry[];

/**
 What a scan's list and the listed one differ by.
 */
type EntryDifference = {
  /**
   Entries the scan found more often than the list holds them, one line per
   extra copy, in the scan's order.
   */
  readonly foundNotListed: readonly string[];

  /**
   Entries the list holds more often than the scan found them, one line per
   missing copy, in the list's order.
   */
  readonly listedNotFound: readonly string[];

  /**
   Whether the two hold the same entries as often, in another order.
   */
  readonly reordered: boolean;
};

/**
 Asserts a scan over the package found nothing, failing with each finding on
 a line of its own.

 @param findings - what the scan found, each naming its own place, so the
 failure reads whole without the scan

 @throws Error listing every finding, when there is any

 @example
 ```ts
 expectNoFindings({ findings: ['cat.ts:4'], },);
 // throws: the scan found these, one per line:
 // cat.ts:4
 ```
 */
export function expectNoFindings({ findings, }: { readonly findings: readonly string[]; },): void {
  if (findings.length > 0)
    throw new Error([
      'the scan found these, one per line:',
      ...findings,
    ].join('\n',),);
  expect(findings,)
    .toEqual([],);
}

/**
 How an entry reads in a failure: a text as it is written, a number or a
 group as JSON.

 @param entry - entry shown

 @returns Its text, whole

 @example
 ```ts
 const shown = entryText({ entry: ['cat.ts#nap', 'kitten.ts#nap',], },); // '["cat.ts#nap","kitten.ts#nap"]'
 ```
 */
function entryText({ entry, }: { readonly entry: ListedEntry; },): string {
  return ((typeof entry) === 'string') ? entry : JSON.stringify(entry,);
}

/**
 The entries of a list in order, as one text two lists compare by: each
 entry's JSON, which tells a text from the number it spells and never holds a
 line feed, so joining them on one keeps the entries apart.

 @param entries - list read

 @returns Its entries' JSON, one per line

 @example
 ```ts
 const order = entryOrder({ entries: ['nap', 3,], },); // '"nap"\n3'
 ```
 */
function entryOrder({ entries, }: { readonly entries: readonly ListedEntry[]; },): string {
  return entries
    .map(function keyOf(entry,): string {
      return JSON.stringify(entry,);
    },)
    .join('\n',);
}

/**
 What a scan's list and the listed one differ by, copy for copy, so an entry
 found twice and listed once shows its extra copy.

 @param findings - what the scan found

 @param listed - what the scan's inventory holds

 @returns Entries on each side the other lacks, and whether the two differ
 only in order

 @throws Error, unreachable, when a listed entry's count is missing after
 every listed entry was counted

 @example
 ```ts
 const { foundNotListed, } = entryDifference({ findings: ['cat.ts#nap',], listed: [], },); // ['cat.ts#nap']
 ```
 */
function entryDifference(
  {
    findings,
    listed,
  }: {
    readonly findings: readonly ListedEntry[];
    readonly listed: readonly ListedEntry[];
  },
): EntryDifference {
  /**
   Copies of each listed entry, by its JSON, that no finding has matched yet.
   */
  const unmatched = new Map<string, number>();
  for (const entry of listed) {
    /**
     The entry's JSON.
     */
    const key = JSON.stringify(entry,);
    unmatched.set(
      key,
      (unmatched.get(key,) ?? 0) + 1,
    );
  }
  /**
   Findings past the copies the list holds.
   */
  const foundNotListed: string[] = [];
  for (const entry of findings) {
    /**
     The finding's JSON.
     */
    const key = JSON.stringify(entry,);
    /**
     Listed copies still unmatched, none where the list never holds the entry.
     */
    const left = unmatched.get(key,) ?? 0;
    if (left === 0) {
      foundNotListed.push(entryText({ entry, },),);
      continue;
    }
    unmatched.set(
      key,
      left - 1,
    );
  }
  /**
   Listed copies no finding matched.
   */
  const listedNotFound: string[] = [];
  for (const entry of listed) {
    /**
     The entry's JSON.
     */
    const key = JSON.stringify(entry,);
    /**
     Copies of it still unmatched.
     */
    const left = unmatched.get(key,);
    if (left === undefined)
      throw new Error('unreachable: every listed entry was counted before any finding was matched',);
    if (left === 0)
      continue;
    listedNotFound.push(entryText({ entry, },),);
    unmatched.set(
      key,
      left - 1,
    );
  }
  return {
    foundNotListed,
    listedNotFound,
    reordered: (foundNotListed.length === 0)
      && (listedNotFound.length === 0)
      && (entryOrder({ entries: findings, },) !== entryOrder({ entries: listed, },)),
  };
}

/**
 The lines a failure prints for two lists that differ: each entry found and
 not listed, then each listed and not found, or, where they hold the same
 entries in another order, both lists whole in their orders.

 @param findings - what the scan found

 @param listed - what the scan's inventory holds

 @param prefix - text opening every line, which names the record key the lists
 sit under, or nothing for a list of its own

 @returns One line per difference, none where the lists are equal

 @example
 ```ts
 const lines = differenceLines({ findings: ['cat.ts#nap',], listed: [], prefix: '', },); // ['found and not listed: cat.ts#nap']
 ```
 */
function differenceLines(
  {
    findings,
    listed,
    prefix,
  }: {
    readonly findings: readonly ListedEntry[];
    readonly listed: readonly ListedEntry[];
    readonly prefix: string;
  },
): readonly string[] {
  /**
   What the two lists differ by.
   */
  const {
    foundNotListed,
    listedNotFound,
    reordered,
  } = entryDifference({
    findings,
    listed,
  },);
  if (reordered) {
    return [
      ...findings.map(function foundLine(entry,): string {
        return `${prefix}found in this order: ${entryText({ entry, },)}`;
      },),
      ...listed.map(function listedLine(entry,): string {
        return `${prefix}listed in this order: ${entryText({ entry, },)}`;
      },),
    ];
  }
  return [
    ...foundNotListed.map(function foundLine(text,): string {
      return `${prefix}found and not listed: ${text}`;
    },),
    ...listedNotFound.map(function listedLine(text,): string {
      return `${prefix}listed and not found: ${text}`;
    },),
  ];
}

/**
 Asserts a scan over the package found exactly the entries its inventory or
 allowlist holds, as often and in the same order, failing with each entry
 found and not listed and each listed and not found on a line of its own, or
 with both lists whole where only their order differs.

 @param findings - what the scan found, each entry naming its own place

 @param listed - what the scan's inventory or allowlist holds, in the order
 the scan reports

 @throws Error listing every difference, when there is any

 @example
 ```ts
 expectFindingsAsListed({ findings: ['cat.ts#nap',], listed: ['cat.ts#purr',], },);
 // throws: the scan's findings differ from the listed ones, one difference per line:
 // found and not listed: cat.ts#nap
 // listed and not found: cat.ts#purr
 ```
 */
export function expectFindingsAsListed(
  {
    findings,
    listed,
  }: {
    readonly findings: readonly ListedEntry[];
    readonly listed: readonly ListedEntry[];
  },
): void {
  /**
   One line per difference.
   */
  const lines = differenceLines({
    findings,
    listed,
    prefix: '',
  },);
  if (lines.length > 0)
    throw new Error([
      'the scan\'s findings differ from the listed ones, one difference per line:',
      ...lines,
    ].join('\n',),);
  expect(findings,)
    .toEqual(listed,);
}

/**
 Whether a record's value is a list of entries rather than a count or a flag.

 @param value - value read

 @returns Whether it is a list

 @example
 ```ts
 const listed = isEntryList(['cat.ts#nap',],); // true
 ```
 */
function isEntryList(value: RecordValue,): value is readonly ListedEntry[] {
  return Array.isArray(value,);
}

/**
 The lines a failure prints for one key of two records: the key with its value
 where only one record holds it, its two values where they differ, or, where
 both values are lists, each entry the lists differ by under the key.

 @param key - key read

 @param found - scan's record, by key

 @param listed - listed record, by key

 @returns One line per difference under the key, none where its values are
 equal

 @throws Error, unreachable, when neither record holds the key, since every key
 read comes from one of them

 @example
 ```ts
 const lines = keyDifferenceLines({ key: 'naps', found: new Map([['naps', 2,],],), listed: new Map([['naps', 1,],],), },);
 // ['"naps": found 2 and listed 1']
 ```
 */
function keyDifferenceLines(
  {
    key,
    found,
    listed,
  }: {
    readonly key: string;
    readonly found: ReadonlyMap<string, RecordValue>;
    readonly listed: ReadonlyMap<string, RecordValue>;
  },
): readonly string[] {
  /**
   The key as JSON, so a key holding a colon or a space reads apart from the
   words after it.
   */
  const label = JSON.stringify(key,);
  /**
   The scan's value under the key, absent where the scan's record lacks it.
   */
  const foundValue = found.get(key,);
  /**
   The listed value under the key, absent where the listed record lacks it.
   */
  const listedValue = listed.get(key,);
  if (listedValue === undefined) {
    if (foundValue === undefined)
      throw new Error(`unreachable: neither record holds the key ${label} read off one of them`,);
    return [`${label}: found ${JSON.stringify(foundValue,)} and not listed`,];
  }
  if (foundValue === undefined)
    return [`${label}: listed ${JSON.stringify(listedValue,)} and not found`,];
  if (isEntryList(foundValue,) && isEntryList(listedValue,)) {
    return differenceLines({
      findings: foundValue,
      listed: listedValue,
      prefix: `${label}: `,
    },);
  }
  return Object.is(
    foundValue,
    listedValue,
  )
    ? []
    : [`${label}: found ${JSON.stringify(foundValue,)} and listed ${JSON.stringify(listedValue,)}`,];
}

/**
 Asserts a record a scan built over the package equals the listed one,
 failing with each key found and not listed, each listed and not found, and
 each whose values differ with both values, on a line of its own; under a key
 whose values are both lists, each entry the lists differ by is a line of its
 own, so a long list is never printed whole beside another to compare by eye.

 @param found - what the scan built: counts, flags or lists by key

 @param listed - what the scan's inventory or allowlist holds by key

 @throws Error listing every difference, when there is any

 @example
 ```ts
 expectRecordAsListed({ found: { 'cat.ts#nap': 2, }, listed: { 'cat.ts#nap': 1, }, },);
 // throws: the scan's record differs from the listed one, one difference per line:
 // "cat.ts#nap": found 2 and listed 1
 ```
 */
export function expectRecordAsListed(
  {
    found,
    listed,
  }: {
    readonly found: Readonly<Record<string, RecordValue>>;
    readonly listed: Readonly<Record<string, RecordValue>>;
  },
): void {
  /**
   The scan's values by key.
   */
  const foundValues: ReadonlyMap<string, RecordValue> = new Map(Object.entries(found,),);
  /**
   The listed values by key.
   */
  const listedValues: ReadonlyMap<string, RecordValue> = new Map(Object.entries(listed,),);
  /**
   One line per difference, key by key, the scan's keys first.
   */
  const lines = [
    ...new Set([
      ...foundValues.keys(),
      ...listedValues.keys(),
    ],),
  ].flatMap(function linesOf(key,): readonly string[] {
    return keyDifferenceLines({
      key,
      found: foundValues,
      listed: listedValues,
    },);
  },);
  if (lines.length > 0)
    throw new Error([
      'the scan\'s record differs from the listed one, one difference per line:',
      ...lines,
    ].join('\n',),);
  expect(found,)
    .toEqual(listed,);
}

//endregion Scan findings
