/**
 Characterization test pinning the retirement table against pi-ai's bundled catalog.

 The rule retires models with no runtime override and no curated exception list, so
 this table is the only place a mistaken retirement becomes visible before it costs a
 session. A pi upgrade that changes any single retirement fails here until the diff is
 reviewed and the fixture is regenerated with
 `mise run //package/pi-plugin/model-retirement:fixture:retirement-table`.

 @module
 */

import {
  readdirSync,
  readFileSync,
} from 'node:fs';
import { fileURLToPath, } from 'node:url';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  compareRecency,
  decideRetirements,
  parseModelId,
  type AbstentionCounts,
  type CatalogEntry,
  type Retirement,
} from '../dist/final/node/index.mjs';

//region Types

/** Shape of the committed fixture file. */
type Fixture = {
  readonly catalogGeneratedAt: string;
  readonly catalogSchemaVersion: number;
  readonly entryCount: number;
  readonly retirementCount: number;
  readonly abstentions: AbstentionCounts;
  readonly retirements: readonly Retirement[];
};

/** Reasons the fixture counts abstentions. */
type TallyFields = {
  readonly keeperAmbiguity: number;
  readonly unorderedPair: number;
  readonly loserNewerThanKeeper: number;
  readonly versionlessProtected: number;
  readonly duplicateIdentity: number;
};

//endregion Types

//region Guards

/**
 Test whether an unknown value is a plain object record.

 @param value - parsed JSON value

 @returns whether the value can be read by key

 @example
 ```typescript
 isRecord({}); // true
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null) && (!Array.isArray(value,));
}

/**
 Test whether an unknown value is one catalog model record.

 @param value - value from a catalog JSON document

 @returns whether the value carries a string model id

 @example
 ```typescript
 isCatalogModel({ id: 'glm-5.3' }); // true
 ```
 */
function isCatalogModel(value: unknown,): value is { readonly id: string; } {
  return isRecord(value,) && ((typeof value.id) === 'string');
}

/**
 Test whether an unknown value is one retirement row.

 @param value - value from the fixture's retirement list

 @returns whether the value carries all four retirement fields as strings

 @example
 ```typescript
 isRetirement({ provider: 'hyper', api: 'a', retiredId: 'x', keeperId: 'y' }); // true
 ```
 */
function isRetirement(value: unknown,): value is Retirement {
  return isRecord(value,)
    && ((typeof value.provider) === 'string')
    && ((typeof value.api) === 'string')
    && ((typeof value.retiredId) === 'string')
    && ((typeof value.keeperId) === 'string');
}

/**
 Read one number field out of a record.

 @param record - object to read

 @param key - field name

 @returns the field value

 @throws when the field is missing or is not a number

 @example
 ```typescript
 readNumber({ entryCount: 1604 }, 'entryCount'); // 1604
 ```
 */
function readNumber(
  {
    record,
    key,
  }: {
    readonly record: Record<string, unknown>;
    readonly key: string;
  },
): number {
  /**
   Value stored under the key.
   */
  const value = record[key];
  if ((typeof value) !== 'number')
    throw new Error(`retirement table fixture field ${key} is not a number`,);
  return value;
}

/**
 Read one string field out of a record.

 @param record - object to read

 @param key - field name

 @returns the field value

 @throws when the field is missing or is not a string

 @example
 ```typescript
 readString({ catalogGeneratedAt: 'now' }, 'catalogGeneratedAt'); // 'now'
 ```
 */
function readString(
  {
    record,
    key,
  }: {
    readonly record: Record<string, unknown>;
    readonly key: string;
  },
): string {
  /**
   Value stored under the key.
   */
  const value = record[key];
  if ((typeof value) !== 'string')
    throw new Error(`retirement table fixture field ${key} is not a string`,);
  return value;
}

/**
 Read the abstention tally out of a fixture record.

 @param record - fixture object

 @returns tally with all five reasons

 @throws when the tally is missing a numeric reason

 @example
 ```typescript
 readTally(fixtureRecord);
 ```
 */
function readTally(record: Record<string, unknown>,): TallyFields {
  /**
   Value stored under the abstentions key.
   */
  const value = record.abstentions;
  if (!isRecord(value,))
    throw new Error('retirement table fixture carries no abstention tally object',);
  return {
    keeperAmbiguity: readNumber({ record: value, key: 'keeperAmbiguity', },),
    unorderedPair: readNumber({ record: value, key: 'unorderedPair', },),
    loserNewerThanKeeper: readNumber({
      record: value,
      key: 'loserNewerThanKeeper',
    },),
    versionlessProtected: readNumber({
      record: value,
      key: 'versionlessProtected',
    },),
    duplicateIdentity: readNumber({ record: value, key: 'duplicateIdentity', },),
  };
}

//endregion Guards

//region Catalog reading

/**
 Resolve the bundled catalog directory through pi-ai's public export subpath.

 @returns absolute path of the directory holding one JSON file per provider

 @example
 ```typescript
 catalogDataDirectory(); // '/.../pi-ai/dist/providers/data'
 ```
 */
function catalogDataDirectory(): string {
  /**
   Resolved URL of pi-ai's provider entry, which sits beside the data directory.
   */
  const providersEntry = import.meta.resolve('@earendil-works/pi-ai/providers/all');
  return fileURLToPath(new URL('data/', providersEntry,),);
}

/**
 Parse one JSON file from the catalog directory.

 @param path - file to read

 @returns parsed value as a record

 @throws when the file does not hold a JSON object

 @example
 ```typescript
 readJsonRecord('/.../openai-codex.json');
 ```
 */
