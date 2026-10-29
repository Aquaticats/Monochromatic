/**
 Tests for the ways one chunk of the repair lane settles unchanged, and for
 what the editor is told and what the record says about refused edits.

 Every early exit returns the archive's own text for the chunk: when the panel
 accepts nothing, when every edit the editor wrote is refused at the gate, and
 when the checkers vote every edit worse. Each case scripts the stages up to
 its exit and asserts the chunk ships the archive and says why. The chunk
 stands for its whole document here, as production runs one whose slice is the
 document, so no same-entry original is passed beside it.

 NO NETWORK. The client scripts each stage by the schema it asks for and
 records every message it was sent. Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  cacheRefusalsOf,
  repairChunk,
  repairPreparedDocument,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the chunk under test.
 */
const l = tagged({ tag: 'repair-chunk-exits-test', },);

//region Fixtures

/**
 Invented original of the chunk: a nap, a chase and a bowl.
 */
const SOURCE_TEXT = '小猫在窗台上打盹。猫也喜欢追蝴蝶。碗是满的。\n';

/**
 Invented archive English, one sentence per original sentence, the first
 carrying a link a rewrite must keep.
 */
const TARGET_TEXT = 'The kitten dozes on the [windowsill](https://cats.example/sill). '
  + 'The cat hates butterflies. The bowl is empty.\n';

/**
 Archive sentence carrying the link.
 */
const NAP = 'The kitten dozes on the [windowsill](https://cats.example/sill).';

/**
 Archive sentence mistranslating the chase.
 */
const CHASE = 'The cat hates butterflies.';

/**
 Archive sentence mistranslating the bowl.
 */
const BOWL = 'The bowl is empty.';

/**
 Critic claims on each sentence, as the critic wire carries them.
 */
const CLAIMS = {
  nap: {
    category: 'accuracy/mistranslation',
    severity: 'major',
    summary: 'Napping is rendered as dozing.',
    sourceQuote: '小猫在窗台上打盹。',
    targetQuote: NAP,
  },
  chase: {
    category: 'accuracy/mistranslation',
    severity: 'major',
    summary: 'Chasing butterflies is rendered as hating them.',
    sourceQuote: '猫也喜欢追蝴蝶。',
    targetQuote: CHASE,
  },
  bowl: {
    category: 'accuracy/mistranslation',
    severity: 'major',
    summary: 'A full bowl is rendered as empty.',
    sourceQuote: '碗是满的。',
    targetQuote: BOWL,
  },
} as const;

/**
 Roster with three panelists, one editor, and checkers apart from both,
 which the chunk asserts before it buys anything.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 One edit as the editor wire carries it.
 */
type EditWire = {
  readonly region: number;
  readonly newText: string;
};

/**
 How each stage of one scripted chunk answers.
 */
type ChunkScript = {
  /**
   Claims every critic files.
   */
  readonly claims: readonly Record<string, string>[];
  /**
   How every panelist votes on every claim.
   */
  readonly panelVote: 'supported' | 'unsupported';
  /**
   Whether every panelist answers in a shape nothing can read, so the panel hears nobody.
   */
  readonly panelDeaf?: boolean;
  /**
   Edits the editor writes, by region.
   */
  readonly edits: readonly EditWire[];
  /**
   Verdict of every checker on each issue, by the sheet's one-based issue
   number; an issue the function leaves out is not ruled on at all.
   */
  readonly checks: (issueCount: number) => readonly {
    readonly issue: number;
    readonly verdict: 'fixed' | 'not-fixed' | 'worse';
  }[];
};

/**
 Occurrences of a marker in the user sheet of one exchange.

 @param request - exchange as the stage sent it

 @param marker - text each item of the sheet begins with

 @returns How many items the sheet carries
 */
function countMarker(
  {
    request,
    marker,
  }: {
    readonly request: ChatJsonRequest<unknown>;
    readonly marker: string;
  },
): number {
  /**
   User sheet of this exchange.
   */
  const last = request.messages.at(-1,);
  return ((last === undefined) ? '' : messageText({ message: last, },)).split(marker,).length - 1;
}

