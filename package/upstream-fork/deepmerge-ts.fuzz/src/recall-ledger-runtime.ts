/**
 Runtime rows of the historical-recall ledger: every released runtime fix in
 deepmerge-ts history (CHANGELOG `Bug Fixes`, `fix:` commits, closed bug
 issues), each re-created on top of v8.0.2 or taken from the fix commit's
 parent, with a control ported from upstream's regression test.

 Transplants re-apply the historical fault to today's code at the place the
 fix now lives, so the sidecar runs against v8.0.2's API and semantics and
 only the fault differs.

 @module
 */

import type { RuntimeBug, } from './recall-ledger.ts';

/**
 Runtime bugs in release order.
 */
export const RUNTIME_BUGS: readonly RuntimeBug[] = [
  {
    broke: '`deepmerge()` with no arguments returned `{}` instead of `undefined`',
    commit: '105b646',
    id: 'empty-call-record',
    issue: '',
    regressionTest: 'tests/deepmerge.test.ts "return undefined when nothing to merge"',
    reproduces: function emptyCallReturnsRecord(library,) {
      /**
       Zero-argument call, typed to return a value so the check reads it.
       */
      const mergeNothing: () => unknown = library.deepmerge;
      return mergeNothing() !== undefined;
    },
    source: {
      edits: [{
        count: 1,
        file: 'deepmerge.ts',
        find: '  ): DeepMergeHKT<Ts, GetDeepMergeFunctionsURIs<PMF>, MetaData> {\n    return mergeUnknowns<',
        replace: '  ): DeepMergeHKT<Ts, GetDeepMergeFunctionsURIs<PMF>, MetaData> {\n    if (objects.length === 0) {\n      return {} as never;\n    }\n    return mergeUnknowns<',
      },],
      kind: 'transplant',
    },
    version: '1.1.0',
  },
  {
    broke: 'a later input\'s inherited or non-enumerable property joined the merge (`in` instead of an own enumerable check)',
    commit: '0967070',
    id: 'inherited-property',
    issue: '#8',
    regressionTest: 'none in the fix commit; enumerable-key test added later in a8de1ca',
    reproduces: function inheritedPropertyMerged(library,) {
      return (typeof Reflect.get(
        library.deepmerge(
          { toString: 1, },
          {},
        ),
        'toString',
      )) === 'function';
    },
    source: {
      edits: [{
        count: 1,
        file: 'utils.ts',
        find: 'return typeof object === "object" && Object.prototype.propertyIsEnumerable.call(object, property);',
        replace: 'return typeof object === "object" && property in object;',
      },],
      kind: 'transplant',
    },
    version: '1.1.1',
  },
  {
    broke: 'a `__proto__` own key was assigned with `result[key] =`, replacing the result\'s prototype',
    commit: 'd637db7',
    id: 'proto-assignment',
    issue: '',
    regressionTest: 'tests/deepmerge.test.ts "prototype pollution" (d637db7)',
    reproduces: function protoReplacesPrototype(library,) {
      return Object.getPrototypeOf(library.deepmerge(
        {},
        JSON.parse('{"__proto__":{"polluted":1}}',),
      ),) !== Object.prototype;
    },
    source: {
      edits: [{
        count: 3,
        file: 'defaults/vanilla.ts',
        find: 'if (key === "__proto__") {',
        replace: 'if (key === "__proto__" && false) {',
      },],
      kind: 'transplant',
    },
    version: '4.0.2',
  },
  {
    broke: 'non-enumerable symbol keys were collected, so they reached the result (as `undefined` on the general path)',
    commit: '3363570',
    id: 'hidden-symbol-key',
    issue: '',
    regressionTest: 'tests/deepmerge.test.ts "enumerable keys" (a8de1ca)',
    reproduces: function hiddenSymbolReachesResult(library,) {
      /**
       Non-enumerable symbol key.
       */
      const hidden = Symbol('non-enumerable key that must never reach a merge result',);
      /**
       Record holding the hidden key.
       */
      const first = Object.defineProperty(
        {},
        hidden,
        {
          enumerable: false,
          value: 1,
        },
      );
      return Object.getOwnPropertySymbols(library.deepmerge(
        first,
        { b: 1, },
      ),)
        .includes(hidden,);
    },
    source: {
      edits: [
        {
          count: 1,
          file: 'utils.ts',
          find: 'if (Object.prototype.propertyIsEnumerable.call(currentObject, symbol)) {',
          replace: 'if (typeof symbol === "symbol") {',
        },
        {
          count: 1,
          file: 'utils.ts',
          find: 'const symbols = Object.getOwnPropertySymbols(object).filter((symbol) =>\n    Object.prototype.propertyIsEnumerable.call(object, symbol),\n  );',
          replace: 'const symbols = Object.getOwnPropertySymbols(object);',
        },
      ],
      kind: 'transplant',
    },
    version: '4.0.4',
  },
  {
    broke: '`deepmergeInto` defined a `__proto__` own key on its internal reference instead of the target',
    commit: '6b04863',
    id: 'into-proto-wrapper',
    issue: '',
    regressionTest: 'tests/deepmerge-into.test.ts own `__proto__` test; the fix dropped its expected throw',
    reproduces: function intoTargetMissesProtoKey(library,) {
      /**
       Into target.
       */
      const intoTarget = {};
      library.deepmergeInto(
        intoTarget,
        JSON.parse('{"__proto__":{"x":1}}',),
      );
      return !Object.hasOwn(
        intoTarget,
        '__proto__',
      );
    },
    source: {
      edits: [{
        count: 2,
        file: 'defaults/into.ts',
        find: 'Object.defineProperty(mut_target.value, key, {',
        replace: 'Object.defineProperty(mut_target, key, {',
      },],
      kind: 'transplant',
    },
    version: '6.0.3',
  },
  {
    broke: 'an `undefined` among 3+ values sent the merge to `mergeOthers`, so the last record replaced the rest',
    commit: '0784f63',
    id: 'undefined-middle',
    issue: '#460',
    regressionTest: 'tests/deepmerge.test.ts "undefined doesn\'t intefer with merging" (0784f63)',
    reproduces: function undefinedMiddleReplaces(library,) {
      return Reflect.get(
        Reflect.get(
          library.deepmerge(
            { a: { b: 1, }, },
            { a: undefined, },
            { a: { c: 3, }, },
          ),
          'a',
        ),
        'b',
      ) !== 1;
    },
    source: {
      edits: [{
        count: 1,
        file: 'deepmerge.ts',
        find: '  const type = getObjectType(filteredValues[0]);\n',
        replace: '  if (values.some((value) => getObjectType(value) !== getObjectType(filteredValues[0]))) {\n    return mergeOthers<U, M, MI>(filteredValues, utils, meta) as DeepMergeHKT<Ts, Fs, M>;\n  }\n  const type = getObjectType(filteredValues[0]);\n',
      },],
      kind: 'transplant',
    },
    version: '7.0.0',
  },
  {
    broke: '`deepmergeInto` wrote merged elements into the first source\'s nested array, Set, or Map when the target lacked the key',
    commit: '2cd7824',
    id: 'into-source-leak',
    issue: '',
    regressionTest: 'tests/deepmerge-into.test.ts "does not mutate nested array input containers" and seven siblings (2cd7824)',
    reproduces: function intoMutatesSource(library,) {
      /**
       Source whose nested array must stay untouched.
       */
      const source = { a: [1,], };
      library.deepmergeInto(
        {},
        source,
        { a: [2,], },
      );
      return source.a
        .length
        !== 1;
    },
    source: {
      buggyTree: '2cd7824-parent',
      fixedTree: '2cd7824',
      kind: 'parent',
      shimFastUnsafe: false,
    },
    version: '8.0.0',
  },
  {
    broke: 'a cyclic input recursed until the stack overflowed (published as GHSA-ggr8-5vv4-36mx)',
    commit: '3984927',
    id: 'cycle-stack-exhaustion',
    issue: '#707, #715',
    regressionTest: 'tests/deepmerge-circular.test.ts "merging simple circular objects" and siblings (3984927)',
    reproduces: function cycleOverflowsStack(library,) {
      /**
       Record that holds itself.
       */
      const looped: Record<string, unknown> = {};
      looped.self = looped;
      /**
       Second self-holding record, so the merge recurses at `self`.
       */
      const other: Record<string, unknown> = { x: 1, };
      other.self = other;
      try {
        library.deepmerge(
          looped,
          other,
        );
        return false;
      } catch (error) {
        return error instanceof RangeError;
      }
    },
    source: {
      buggyTree: '3984927-parent',
      fixedTree: '3984927',
      kind: 'parent',
      shimFastUnsafe: true,
    },
    version: '8.0.0',
  },
];
