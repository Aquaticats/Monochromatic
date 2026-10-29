/**
 Tests for the consolidation's deciding half, composed end to end.

 WHAT THIS FILE EXISTS TO STOP is the defect the stage was built to close, at
 one level up. `floorConsolidateSlate`, `gateConsolidatedSlice` and
 `wrapConsolidation` were each built, tested, and called by nothing. A unit
 test of a part cannot say the part is reached, so these drive the composition
 and assert which rounds were bought.

 THE TRANSPORT ROUTES ON SHEET CONTENT, NOT ON CALL ORDER. A counter over
 calls looks like it works and breaks silently: a schema-mismatch retry or a
 quorum that proceeds early shifts every later index, and the wrong route then
 answers with a well-formed reply nobody notices is misaddressed. Each case
 asserts how many calls each route served, which is the router's own positive
 control.

 Cat-themed invention throughout; no corpus content appears here.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createSyntheticClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buildTranslateCandidates,
  describeSlate,
  type LaneText,
  rotateCandidates,
  TRANSLATE_LINE_STRUCTURE_CRITERION,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING,
  INELIGIBLE_STANDING_WITHHELD_FINDING,
  type ProposalValidity,
  settleConsolidation,
  UNDECIDED_GATE_SHIPS_PROPOSAL_FINDING,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { ballot, } from '@monochromatic-dev/translation-repair-test-fixture/ts/structural-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { tenseBlocksOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/tense-sheet.test-fixture.ts';

/**
 Logger the stage writes through, whose output is not under test.
 */
const l = tagged({ tag: 'consolidate-settle-test', },);

/**
 Roster of three, the smallest that can produce a two-to-one split.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 One seated voice, narrowed to the roster so a fixture cannot invent a model
 the provider catalogue does not carry.
 */
type FixtureModelId = (typeof ROSTER)[number];

/**
 Phrase separating the gate's sheet from the selector's, taken from the sheet
 itself rather than invented, so a rewording breaks the router loudly instead
 of silently routing every call to one side.

 CARRIES NO QUOTE CHARACTERS ON PURPOSE. The router reads the serialised
 exchange, where every quote inside a sheet arrives escaped, so a marker
 spelled with quotes matches nothing and every gate call is answered with a
 selector ballot the gate then reads as a schema mismatch. That is what the
 per-route counts exist to catch, and they caught exactly this.

 `lane-contest-wire.ts` carries the same sentence, which does not matter here:
 this stage never asks the lane contest anything, so within one settlement the
 phrase appears in the gate sheet alone.
 */
const GATE_MARKER = 'Return JSON: assessment one sentence';

/**
 The slice every case decides about.
 */
const SUBJECT = {
  sourceText: '猫在窗边睡着了。她四点醒来。',
  incumbentText: 'The cat fell asleep by the window and woke at four in the afternoon.',
};

/**
 Wording in place when the stage begins, written as the wrap rule would have
 it so a consolidation is not demoted by accident.
 */
const STANDING = 'The cat fell asleep by the window.\nShe woke at four.';

/**
 A consolidation that differs from what stands in its content, emitted as one
 line because producers do that.
 */
const FRESH = 'The cat fell asleep beside the window. She woke at four in the afternoon.';

/**
 Idiomatic final rewrite used to prove standing-text polish reachability.
 */
const POLISHABLE_STANDING = 'She viewed rainy days proactively and spent many a cozy afternoon with the other cats, while doing her best to stay curious and close to the cats around her.';

/**
 Faithful idiomatic rewrite of polishable standing text, as the refiner emits it.
 */
const POLISHED_STANDING = 'She kept a cheerful outlook on rainy days and spent many cozy afternoons with the other cats, doing her best to stay curious and close to those around her.';

/**
 That rewrite as it ships: wrapped at its semantic boundary before the polish
 gate judged it (one entry, 2026-09-03).
 */
const WRAPPED_POLISHED_STANDING = 'She kept a cheerful outlook on rainy days and spent many cozy afternoons with the other cats,\n'
  + 'doing her best to stay curious and close to those around her.';

/**
 Builds one voice as the producing half hands them over.

 @param modelId - voice that wrote it

 @param translation - wording it proposed

 @returns Voice shaped as the gather round returns one

 @example
 ```ts
 const voice = voiceOf({ modelId: ROSTER[0], translation: FRESH, },);
 ```
 */
function voiceOf(
  {
    modelId,
    translation,
  }: {
    readonly modelId: FixtureModelId;
    readonly translation: string;
  },
) {
  return {
    modelId,
    value: { translation, },
  };
}

/**
 Builds one structural verdict.

 @param modelId - voice the verdict belongs to

 @param valid - whether the guard passed it

 @returns Verdict shaped as the produce half reports one

 @example
 ```ts
 const checked = validityOf({ modelId: ROSTER[0], valid: true, },);
 ```
 */
function validityOf(
  { modelId, valid, }: { readonly modelId: string; readonly valid: boolean; },
): ProposalValidity {
  return {
    modelId,
    validation: valid
      ? {
        kind: 'valid',
        pageGrammar: 'strict',
      }
      : {
        kind: 'invalid',
        findings: ['The page as it stands is 2 blocks and your rendering is 1.',],
      },
  } as ProposalValidity;
}

/**
 Works out which numbered candidate carries one rendering, by replaying the
 rotation the judges will see.

 NOT HARDCODED, because the slate is rotated by a hash of the source so the
 incumbent does not sit in one position across a document. A fixture that
 guessed the number would pass or fail on the fixture's wording rather than on
 the stage's behaviour.

 @param texts - proposals reaching the slate, in roster order

 @param wanted - rendering whose position is sought

 @returns One-based ballot index naming it

 @example
 ```ts
 const best = positionOfText({ texts: [FRESH,], wanted: FRESH, },);
 ```
 */
function positionOfText(
  {
    texts,
    wanted,
    incumbentText = STANDING,
    laneTexts = [],
    sourceText = SUBJECT.sourceText,
  }: {
    readonly texts: readonly string[];
    readonly wanted: string;

    /**
     What the slate offers to keep; empty when the standing is withheld.
     */
    readonly incumbentText?: string;

    /**
     Lane texts the slate offers beside the proposals (class forty).
     */
    readonly laneTexts?: readonly LaneText[];

    /**
     Original the rotation is hashed from, for a case deciding another slice.
     */
    readonly sourceText?: string;
  },
): number {
  const built = buildTranslateCandidates({
    voices: texts.map(function toVoice(translation, at,) {
      return voiceOf({ modelId: ROSTER[at] ?? ROSTER[0], translation, },);
    },),
    translatorModelIds: ROSTER,
    incumbentText,
    lineStructured: false,
    laneTexts,
  },);

  const entries = describeSlate({
    candidates: rotateCandidates({
      candidates: built.candidates,
      sourceText,
    },),
  },);

  const found = entries.find(function carriesWanted(entry,) {
    return entry.text === wanted;
  },);
  if (found === undefined)
    throw new Error('the wanted rendering never reached the slate, so the fixture is wrong',);
  return found.index;
}

/**
 One-based number of the candidate block carrying a rendering on the sheet a
 judge was shown, or zero when absent. Read off the sheet because a run-off
 round renumbers the finalists.

 @param sent - serialised request a judge was shown

 @param wanted - fragment of the rendering whose number is sought

 @returns One-based candidate number, or zero

 @example
 ```ts
 const best = numberedOn({ sent, wanted: FRESH_MARK, },);
 ```
 */
