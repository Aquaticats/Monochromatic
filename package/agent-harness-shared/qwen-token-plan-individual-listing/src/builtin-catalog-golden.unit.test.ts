import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  expandQwenTokenPlanThinkingLevelMap,
  qwenTokenPlanIndividualFacts,
} from '../dist/final/node/index.mjs';

import { piAiQwenTokenPlanIndividualCatalog, } from './test-fixtures/fixture.pi-ai-qwen-token-plan-individual-catalog.ts';

/**
 Facts row one builtin id earns, or a thrown error when the curated table
 lost it, since the golden pins the table against the builtin catalog.
 */
function factsRowOf(id: string) {
  const row = qwenTokenPlanIndividualFacts.find(function matchesId(candidate) {
    return candidate.id === id;
  });
  if (row === undefined) {
    throw new Error(`facts table carries no row for builtin id ${id}`);
  }
  return row;
}

await describe({
  name: 'builtin catalog golden expansion',
  children: [
    describe({
      name: 'fixture completeness',
      children: [
        it({
          name: 'pins the fixture to the 10 rows the builtin catalog carries',
          fn: async () => {
            expect(Object.keys(piAiQwenTokenPlanIndividualCatalog))
              .toHaveLength(10);
          },
        }),
      ],
    }),
    describe({
      name: 'expansion equality',
      children: Object.values(piAiQwenTokenPlanIndividualCatalog).map(
        function goldenRow(row) {
          return it({
            name: row.thinkingLevelMap === undefined
              ? `keeps ${row.id} efforts absent, matching its builtin row without a map`
              : `expands ${row.id} efforts to its builtin thinkingLevelMap`,
            fn: async () => {
              const factsRow = factsRowOf(row.id);
              if (row.thinkingLevelMap === undefined) {
                expect(factsRow.efforts).toBeUndefined();
                return;
              }
              expect(expandQwenTokenPlanThinkingLevelMap({
                efforts: factsRow.efforts ?? {},
              })).toEqual(row.thinkingLevelMap);
            },
          });
        },
      ),
    }),
  ],
});
