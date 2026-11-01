/**
 Scenario families for `./surface-type-cost.ts`: one source body per shape
 family and size, and the size series each sweeps.

 @module
 */

/**
 Consecutive integers from zero.

 @param count - How many.

 @returns `[0, 1, ..., count - 1]`.

 @example
 ```ts
 range(3,); // [0, 1, 2]
 ```
 */
export function range(count: number,): readonly number[] {
  return Array.from(
    { length: count, },
    function index(
      _unused,
      position,
    ) {
      return position;
    },
  );
}

/**
 Nested record literal `depth` levels deep, with a distinct leaf key per level.

 @param depth - Nesting levels.

 @param leaf - Suffix keeping two inputs' leaf keys apart.

 @returns Object literal source.

 @example
 ```ts
 nested({ depth: 1, leaf: 'a', },); // '{ n: { enda: true }, l0a: 0 }'
 ```
 */
function nested({
  depth,
  leaf,
}: {
  readonly depth: number;
  readonly leaf: string
},): string {
  return range(depth,)
    .reduce(
    function wrap(
      inner,
      level,
    ) {
      return `{ n: ${inner}, l${String(level,)}${leaf}: ${String(level,)} }`;
    },
    `{ end${leaf}: true }`,
    );
}

/**
 Sizes a family sweeps: an arithmetic or a geometric sequence from `first`
 up to and including `last`.
 */
export type Series = {
  readonly kind: 'linear';
  readonly first: number;
  readonly step: number;
  readonly last: number;
} | {
  readonly kind: 'geometric';
  readonly first: number;
  readonly ratio: number;
  readonly last: number;
};

/**
 Expand a {@link Series} into its sizes.

 @param series - Sweep description.

 @returns Sizes in increasing order.

 @example
 ```ts
 seriesOf({ first: 1, kind: 'geometric', last: 8, ratio: 2, },); // [1, 2, 4, 8]
 ```
 */
export function seriesOf(series: Series,): readonly number[] {
  /**
   Sizes collected so far.
   */
  const sizes: number[] = [];
  for (let size = series.first; size <= series.last; size = (series.kind === 'linear') ? size + series.step : size * series.ratio)
    sizes.push(size,);
  return sizes;
}

/**
 Shape families measured by the surface audit; each reads the result so
 the checker resolves it.
 */
const SHAPE_FAMILIES: Readonly<Record<string, {
  readonly series: Series;
  readonly body: (size: number,) => string
}>> = {
  args: {
    body: function args(size,) {
      return `const merged = deepmerge(${range(size,)
        .map(function input(i,) {
        return `{ k${String(i,)}: ${String(i,)}, shared: { x${String(i,)}: "${String(i,)}" }, list: [${String(i,)}] }`;
      },)
        .join(', ',)});\nexport const probe: number = merged.k${String(size - 1,)};`;
    },
    series: {
      first: 25,
      kind: 'linear',
      last: 200,
      step: 25,
    },
  },
  deep: {
    body: function deep(size,) {
      return `const merged = deepmerge(${nested({
        depth: size,
        leaf: 'a',
      },)}, ${nested({
        depth: size,
        leaf: 'b',
      },)});\nexport const probe: boolean = merged${'.n'.repeat(size,)}.endb;`;
    },
    series: {
      first: 10,
      kind: 'linear',
      last: 70,
      step: 20,
    },
  },
  intowide: {
    body: function intoWide(size,) {
      return `const target = { ${range(size,)
        .map(function key(i,) { return `k${String(i,)}: ${String(i,)}`; },)
        .join(', ',)} };\ndeepmergeInto(target, { ${range(size,)
          .map(function key(i,) { return `k${String(i + (size / 2),)}: "s"`; },)
          .join(', ',)} });\nexport const probe = target.k${String(size - 1,)};`;
    },
    series: {
      first: 100,
      kind: 'geometric',
      last: 800,
      ratio: 2,
    },
  },
  recursive: {
    body: function recursive(size,) {
      return `${range(size,)
        .map(function declare(i,) {
        return `type T${String(i,)} = { v${String(i,)}: number; child?: T${String(i,)}; list?: T${String(i,)}[] };\ndeclare const t${String(i,)}: T${String(i,)};`;
      },)
        .join('\n',)}\nconst merged = deepmerge(${range(size,)
          .map(function name(i,) { return `t${String(i,)}`; },)
          .join(', ',)});\nexport const probe = merged.child?.child?.child;`;
    },
    series: {
      first: 5,
      kind: 'linear',
      last: 50,
      step: 5,
    },
  },
  sets: {
    body: function sets(size,) {
      return `export const probe = deepmerge(${range(size,)
        .map(function set(i,) { return `new Set<"v${String(i,)}">()`; },)
        .join(', ',)});`;
    },
    series: {
      first: 10,
      kind: 'linear',
      last: 50,
      step: 20,
    },
  },
  tuple: {
    body: function tuple(size,) {
      return `export const probe = deepmerge([${range(size,)
        .join(', ',)}] as const, [${range(size,)
          .join(', ',)}] as const).length;`;
    },
    series: {
      first: 100,
      kind: 'geometric',
      last: 800,
      ratio: 2,
    },
  },
  union: {
    body: function union(size,) {
      /**
       Union of `size` object types.
       */
      const members = range(size,)
        .map(function member(i,) { return `{ t: ${String(i,)}; v${String(i,)}: string }`; },)
        .join(' | ',);
      return `declare const a: { f: ${members} };\ndeclare const b: { f: ${members} };\nexport const probe = deepmerge(a, b).f;`;
    },
    series: {
      first: 4,
      kind: 'geometric',
      last: 64,
      ratio: 2,
    },
  },
  wide: {
    body: function wide(size,) {
      return `const merged = deepmerge({ ${range(size,)
        .map(function key(i,) { return `k${String(i,)}: ${String(i,)}`; },)
        .join(', ',)} }, { ${range(size,)
          .map(function key(i,) { return `k${String(i + (size / 2),)}: "s"`; },)
          .join(', ',)} });\nexport const probe: string = merged.k${String(size - 1,)};`;
    },
    series: {
      first: 100,
      kind: 'geometric',
      last: 1_600,
      ratio: 2,
    },
  },
};

