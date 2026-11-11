/**
 Characterization test pinning the builtin `qwen-token-plan-individual`
 catalog rows against the pi-ai installation this workspace resolves.

 The override must equal the builtin rows value for value on every overlapping
 id,
 so the pinned table is the place a pi-ai upgrade that changes a row becomes
 visible before it costs a session.
 A pi-ai upgrade that changes any row fails here until the diff is reviewed
 and the fixture is regenerated with
 `mise run //package/pi-plugin/qwen-token-plan-individual:fixture:builtin-catalog`.

 @module
 */

import {
  readFileSync,
} from 'node:fs';
import {
  fileURLToPath,
} from 'node:url';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_API,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Types

/**
 Shape of the committed fixture file
 (`src/builtin-catalog.fixture.json`).
 */
type Fixture = {
  readonly catalogGeneratedAt: string;
  readonly catalogSchemaVersion: number;
  readonly catalogFileHash: string;
  readonly piAiVersion: string;
  readonly providerId: string;
  readonly api: string;
  readonly rowCount: number;
  readonly rows: Readonly<Record<string, unknown>>;
};

//endregion Types

//region Fixtures

/**
 Read the pinned fixture this package checks in.

 @returns the fixture document

 @example
 ```typescript
 const fixture = readFixture();
 ```
 */
function readFixture(): Fixture {
  return JSON.parse(readFileSync(fileURLToPath(new URL(
    'builtin-catalog.fixture.json',
    import.meta.url,
  ),), 'utf8',),) as Fixture;
}

/**
 Read the builtin rows the installed pi-ai catalog ships.

 @returns the `openai-completions` row table of the builtin catalog

 @example
 ```typescript
 const rows = readInstalledRows();
 ```
 */
function readInstalledRows(): Record<string, unknown> {
  /**
   Data directory beside pi-ai's bundled providers entry.
   */
  const dataDirectory = fileURLToPath(new URL(
    'data/',
    import.meta.resolve('@earendil-works/pi-ai/providers/all'),
  ),);
  /**
   Parsed builtin catalog document for this route.
   */
  const document = JSON.parse(readFileSync(
    `${dataDirectory}/qwen-token-plan-individual.json`,
    'utf8',
  ),) as Record<string, unknown>;
  return document['openai-completions'] as Record<string, unknown>;
}

//endregion Fixtures

//region Characterization

await describe({
  name: 'builtin catalog characterization',
  children: [
    it({
      name: 'pins the installed builtin rows value for value',
      fn: async () => {
        expect(readFixture().rows,)
          .toEqual(readInstalledRows(),);
      },
    },),
    it({
      name: 'pins the row count the fixture records',
      fn: async () => {
        /**
         Pinned fixture document.
         */
        const fixture = readFixture();
        expect(Object.keys(fixture.rows,)
          .length,)
          .toBe(fixture.rowCount);
        expect(fixture.rowCount,)
          .toBe(Object.keys(readInstalledRows(),)
            .length);
      },
    },),
    it({
      name: 'pins the route and api the fixture was generated for',
      fn: async () => {
        /**
         Pinned fixture document.
         */
        const fixture = readFixture();
        expect(fixture.providerId,)
          .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE);
        expect(fixture.api,)
          .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_API);
      },
    },),
  ],
},);

//endregion Characterization