function readJsonRecord(path: string,): Record<string, unknown> {
  /**
   Parsed file contents.
   */
  const parsed: unknown = JSON.parse(readFileSync(path, 'utf8',),) as unknown;
  if (!isRecord(parsed,))
    throw new Error(`catalog file ${path} does not hold a JSON object`,);
  return parsed;
}

/**
 Read every catalog entry the bundled data files carry.

 @returns entries in provider file and API order

 @example
 ```typescript
 readCatalogEntries().length; // 1604
 ```
 */
function readCatalogEntries(): CatalogEntry[] {
  /**
   Directory holding one JSON file per provider.
   */
  const directory = catalogDataDirectory();
  /**
   Entries collected so far.
   */
  const entries: CatalogEntry[] = [];
  for (const file of readdirSync(directory,).toSorted()) {
    if ((!file.endsWith('.json')) || (file === '.manifest.json'))
      continue;
    /**
     Provider id the file name carries.
     */
    const provider = file.slice(0, -'.json'.length,);
    /**
     Parsed catalog document for that provider.
     */
    const document = readJsonRecord(`${directory}/${file}`,);
    for (const [api, models,] of Object.entries(document,)) {
      if (!isRecord(models,))
        continue;
      for (const model of Object.values(models,)) {
        if (isCatalogModel(model,))
          entries.push({ provider, api, modelId: model.id, },);
      }
    }
  }
  return entries;
}

/**
 Read the committed fixture.

 @returns parsed fixture

 @throws when the file does not hold every field this test compares

 @example
 ```typescript
 readFixture().retirementCount; // 551
 ```
 */
function readFixture(): Fixture {
  /**
   Parsed fixture file.
   */
  const record = readJsonRecord(
    fileURLToPath(new URL('retirement-table.fixture.json', import.meta.url,),),
  );
  /**
   Fixture retirement rows.
   */
  const rows = record.retirements;
  if (!Array.isArray(rows,))
    throw new Error('retirement table fixture carries no retirement list',);
  for (const row of rows) {
    if (!isRetirement(row,))
      throw new Error('retirement table fixture holds a malformed retirement row',);
  }
  return {
    catalogGeneratedAt: readString({ record, key: 'catalogGeneratedAt', },),
    catalogSchemaVersion: readNumber({ record, key: 'catalogSchemaVersion', },),
    entryCount: readNumber({ record, key: 'entryCount', },),
    retirementCount: readNumber({ record, key: 'retirementCount', },),
    abstentions: readTally(record,),
    retirements: rows,
  };
}

/**
 Render one retirement as a single comparable row.

 @param retirement - retirement to render

 @returns row naming provider, api, retired id, and keeper

 @example
 ```typescript
 retirementRow({ provider: 'hyper', api: 'a', retiredId: 'glm-5.2', keeperId: 'glm-5.3' });
 ```
 */
function retirementRow(retirement: Retirement,): string {
  return `${retirement.provider} ${retirement.api} ${retirement.retiredId} -> ${retirement.keeperId}`;
}

//endregion Catalog reading

//region Shared state

/**
 Catalog entries read once for every case in this file.
 */
const CATALOG_ENTRIES = readCatalogEntries();

/**
 Rule outcome for the whole bundled catalog.
 */
const DECISION = decideRetirements({ entries: CATALOG_ENTRIES, },);

/**
 Committed table this run is compared against.
 */
const FIXTURE = readFixture();

//endregion Shared state

await describe({
  name: 'retirement table characterization',
  children: [
    it({
      name: 'reads the catalog the fixture was generated from',
      fn: async function runEntryCount() {
        expect(CATALOG_ENTRIES.length,).toBe(FIXTURE.entryCount,);
      },
    },),
    it({
      name: 'reports the catalog stamp the fixture names, so an upgrade asks for review',
      fn: async function runCatalogStamp() {
        /**
         Generation stamp of the installed catalog.
         */
        const installed = readString({
          record: readJsonRecord(`${catalogDataDirectory()}/.manifest.json`,),
          key: 'generatedAt',
        },);
        expect(installed,).toBe(FIXTURE.catalogGeneratedAt,);
      },
    },),
    it({
      name: 'reproduces the pinned abstention tally',
      fn: async function runAbstentionTally() {
        expect(DECISION.abstentions,).toEqual(FIXTURE.abstentions,);
      },
    },),
    it({
      name: 'reproduces the pinned retirement count',
      fn: async function runRetirementCount() {
        expect(DECISION.retirements.length,).toBe(FIXTURE.retirementCount,);
      },
    },),
    it({
      name: 'reproduces the pinned retirement table exactly',
      fn: async function runRetirementTable() {
        /**
         Rows the fixture pins.
         */
        const pinned = FIXTURE.retirements.map(retirementRow,);
        /**
         Rows this run produced.
         */
        const actual = DECISION.retirements.map(retirementRow,);
        /**
         Rows that disappeared since the fixture was generated.
         */
        const missing = pinned.filter(function absentNow(row,) {
          return !actual.includes(row,);
        },);
        /**
         Rows that are new since the fixture was generated.
         */
        const unexpected = actual.filter(function absentBefore(row,) {
          return !pinned.includes(row,);
        },);
        expect({
          missing,
          unexpected,
        },).toEqual({
          missing: [],
          unexpected: [],
        },);
      },
    },),
    it({
      name: 'never retires a model whose keeper does not order newer',
      fn: async function runKeeperInvariant() {
        /**
         Retirements whose keeper does not order strictly newer than the retired entry.
         */
        const misordered = DECISION.retirements.filter(function keeperIsNotNewer(retirement,) {
          return compareRecency({
            left: parseModelId(retirement.keeperId,),
            right: parseModelId(retirement.retiredId,),
          },) !== 'left';
        },);
        expect(misordered.map(retirementRow,),).toEqual([],);
      },
    },),
  ],
},);