/**
 Client answering each stage as the script says and recording every message
 each stage was sent.

 @param script - how each stage answers

 @param sent - every exchange's messages, joined, by stage

 @returns Client for the chunk under test
 */
function scriptedChunkClient(
  {
    script,
    sent,
  }: {
    readonly script: ChunkScript;
    readonly sent: Map<string, string[]>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the repair chunk',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';
      sent.set(
        stage,
        [
          ...(sent.get(stage,) ?? []),
          request.messages
            .map(function toText(message,) {
              return messageText({ message, },);
            },)
            .join('\n',),
        ],
      );
      if ((stage === 'panel_ballot') && (script.panelDeaf === true))
        return { kind: 'schema-mismatch', rawText: '{}', detail: 'the script made the panel deaf', };
      /**
       Scripted wire reply for the stage.
       */
      const scripted: unknown = (stage === 'critic_report')
        ? { issues: script.claims, }
        : (stage === 'panel_ballot')
          ? {
            verdicts: Array.from(
              { length: countMarker({ request, marker: '\nCLAIM ', },), },
              function toVerdict(_unused, index,) {
                return {
                  claim: index + 1,
                  reason: 'The quotes show it.',
                  vote: script.panelVote,
                };
              },
            ),
          }
          : (stage === 'editor_report')
            ? { edits: script.edits, }
            : (stage === 'resolution_report')
              ? { checks: script.checks(countMarker({ request, marker: '\nISSUE ', },),), }
              : (stage === 'introduced_defect_report')
                ? {
                  checks: Array.from(
                    { length: countMarker({ request, marker: ' REGION ', },), },
                    function toCheck(_unused, index,) {
                      return {
                        region: index + 1,
                        verdict: 'no-introduced-defect-found',
                        category: '',
                        severity: '',
                        evidence: '',
                        omittedText: '',
                        verdictReason: '',
                      };
                    },
                  ),
                }
                : undefined;
      if (scripted === undefined)
        throw new Error(`this chunk has no script for stage ${stage}`,);
      if (!request.validate(scripted,))
        throw new Error(`the script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the repair chunk',);
    },
  };
}

/**
 Every checker ruling every issue the same way.

 @param verdict - ruling on every issue

 @returns Checks function for a script
 */
function everyIssue(
  { verdict, }: { readonly verdict: 'fixed' | 'not-fixed' | 'worse'; },
): ChunkScript['checks'] {
  return function rule(issueCount,) {
    return Array.from({ length: issueCount, }, function toCheck(_unused, index,) {
      return {
        issue: index + 1,
        verdict,
      };
    },);
  };
}

/**
 Runs the fixture chunk under one script.

 @param script - how each stage answers

 @param lineStructured - whether the enclosing chunk's original is
 line-structured

 @param models - roster, the fixture's by default

 @returns Outcome and every message each stage was sent
 */
async function runChunk(
  {
    script,
    lineStructured = false,
    models = MODELS,
  }: {
    readonly script: ChunkScript;
    readonly lineStructured?: boolean;
    readonly models?: RepairModels;
  },
) {
  /**
   Every exchange's messages, by stage.
   */
  const sent = new Map<string, string[]>();
  /**
   What the chunk settled on.
   */
  const outcome = await repairChunk({
    client: scriptedChunkClient({
      script,
      sent,
    },),
    sliceIndex: 0,
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    lineStructured,
    models,
    declaredNames: [],
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    outcome,
    sent,
  };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: repairChunk.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHIPS THE ARCHIVE when the panel accepts no claim, since there is nothing to edit, and '
            + 'asks no editor',
          fn: async () => {
            const { outcome, sent, } = await runChunk({
              script: {
                claims: [CLAIMS.chase,],
                panelVote: 'unsupported',
                edits: [],
                checks: everyIssue({ verdict: 'fixed', },),
              },
            },);
            expect([
              outcome.changed,
              outcome.repairedText,
              sent.has('editor_report',),
            ],).toEqual([false, TARGET_TEXT, false,],);
            expect(outcome.issues
              .map(function toStatus(issue,) {
                return issue.status;
              },),).toEqual(['rejected',],);
          },
        },),
        it({
          name: 'RECORDS A PANEL THAT HEARD NOBODY AS DATA on the outcome and refuses to cache it, though the '
            + 'panel\'s own finding is located by cluster and the critics were all heard: the settlement reads '
            + '"nothing to edit" exactly as XingZ6018 slice 85 did',
          fn: async () => {
            const { outcome, sent, } = await runChunk({
              script: {
                claims: [CLAIMS.chase,],
                panelVote: 'supported',
                panelDeaf: true,
                edits: [],
                checks: everyIssue({ verdict: 'fixed', },),
              },
            },);
            expect(sent.has('editor_report',),).toBe(false,);
            expect(outcome.heardCritics,).toBe(MODELS.criticModelIds.length,);
            expect(outcome.silentStages.length,).toBe(1,);
            expect(outcome.silentStages[0],).toContain('panel 0/3 (cluster/',);
            expect(outcome.findings.some(function isLocated(finding,): boolean {
              return finding.startsWith('panel-packet (cluster/',) && finding.includes('stage-quorum-unmet (panel 0/3)',);
            },),).toBe(true,);
            expect(cacheRefusalsOf({ outcome, },).length,).toBe(1,);
          },
        },),
        it({
          name: 'SHIPS THE ARCHIVE when the gate refuses every edit the editor wrote, and names the '
            + 'editor that repaired nothing',
          fn: async () => {
            const { outcome, sent, } = await runChunk({
              script: {
                claims: [CLAIMS.nap, CLAIMS.chase,],
                panelVote: 'supported',
                edits: [
                  // Drops the link the archive carries.
                  { region: 1, newText: 'The kitten naps on the windowsill.', },
                  // Writes the region back as it stands.
                  { region: 2, newText: CHASE, },
                ],
                checks: everyIssue({ verdict: 'fixed', },),
              },
            },);
            expect([
              outcome.changed,
              outcome.repairedText,
              sent.has('resolution_report',),
            ],).toEqual([false, TARGET_TEXT, false,],);
            expect(outcome.findings,).toContain('editor-candidates (1/1 heard, 0 repairing)',);
          },
        },),
        it({
          name: 'COUNTS the edits the gate refused by reason, detail stripped and sorted, beside the one '
            + 'it let through, so a run whose gate refused most of a patch does not read as ordinary',
          fn: async () => {
            const { outcome, } = await runChunk({
              script: {
                claims: [CLAIMS.nap, CLAIMS.chase, CLAIMS.bowl,],
                panelVote: 'supported',
                edits: [
                  { region: 1, newText: 'The kitten naps on the windowsill.', },
                  { region: 2, newText: CHASE, },
                  { region: 3, newText: 'The bowl is full.', },
                ],
                checks: everyIssue({ verdict: 'fixed', },),
              },
            },);
            expect(outcome.repairedText,).toContain('The bowl is full.',);
            expect(outcome.findings
              .filter(function isRefusalCount(finding,): boolean {
                return finding.startsWith('editor-rejected ',);
              },),).toEqual([
                'editor-rejected preservation-lost-markup (1)',
                'editor-rejected unchanged-region (1)',
              ],);
          },
        },),
        it({
          name: 'SHIPS THE ARCHIVE when every checker votes every edit worse, stripping them all and '
            + 'buying no recheck of a patch with nothing left in it',
          fn: async () => {
            const { outcome, sent, } = await runChunk({
              script: {
                claims: [CLAIMS.chase, CLAIMS.bowl,],
                panelVote: 'supported',
                edits: [
                  { region: 1, newText: 'The cat loves chasing butterflies.', },
                  { region: 2, newText: 'The bowl is full.', },
                ],
                checks: everyIssue({ verdict: 'worse', },),
              },
            },);
            expect([
              outcome.changed,
              outcome.repairedText,
              sent.get('resolution_report',)?.length,
            ],).toEqual([false, TARGET_TEXT, MODELS.checkerModelIds.length,],);
            expect(outcome.findings.some(function isStrip(finding,): boolean {
              return finding.startsWith('repair-stripped-worse-voted (',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS an edit no checker ruled on, since only an edit the checkers did not confirm and '
            + 'voted worse is stripped, and credits only the issue they confirmed',
          fn: async () => {
            const { outcome, } = await runChunk({
              script: {
                claims: [CLAIMS.chase, CLAIMS.bowl,],
                panelVote: 'supported',
                edits: [
                  { region: 1, newText: 'The cat loves chasing butterflies.', },
                  { region: 2, newText: 'The bowl is full.', },
                ],
                // The checkers rule on the first issue and leave the second alone.
                checks: function firstOnly() {
                  return [{ issue: 1, verdict: 'fixed', },];
                },
              },
            },);
            expect([
              outcome.repairedText.includes('The cat loves chasing butterflies.',),
              outcome.repairedText.includes('The bowl is full.',),
              outcome.resolvedIssueIds.length,
            ],).toEqual([true, true, 1,],);
          },
        },),
        it({
          name: 'TELLS the editor the roster\'s rule addendum and the line-structure fact of the enclosing '
            + 'chunk, which it reads nowhere else',
          fn: async () => {
            /** Rule only this roster adds. */
            const addendum = 'Keep every cat named as the archive names it.';
            const { sent, } = await runChunk({
              script: {
                claims: [CLAIMS.chase,],
                panelVote: 'supported',
                edits: [{ region: 1, newText: 'The cat loves chasing butterflies.', },],
                checks: everyIssue({ verdict: 'fixed', },),
              },
              lineStructured: true,
              models: {
                ...MODELS,
                editorRuleAddendum: addendum,
              },
            },);
            /** What the editor was sent. */
            const [editorSheet = '',] = sent.get('editor_report',) ?? [];
            expect([
              editorSheet.includes(addendum,),
              editorSheet.includes('This region\'s ORIGINAL IS line-structured',),
            ],).toEqual([true, true,],);
          },
        },),
      ],
    },),

    describe({
      name: repairPreparedDocument.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REJECTS an addition claim on an archive detail a cited reference states, before the '
            + 'panel, and says so in the run\'s findings, when preparation attested that detail',
          fn: async () => {
            /** Archive sentence a cited page states word for word. */
            const attested = 'She has an older sister.';
            /** Pair whose archive carries the attested detail. */
            const prepared = prepareDocumentPair({
              sourceText: '小猫在窗台上打盹。\n',
              targetText: `The kitten dozes on the windowsill. ${attested}\n`,
            },);
            /** Every exchange's messages, by stage. */
            const sent = new Map<string, string[]>();
            /** Run over the pair with the detail attested. */
            const result = await repairPreparedDocument({
              client: scriptedChunkClient({
                script: {
                  claims: [{
                    category: 'accuracy/addition',
                    severity: 'major',
                    summary: 'The older sister has no source.',
                    sourceQuote: '小猫在窗台上打盹。',
                    targetQuote: attested,
                  },],
                  panelVote: 'supported',
                  edits: [],
                  checks: everyIssue({ verdict: 'fixed', },),
                },
                sent,
              },),
              prepared: {
                ...prepared,
                attestedDetails: [{
                  archiveQuote: attested,
                  reference: 1,
                  referenceQuote: 'The kitten has an older sister.',
                  voices: 2,
                  heard: 3,
                },],
              },
              models: MODELS,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([
              result.repairedText,
              sent.has('panel_ballot',),
            ],).toEqual([prepared.targetText, false,],);
            expect(result.findings.some(function isScreen(finding,): boolean {
              return finding.startsWith('addition claim rejected before the panel, reference-attested:',);
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