function numberedOn(
  { sent, wanted, }: { readonly sent: string; readonly wanted: string; },
): number {
  /**
   Candidate heading marker.
   */
  const marker = 'CANDIDATE ';
  for (let cursor = sent.indexOf(marker,); cursor >= 0;) {
    /**
     Start of the candidate block whose marker follows this one, or the sheet's end.
     */
    const next = sent.indexOf(marker, cursor + marker.length,);
    if (sent.slice(cursor, (next === (-1)) ? sent.length : next,)
      .includes(wanted,))
      return Number(sent.charAt(cursor + marker.length,),);
    cursor = next;
  }
  return 0;
}

/**
 Reply body for one slate judge, fixed or scripted from the sheet it was
 shown and how many judge calls came before it.
 */
type JudgeReply = string | ((sent: string, call: number,) => string);

/**
 Reads one judge's reply body off a fixed string or a script.

 @param judgeReply - fixed body or script over the sheet

 @param sent - sheet this judge was shown

 @param call - judge calls before this one

 @returns Body this judge returns

 @example
 ```ts
 const body = judgeBodyOf({ judgeReply, sent, call: 0, },);
 ```
 */
function judgeBodyOf(
  {
    judgeReply,
    sent,
    call,
  }: {
    readonly judgeReply: JudgeReply;
    readonly sent: string;
    readonly call: number;
  },
): string {
  if ((typeof judgeReply) === 'string')
    return judgeReply;
  return judgeReply(sent, call,);
}

/**
 Builds a client that answers each round from its own script.

 @param judgeReply - body every slate judge returns, or a script over the sheet

 @param gateReply - body every gate voice returns

 @param served - counter the caller reads afterwards

 @returns Client over a routing transport

 @example
 ```ts
 const client = routedClient({ judgeReply, gateReply, served, },);
 ```
 */
function routedClient(
  {
    judgeReply,
    gateReply,
    served,
    judgeSheets,
    gateSheets,
  }: {
    readonly judgeReply: JudgeReply;
    readonly gateReply: string;
    readonly served: { judge: number; gate: number; };

    /**
     Where to record each slate judge's request, for the cases that read what
     the judges were shown rather than what they answered.
     */
    readonly judgeSheets?: string[];
    /**
     Where to record each gate voice's request, for the cases that read what
     the gate was shown.
     */
    readonly gateSheets?: string[];
  },
) {
  return createSyntheticClient({
    apiKey: 'test-key',
    transport: async function routingTransport(exchange,) {
      /**
       Everything the call is sending, which is where the sheet lives.
       */
      const sent = JSON.stringify(exchange,);

      /**
       Whether this call carries the gate's sheet rather than the selector's.
       */
      const isGate = sent.includes(GATE_MARKER,);
      /**
       Judge calls before this one, which a scripted reply may read.
       */
      const judgeCall = served.judge;
      if (isGate) {
        served.gate += 1;
        gateSheets?.push(sent,);
      }
      else {
        served.judge += 1;
        judgeSheets?.push(sent,);
      }

      /**
       Body this voice returns.
       */
      const content = isGate
        ? gateReply
        : judgeBodyOf({
          judgeReply,
          sent,
          call: judgeCall,
        },);
      return {
        status: 200,
        bodyText: `data: ${
          JSON.stringify({
            choices: [
              {
                index: 0,
                delta: { content, },
              },
            ],
          },)
        }\n\ndata: [DONE]\n\n`,
      };
    },
  },);
}

/**
 Builds direct client proving final polish remains reachable after slate decline.

 @param servedSchemas - schema names called in execution order

 @returns Client declining consolidation slate but approving final polish

 @example
 ```ts
 const client = standingPolishClient({ servedSchemas: [], });
 ```
 */
