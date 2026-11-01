/**
 JSON-serializable merge cases with their documented results.

 The seed of a later export for the Rust unified-linter merge (Q13 in
 `doc/handover/deepmerge-ts-hardening.md`): every input and expected output is
 plain JSON, so the list can be written to a file verbatim. Here the cases
 check the model and the build under test against the same expectations.

 JSON has no `undefined`, Sets, or Maps; those rules stay covered by the
 properties. JSON-ness is checked at runtime by a round-trip test in
 `./model.unit.test.ts` rather than by a recursive JSON type, whose `null`
 member this repo's no-nullish-union rule forbids.

 Array elements are strings, not numbers above 2, so the data needs no magic
 number constants; the merge rules under test do not depend on element type.

 @module
 */

/**
 One corpus entry.
 */
export type JsonCase = {
  /**
   Rule this case pins, phrased as the documented behaviour.
   */
  readonly name: string;
  /**
   Arguments in call order.
   */
  readonly inputs: readonly unknown[];
  /**
   Documented merge result.
   */
  readonly expected: unknown;
};

/**
 Corpus, one case per documented rule plus the README example's JSON subset.
 */
export const JSON_CASES: readonly JsonCase[] = [
  {
    name: 'a single input is returned unchanged',
    inputs: [{ a: 1, },],
    expected: { a: 1, },
  },
  {
    name: 'records merge key-wise and the later scalar wins',
    inputs: [
      {
        a: 1,
        b: 2,
      },
      {
        a: 3,
        c: 4,
      },
    ],
    expected: {
      a: 3,
      b: 2,
      c: 4,
    },
  },
  {
    name: 'keys appear in first-seen order',
    inputs: [
      { b: 1, },
      {
        a: 2,
        b: 3,
      },
    ],
    expected: {
      b: 3,
      a: 2,
    },
  },
  {
    name: 'nested records merge recursively',
    inputs: [
      { r: {
        x: 1,
        y: 2,
      }, },
      { r: {
        y: 3,
        z: 4,
      }, },
    ],
    expected: { r: {
      x: 1,
      y: 3,
      z: 4,
    }, },
  },
  {
    name: 'arrays concatenate in argument order',
    inputs: [
      [
        'p',
        'q',
      ],
      [
        'q',
        'r',
      ],
      ['s',],
    ],
    expected: [
      'p',
      'q',
      'q',
      'r',
      's',
    ],
  },
  {
    name: 'arrays inside records concatenate',
    inputs: [
      { list: [1,], },
      { list: [2,], },
    ],
    expected: { list: [
      1,
      2,
    ], },
  },
  {
    name: 'array elements are not merged with each other',
    inputs: [
      [{ a: 1, },],
      [{ b: 2, },],
    ],
    expected: [
      { a: 1, },
      { b: 2, },
    ],
  },
  {
    name: 'differing kinds resolve to the last value',
    inputs: [
      { v: [1,], },
      { v: { a: 1, }, },
    ],
    expected: { v: { a: 1, }, },
  },
  {
    name: 'a kind mismatch anywhere among n inputs resolves to the last value',
    inputs: [
      null,
      [1,],
      [2,],
    ],
    expected: [2,],
  },
  {
    name: 'null is a value and replaces earlier values',
    inputs: [
      { a: { b: 1, }, },
      { a: null, },
    ],
    expected: { a: null, },
  },
  {
    name: 'README example shape, JSON subset, with string array elements',
    inputs: [
      {
        record: {
          prop1: 'value1',
          prop2: 'value2',
        },
        array: [
          'one',
          'two',
          'three',
        ],
      },
      {
        record: {
          prop1: 'changed',
          prop3: 'value3',
        },
        array: [
          'two',
          'three',
          'four',
        ],
      },
    ],
    expected: {
      record: {
        prop1: 'changed',
        prop2: 'value2',
        prop3: 'value3',
      },
      array: [
        'one',
        'two',
        'three',
        'two',
        'three',
        'four',
      ],
    },
  },
];