/**
 Union of `size` string literals, the shape of a status or country code.

 @param size - Members.

 @returns Type source.

 @example
 ```ts
 literalUnion(2,); // '"s0" | "s1"'
 ```
 */
function literalUnion(size: number,): string {
  return range(size,)
    .map(function member(i,) { return `"s${String(i,)}"`; },)
    .join(' | ',);
}

/**
 Literal-union sizes: past the 50 members where the recall audit's
 no-filtering case fails, and past the 120 the default filter handles.
 */
const LITERAL_SERIES: Series = {
  first: 25,
  kind: 'geometric',
  last: 400,
  ratio: 2,
};

/**
 Literal-union families across entry points and filter URIs, added after the
 recall audit found TS2589 at 50 members with `DeepMergeNoFilteringURI`
 (`src/recall-known-defect-union-depth.unit.test.ts`); each reads the result
 into its declared type so the checker resolves it.
 */
const LITERAL_FAMILIES: Readonly<Record<string, {
  readonly series: Series;
  readonly body: (size: number,) => string
}>> = {
  literal: {
    body: function literal(size,) {
      return `type S = ${literalUnion(size,)};\ndeclare const a: { s: S };\ndeclare const b: { s: S };\nexport const probe: { s: S } = deepmerge(a, b);`;
    },
    series: LITERAL_SERIES,
  },
  literalcustomfilter: {
    body: function literalCustomFilter(size,) {
      return `import { deepmergeCustom, type DeepMergeFunctionsURIs, type DeepMergeLeaf } from "deepmerge-ts";\ndeclare module "deepmerge-ts" {\n  interface DeepMergeFunctionURItoKind<Ts extends readonly unknown[], Fs extends DeepMergeFunctionsURIs, in out M> {\n    readonly CostKeepURI: Ts;\n  }\n}\ntype S = ${literalUnion(size,)};\nconst merge = deepmergeCustom<unknown, { DeepMergeFilterValuesURI: "CostKeepURI" }>({ filterValues: (values) => values });\ndeclare const a: { s: S };\ndeclare const b: { s: S };\nexport const probe: { s: S } = merge(a, b);`;
    },
    series: LITERAL_SERIES,
  },
  literalfast: {
    body: function literalFast(size,) {
      return `import { deepmergeFastUnsafe } from "deepmerge-ts";\ntype S = ${literalUnion(size,)};\ndeclare const a: { s: S };\ndeclare const b: { s: S };\nexport const probe: { s: S } = deepmergeFastUnsafe(a, b);`;
    },
    series: LITERAL_SERIES,
  },
  literalinto: {
    body: function literalInto(size,) {
      return `type S = ${literalUnion(size,)};\ndeclare const a: { s: S };\ndeclare const b: { s: S };\ndeepmergeInto(a, b);\nexport const probe: { s: S } = a;`;
    },
    series: LITERAL_SERIES,
  },
  literalnofilter: {
    body: function literalNoFilter(size,) {
      return `import { deepmergeCustom, type DeepMergeNoFilteringURI } from "deepmerge-ts";\ntype S = ${literalUnion(size,)};\nconst merge = deepmergeCustom<unknown, { DeepMergeFilterValuesURI: DeepMergeNoFilteringURI }>({ filterValues: false });\ndeclare const a: { s: S };\ndeclare const b: { s: S };\nexport const probe: { s: S } = merge(a, b);`;
    },
    series: LITERAL_SERIES,
  },
  literalthree: {
    body: function literalThree(size,) {
      return `type S = ${literalUnion(size,)};\ndeclare const a: { s: S };\ndeclare const b: { s: S };\ndeclare const c: { s: S };\nexport const probe: { s: S } = deepmerge(a, b, c);`;
    },
    series: LITERAL_SERIES,
  },
};

/**
 Every family, by name; `./surface-type-cost.ts` selects them by prefix.
 */
export const FAMILIES: Readonly<Record<string, {
  readonly series: Series;
  readonly body: (size: number,) => string
}>> = {
  ...SHAPE_FAMILIES,
  ...LITERAL_FAMILIES,
};
