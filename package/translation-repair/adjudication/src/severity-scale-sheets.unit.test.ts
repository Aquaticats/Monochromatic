/**
 Tests that the critic and panel sheets define the severity scale the tally
 reads and refuse a claim that finds nothing wrong (ledger L5).
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type ClaimCluster,
  ISSUE_SEVERITIES,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '../dist/final/node/index.mjs';

/**
 Invented source both sheets carry.
 */
const SOURCE_TEXT = '猫猫在中午打盹。';

/**
 Invented translation both sheets carry.
 */
const TARGET_TEXT = 'The cat naps at noon.';

/**
 One claim for the panel sheet to number.
 */
const CLUSTERS: readonly ClaimCluster[] = [
  {
    clusterId: 'cluster/noon',
    position: 0,
    members: [
      {
        claimId: 'issue/noon',
        claim: {
          category: 'accuracy/omission',
          severity: 'neutral',
          summary: 'The time of the nap is rendered correctly.',
          spans: [
            {
              side: 'target',
              nodeId: 'block/1',
              nodeHash: hashContent({ content: TARGET_TEXT, },),
              startOffset: 0,
              endOffset: TARGET_TEXT.length,
              quotedText: TARGET_TEXT,
            },
          ],
        },
      },
    ],
  },
];

/**
 Critic system instructions.
 */
const CRITIC_SYSTEM = buildCriticMessages({ sourceText: SOURCE_TEXT, targetText: TARGET_TEXT, },)[0]?.content ?? '';

/**
 Panel system instructions.
 */
const PANEL_SYSTEM = buildAdjudicationMessages({
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
  clusters: CLUSTERS,
},)
  .messages[0]
  ?.content ?? '';

await describe({
  name: 'severity scale on the sheets',
  children: [
    it({
      name: 'DEFINES every severity on its own line of the critic sheet, since the tally holds a neutral acceptance and the archive dispute reads major',
      fn: async () => {
        for (const severity of ISSUE_SEVERITIES)
          expect(CRITIC_SYSTEM,).toContain(`\n- ${severity}: `,);
      },
    },),

    it({
      name: 'DEFINES every severity on its own line of the panel sheet, whose re-grades settle the severity',
      fn: async () => {
        for (const severity of ISSUE_SEVERITIES)
          expect(PANEL_SYSTEM,).toContain(`\n- ${severity}: `,);
      },
    },),

    it({
      name: 'TELLS THE CRITIC a claim that finds nothing wrong is no issue, the eight accepted claims calling a rendering accurate or correct',
      fn: async () => {
        expect(CRITIC_SYSTEM,).toContain('names nothing wrong',);
      },
    },),

    it({
      name: 'TELLS THE PANEL such a claim is unsupported, yet keeps an ambiguity or a suspected source error outside that rule',
      fn: async () => {
        expect(PANEL_SYSTEM,).toContain('names nothing wrong',);
        expect(PANEL_SYSTEM,).toContain('extension/interpretive-ambiguity',);
        expect(PANEL_SYSTEM,).toContain('extension/suspected-source-error',);
      },
    },),

    it({
      name: 'ASKS A SUPPORTER to lift a real defect filed neutral to minor, the one path that keeps its fix now that neutral asks for no edit',
      fn: async () => {
        expect(PANEL_SYSTEM,).toContain('at least minor',);
      },
    },),
  ],
},);
