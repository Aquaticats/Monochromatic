/**
 Round-trip and metamorphic properties for the parse/emit cycle.
 
 Three oracles, each over both generated documents and the committed corpus:
 
 1. Splice fidelity: re-emitting an unmutated splice-mode parse is byte-for-byte
    identical to the source.
 2. Canonical semantic round-trip: parsing in canonical mode, re-emitting from
    the AST, and reparsing preserves the normalized semantic model. This drives
    the emitter, where re-spelling a value could silently corrupt it.
 3. Metamorphic invariance: adding a leading comment, a trailing comment, or
    trailing blank lines never changes a document's meaning.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constantFrom,
  oneof,
  pre,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseTomlEdit,
  tomlGetValue,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

import {
  fuzzRunPlan,
  isCampaignMode,
} from './fuzz-budget.ts';
import { documentArbitrary, } from './arb-documents.ts';
import {
  discoverRepoToml,
  loadValidFixtures,
} from './corpus.ts';
import {
  semanticEquals,
  semanticModel,
  staticSemanticOracleSupports,
} from './equality.ts';

//region Setup

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Upper bound on live repository TOML files pulled into the campaign corpus.
 */
const REPO_CORPUS_LIMIT = 300;

/** Inherited setter name mishandled by upstream static projection. */
const UPSTREAM_PROTOTYPE_KEY = '__proto__';

/**
 Minimized source whose `__proto__` datetime triggers upstream static-oracle prototype assignment.
 */
const UPSTREAM_PROTOTYPE_SOURCE = `value = { "${UPSTREAM_PROTOTYPE_KEY}" = 1979-05-27T07:32:00Z }\n`;

/**
 Whether the package accepts `source` under the default dialect.
 
 Live-discovered repository TOML can be version-specific or malformed; the
 valid-requiring properties keep only sources the package actually parses, so
 a parse failure here classifies corpus rather than failing the property.
 
 @returns Whether `parseTomlEdit` accepts `source`.
 */
function parsesUnderDefault(source: string,): boolean {
  try {
    parseTomlEdit({ source, },);
    return true;
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    return false;
  }
}

/**
 Committed valid fixtures plus, in campaign mode, live repository TOML, kept to
 what the package parses so the round-trip oracles see only valid input.
 */
const validSources = [
  ...(await loadValidFixtures()).map(function source(entry,) { return entry.source; },),
  ...(await discoverRepoToml({
    campaign: isCampaignMode(),
    limit: REPO_CORPUS_LIMIT,
  },)).map(function source(entry,) { return entry.source; },),
].filter(function keepValid(source,) { return parsesUnderDefault(source,); },);

/**
 Valid source arbitrary: generated documents unioned with the real corpus.
 */
const validSourceArbitrary = oneof(
  documentArbitrary,
  constantFrom(...validSources,),
);

/**
 Metamorphic transform tags applied to a valid source.
 */
const TRANSFORMS = [
  'leading-comment',
  'trailing-comment',
  'trailing-blanks',
] as const;

/**
 Apply a meaning-preserving transform named by `tag` to `source`.
 
 @returns Transformed source that must share `source`'s semantic model.
 */
function applyTransform(
  {
    source,
    tag,
  }: {
    readonly source: string;
    readonly tag: typeof TRANSFORMS[number];
  },
): string {
  /**
   Source guaranteed to end in a newline so an appended line is well-formed.
   */
  const terminated = source.endsWith('\n',) ? source : `${source}\n`;
  const transforms: Record<typeof TRANSFORMS[number], () => string> = {
    'leading-comment': function run() { return `# metamorphic\n${source}`; },
    'trailing-comment': function run() { return `${terminated}# metamorphic\n`; },
    'trailing-blanks': function run() { return `${terminated}\n\n`; },
  };
  return transforms[tag]();
}

//endregion Setup

await describe({
  name: 'parse/emit round-trip',
  children: [
    it({
      name: 'splice mode re-emits a parsed document byte-for-byte',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(validSourceArbitrary, async function identical(source,) {
            expect(tomlStringify({ edit: parseTomlEdit({ source, },), },),).toBe(source,);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'classifies the upstream oracle boundary and round-trips supported semantic models',
      timeout: RUN.timeout,
      fn: async () => {
        expect(staticSemanticOracleSupports({ source: 'value = 1\n', },)).toBe(true,);
        expect(semanticEquals({
          left: semanticModel({ source: '[constructor.prototype]\nvalue = 1\n', },),
          right: {
            constructor: {
              prototype: { value: 1, },
            },
          },
        },)).toBe(true,);
        expect(staticSemanticOracleSupports({
          source: UPSTREAM_PROTOTYPE_SOURCE,
        },)).toBe(false,);
        /** Upstream projection that loses the own key and changes nested object prototype. */
        const unsupportedModel = semanticModel({ source: UPSTREAM_PROTOTYPE_SOURCE, },);
        if (((typeof unsupportedModel) !== 'object')
          || (unsupportedModel === null)
          || Array.isArray(unsupportedModel,)
          || (unsupportedModel instanceof Date)
          || (!('value' in unsupportedModel)))
          throw new Error('Expected projected root object.',);
        /** Nested inline-table projection altered by inherited `__proto__` setter. */
        const unsupportedTable = unsupportedModel.value;
        if (((typeof unsupportedTable) !== 'object')
          || (unsupportedTable === null)
          || Array.isArray(unsupportedTable,))
          throw new Error('Expected projected inline table.',);
        expect(Object.hasOwn(unsupportedTable, UPSTREAM_PROTOTYPE_KEY,),).toBe(false,);
        expect(Object.getPrototypeOf(unsupportedTable,) instanceof Date,).toBe(true,);
        /** Package-owned materialization of the same inline table. */
        const materializedTable = tomlGetValue({
          edit: parseTomlEdit({ source: UPSTREAM_PROTOTYPE_SOURCE, },),
          path: ['value',],
        },);
        if (((typeof materializedTable) !== 'object')
          || (materializedTable === null)
          || Array.isArray(materializedTable,))
          throw new Error('Expected materialized inline table.',);
        expect(Object.hasOwn(materializedTable, UPSTREAM_PROTOTYPE_KEY,),).toBe(true,);
        expect(Object.getPrototypeOf(materializedTable,),).toBe(Object.prototype,);
        await assert(
          asyncProperty(validSourceArbitrary, async function preserves(source,) {
            pre(staticSemanticOracleSupports({ source, },),);
            /**
             Canonical re-emission, rebuilt from the AST rather than spliced.
             */
            const emitted = tomlStringify({
              edit: parseTomlEdit({
                source,
                mode: 'canonical',
              },),
            },);
            expect(
              semanticEquals({
                left: semanticModel({ source, },),
                right: semanticModel({ source: emitted, },),
              },),
            ).toBe(true,);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'meaning is invariant under leading/trailing comments and blank lines',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            validSourceArbitrary,
            constantFrom(...TRANSFORMS,),
            async function invariant(source, tag,) {
              pre(staticSemanticOracleSupports({ source, },),);
              /**
               Transformed source whose meaning must equal the original.
               */
              const transformed = applyTransform({
                source,
                tag,
              },);
              expect(
                semanticEquals({
                  left: semanticModel({ source, },),
                  right: semanticModel({ source: transformed, },),
                },),
              ).toBe(true,);
            },
          ),
          RUN.params,
        );
      },
    },),
  ],
},);