function standingPolishClient(
  { servedSchemas, }: { readonly servedSchemas: string[]; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by structured consolidation stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Structured schema naming current stage.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name ?? '';
      servedSchemas.push(schema,);
      /**
       Whether candidate ballot belongs to final naturalness selector.
       */
      const choosingPolish = JSON.stringify(request.messages,)
        .includes(POLISHED_STANDING,);
      /**
       Reply preserving declined consolidation while approving polish.
       */
      const value: unknown = (schema === 'refine_report')
        ? {
          rewrites: [{
            paragraph: 1,
            newText: POLISHED_STANDING,
          },],
        }
        : (schema === 'candidate_ballot')
          ? {
            best: choosingPolish ? 1 : 0,
            assessment: choosingPolish
              ? 'same meaning in more idiomatic English'
              : 'no consolidation clearly improves the standing text',
          }
          : (schema === 'consolidation_polish_gate')
            ? {
              verdict: 'polished',
              unsupported: [],
              dropped: [],
              assessment: 'same meaning in more idiomatic English',
            }
            : (schema === 'absolute_naturalness_review')
              ? {
                isAcceptable: true,
                findings: [],
                assessment: 'whole passage is publication-ready',
              }
              : {};
      if (!request.validate(value,))
        throw new Error(`synthetic ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by consolidation stages',);
    },
  };
}

/**
 Builds a slate judge's reply.

 @param best - ballot index, zero to decline

 @returns Reply body a judge would return

 @example
 ```ts
 const reply = judgeBallot({ best: 0, },);
 ```
 */
function judgeBallot({ best, }: { readonly best: number; },): string {
  return JSON.stringify({
    best,
    assessment: 'it says what the original says',
  },);
}

/**
 Runs one settlement over a scripted roster.

 @param voices - proposals reaching the stage

 @param validity - what the guard made of each

 @param standingText - wording in place, overridable to test its absence

 @param judgeReply - body every slate judge returns

 @param gateReply - body every gate voice returns

 @param producedFindings - what gathering and repairing recorded

 @param served - counter the caller holds, so a case whose settlement throws
 can still read what was bought before the throw

 @returns Settlement beside the calls each round served

 @example
 ```ts
 const { settled, served, } = await settleWith({ voices, validity, },);
 ```
 */
async function settleWith(
  {
    voices,
    validity,
    standingText = STANDING,
    judgeReply = judgeBallot({ best: 0, },),
    gateReply = ballot({ choice: 'standing', },),
    producedFindings = [],
    lineStructured = false,
    standingEligible = true,
    standingRefusal,
    laneTexts = [],
    runoffOverStanding = false,
    served = {
      judge: 0,
      gate: 0,
    },
    subject = SUBJECT,
  }: {
    readonly voices: readonly {
      readonly modelId: FixtureModelId;
      readonly value: { readonly translation: string; };
    }[];
    readonly validity: readonly ProposalValidity[];
    readonly standingText?: string;
    readonly judgeReply?: JudgeReply;
    readonly gateReply?: string;
    readonly producedFindings?: readonly string[];

    /**
     Whether the enclosing chunk is governed by the verse rule, which decides
     what the judges of this round are asked.
     */
    readonly lineStructured?: boolean;

    /**
     Whether the standing text passed the deterministic gate; false withholds
     it from the slate.
     */
    readonly standingEligible?: boolean;
    /**
     Why the deterministic rule refused the standing, shown to the gate.
     */
    readonly standingRefusal?: string;

    /**
     Lane texts offered on the slate beside the proposals (class forty).
     */
    readonly laneTexts?: readonly LaneText[];

    /**
     Whether a strict majority of the contest ballots called the archive
     flawed, so a tied slate over an eligible standing is run off (class one
     hundred six) and an undecided gate over it ships the slate's choice (class
     one hundred seventy-seven); the owner put both on the majority on
     2026-10-11.
     */
    readonly runoffOverStanding?: boolean;

    /**
     Calls each route served, which every case asserts on; a case expecting
     a throw passes its own, since nothing is returned to read it from.
     */
    readonly served?: {
      judge: number;
      gate: number;
    };

    /**
     The slice decided about, for a case needing another than every other
     case's.
     */
    readonly subject?: Parameters<typeof settleConsolidation>[0]['subject'];
  },
) {
  /**
   Every slate judge's request, for the cases reading what was shown.
   */
  const judgeSheets: string[] = [];
  /**
   Every gate voice's request, for the cases reading what the gate was shown.
   */
  const gateSheets: string[] = [];

  const settled = await settleConsolidation({
    standingMayShip: true,
    client: routedClient({
      judgeReply,
      gateReply,
      served,
      judgeSheets,
      gateSheets,
    },),
    roster: ROSTER,
    subject,
    voices,
    validity,
    producedFindings,
    standingText,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    lineStructured,
    standingEligible,
    ...((standingRefusal === undefined) ? {} : { standingRefusal, }),
    laneTexts,
    runoffOverStanding,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return {
    settled,
    served,
    judgeSheets,
    gateSheets,
  };
}

await describe({
  name: settleConsolidation.name,
  children: [
    it({
      name: 'BUYS NOTHING AT ALL FOR A SLATE THE GUARD REJECTED ENTIRELY, and records it as '
        + 'incumbent-only rather than as a decision. This is the case the band pair hit twice: '
        + 'one entry finished its repair round with five candidates and zero valid ones, in both runs, '
        + 'and a consolidation shipped at both',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: ROSTER.map(function toVoice(modelId,) {
            return voiceOf({ modelId, translation: FRESH, },);
          },),
          validity: ROSTER.map(function toVerdict(modelId,) {
            return validityOf({ modelId, valid: false, },);
          },),
        },);

        expect(settled.terminal,).toBe('incumbent-only',);
        expect(settled.text,).toBe(STANDING,);
        expect(served.judge,).toBe(0,);
        expect(served.gate,).toBe(0,);
      },
    },),

    it({
      name: 'KEEPS WHAT THE PRODUCE HALF RECORDED ON AN INCUMBENT-ONLY SLICE, which is the '
        + 'terminal those findings explain. This path ends before a judged round and before a '
        + 'gate round, and the findings rode inside those rounds, so what said WHY a slate had '
        + 'nothing valid on it was dropped at exactly the slices reporting that it had nothing',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: ROSTER.map(function toVoice(modelId,) {
            return voiceOf({ modelId, translation: FRESH, },);
          },),
          validity: ROSTER.map(function toVerdict(modelId,) {
            return validityOf({ modelId, valid: false, },);
          },),
          producedFindings: ['cat-gather-lost-a-voice',],
        },);

        expect(settled.terminal,).toBe('incumbent-only',);
        expect(settled.findings,).toContain('cat-gather-lost-a-voice',);
      },
    },),

    it({
      name: 'KEEPS THEM ON A SLICE WITH NO STANDING TEXT TOO, the other terminal ending before '
        + 'either round. Both were losing the same list, and neither loss depended on a cache: a '
        + 'fresh run dropped them exactly as a resumed one did',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: ROSTER.map(function toVoice(modelId,) {
            return voiceOf({ modelId, translation: FRESH, },);
          },),
          validity: ROSTER.map(function toVerdict(modelId,) {
            return validityOf({ modelId, valid: true, },);
          },),
          standingText: '',
          producedFindings: ['cat-gather-lost-a-voice',],
        },);

        expect(settled.terminal,).toBe('no-standing-text',);
        expect(settled.findings,).toContain('cat-gather-lost-a-voice',);
      },
    },),

    it({
      name: 'REPORTS EACH FINDING ONCE ON A PATH THAT REACHED BOTH ROUNDS, which is the trap in '
        + 'fixing the "KEEPS WHAT THE PRODUCE HALF RECORDED ON AN INCUMBENT-ONLY SLICE" and "KEEPS THEM ON '
        + 'A SLICE WITH NO STANDING TEXT TOO" cases. The judged round is already handed what the produce half '
        + 'recorded, so a settlement appending that list beside the judged round would report one '
        + 'lost voice twice and double every count taken over it',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: ROSTER.map(function toVoice(modelId,) {
            return voiceOf({ modelId, translation: FRESH, },);
          },),
          validity: ROSTER.map(function toVerdict(modelId,) {
            return validityOf({ modelId, valid: true, },);
          },),
          producedFindings: ['cat-gather-lost-a-voice',],
        },);

        expect(
          settled.findings
            .filter(function isTheOne(finding,): boolean {
              return finding === 'cat-gather-lost-a-voice';
            },)
            .length,
        ).toBe(1,);
      },
    },),

    it({
      name: 'READS AN ABSENT STANDING TEXT AHEAD OF AN EMPTY SLATE when a slice is both at once, '
        + 'because the missing standing text is the older fact: the floor would have refused a slate '
        + 'this slice was never going to use. A census counting terminal states has to be told which '
        + 'one it is',
      fn: async () => {
        const { settled, served, } = await settleWith({
          standingText: '',
          voices: ROSTER.map(function toVoice(modelId,) {
            return voiceOf({ modelId, translation: FRESH, },);
          },),
          validity: ROSTER.map(function toVerdict(modelId,) {
            return validityOf({ modelId, valid: false, },);
          },),
        },);

        expect(settled.terminal,).toBe('no-standing-text',);
        expect(served.judge,).toBe(0,);
        expect(served.gate,).toBe(0,);
      },
    },),

    it({
      name: 'KEEPS A REFUSED VOICE OFF THE SLATE AND ON THE RECORD, which the floor alone cannot do: '
        + 'it names refusals only when nothing survived, and run 8 carried 7 invalid candidates across '
        + 'slices that all shipped normally with nothing downstream able to see them',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: ROSTER.map(function toVoice(modelId,) {
            return voiceOf({ modelId, translation: FRESH, },);
          },),
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: false, },),
            validityOf({ modelId: ROSTER[2], valid: false, },),
          ],
        },);

        expect(settled.floor.kind,).toBe('proposals',);
        expect(settled.verdicts.length,).toBe(3,);
        expect(
          settled.verdicts.filter(function refused(verdict,) {
            return verdict.kind === 'invalid';
          },).length,
        ).toBe(2,);
        expect(settled.verdicts[1]?.findings.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'KEEPS THE STANDING TEXT WHEN EVERY JUDGE DECLINES, and never reaches the gate. The gate '
        + 'question is whether a consolidation beats what stands, and there is no consolidation to ask '
        + 'about once the judges have kept the incumbent',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: 0, },),
        },);

        // NAMED AS A DECLINE, which the merged terminal could not say. This
        // test always described a declining panel, and always asserted the
        // name a panel ENDORSING the archive would carry.
        expect(settled.terminal,).toBe('slate-declined-standing',);
        expect(settled.text,).toBe(STANDING,);
        expect(served.judge,).toBeGreaterThan(0,);
        expect(served.gate,).toBe(0,);
      },
    },),

    it({
      name: 'POLISHES AN ENDORSED STANDING TEXT after consolidation judges decline their slate',
      fn: async () => {
        /**
         Structured stages reached by settlement.
         */
        const servedSchemas: string[] = [];
        const settled = await settleConsolidation({
          standingEligible: true,
          client: standingPolishClient({ servedSchemas, }),
          roster: ROSTER,
          subject: {
            sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
            incumbentText: POLISHABLE_STANDING,
          },
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          producedFindings: [],
          standingText: POLISHABLE_STANDING,
          lineStructured: false,
          sliceIndex: 1,
          polishConfig: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          standingMayShip: true,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(settled.terminal,).toBe('slate-declined-standing',);
        expect(settled.text,).toBe(WRAPPED_POLISHED_STANDING,);
        expect(settled.polish?.kind,).toBe('settled',);
        expect(settled.polish?.kind === 'settled' ? settled.polish.changed : false,).toBe(true,);
        expect(servedSchemas,).toContain('refine_report',);
        expect(servedSchemas,).toContain('consolidation_polish_gate',);
      },
    },),

    it({
      name: 'REFUSES TO POLISH AN UNENDORSED STANDING TEXT after consolidation judges decline',
      fn: async () => {
        /**
         Structured stages reached before provenance refusal.
         */
        const servedSchemas: string[] = [];
        const settled = await settleConsolidation({
          standingEligible: true,
          client: standingPolishClient({ servedSchemas, }),
          roster: ROSTER,
          subject: {
            sourceText: '她总是乐观地看待下雨天，和猫友们度过了许多惬意的午后。',
            incumbentText: POLISHABLE_STANDING,
          },
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          producedFindings: [],
          standingText: POLISHABLE_STANDING,
          lineStructured: false,
          sliceIndex: 1,
          polishConfig: {
            refinerModelIds: [ROSTER[0],],
            judgeModelIds: ROSTER,
            gateModelIds: ROSTER,
            declaredNames: [],
            definitions: '',
          },
          standingMayShip: false,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(settled.terminal,).toBe('slate-declined-standing',);
        expect(settled.text,).toBe(POLISHABLE_STANDING,);
        expect(settled.polish,).toEqual({
          kind: 'not-run',
          reason: 'unsafe-baseline',
        },);
        expect(servedSchemas,).not.toContain('refine_report',);
        expect(servedSchemas,).not.toContain('consolidation_polish_gate',);
      },
    },),

    it({
      name: 'CARRIES THE PRODUCE HALF\'S FINDINGS INTO WHAT THE JUDGING HALF REPORTS, which ProducedSlate '
        + 'requires by contract and the stage did not do: a repair round\'s record stopped at the stage '
        + 'boundary, so a run could not tell a clean slate from one that took two attempts',
      fn: async () => {
        /**
         A finding only the gather round could have produced.
         */
        const gathered = 'hf:cat/Cat-A answered with nothing to ship, so its voice was lost';

        const { settled, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          producedFindings: [gathered,],
        },);

        expect(settled.decided?.findings.includes(gathered,),).toBe(true,);
      },
    },),

    it({
      name: 'SHIPS A CONSOLIDATION BOTH ROUNDS BACKED, CARRYING THE SEMANTIC LINE BREAKS both lanes '
        + 'apply at their own assembly step and this stage had none. REWRAPPED IS FALSE AND THAT IS '
        + 'THE POINT since 2026-08-22: the breaks are there before either decider reads the proposal, so '
        + 'the shipping wrap finds nothing left to correct. It asserted true until 2026-08-22, when '
        + 'what shipped was still being altered after the gate had approved it',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'consolidated', },),
        },);

        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.rewrapped,).toBe(false,);
        expect(settled.demoted,).toBe(false,);
        expect(settled.text.split('\n',).length,).toBeGreaterThan(1,);
        expect(served.gate,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'SHOWS THE SLATE JUDGES AND THE GATE what the pages the original cites say (class forty-one), when the '
        + 'slice carries them; the same settlement carrying none is the control, whose sheets never mention them',
      fn: async () => {
        /** What the cited page says, marked so a sheet carrying it is plain. */
        const references = 'CITED PAGE SAYS: the cat sleeps by the window until four.';
        /**
         Whether every judge sheet and every gate sheet of one settlement
         carries the references.

         @param referenceContext - slice's references, none left out

         @returns Judge and gate answers, false for a round never asked
         */
        async function carriedBy(
          { referenceContext, }: { readonly referenceContext?: string; },
        ): Promise<readonly boolean[]> {
          /** Sheets the settlement sent. */
          const { judgeSheets, gateSheets, } = await settleWith({
            subject: {
              ...SUBJECT,
              ...((referenceContext === undefined) ? {} : { referenceContext, }),
            },
            voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
            validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
            judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
            gateReply: ballot({ choice: 'consolidated', },),
          },);
          return [judgeSheets, gateSheets,].map(function carried(sheets,): boolean {
            return (sheets.length > 0) && sheets.every(function carries(sheet,): boolean {
              return sheet.includes(references,);
            },);
          },);
        }
        expect([
          await carriedBy({},),
          await carriedBy({ referenceContext: references, },),
        ],).toEqual([
          [false, false,],
          [true, true,],
        ],);
      },
    },),

    it({
      name: 'SHOWS THE SLATE JUDGES AND THE GATE the tense candidates of the texts they weigh, when the slice\'s entry '
        + 'records a death and names its subject; the same settlement for an entry recording none is the control',
      fn: async () => {
        /** Standing text told in the present, which both rounds weigh. */
        const presentStanding = 'The cat is asleep by the window.\nShe wakes at four.';
        /**
         Whether every judge sheet and every gate sheet of one settlement
         carries exactly one tense block.

         @param recorded - slice's death record, none left out

         @returns Judge and gate answers, false for a round never asked
         */
        async function carriedBy(
          { recorded, }: { readonly recorded?: { readonly lifeEnded: boolean; readonly subjectNames: readonly string[]; }; },
        ): Promise<readonly boolean[]> {
          /** Sheets the settlement sent. */
          const { judgeSheets, gateSheets, } = await settleWith({
            subject: {
              ...SUBJECT,
              ...recorded,
            },
            standingText: presentStanding,
            voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
            validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
            judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
            gateReply: ballot({ choice: 'consolidated', },),
          },);
          return [judgeSheets, gateSheets,].map(function carried(sheets,): boolean {
            return (sheets.length > 0) && sheets.every(function carries(sheet,): boolean {
              return tenseBlocksOf({ content: sheet, },).length === 1;
            },);
          },);
        }
        expect([
          await carriedBy({},),
          await carriedBy({
            recorded: {
              lifeEnded: true,
              subjectNames: ['The cat',],
            },
          },),
        ],).toEqual([
          [false, false,],
          [true, true,],
        ],);
      },
    },),

    it({
      name: 'SHOWS THE GATE THE FRONT-MATTER RULE on a front-matter slice, which the gate sheet carries only when '
        + 'the slice\'s syntax role reaches it; the prose slice every other case decides is the control',
      fn: async () => {
        /** The slice's metadata, whose name the alias repeats. */
        const sourceText = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';
        /** The archive's metadata, which stands. */
        const page = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n';
        /** The consolidation, valid under the front-matter rule. */
        const proposed = '---\nname: Maomao Cat\ninfo:\n  alias: Maomao Cat\n---\n';
        /** Opening words of the rule the gate reads for front matter. */
        const rule = 'The candidates are complete YAML front matter';
        const prose = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        const frontMatter = await settleWith({
          subject: {
            sourceText,
            incumbentText: page,
            syntax: 'front-matter',
          },
          standingText: page,
          voices: [voiceOf({ modelId: ROSTER[0], translation: proposed, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({
            best: positionOfText({
              texts: [proposed,],
              wanted: proposed,
              incumbentText: page,
              sourceText,
            },),
          },),
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        /**
         Whether every gate sheet of a settlement carries the rule.

         @param gateSheets - what the settlement's gate was sent

         @returns False when the gate was never asked
         */
        function everyGateSheetCarriesRule(gateSheets: readonly string[],): boolean {
          return (gateSheets.length > 0) && gateSheets.every(function carries(sheet,): boolean {
            return sheet.includes(rule,);
          },);
        }
        expect([
          [prose.settled.terminal, everyGateSheetCarriesRule(prose.gateSheets,), prose.gateSheets.length > 0,],
          [frontMatter.settled.terminal, everyGateSheetCarriesRule(frontMatter.gateSheets,), frontMatter.settled.text,],
        ],).toEqual([
          ['consolidated', false, true,],
          ['consolidated', true, proposed,],
        ],);
      },
    },),

    it({
      name: 'KEEPS THE STANDING TEXT WHEN THE GATE REFUSES A CONSOLIDATION THE JUDGES CHOSE, because '
        + 'the selector question is a preference while the gate question is what the original supports, '
        + 'and the second measured better on the lane pair',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'standing', },),
        },);

        expect(settled.terminal,).toBe('gate-kept-standing',);
        expect(settled.text,).toBe(STANDING,);
        expect(settled.demoted,).toBe(false,);
        expect(served.gate,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'WITHHOLDS AN INELIGIBLE STANDING FROM THE SLATE AND SHIPS THE PROPOSAL BOTH ROUNDS BACKED, '
        + 'recording the withholding: the owner\'s decision of 2026-09-04 after one entry shipped an '
        + 'archive front matter the gate had refused and lost the entry at assembly',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({
            best: positionOfText({
              texts: [FRESH,],
              wanted: FRESH,
              incumbentText: '',
            },),
          },),
          gateReply: ballot({ choice: 'consolidated', },),
          standingEligible: false,
        },);

        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(settled.findings.includes(INELIGIBLE_STANDING_WITHHELD_FINDING,),).toBe(true,);
        expect(served.judge,).toBeGreaterThan(0,);
        expect(served.gate,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'SHIPS THE PROPOSAL THE SLATE CHOSE when the standing is ineligible and the gate prefers it '
        + 'anyway (class one hundred eighty-five, one entry\'s slice 9, 2026-09-27): a standing the '
        + 'deterministic rule refused cannot ship whatever the gate thinks of it, and the owner\'s '
        + '2026-09-04 rule prefers the best valid proposal and fails the slice only where there is none',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({
            best: positionOfText({
              texts: [FRESH,],
              wanted: FRESH,
              incumbentText: '',
            },),
          },),
          gateReply: ballot({ choice: 'standing', },),
          standingEligible: false,
        },);
        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(settled.findings.some(function namesPreference(finding,): boolean {
          return finding.startsWith('gate-preferred-ineligible-standing',);
        },),).toBe(true,);
        expect(settled.gate?.choice,).toBe('standing',);
        expect(served.gate,).toBeGreaterThan(0,);

        const eligible = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'standing', },),
        },);
        expect(eligible.settled.terminal,).toBe('gate-kept-standing',);
        expect(eligible.settled.text,).toBe(STANDING,);
      },
    },),

    it({
      name: 'SHIPS THE PROPOSAL THE SLATE CHOSE when the standing is ineligible and the gate settles on '
        + 'neither (class fifty-four, one entry\'s slice 13, 2026-09-18): the gate\'s indecision is not a '
        + 'refusal, and with nothing eligible to keep the owner\'s 2026-09-04 rule prefers the best valid '
        + 'proposal; an eligible standing still keeps the slice on the same indecision',
      fn: async () => {
        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({
            best: positionOfText({
              texts: [FRESH,],
              wanted: FRESH,
              incumbentText: '',
            },),
          },),
          gateReply: ballot({ choice: 'neither', },),
          standingEligible: false,
        },);
        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(settled.findings.includes(UNDECIDED_GATE_SHIPS_PROPOSAL_FINDING,),).toBe(true,);
        expect(settled.gate?.choice,).toBe('neither',);
        expect(served.gate,).toBeGreaterThan(0,);

        const eligible = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'neither', },),
        },);
        expect(eligible.settled.terminal,).toBe('gate-kept-standing',);
        expect(eligible.settled.text,).toBe(STANDING,);
        expect(eligible.settled.findings.includes(UNDECIDED_GATE_SHIPS_PROPOSAL_FINDING,),).toBe(false,);
      },
    },),
    it({
      name: 'SHIPS THE PROPOSAL THE SLATE CHOSE when the gate settles on neither over an eligible standing '
        + 'every contest ballot called flawed (class one hundred seventy-seven, one entry\'s slice 13, '
        + 'owner answer 2026-09-27: "Slate\'s choice"; read from the same majority flag as the run-off since '
        + 'the owner answer of 2026-10-11, "Majority, like the run-off"); a gate that refuses the proposal '
        + 'still keeps the standing',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'neither', },),
          runoffOverStanding: true,
        },);
        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(settled.gate?.choice,).toBe('neither',);
        expect(settled.findings.some(function namesFlawedStanding(finding,): boolean {
          return finding.startsWith('undecided-gate-ships-proposal (standing flawed by a majority of contest ballots)',);
        },),).toBe(true,);

        const refused = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'standing', },),
          runoffOverStanding: true,
        },);
        expect(refused.settled.terminal,).toBe('gate-kept-standing',);
        expect(refused.settled.text,).toBe(STANDING,);
      },
    },),
    it({
      name: 'SEATS THE PROPOSAL TWO BALLOTS BACKED over a standing, flawed or not, though '
        + 'every cast ballot fell on that one proposal (one entry\'s slice 45, 2026-10-19): the writer names '
        + 'it at half weight, one disinterested judge at full weight and the third declines, so 1.5 from two '
        + 'ballots and no run-off to narrow, which kept the archive wording no ballot endorsed while a weight '
        + 'minimum of 2 stood (removed by the owner on 2026-10-25)',
      fn: async () => {
        /**
         Fragment naming the proposal on a sheet.
         */
        const FRESH_MARK = 'beside the window';
        /**
         Ballots by seat: the writer and one other name the proposal, the third declines.
         */
        function writerAndOneOther(sent: string,): string {
          if (sent.includes(ROSTER[2],))
            return judgeBallot({ best: 0, },);
          return judgeBallot({ best: numberedOn({ sent, wanted: FRESH_MARK, },), },);
        }
        const flawed = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: writerAndOneOther,
          gateReply: ballot({ choice: 'consolidated', },),
          runoffOverStanding: true,
        },);
        expect(flawed.settled.terminal,).toBe('consolidated',);
        expect(flawed.settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(flawed.settled.findings.includes('select-condemned-fallback-under-minimum',),).toBe(false,);
        // The first round decides, so no challenge round is asked.
        expect(flawed.judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes('A prior panel declined this exact slate',);
        },),).toBe(false,);

        const endorsed = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: writerAndOneOther,
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        // The flag no longer matters: the same ballots decide where nobody called the standing flawed.
        expect(endorsed.settled.terminal,).toBe('consolidated',);
        expect(endorsed.settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
      },
    },),
    it({
      name: 'STILL KEEPS THE ARCHIVE over a flawed standing when one judge alone named the proposal, so one '
        + 'model never decides (owner, 2026-09-09), and when no proposal passed the floor, so the archive '
        + 'stays at a slice with no valid wording (owner, 2026-09-27)',
      fn: async () => {
        /**
         Fragment naming the proposal on a sheet.
         */
        const FRESH_MARK = 'beside the window';
        /**
         Ballots by seat: only the second seat names the proposal.
         */
        function oneJudgeAlone(sent: string,): string {
          if (sent.includes(ROSTER[1],))
            return judgeBallot({ best: numberedOn({ sent, wanted: FRESH_MARK, },), },);
          return judgeBallot({ best: 0, },);
        }
        const alone = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: oneJudgeAlone,
          gateReply: ballot({ choice: 'consolidated', },),
          runoffOverStanding: true,
        },);
        expect(alone.settled.terminal,).toBe('slate-declined-standing',);
        expect(alone.settled.text,).toBe(STANDING,);

        const nothingValid = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: false, },),],
          gateReply: ballot({ choice: 'consolidated', },),
          runoffOverStanding: true,
        },);
        expect(nothingValid.settled.terminal,).toBe('incumbent-only',);
        expect(nothingValid.settled.text,).toBe(STANDING,);
        expect(nothingValid.served.judge,).toBe(0,);
      },
    },),
    it({
      name: 'JUDGES THE LANE TEXTS OVER AN EMPTY STANDING (class eighty-seven, one entry\'s slice 89, '
        + '2026-09-22): the contest declined both lanes at a passage the archive never carried, so nothing '
        + 'stood, and the no-standing-text exit ran ahead of the class forty offer; the translate lane\'s '
        + 'text passed the rule and was never put to a judge, the slice shipped nothing and the entry '
        + 'stopped at publish. With a lane text to offer, the slate is judged and the winner gated',
      fn: async () => {
        /**
         The one lane text the rule admits, the translate lane\'s.
         */
        const laneTexts: readonly LaneText[] = [
          {
            lane: 'translate',
            text: FRESH,
          },
        ];
        const { settled, served, } = await settleWith({
          voices: [],
          validity: [],
          standingText: '',
          standingEligible: false,
          laneTexts,
          judgeReply: judgeBallot({
            best: positionOfText({
              texts: [],
              wanted: FRESH,
              incumbentText: '',
              laneTexts,
            },),
          },),
          gateReply: ballot({ choice: 'consolidated', },),
        },);

        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(served.judge,).toBeGreaterThan(0,);
        expect(served.gate,).toBeGreaterThan(0,);

        /**
         Calls the unoffered settlement bought.
         */
        const unofferedServed = {
          judge: 0,
          gate: 0,
        };
        /**
         Settlement where no lane text is on offer either.
         */
        const unoffered = await settleWith({
          voices: [],
          validity: [],
          standingText: '',
          standingEligible: false,
          served: unofferedServed,
        },);
        // WITHOUT A LANE TEXT NO WORDING PASSES THE RULE, so the slice keeps the
        // archive's and the page ships with it reported (owner, 2026-09-27, "Keep
        // archive, ship"; ledger E4), unbought. It used to settle with no polish,
        // and the final naturalness check stopped the entry at persist after
        // every later slice was bought.
        expect(unoffered.settled.terminal,).toBe('no-standing-text',);
        expect(unoffered.settled.archiveKept,).toBe(true,);
        expect(unoffered.settled.findings.some(function namesIt(finding,): boolean {
          return finding.startsWith('no-valid-wording',);
        },),).toBe(true,);
        expect(unoffered.settled.polish,).toStrictEqual({
          kind: 'not-run',
          reason: 'unsafe-baseline',
        },);
        expect(unofferedServed,).toStrictEqual({
          judge: 0,
          gate: 0,
        },);
      },
    },),

    it({
      name: 'CHALLENGES A TIED SLATE ONCE when the standing is ineligible (class fifty-five, one '
        + 'entry\'s slice 13, 2026-09-18): the translate lane\'s run-off challenge of class fifty-three reaches the '
        + 'consolidation, whose tie over a withheld standing otherwise stops the entry; an eligible '
        + 'standing keeps the slice on the same tie without a second round',
      fn: async () => {
        /**
         Second proposal, so a slate of two can tie.
         */
        const OTHER = 'The cat naps on the sill each afternoon.';
        /**
         Wording the challenge round's task carries and the first round's does not.
         */
        const CHALLENGE_MARKER = 'A prior panel declined this exact slate';
        /**
         Ballots: the first round splits one voice each way and a third declines,
         the challenge round backs candidate 1 unanimously.
         */
        function tiedThenSettled(sent: string, call: number,): string {
          if (sent.includes(CHALLENGE_MARKER,))
            return judgeBallot({ best: 1, },);
          return judgeBallot({ best: [1, 2, 0,][call % 3] ?? 0, },);
        }
        const { settled, served, judgeSheets, } = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
          ],
          judgeReply: tiedThenSettled,
          gateReply: ballot({ choice: 'consolidated', },),
          standingEligible: false,
        },);
        expect(settled.terminal,).toBe('consolidated',);
        expect(settled.findings.includes('translate-declined-retried',),).toBe(true,);
        expect(judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },),).toBe(true,);
        expect(served.gate,).toBeGreaterThan(0,);

        const eligible = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
          ],
          judgeReply: tiedThenSettled,
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        expect(eligible.settled.terminal,).toBe('slate-declined-standing',);
        expect(eligible.judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'RUNS THE RUN-OFF OVER AN ELIGIBLE STANDING when a strict majority of contest ballots called the archive '
        + 'flawed (class one hundred six, one entry\'s slice 9, 2026-09-24): the tie is challenged and a '
        + 'round that decides ships its choice; a run-off that stays undecided ships the standing '
        + 'rather than stopping the slice',
      fn: async () => {
        /**
         Second proposal, so a slate of two can tie.
         */
        const OTHER = 'The cat naps on the sill each afternoon.';
        /**
         Fragments naming each proposal on a sheet, since the wrap step may
         rewrap a proposal across lines and the whole text then never matches.
         */
        const FRESH_MARK = 'beside the window';
        const OTHER_MARK = 'naps on the sill';
        /**
         Wording the challenge round's task carries and the first round's does not.
         */
        const CHALLENGE_MARKER = 'A prior panel declined this exact slate';
        /**
         Ballots: the first round splits one voice each way between the two
         proposals and a third declines; the challenge round backs the fresh
         proposal unanimously.
         */
        function tiedThenSettled(sent: string, call: number,): string {
          if (sent.includes(CHALLENGE_MARKER,))
            return judgeBallot({ best: numberedOn({ sent, wanted: FRESH_MARK, },), },);
          return judgeBallot({
            best: [
              numberedOn({ sent, wanted: FRESH_MARK, },),
              numberedOn({ sent, wanted: OTHER_MARK, },),
              0,
            ][call % 3] ?? 0,
          },);
        }
        const decided = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
          ],
          judgeReply: tiedThenSettled,
          gateReply: ballot({ choice: 'consolidated', },),
          runoffOverStanding: true,
        },);
        expect(decided.settled.terminal,).toBe('consolidated',);
        expect(decided.settled.findings.includes('translate-declined-retried',),).toBe(true,);
        expect(decided.judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },),).toBe(true,);

        /**
         Ballots that tie on every round: one voice each way between the two
         proposals, a third declining.
         */
        function tiedThroughout(sent: string, call: number,): string {
          return judgeBallot({
            best: [
              numberedOn({ sent, wanted: FRESH_MARK, },),
              numberedOn({ sent, wanted: OTHER_MARK, },),
              0,
            ][call % 3] ?? 0,
          },);
        }
        const undecided = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
          ],
          judgeReply: tiedThroughout,
          gateReply: ballot({ choice: 'consolidated', },),
          runoffOverStanding: true,
        },);
        expect(undecided.settled.terminal,).toBe('slate-declined-standing',);
        expect(undecided.settled.text,).toBe(STANDING,);
        expect(undecided.judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'RUNS OFF THE TIED FINALISTS when an eligible standing drew no ballot in a three-way tie, so the '
        + 'same panel decides between the tied proposals (owner answer 2026-10-11, "Run off the tied finalists"): '
        + 'the run-off names the finalist it backs and the consolidation ships it',
      fn: async () => {
        /**
         Three proposals, one per roster voice, so the three-way tie leaves the
         standing at zero ballots, as the live slice did.
         */
        const OTHER = 'The cat naps on the sill each afternoon.';
        /**
         Third proposal, the one the run-off backs.
         */
        const THIRD = 'The cat dozes in a patch of sun, then wakes at four.';
        /**
         Fragment naming the third proposal on a sheet.
         */
        const THIRD_MARK = 'patch of sun';
        /**
         Fragments naming the first two proposals on a sheet.
         */
        const FRESH_MARK = 'beside the window';
        const OTHER_MARK = 'naps on the sill';
        /**
         Wording the challenge round's task carries and the first round's does not.
         */
        const CHALLENGE_MARKER = 'A prior panel declined this exact slate';
        /**
         Ballots: the first round gives each proposal one voice, so no candidate
         leads; the run-off round votes for the third proposal.
         */
        function splitThenThird(sent: string, call: number,): string {
          if (sent.includes(CHALLENGE_MARKER,))
            return judgeBallot({ best: numberedOn({ sent, wanted: THIRD_MARK, },), },);
          return judgeBallot({
            best: [
              numberedOn({ sent, wanted: FRESH_MARK, },),
              numberedOn({ sent, wanted: OTHER_MARK, },),
              numberedOn({ sent, wanted: THIRD_MARK, },),
            ][call % 3] ?? 0,
          },);
        }
        const decided = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
            voiceOf({ modelId: ROSTER[2], translation: THIRD, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
            validityOf({ modelId: ROSTER[2], valid: true, },),
          ],
          judgeReply: splitThenThird,
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        expect(decided.settled.terminal,).toBe('consolidated',);
        expect(decided.settled.findings.includes('translate-declined-retried',),).toBe(true,);
        expect(decided.judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'SHOWS THE RUN-OFF OVER THE TIED FINALISTS THE TENSE CANDIDATES of its texts as the first round shows them, '
        + 'since every ask of a slate carries the same blocks, when the entry records a death',
      fn: async () => {
        /**
         Standing told in the present, which the round and the run-off both weigh.
         */
        const presentStanding = 'The cat is asleep by the window.\nShe wakes at four.';
        /**
         Other proposal, worded in the present too.
         */
        const OTHER = 'The cat naps on the sill each afternoon.';
        /**
         Third proposal, the one the run-off backs.
         */
        const THIRD = 'The cat dozes in a patch of sun, then wakes at four.';
        /**
         Wording the challenge round's task carries and the first round's does not.
         */
        const CHALLENGE_MARKER = 'A prior panel declined this exact slate';
        /**
         Ballots: the first round gives each proposal one voice, so no candidate
         leads; the run-off round votes for the third proposal.
         */
        function splitThenThird(sent: string, call: number,): string {
          if (sent.includes(CHALLENGE_MARKER,))
            return judgeBallot({ best: numberedOn({ sent, wanted: 'patch of sun', },), },);
          return judgeBallot({
            best: [
              numberedOn({ sent, wanted: 'beside the window', },),
              numberedOn({ sent, wanted: 'naps on the sill', },),
              numberedOn({ sent, wanted: 'patch of sun', },),
            ][call % 3] ?? 0,
          },);
        }
        const decided = await settleWith({
          subject: {
            ...SUBJECT,
            lifeEnded: true,
            subjectNames: ['The cat',],
          },
          standingText: presentStanding,
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
            voiceOf({ modelId: ROSTER[2], translation: THIRD, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
            validityOf({ modelId: ROSTER[2], valid: true, },),
          ],
          judgeReply: splitThenThird,
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        /**
         Judge sheets of the run-off round alone.
         */
        const challenged = decided.judgeSheets.filter(function inRunOff(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },);
        expect(challenged.length > 0,).toBe(true,);
        expect(decided.judgeSheets.every(function carriesOneBlock(sheet,): boolean {
          return tenseBlocksOf({ content: sheet, },).length === 1;
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS THE SINGLE-ROUND TIE when the standing drew a ballot in it, so no second ask is bought '
        + 'for a tie the standing took part in (owner answer 2026-10-11, "Run off the tied finalists")',
      fn: async () => {
        /**
         Fragment naming the standing on a sheet.
         */
        const STANDING_MARK = 'by the window';
        /**
         Fragments naming the two proposals on a sheet.
         */
        const FRESH_MARK = 'beside the window';
        const OTHER_MARK = 'naps on the sill';
        const OTHER = 'The cat naps on the sill each afternoon.';
        /**
         Wording the challenge round's task carries and the first round's does not.
         */
        const CHALLENGE_MARKER = 'A prior panel declined this exact slate';
        /**
         Ballots: the first judge backs the fresh proposal, the second the
         standing and the third the other proposal, so the standing draws a
         ballot and the tie is the single round's own.
         */
        function splitWithStanding(sent: string, call: number,): string {
          return judgeBallot({
            best: [
              numberedOn({ sent, wanted: FRESH_MARK, },),
              numberedOn({ sent, wanted: STANDING_MARK, },),
              numberedOn({ sent, wanted: OTHER_MARK, },),
            ][call % 3] ?? 0,
          },);
        }
        const { settled, served, judgeSheets, } = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
          ],
          judgeReply: splitWithStanding,
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        expect(settled.terminal,).toBe('slate-declined-standing',);
        expect(settled.text,).toBe(STANDING,);
        expect(served.judge,).toBe(3,);
        expect(judgeSheets.some(function challenged(sheet,): boolean {
          return sheet.includes(CHALLENGE_MARKER,);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'KEEPS THE STANDING when the run-off over the tied finalists ties again, after exactly one '
        + 'run-off ask, so the standing ships as it did before the run-off (owner answer 2026-10-11, '
        + '"Run off the tied finalists")',
      fn: async () => {
        const OTHER = 'The cat naps on the sill each afternoon.';
        const THIRD = 'The cat dozes in a patch of sun, then wakes at four.';
        const FRESH_MARK = 'beside the window';
        const OTHER_MARK = 'naps on the sill';
        const THIRD_MARK = 'patch of sun';
        /**
         Ballots that split three ways on every round, so the run-off ties too.
         */
        function splitThroughout(sent: string, call: number,): string {
          return judgeBallot({
            best: [
              numberedOn({ sent, wanted: FRESH_MARK, },),
              numberedOn({ sent, wanted: OTHER_MARK, },),
              numberedOn({ sent, wanted: THIRD_MARK, },),
            ][call % 3] ?? 0,
          },);
        }
        const { settled, served, } = await settleWith({
          voices: [
            voiceOf({ modelId: ROSTER[0], translation: FRESH, },),
            voiceOf({ modelId: ROSTER[1], translation: OTHER, },),
            voiceOf({ modelId: ROSTER[2], translation: THIRD, },),
          ],
          validity: [
            validityOf({ modelId: ROSTER[0], valid: true, },),
            validityOf({ modelId: ROSTER[1], valid: true, },),
            validityOf({ modelId: ROSTER[2], valid: true, },),
          ],
          judgeReply: splitThroughout,
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        expect(settled.terminal,).toBe('slate-declined-standing',);
        expect(settled.text,).toBe(STANDING,);
        expect(served.judge,).toBe(6,);
      },
    },),
    it({
      name: 'KEEPS THE STANDING ON AN UNDECIDED GATE when no majority of contest ballots called the archive '
        + 'flawed, and ships the slate\'s choice when one did though not every ballot did (owner answer '
        + '2026-10-11, "Majority, like the run-off"; TianqiChen66622 slice 4 had four of five)',
      fn: async () => {
        const kept = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'neither', },),
          runoffOverStanding: false,
        },);
        expect(kept.settled.terminal,).toBe('gate-kept-standing',);
        expect(kept.settled.text,).toBe(STANDING,);
        expect(kept.settled.findings.includes(FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING,),).toBe(false,);

        const forfeit = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'neither', },),
          runoffOverStanding: true,
        },);
        expect(forfeit.settled.terminal,).toBe('consolidated',);
        expect(forfeit.settled.text.replaceAll('\n', ' ',),).toBe(FRESH,);
        expect(forfeit.settled.findings.includes(FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING,),).toBe(true,);
      },
    },),
    it({
      name: 'SHOWS THE GATE WHY THE STANDING CANNOT SHIP when it is ineligible (class fifty-six, one '
        + 'entry\'s slice 33, 2026-09-18), and shows nothing of the kind over an eligible standing',
      fn: async () => {
        /**
         Why the deterministic rule refused the standing, as the run log words it.
         */
        const refusal = 'Your translation carries the pronoun untranslated as Ta (1 time)';
        const { settled, gateSheets, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({
            best: positionOfText({
              texts: [FRESH,],
              wanted: FRESH,
              incumbentText: '',
            },),
          },),
          gateReply: ballot({ choice: 'consolidated', },),
          standingEligible: false,
          standingRefusal: refusal,
        },);
        expect(settled.terminal,).toBe('consolidated',);
        expect(gateSheets.length,).toBeGreaterThan(0,);
        expect(gateSheets.every(function toldWhy(sheet,): boolean {
          return sheet.includes(refusal,);
        },),).toBe(true,);

        const eligible = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: positionOfText({ texts: [FRESH,], wanted: FRESH, },), },),
          gateReply: ballot({ choice: 'consolidated', },),
        },);
        expect(eligible.gateSheets.length,).toBeGreaterThan(0,);
        expect(eligible.gateSheets.some(function saysCannotShip(sheet,): boolean {
          return sheet.includes('CANNOT SHIP',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'KEEPS THE ARCHIVE when the standing is ineligible and every judge declines the slate in both '
        + 'rounds, and ships no proposal by a fixed order that no two judges named, rather than stopping the '
        + 'entry (owner, 2026-09-27, "Keep archive, ship", and 2026-10-25, "Two judges must name it", '
        + 'superseding the shipping by preference of "Preference + polish")',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: 0, },),
          gateReply: ballot({ choice: 'consolidated', },),
          standingEligible: false,
        },);
        expect(settled.terminal,).toBe('slate-declined-standing',);
        expect(settled.archiveKept,).toBe(true,);
        expect(settled.findings.includes('translate-slate-declined-archive-kept',),).toBe(true,);
        expect(settled.findings.some(function byPreference(finding,): boolean {
          return finding.startsWith('translate-slate-declined-shipped-by-preference',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'KEEPS THE ARCHIVE when the standing is ineligible and no proposal survived the floor, before '
        + 'any judge is bought, and the page ships with it reported (owner, 2026-09-27, "Keep archive, '
        + 'ship"; until then this case stopped the entry)',
      fn: async () => {
        /**
         Calls served, held here so a settlement that throws would still show
         what it bought; reading them off a return left this check at its
         initial zero whatever was bought.
         */
        const served = {
          judge: 0,
          gate: 0,
        };
        const { settled, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: false, },),],
          standingEligible: false,
          served,
        },);
        expect(settled.terminal,).toBe('incumbent-only',);
        expect(settled.archiveKept,).toBe(true,);
        expect(settled.findings.some(function namesIt(finding,): boolean {
          return finding.startsWith('no-valid-wording',);
        },),).toBe(true,);
        expect(settled.polish,).toStrictEqual({
          kind: 'not-run',
          reason: 'unsafe-baseline',
        },);
        expect(served,).toStrictEqual({
          judge: 0,
          gate: 0,
        },);
      },
    },),

    it({
      name: 'KEEPS THE ARCHIVE when the gate settles on the refused standing, here through a wrap that '
        + 'erased the only difference: the proposal the floor was told is valid is the refused standing '
        + 'itself, so what the gate leaves in place is wording the rule refused',
      fn: async () => {
        const { settled, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: STANDING, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          judgeReply: judgeBallot({ best: 1, },),
          gateReply: ballot({ choice: 'consolidated', },),
          standingEligible: false,
        },);
        expect(settled.terminal,).toBe('wrap-erased-difference',);
        expect(settled.archiveKept,).toBe(true,);
        expect(settled.polish,).toStrictEqual({
          kind: 'not-run',
          reason: 'unsafe-baseline',
        },);
      },
    },),

    it({
      name: 'KEEPS THE ARCHIVE when the withheld slate reaches the judges with no candidate at all: the one '
        + 'proposal the floor passed says nothing, so the judging returns "nothing proposed" with a voice '
        + 'heard (ledger B39; this case named an absence the judging stopped raising there)',
      fn: async () => {
        /**
         Calls served.
         */
        const served = {
          judge: 0,
          gate: 0,
        };
        const { settled, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: '', },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          standingEligible: false,
          served,
        },);
        expect(settled.terminal,).toBe('slate-unjudged-standing',);
        expect(settled.archiveKept,).toBe(true,);
        expect(settled.findings,).toContain('translate-no-candidate',);
        expect(settled.polish,).toStrictEqual({
          kind: 'not-run',
          reason: 'unsafe-baseline',
        },);
        expect(served,).toStrictEqual({
          judge: 0,
          gate: 0,
        },);
      },
    },),

    it({
      name: 'SETTLES A PURE RE-WRAPPING WITHOUT BUYING EITHER ROUND, which is what changed on 2026-08-22. '
        + 'This case asserted wrap-erased-difference until 2026-08-22: the proposal reached the slate '
        + 'unwrapped, both deciders spent ballots on where the lines broke, and the shipping wrap '
        + 'demoted it at the end. Wrapped before the slate it IS the standing text, the candidate '
        + 'dedup folds it in, and nothing is bought. Since 2026-09-29 the dedup reads the wording as the '
        + 'page shows it (ledger B26), so a proposal whose soft breaks fall anywhere else folds in too',
      fn: async () => {
        /**
         The standing text as a producer that ignored the wrap rule emits it.
         */
        const unwrapped = STANDING.replaceAll('\n', ' ',);

        const { settled, served, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: unwrapped, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          // No judge is asked, which `served.judge` pins, so the ballot a
          // judge would cast is never read.
          judgeReply: judgeBallot({ best: 1, },),
          gateReply: ballot({ choice: 'consolidated', },),
        },);

        expect(settled.terminal,).toBe('slate-unjudged-standing',);
        expect(settled.text,).toBe(STANDING,);
        expect(settled.demoted,).toBe(false,);
        expect(served.judge,).toBe(0,);
        expect(served.gate,).toBe(0,);
      },
    },),

    it({
      name: 'SHOWS A GOVERNED ROUND\'S JUDGES THE RULE AGAINST MERGING LINES. One change gave this fact to the '
        + 'consolidation PRODUCERS and stopped there, so for a day the judges weighed proposals written to '
        + 'unmerge a passage against a criterion telling them a shape the ORIGINAL lacks is no fault. This '
        + 'reads the judges\' own request rather than the settlement, because a settlement decided the right '
        + 'way on a sheet missing the rule would look identical',
      fn: async () => {
        const { judgeSheets, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          lineStructured: true,
        },);

        expect(judgeSheets.length,).toBeGreaterThan(0,);
        expect(
          judgeSheets.some(function carriesCriterion(sheet,): boolean {
            return sheet.includes(TRANSLATE_LINE_STRUCTURE_CRITERION,);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES IT OUT OF A ROUND IT DOES NOT GOVERN, which is what makes the "SHOWS A GOVERNED ROUND\'S '
        + 'JUDGES THE RULE AGAINST MERGING LINES" case evidence. A '
        + 'sheet carrying the criterion unconditionally would satisfy that one just as well and would mean '
        + 'the flag this function has always received was still reaching nobody',
      fn: async () => {
        const { judgeSheets, } = await settleWith({
          voices: [voiceOf({ modelId: ROSTER[0], translation: FRESH, },),],
          validity: [validityOf({ modelId: ROSTER[0], valid: true, },),],
          lineStructured: false,
        },);

        expect(judgeSheets.length,).toBeGreaterThan(0,);
        expect(
          judgeSheets.some(function carriesCriterion(sheet,): boolean {
            return sheet.includes(TRANSLATE_LINE_STRUCTURE_CRITERION,);
          },),
        ).toBe(false,);
      },
    },),
  ],
},);
