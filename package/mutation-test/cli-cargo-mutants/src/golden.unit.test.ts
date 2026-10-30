import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
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
               Text of the outcome pinned from the script as it was before the shared package replaced its duplicated
               parts.
               */
              const golden = await readFile(
                join(
                  import.meta.dirname,
                  'golden',
                  `${scenario.name}.json`,
                ),
                'utf8',
              );
              expect(`${JSON.stringify(
                await runScenario(scenario),
                null,
                2,
              )}\n`).toBe(golden);
            },
            timeout: 60_000,
          });
        }),
    }),
  ],
});
