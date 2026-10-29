/**
 Result-type rows of the historical-recall ledger: every released type fix,
 checked by type-checking the whole sidecar against the npm release before
 the fix (`buggy`) and the release that shipped it (`fixed`).

 Each `control` is upstream's regression assertion (tsd `expectType`, which
 requires identical types) rewritten as a standalone module with an identity
 check, or the issue's repro when the fix added no test; it must fail to
 type-check against `buggy` and pass against `fixed`. An empty `control`
 means none could be written, so a miss on that row is not evidence.

 @module
 */

import type { TypeBug, } from './recall-ledger.ts';

/**
 Identity check shared by the controls; tsd's `expectType` is as strict.
 */
const EQUAL = 'type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;';

/**
 Build a control that merges with `deepmerge` and checks the result type.

 @param setup - Declarations before the call.

 @param call - Expression producing the merged value.

 @param expected - Type the fixed release returns.

 @returns Control module source.

 @example
 ```ts
 mergeControl({ call: 'deepmerge(q, q)', expected: 'Record<string, string>', setup: 'const q: Record<string, string> = {};', });
 ```
 */
function mergeControl(
  {
    setup,
    call,
    expected,
  }: {
    readonly setup: string;
    readonly call: string;
    readonly expected: string;
  },
): string {
  return [
    'import { deepmerge } from \'deepmerge-ts\';',
    EQUAL,
    setup,
    `const merged = ${call};`,
    `const check: Equal<typeof merged, ${expected}> = true;`,
    'export { check };',
    '',
  ].join('\n',);
}

/**
 Record `a` from upstream's `tests/deepmerge.test-d.ts`.
 */
const RECORD_A = 'const a = { foo: "abc", baz: { quux: ["def", "ghi"] }, garply: 42 };';

/**
 String-literal union of 120 members, past the depth the non-tail-recursive
 `UnionToTuple` of 8.0.0 reaches on a leaf that may be `undefined`.
 */
const BIG_UNION = Array.from(
  { length: 120, },
  function member(
    _unused: unknown,
    index: number,
  ) {
    return JSON.stringify(`m${String(index,)}`,);
  },
)
  .join(' | ',);

/**
 Type bugs in release order.
 */
