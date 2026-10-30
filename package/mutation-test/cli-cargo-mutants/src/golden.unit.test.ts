import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GoldenOutcome,
  goldenScenarios,
  runScenario,
} from './golden-scenarios.ts';

await describe({
  name: '',
  children: [
    describe({
      name: 'product campaign scripts against the outcomes captured before they were shared',
      children: goldenScenarios()
        .map(function goldenCase(scenario) {
          return it({
            name: scenario.name,
            fn: async () => {
              /**
               Outcome pinned from the script as it was before the shared package replaced its duplicated parts.
               */
              const golden = JSON.parse(await readFile(
                join(
                  import.meta.dirname,
                  'golden',
                  `${scenario.name}.json`,
                ),
                'utf8',
              )) as GoldenOutcome;
              expect(await runScenario(scenario)).toEqual(golden);
            },
            timeout: 60_000,
          });
        }),
    }),
  ],
});
