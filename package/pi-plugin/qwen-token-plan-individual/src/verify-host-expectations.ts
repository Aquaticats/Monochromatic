/**
 Expectations for the gated disposable-host verification:
 the projected rows the checked-in listing earns,
 the pinned builtin fixture the stale-only comparison reads,
 and the parsing of `pi --list-models` output.

 Split out of `verify-host.ts` so each file stays inside the line budget.
 Everything here is verification-only:
 the package entry never imports this module.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import {
  fileURLToPath,
} from 'node:url';
import {
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualListingSnapshot,
  type QwenTokenPlanResolvedModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  projectQwenTokenPlanIndividualModels,
} from '../dist/final/node/index.mjs';
import {
  TARGET_PROVIDER,
} from './disposable-host.ts';

//region Constants

/**
 Column label `pi --list-models` prints above the provider column,
 so parsed rows are recognized by their provider field instead of by position.
 */
const LIST_MODELS_PROVIDER_LABEL = 'provider';

//endregion Constants

//region Types

/**
 Committed shape of `src/builtin-catalog.fixture.json`,
 the pinned builtin rows the stale-only comparison reads.
 */
export type BuiltinCatalogFixture = {
  readonly rows: Readonly<Record<string, { readonly id: string }>>;
};

/**
 Projected row fields the probe reports and the assertions compare.
 */
export type QwenTokenPlanIndividualExpectedRow = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelCount: number;
};

/**
 Project-owned deep readonly projection of the pi row fields the expectations
 read,
 so the map callback parameter stays deeply readonly while pi's own `Model`
 declaration stays externally mutable.
 */
type ReadonlyProjectedRow = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelMap?: Readonly<Record<string, unknown>>;
};

//endregion Types

//region Expectations

/**
 The rows the checked-in listing earns under the shared mapping and this
 package's projection.

 @returns the projected rows in listing order

 @example
 ```typescript
 const expected = expectedRows();
 ```
 */
export function expectedRows(): QwenTokenPlanIndividualExpectedRow[] {
  /**
   Resolved facts the shared mapping earns from the checked-in listing.
   */
  const facts: readonly QwenTokenPlanResolvedModelFacts[] =
    mapQwenTokenPlanIndividualModels({
      listing: qwenTokenPlanIndividualListingSnapshot.entries,
    },);
  return projectQwenTokenPlanIndividualModels({ facts, },)
    .map(function toExpectedRow(row: ReadonlyProjectedRow,) {
      return {
        id: row.id,
        contextWindow: row.contextWindow,
        maxTokens: row.maxTokens,
        reasoning: row.reasoning,
        thinkingLevelCount: row.thinkingLevelMap === undefined
          ? 0
          : Object.keys(row.thinkingLevelMap,)
            .length,
      };
    },);
}

/**
 Test whether an unknown value is the pinned fixture document.

 @param value - decoded fixture JSON

 @returns whether the value carries the rows table the assertions read
 */
function isBuiltinCatalogFixture(value: unknown,): value is BuiltinCatalogFixture {
  if (((typeof value) !== 'object')
    || (value === null)
    || (Array.isArray(value,)))
    return false;
  return ('rows' in value)
    && ((typeof value.rows) === 'object')
    && (value.rows !== null);
}

/**
 Read the pinned builtin rows the stale-only comparison derives from.

 @returns the fixture document

 @throws when the checked-in fixture does not carry the rows table

 @example
 ```typescript
 const fixture = await readBuiltinFixture();
 ```
 */
export async function readBuiltinFixture(): Promise<BuiltinCatalogFixture> {
  /**
   Pinned fixture text beside this module's sibling.
   */
  const text = await readFile(
    fileURLToPath(new URL(
      'builtin-catalog.fixture.json',
      import.meta.url,
    ),),
    'utf8',
  );
  /**
   Decoded fixture document, narrowed before its shape is trusted.
   */
  const decoded: unknown = JSON.parse(text,);
  if (!isBuiltinCatalogFixture(decoded,))
    throw new Error(
      'src/builtin-catalog.fixture.json does not carry the pinned rows table',
    );
  return decoded;
}

//endregion Expectations

//region Listing output parsing

/**
 Parse the model ids `pi --list-models` prints for the override route.

 @param stdout - everything the listing command printed

 @returns the route's model ids in printed order

 @example
 ```typescript
 parseListedIds({ stdout });
 ```
 */
export function parseListedIds({ stdout, }: { readonly stdout: string; },): string[] {
  /**
   Printed ids collected in row order.
   */
  const ids: string[] = [];
  for (const line of stdout.split('\n',)) {
    /**
     Whitespace-separated columns of one printed row,
     with the padding gaps the table prints dropped.
     */
    const columns = line
      .trim()
      .split(' ',)
      .filter(function nonEmpty(column,) {
        return column.length > 0;
      },);
    /**
     Provider and model columns of one printed row.
     */
    const [provider, model,] = columns;
    if ((provider === undefined) || (model === undefined))
      continue;
    if (provider === LIST_MODELS_PROVIDER_LABEL)
      continue;
    if (provider !== TARGET_PROVIDER)
      continue;
    ids.push(model,);
  }
  return ids;
}

//endregion Listing output parsing