export const TYPE_BUGS: readonly TypeBug[] = [
  {
    broke: 'merging 3+ readonly tuples produced the wrong tuple type',
    buggy: '1.1.6',
    commit: '696a1b2',
    control: mergeControl({
      call: 'deepmerge(e, f, f)',
      expected: '{ foo: [1, 2, 3, "a", "b", "c", "a", "b", "c"] }',
      setup: 'const e = { foo: [1, 2, 3] } as const;\nconst f = { foo: ["a", "b", "c"] } as const;',
    },),
    fixed: '1.1.7',
    id: 'readonly-tuples-3',
    issue: '#16, #20',
    regressionTest: 'tests/types/test.ts `$ExpectType` for `deepmerge(e, f, f)`',
    version: '1.1.7',
  },
  {
    broke: 'readonly inputs and the custom HKT interface did not accept readonly tuples',
    buggy: '1.1.7',
    commit: 'ee59064',
    control: '',
    fixed: '2.0.0',
    id: 'readonly-support',
    issue: '#17',
    regressionTest: 'custom HKT augmentations in tests/deepmerge-custom.test.ts; no consumer-level assertion',
    version: '2.0.0',
  },
  {
    broke: 'a readonly record head was skipped when collecting a property\'s values',
    buggy: '3.0.0',
    commit: 'fc85dfa',
    control: '',
    fixed: '3.0.1',
    id: 'readonly-record-head',
    issue: '#60',
    regressionTest: 'none',
    version: '3.0.1',
  },
  {
    broke: 'custom meta data typings: `MetaMetaData` was not tied to the built-in meta data',
    buggy: '3.0.1',
    commit: '9a881d3',
    control: '',
    fixed: '4.0.0',
    id: 'meta-data-typings',
    issue: '#61',
    regressionTest: 'tests/deepmerge-custom.test.ts key-path examples',
    version: '4.0.0',
  },
  {
    broke: '`DeepMergeMergeFunctionUtils` required its meta-meta-data generic',
    buggy: '4.3.0',
    commit: '944b428',
    control: [
      'import type { DeepMergeMergeFunctionUtils } from \'deepmerge-ts\';',
      'type Utils = DeepMergeMergeFunctionUtils<{ readonly a: 1 }>;',
      'export type { Utils };',
      '',
    ].join('\n',),
    fixed: '5.0.0',
    id: 'utils-default-generic',
    issue: '#304',
    regressionTest: 'none; issue #304 snippet',
    version: '5.0.0',
  },
  {
    broke: 'an optional property merged over a required one gave the wrong result type',
    buggy: '5.1.0',
    commit: 'fa9ace2',
    control: mergeControl({
      call: 'deepmerge(n, o)',
      expected: '{ a: false; b: string | number }',
      setup: 'const n: { a: true; b: string } = { a: true, b: "n" };\nconst o: { a: false; b?: number } = { a: false };',
    },),
    fixed: '6.0.0',
    id: 'optional-over-required',
    issue: '#451',
    regressionTest: 'tests/deepmerge.test-d.ts test17',
    version: '6.0.0',
  },
  {
    broke: 'merging index-signature records added `undefined` to the value type',
    buggy: '6.0.0',
    commit: '5e8b9b6',
    control: mergeControl({
      call: 'deepmerge(q, q)',
      expected: 'Record<string, string>',
      setup: 'const q: Record<string, string> = { a: "a" };',
    },),
    fixed: '6.0.1',
    id: 'index-signature-undefined',
    issue: '#459',
    regressionTest: 'tests/deepmerge.test-d.ts test19',
    version: '6.0.1',
  },
  {
    broke: 'an empty record `{}` argument broke the result type',
    buggy: '6.0.1',
    commit: '6b4ff3f',
    control: mergeControl({
      call: 'deepmerge({}, a)',
      expected: '{ foo: string; baz: { quux: string[] }; garply: number }',
      setup: RECORD_A,
    },),
    fixed: '6.0.2',
    id: 'empty-record',
    issue: '#465',
    regressionTest: 'tests/deepmerge.test-d.ts test20',
    version: '6.0.2',
  },
  {
    broke: 'merging records whose keys are all optional returned `never`',
    buggy: '7.0.0',
    commit: '1832bd0',
    control: mergeControl({
      call: 'deepmerge(r, r)',
      expected: '{ a?: string; b?: number; c?: boolean }',
      setup: 'const r: { a?: string; b?: number; c?: boolean } = { a: "a", b: 1 };',
    },),
    fixed: '7.0.1',
    id: 'all-optional-never',
    issue: '#476',
    regressionTest: 'tests/deepmerge.test-d.ts test23',
    version: '7.0.1',
  },
  {
    broke: 'the bundled declarations typed `utils.defaultMergeFunctions` with the into signatures',
    buggy: '7.0.2',
    commit: 'ca94270',
    control: [
      'import { deepmergeCustom } from \'deepmerge-ts\';',
      'export const mergeCustom = deepmergeCustom({',
      '  mergeOthers: (values, utils) => utils.defaultMergeFunctions.mergeOthers(values.filter((v) => v !== undefined && v !== null)),',
      '  mergeArrays: (values) => values[values.length - 1],',
      '});',
      '',
    ].join('\n',),
    fixed: '7.0.3',
    id: 'default-functions-signature',
    issue: '#482',
    regressionTest: 'none; issue #482 snippet from docs/deepmergeCustom.md',
    version: '7.0.3',
  },
  {
    broke: 'a leaf union with `undefined` ignored `filterValues`, so the result type kept `undefined`',
    buggy: '7.1.3',
    commit: '6d85163',
    control: mergeControl({
      call: 'deepmerge(a, s)',
      expected: '{ foo: string | number; baz: { quux: string[] }; garply: number }',
      setup: `${RECORD_A}\nconst s: { foo: number | undefined } = { foo: undefined };`,
    },),
    fixed: '7.1.4',
    id: 'leaf-filtering',
    issue: '#524',
    regressionTest: 'tests/deepmerge.test-d.ts test24',
    version: '7.1.4',
  },
  {
    broke: 'nested optional properties merged over required ones made the nested keys optional',
    buggy: '7.1.4',
    commit: '349fd14',
    control: mergeControl({
      call: 'deepmerge(u, v)',
      expected: '{ outer: { inner: number } }',
      setup: 'const u: { outer: { inner: number } } = { outer: { inner: 1 } };\nconst v: { outer?: { inner?: number } } = {};',
    },),
    fixed: '7.1.5',
    id: 'nested-optional',
    issue: '#529',
    regressionTest: 'tests/deepmerge.test-d.ts test28',
    version: '7.1.5',
  },
  {
    broke: 'an index signature absorbed the known keys of the same record',
    buggy: '7.1.5',
    commit: 'e86cd8c',
    control: mergeControl({
      call: 'deepmerge(tiIndexable, tiIndexable as Partial<typeof tiIndexable>)',
      expected: '{ foo: string; bar?: string }',
      setup: 'interface IndexableInterface { [key: PropertyKey]: unknown; readonly foo: string; bar?: string; }\nconst tiIndexable: IndexableInterface = { foo: "abc" };',
    },),
    fixed: '7.1.6',
    id: 'index-signature-known-keys',
    issue: '#692',
    regressionTest: 'tests/deepmerge.test-d.ts test28 (7.1.6)',
    version: '7.1.6',
  },
  {
    broke: 'the shipped declarations failed to type-check for consumers on TypeScript 7',
    buggy: '8.0.0',
    commit: 'b307b77',
    control: [
      'import { deepmerge } from \'deepmerge-ts\';',
      `type Big = ${BIG_UNION};`,
      'const x: { v: string } = { v: "a" };',
      'const y: { v: Big | undefined } = { v: undefined };',
      'const merged = deepmerge(x, y);',
      'const check: { v: string } = merged;',
      'export { check };',
      '',
    ].join('\n',),
    fixed: '8.0.1',
    id: 'typescript-7',
    issue: '#714',
    regressionTest: 'tests/consumer-ts7/fixture.ts, which compiles against 8.0.0 under TypeScript 7.0.2; control is a probe of the rewritten tail-recursive UnionToTuple',
    version: '8.0.1',
  },
];
