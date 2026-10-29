/**
 Tests for the naturalness phase: when it runs, when it rolls back, and that
 a refinement-only change reaches the shipped text.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { IssueAuthorship, } from '@monochromatic-dev/translation-repair-house-rule';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  CheckerQuorumError,
  type ChunkRepairOutcome,
  type RepairModels,
  type RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly';
import {
  OverlapRefusedError,
  UnpreparedSliceError,
} from '@monochromatic-dev/translation-repair-stage';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  persistRefinePhaseSlice,
  type RefinedSliceSettlement,
  runRefinePhase,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Logger for the phase under test.
 */
const l = tagged({ tag: 'refine-phase-test', },);

/**
 Repaired slice text, one long single-line paragraph so it is eligible.
 */
const REPAIRED_TEXT =
  'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is moving across the floor she is following it without any hurry at all.';

/**
 Smoother rendering of the same content.
 */
const SMOOTH_TEXT =
  'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor she follows it without hurry.';

/**
 Repaired text shorter than the lane's minimum paragraph, so it is never
 offered to a rewriter.
 */
const SHORT_TEXT = 'The cat naps on the windowsill.';

/**
 Original the refinement is checked against.
 */
const SOURCE_TEXT = '猫猫每天下午都在窗台上晒太阳。';

/**
 Editor named as the author of `T1` where a case sets one.

 KEPT OUT OF THE REFINER ROSTER so a stored authorship carries two
 distinguishable ids, which is what lets an assertion tell a union of both
 stages apart from either stage alone.
 */
const EDITOR_WHO_DID_NOT_REFINE: RosterModelId = SEAT_HYPER_VISION;

/**
 Refiner the scripted client answers as, which is the fixture's other editor.
 */
const REFINER_THAT_REWROTE: RosterModelId = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Roster with the lane on, refiners disjoint from checkers.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [
    REFINER_THAT_REWROTE,
    EDITOR_WHO_DID_NOT_REFINE,
  ],
  judgeModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
  refinerModelIds: [REFINER_THAT_REWROTE,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 Slice pair covering the whole fixture translation.
 */
const SLICES: readonly ChunkPair[] = [
  {
    source: {
      sliceIndex: 0,
      text: SOURCE_TEXT,
      startOffset: 0,
      endOffset: SOURCE_TEXT.length,
      nodes: [],
    },
    target: {
      sliceIndex: 0,
      text: REPAIRED_TEXT,
      startOffset: 0,
      endOffset: REPAIRED_TEXT.length,
      nodes: [],
    },
  },
];

/**
 Authorship of hand-written fixture text: no model wrote it, so no checker
 in a case that leaves this alone is judging its own work.
 */
const NO_MODEL_WROTE_THE_FIXTURE: IssueAuthorship = {
  perIssue: {},
  everyIssue: [],
};

/**
 Successful refiner calls in flight and peak observed by fixture.
 */
type RefinerConcurrency = {
  now: number;
  peak: number;
  started: number;

  /**
   Start positions in the order their calls finished, so a test can show a
   later call answering first rather than assume it (ledger T6).
   */
  readonly finished: number[];

  /**
   Gate the first-started call holds on until the second-started call has
   finished, where a case needs the second to answer first and both to be in
   flight at once whatever the machine's load.
   */
  readonly secondFinished?: PromiseWithResolvers<undefined>;
};

/**
 Delays refiner answers so a test can observe driver overlap.

 First refiner waits longer than second, making completion order differ from
 input order when both are active.

 @param inner - client providing scripted answers

 @param activity - mutable test instrument for active refiner calls

 @returns Client forwarding every call after measuring refiners

 @example
 ```ts
 const client = measuringRefiners({ inner, activity, },);
 ```
 */
function measuringRefiners(
  {
    inner,
    activity,
  }: {
    readonly inner: ModelClient;
    readonly activity: RefinerConcurrency;
  },
): ModelClient {
  return {
    chatText: inner.chatText,
    chatJson: async (request) => {
      if (request.responseFormat
        ?.json_schema
        .name
        === 'refine_report') {
        /**
         Start position deciding deterministic delay.
         */
        const startPosition = activity.started;
        activity.started += 1;
        activity.now += 1;
        activity.peak = Math.max(
          activity.peak,
          activity.now,
        );
        // ORDERED BY A GATE where the case supplies one: the head start alone
        // let the first call finish first 1 time in 8 at 0.2 CPU in the
        // consolidation driver's twin of this fixture (2026-09-27). A phase that
        // never overlaps leaves the first call held, and the file fails on the
        // unsettled wait rather than passing.
        await (((startPosition === 0) && (activity.secondFinished !== undefined))
          ? activity.secondFinished.promise
          : wait(startPosition === 0 ? 20 : 5,));
        activity.now -= 1;
        activity.finished.push(startPosition,);
        if (startPosition === 1)
          activity.secondFinished?.resolve(undefined,);
      }
      return await inner.chatJson(request,);
    },
    quotas: inner.quotas,
  };
}

/**
 An issue the panel ruled on that `T1` does not resolve, which is every
 accepted issue of a slice whose accuracy patch lost (ledger L11).
 */
type UnresolvedIssue = {
  /**
   Issue id.
   */
  readonly issueId: string;

  /**
   Panel's ruling, which decides whether the recheck owes it a round.
   */
  readonly status: 'accepted' | 'rejected' | 'needs-human';
};

/**
 Builds one settled accuracy outcome.

 @param resolvedIssueIds - issues the checkers confirmed in `T1`

 @param authorship - who wrote `T1`, which the phase must union with its
 own refiners on any slice where the rewrite ships

 @param unresolvedIssues - issues the panel ruled on that `T1` does not
 resolve, none by default

 @returns Outcome the phase refines

 @example
 ```ts
 const outcome = settledOutcome({ resolvedIssueIds: [], authorship, },);
 ```
 */
function settledOutcome(
  {
    resolvedIssueIds,
    authorship,
    unresolvedIssues = [],
  }: {
    readonly resolvedIssueIds: readonly string[];
    readonly authorship: IssueAuthorship;
    readonly unresolvedIssues?: readonly UnresolvedIssue[];
  },
): ChunkRepairOutcome {
  return {
    sliceIndex: 0,
    repairedText: REPAIRED_TEXT,
    changed: false,
    issues: [
      ...resolvedIssueIds.map(function toIssue(issueId,): UnresolvedIssue {
        return {
          issueId,
          status: 'accepted',
        };
      },),
      ...unresolvedIssues,
    ]
      .map(function toAdjudicated({ issueId, status, },) {
        return {
          issueId,
          status,
          severity: 'major' as const,
          claims: [],
          tallies: {},
        };
      },),
    resolvedIssueIds,
    candidateResolvedIssueIds: [],
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    authorship,
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    nonTranslationVotes: 0,
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 1,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 One checker casting a verdict unlike the rest of the bench.
 */
type DissentingChecker = {
  /**
   Checker that dissents.
   */
  readonly modelId: RosterModelId;

  /**
   Verdict it casts on every issue.
   */
  readonly verdict: string;
};

/**
 A claim of damage every prober makes against the rewrite, quoting one side.
 */
type ProbeClaim = {
  /**
   Wording quoted from the rewrite, for damage it added; empty otherwise.
   */
  readonly evidence: string;

  /**
   Wording quoted from the text it replaced, for content it dropped; empty
   otherwise.
   */
  readonly omittedText: string;
};

/**
 Client scripting the rewriter, the judges, and the recheck.

 @param checkerVerdict - verdict every checker casts during the recheck

 @param dissent - one checker casting another verdict, where a case needs a
 split bench

 @param probeClaim - claim every prober makes, none by default

 @returns Client usable by the phase

 @example
 ```ts
 const client = scriptedPhase({ checkerVerdict: 'fixed', },);
 ```
 */
function scriptedPhase(
  {
    checkerVerdict,
    dissent,
    probeClaim,
  }: {
    readonly checkerVerdict: string;
    readonly dissent?: DissentingChecker;
    readonly probeClaim?: ProbeClaim;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the phase',);
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

      /**
       User prompt, for counting the issues a checker sheet lists.
       */
      const asked = request.messages.at(-1,);
      const content = (asked === undefined) ? '' : messageText({ message: asked, },);

      /**
       Scripted reply for the stage.
       */
      const scripted: unknown = stage === 'refine_report'
        ? {
          rewrites: [
            {
              paragraph: 1,
              newText: SMOOTH_TEXT,
            },
          ],
        }
        : stage === 'candidate_ballot'
          ? {
            best: 1,
            assessment: 'scripted',
          }
        // The naturalness probe answers a DIFFERENT schema from the checker,
        // and the checker-shaped fallback fails its guard. That failure
        // is swallowed as a lost voice, so without this branch the probe would
        // report nothing heard and a case asserting it ran would pass for the
        // wrong reason.
          : stage === 'introduced_defect_report'
            ? {
              checks: [
                (probeClaim === undefined)
                  ? {
                    region: 1,
                    verdict: 'no-introduced-defect-found',
                    category: '',
                    severity: '',
                    evidence: '',
                    omittedText: '',
                    verdictReason: '',
                  }
                  : {
                    region: 1,
                    verdict: 'introduced-defect',
                    category: 'accuracy/mistranslation',
                    severity: 'major',
                    evidence: probeClaim.evidence,
                    omittedText: probeClaim.omittedText,
                    verdictReason: 'scripted',
                  },
              ],
            }
            : {
              checks: [...Array.from(
                { length: content.split('\nISSUE ',).length - 1, },
              ).keys(),]
                .map(function toCheck(index,) {
                  return {
                    issue: index + 1,
                    verdict: (request.modelId === dissent?.modelId) ? dissent.verdict : checkerVerdict,
                  };
                },),
            };
      if (!request.validate(scripted,))
        throw new Error(`stub script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the phase',);
    },
  };
}

/**
 Runs the phase over one settled outcome.

 @param resolvedIssueIds - issues the checkers confirmed in `T1`

 @param checkerVerdict - verdict the recheck receives

 @param models - roster override, defaulting to the lane-on roster

 @param authorship - who wrote `T1`, defaulting to nobody

 @param unresolvedIssues - issues the panel ruled on that `T1` does not
 resolve, none by default

 @param dissent - one checker casting another verdict, none by default

 @param probeClaim - claim every prober makes against the rewrite, none by
 default

 @returns Phase result

 @example
 ```ts
 const phase = await runPhase({ resolvedIssueIds: [], checkerVerdict: 'fixed', },);
 ```
 */
async function runPhase(
  {
    resolvedIssueIds,
    checkerVerdict,
    models = MODELS,
    authorship = NO_MODEL_WROTE_THE_FIXTURE,
    unresolvedIssues = [],
    dissent,
    probeClaim,
  }: {
    readonly resolvedIssueIds: readonly string[];
    readonly checkerVerdict: string;
    readonly models?: RepairModels;
    readonly authorship?: IssueAuthorship;
    readonly unresolvedIssues?: readonly UnresolvedIssue[];
    readonly dissent?: DissentingChecker;
    readonly probeClaim?: ProbeClaim;
  },
) {
  return runRefinePhase({
    declaredNames: [],
    client: scriptedPhase({
      checkerVerdict,
      ...((dissent === undefined) ? {} : { dissent, }),
      ...((probeClaim === undefined) ? {} : { probeClaim, }),
    },),
    targetText: REPAIRED_TEXT,
    slices: SLICES,
    outcomes: [settledOutcome({ resolvedIssueIds, authorship, unresolvedIssues, },),],
    models,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Counts every model call a client is asked to make.

 WHAT IT IS FOR: a resumed slice is only resumed if it bought NOTHING. Reading
 the returned text alone cannot tell a cache hit from a rewriter that happened
 to answer the same way twice, and the scripted client here answers the same
 way every time by construction, so the text would match either way.

 @param inner - client doing the actual scripted answering

 @param calls - counter this bumps on every structured call

 @returns Client forwarding to `inner` and counting

 @example
 ```ts
 const client = countingClient({ inner, calls, },);
 ```
 */
function countingClient(
  {
    inner,
    calls,
  }: {
    readonly inner: ModelClient;
    readonly calls: { count: number; };
  },
): ModelClient {
  return {
    chatText: inner.chatText,
    chatJson: async (request) => {
      calls.count += 1;
      return await inner.chatJson(request,);
    },
    quotas: inner.quotas,
  };
}

/**
 In-memory refinement cache behaving as the disk-backed one does.

 @param stored - map surviving between the two runs of a case

 @returns Cache resuming from `stored` and writing back into it

 @example
 ```ts
 const cache = memoryRefineCache({ stored, },);
 ```
 */
function memoryRefineCache(
  { stored, }: { readonly stored: Map<string, RefinedSliceSettlement>; },
): SliceCache<RefinedSliceSettlement> {
  return {
    resumed: stored,
    persist: async ({ key, serialized, },) => {
      stored.set(
        key,
        JSON.parse(serialized,) as RefinedSliceSettlement,
      );
    },
  };
}

/**
 Runs the phase once against a shared cache, counting what it bought.

 @param stored - cache contents carried between runs

 @returns Phase result beside the number of calls this run made

 @example
 ```ts
 const first = await runCachedPhase({ stored, },);
 ```
 */
async function runCachedPhase(
  { stored, }: { readonly stored: Map<string, RefinedSliceSettlement>; },
) {
  /**
   Calls this run made, which is what separates a resume from a rebuy.
   */
  const calls = { count: 0, };

  /**
   What the phase settled this run.
   */
  const phase = await runRefinePhase({
    declaredNames: [],
    client: countingClient({
      inner: scriptedPhase({ checkerVerdict: 'fixed', },),
      calls,
    },),
    targetText: REPAIRED_TEXT,
    slices: SLICES,
    outcomes: [settledOutcome({ resolvedIssueIds: [], authorship: NO_MODEL_WROTE_THE_FIXTURE, },),],
    models: MODELS,
    refineCache: memoryRefineCache({ stored, },),
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    phase,
    calls: calls.count,
  };
}

//region Checker bench at the refine stage

/**
 Checkers a hold that began inside the lane re-seats onto, disjoint from the
 chunk's own bench and from every writer, so a call reaching either bench is
 attributable.
 */
const RESEATED_CHECKERS: readonly RosterModelId[] = [
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_OPENROUTER_ONLY,
];

/**
 One call a checking stage made: which stage, and which model it asked.
 */
type CheckingCall = {
  /**
   Structured-output schema the call answered.
   */
  readonly stage: string;

  /**
   Model the call went to.
   */
  readonly modelId: RosterModelId;
};

/**
 Runs the phase over one slice whose rewrite ships and whose confirmed issue
 the recheck must re-prove, recording every checking call.

 @param reseat - reads the seating at the stage

 @returns Phase result and checking calls, in order

 @example
 ```ts
 const { calls, } = await runReseatedPhase({ reseat, },);
 ```
 */
async function runReseatedPhase(
  { reseat, }: { readonly reseat: () => Promise<RepairSliceSeating>; },
): Promise<{
  readonly phase: Awaited<ReturnType<typeof runRefinePhase>>;
  readonly calls: readonly CheckingCall[];
}> {
  /**
   Checking calls, in the order made.
   */
  const calls: CheckingCall[] = [];
  /**
   Client the phase would have used, answering every stage.
   */
  const inner = scriptedPhase({ checkerVerdict: 'fixed', },);
  const phase = await runRefinePhase({
    declaredNames: [],
    client: {
      chatText: inner.chatText,
      chatJson: async (request,) => {
        /**
         Stage name from the structured-output constraint.
         */
        const stage = request.responseFormat
          ?.json_schema
          .name
          ?? '';
        if ((stage !== 'refine_report') && (stage !== 'candidate_ballot'))
          calls.push({
            stage,
            modelId: request.modelId,
          },);
        return await inner.chatJson(request,);
      },
      quotas: inner.quotas,
    },
    targetText: REPAIRED_TEXT,
    slices: SLICES,
    outcomes: [settledOutcome({
      resolvedIssueIds: ['adjudicated/one',],
      authorship: NO_MODEL_WROTE_THE_FIXTURE,
    },),],
    models: MODELS,
    reseat,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    phase,
    calls,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runRefinePhase.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ships a refinement-only change and marks the slice changed, which '
            + 'is what puts it into the assembled document',
          fn: async () => {
            // No accepted issue at all: the lane's primary target, and the case a
            // lane placed at the bottom of repairChunk would never have reached.
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(phase.outcomes[0]?.changed,).toBe(true,);
          },
        },),

        it({
          name: 'rolls the whole slice back to T1 when the recheck finds a '
            + 'previously resolved issue no longer resolved',
          fn: async () => {
            /** Recheck where the checkers now say the issue is not fixed. */
            const phase = await runPhase({
              resolvedIssueIds: ['adjudicated/one',],
              checkerVerdict: 'not-fixed',
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(REPAIRED_TEXT,);
            expect(phase.outcomes[0]?.changed,).toBe(false,);
            expect(
              phase.findings
                .some(function namesRollback(finding,) {
                  return finding.includes('refine-rolled-back',)
                    && finding.includes('adjudicated/one',);
                },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'keeps a refinement whose recheck confirms the issue survived',
          fn: async () => {
            /** Recheck where the checkers confirm the issue is still fixed. */
            const phase = await runPhase({
              resolvedIssueIds: ['adjudicated/one',],
              checkerVerdict: 'fixed',
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(
              phase.findings
                .some(function namesRecheck(finding,) {
                  return finding.includes('refine-recheck-passed',);
                },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'KEEPS THE BALLOTS THAT CAUSED A ROLLBACK, which is the one round '
            + 'whose evidence decides what a reader sees: `refine-rolled-back` names '
            + 'the issue and nothing else said who called it a regression, so a '
            + 'slice that lost its rewrite could not be re-read at all',
          fn: async function aRollbackKeepsItsEvidence() {
            const phase = await runPhase({
              resolvedIssueIds: ['adjudicated/one',],
              checkerVerdict: 'not-fixed',
            },);

            /** The recheck round, as the outcome now carries it. */
            const reading = phase.outcomes[0]?.recheckReadings['adjudicated/one'];
            expect(reading?.configuredCheckers,).toBe(MODELS.checkerModelIds.length,);
            expect(reading?.ballots.length,).toBe(MODELS.checkerModelIds.length,);
            expect(
              reading?.ballots
                .every(function saidNotFixed(ballot,) {
                  return ballot.verdict === 'not-fixed';
                },),
            ).toBe(true,);
            expect(reading?.tally.resolved,).toBe(false,);

            // THE DECIDING ROUND IS UNTOUCHED. This fixture's accuracy stage bought
            // no checker round, so a recheck landing in the wrong field would be
            // visible here as a reading this outcome never earned.
            expect(Object.keys(phase.outcomes[0]?.checkerReadings ?? {},).length,).toBe(0,);
          },
        },),

        it({
          name: 'KEEPS the ballots of a recheck that CLEARED as well, so agreement '
            + 'and rollback leave the same kind of evidence and a reader cannot '
            + 'mistake an unrecorded round for a unanimous one',
          fn: async function apassingRecheckKeepsItsEvidence() {
            const phase = await runPhase({
              resolvedIssueIds: ['adjudicated/one',],
              checkerVerdict: 'fixed',
            },);
            const reading = phase.outcomes[0]?.recheckReadings['adjudicated/one'];
            expect(reading?.ballots.length,).toBe(MODELS.checkerModelIds.length,);
            expect(reading?.tally.resolved,).toBe(true,);

            // The refiner is disjoint from every checker in this roster, so no
            // ballot here is a self-vote and the tally runs at full weight.
            expect(
              reading?.ballots
                .some(function judgedItsOwnWork(ballot,) {
                  return ballot.wroteTheText;
                },),
            ).toBe(false,);
          },
        },),

        it({
          name: 'BUYS NO RECHECK on a slice with no confirmed issue, which is the '
            + 'common case, so an empty reading means the round never ran rather '
            + 'than a round that said nothing',
          fn: async function nothingProvedBuysNoRound() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
            },);

            // The rewrite still shipped; it simply had nothing to re-prove.
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(Object.keys(phase.outcomes[0]?.recheckReadings ?? {},).length,).toBe(0,);
          },
        },),

        it({
          name: 'KEEPS A REWRITE OF THE ARCHIVE the checkers find one worse ballot on: one worse ballot is '
            + 'one voice, and one voice rolls nothing back (owner, 2026-10-25, "Two ballots for undo too")',
          fn: async function aLoneWorseBallotRollsNothingBack() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              unresolvedIssues: [{ issueId: 'adjudicated/open', status: 'accepted', },],
              checkerVerdict: 'not-fixed',
              dissent: { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, verdict: 'worse', },
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(phase.findings,).toContain('refine-recheck-passed (1 issue)',);
          },
        },),

        it({
          name: 'RECHECKS A REWRITE OF THE ARCHIVE AFTER A LOST PATCH against the '
            + 'slice\'s accepted issues, and a rewrite two checkers vote worse keeps '
            + 'the text before it (ledger L11, owner 2026-09-28, two ballots since 2026-10-25): '
            + '1,218 of 2,144 refined slices were such rewrites, 457 over accepted issues the '
            + 'rewrite was never shown, and none had a checker round',
          fn: async function aWorseRewriteOfTheArchiveRollsBack() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              unresolvedIssues: [{ issueId: 'adjudicated/open', status: 'accepted', },],
              checkerVerdict: 'worse',
              dissent: { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, verdict: 'not-fixed', },
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(REPAIRED_TEXT,);
            expect(phase.outcomes[0]?.changed,).toBe(false,);
            expect(
              phase.findings
                .some(function namesRollback(finding,) {
                  return finding.includes('refine-rolled-back',)
                    && finding.includes('adjudicated/open',);
                },),
            ).toBe(true,);

            // The round that decided it is kept, as for a regressed confirmed issue.
            const reading = phase.outcomes[0]?.recheckReadings['adjudicated/open'];
            expect(reading?.ballots.length,).toBe(MODELS.checkerModelIds.length,);
          },
        },),

        it({
          name: 'SHIPS a rewrite of the archive the checkers find no worse, since a '
            + 'not-fixed verdict on an issue the patch never fixed is the text as it '
            + 'stood, and keeps that round\'s ballots',
          fn: async function aNoWorseRewriteOfTheArchiveShips() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              unresolvedIssues: [{ issueId: 'adjudicated/open', status: 'accepted', },],
              checkerVerdict: 'not-fixed',
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(phase.findings,).toContain('refine-recheck-passed (1 issue)',);
            const reading = phase.outcomes[0]?.recheckReadings['adjudicated/open'];
            expect(reading?.ballots.length,).toBe(MODELS.checkerModelIds.length,);
          },
        },),

        it({
          name: 'CREDITS NOTHING to a rewrite the checkers call fixed on an issue the '
            + 'patch never fixed: the recheck is a rollback gate, and a resolution it '
            + 'recorded would rest on a round no panel or selection ever weighed',
          fn: async function aFixedVoteOnTheRewriteIsNotAResolution() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              unresolvedIssues: [{ issueId: 'adjudicated/open', status: 'accepted', },],
              checkerVerdict: 'fixed',
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(phase.outcomes[0]?.resolvedIssueIds,).toStrictEqual([],);
          },
        },),

        it({
          name: 'BUYS NO RECHECK over issues the panel rejected or left to a human, '
            + 'which describe no defect the slice is known to carry',
          fn: async function unacceptedIssuesBuyNoRound() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              unresolvedIssues: [
                { issueId: 'adjudicated/rejected', status: 'rejected', },
                { issueId: 'adjudicated/human', status: 'needs-human', },
              ],
              checkerVerdict: 'worse',
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(Object.keys(phase.outcomes[0]?.recheckReadings ?? {},).length,).toBe(0,);
          },
        },),

        it({
          name: 'ROLLS BACK a rewrite the damage probe admits a claim against (ledger L11, decided for quality): '
            + '175 of 2,144 kept rewrites carried one, and a reading of every such region found six of ten true, '
            + 'so keeping them shipped more damage than rolling back loses fluency',
          fn: async function anAdmittedAddedDamageClaimRollsBack() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
              probeClaim: { evidence: 'sunbathes on the windowsill', omittedText: '', },
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(REPAIRED_TEXT,);
            expect(phase.outcomes[0]?.changed,).toBe(false,);
            expect(phase.outcomes[0]?.refined,).toBe(false,);
            expect(
              phase.findings
                .some(function namesProbeRollback(finding,) {
                  return finding.startsWith('refine-rolled-back-by-probe',);
                },),
            ).toBe(true,);

            // NO REPORT AGAINST TEXT THAT NO LONGER SHIPS: the lane contest reads
            // this field as damage evidence against the repair candidate, whose text
            // is T1 again.
            expect(phase.outcomes[0]?.refinementDefects,).toBeUndefined();
          },
        },),

        it({
          name: 'ROLLS BACK a rewrite the probe admits a removal claim against, the other shape an admitted claim '
            + 'takes: content the text before it carried and the rewrite dropped',
          fn: async function anAdmittedRemovalClaimRollsBack() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
              probeClaim: { evidence: '', omittedText: 'without any hurry at all', },
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(REPAIRED_TEXT,);
            expect(phase.outcomes[0]?.refinementDefects,).toBeUndefined();
          },
        },),

        it({
          name: 'STORES BOTH STAGES ON A RECORD WHOSE REWRITE SHIPPED, so what the '
            + 'record says about its own text stays true. The recheck already unions '
            + 'the two for its own weighting, but that union lives in an argument and '
            + 'dies with the call: left unstored, the record credits the editor with '
            + 'words a refiner replaced, and a later reader would let that refiner '
            + 'certify its own rewrite at full weight',
          fn: async function bothStagesRideTheStoredRecord() {
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
              authorship: {
                perIssue: {},
                everyIssue: [EDITOR_WHO_DID_NOT_REFINE,],
              },
            },);

            // The rewrite shipped, so both stages wrote what this record carries.
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(phase.outcomes[0]?.authorship.everyIssue.toSorted(),).toStrictEqual(
              [
                EDITOR_WHO_DID_NOT_REFINE,
                REFINER_THAT_REWROTE,
              ].toSorted(),
            );
          },
        },),

        it({
          name: 'NAMES NO REFINER ON A SLICE IT ROLLED BACK, which is the control '
            + 'proving the "STORES BOTH STAGES ON A RECORD WHOSE REWRITE SHIPPED" union is not stored '
            + 'unconditionally. The rewrite '
            + 'those refiners produced is exactly the text the rollback threw away, '
            + 'and naming them would discount a checker over words no reader saw',
          fn: async function aRolledBackRewriteAddsNobody() {
            const phase = await runPhase({
              resolvedIssueIds: ['adjudicated/one',],
              checkerVerdict: 'not-fixed',
              authorship: {
                perIssue: {},
                everyIssue: [EDITOR_WHO_DID_NOT_REFINE,],
              },
            },);

            // T1 came back, so its editor alone answers for this record.
            expect(phase.outcomes[0]?.repairedText,).toBe(REPAIRED_TEXT,);
            expect(phase.outcomes[0]?.authorship.everyIssue,).toStrictEqual(
              [EDITOR_WHO_DID_NOT_REFINE,],
            );
          },
        },),

        it({
          name: 'AUDITS the rewrite it accepted, attaching a refinement probe '
            + 'report to the refined outcome. retainsResolvedIssues only proves a '
            + 'rewrite did not UNDO a confirmed repair; a rewrite can leave every '
            + 'confirmed repair standing and still damage the wording around it, '
            + 'and before this the lane was the one stage that could change shipped '
            + 'text with nothing asking',
          fn: async () => {
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
            },);

            expect(phase.outcomes[0]?.refined,).toBe(true,);
            /**
             Report the lane attached for its own rewrite.
             */
            const report = phase.outcomes[0]
              ?.refinementDefects;
            expect(report,).toBeDefined();
            // Heard rather than lost: a report of zero heard probers is what a
            // silently broken wiring also produces, so the count is the assertion.
            expect(report
              ?.heardProbers,).toBeGreaterThan(0,);
            expect(report
              ?.regions
              .length,).toBe(1,);
            expect(report
              ?.regions[0]
              ?.envelopeId,).toBe('refinement/0',);
          },
        },),

        it({
          name: 'attaches NO refinement report when the lane changed nothing, so '
            + 'an absent report means no rewrite happened rather than a rewrite '
            + 'nobody checked',
          fn: async () => {
            /** Roster with the lane off. */
            const laneOff: RepairModels = {
              ...MODELS,
              refinerModelIds: [],
            };
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
              models: laneOff,
            },);

            expect(phase.outcomes[0]?.refined,).toBe(false,);
            expect(phase.outcomes[0]?.refinementDefects,).toBeUndefined();
          },
        },),

        it({
          name: 'is off entirely when no refiner roster is configured, spending no '
            + 'calls and returning the outcomes untouched',
          fn: async () => {
            /** Roster with the lane off. */
            const laneOff: RepairModels = {
              ...MODELS,
              refinerModelIds: [],
            };
            const phase = await runPhase({
              resolvedIssueIds: [],
              checkerVerdict: 'fixed',
              models: laneOff,
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(REPAIRED_TEXT,);
            expect(phase.outcomes[0]?.changed,).toBe(false,);
            expect(phase.findings.length,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES invalid overlap while the refiner lane is off, so disabling '
            + 'work cannot make invalid caller configuration valid',
          fn: async () => {
            /**
             Calls made before refusal, which must stay at zero.
             */
            const calls = { count: 0, };

            /**
             Roster turning naturalness lane off.
             */
            const laneOff: RepairModels = {
              ...MODELS,
              refinerModelIds: [],
            };
            await expect(runRefinePhase({
              declaredNames: [],
              client: countingClient({
                inner: scriptedPhase({ checkerVerdict: 'fixed', },),
                calls,
              },),
              targetText: REPAIRED_TEXT,
              slices: SLICES,
              outcomes: [settledOutcome({
                resolvedIssueIds: [],
                authorship: NO_MODEL_WROTE_THE_FIXTURE,
              },),],
              models: laneOff,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              overlap: 0,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),)
              .rejects
              .toThrow(OverlapRefusedError,);
            expect(calls.count,).toBe(0,);
          },
        },),

        it({
          name: 'runs two refinement slices at once when overlap is 2, after a serial '
            + 'positive control proves the refiner instrument distinguishes one from two, '
            + 'and still returns outcomes in input order when the second refiner answers first',
          fn: async () => {
            /**
             Two prepared slices carrying independently refinable paragraphs.
             */
            const twoSlices: readonly ChunkPair[] = [
              ...SLICES,
              {
                source: {
                  sliceIndex: 1,
                  text: SOURCE_TEXT,
                  startOffset: SOURCE_TEXT.length + 2,
                  endOffset: (SOURCE_TEXT.length * 2) + 2,
                  nodes: [],
                },
                target: {
                  sliceIndex: 1,
                  text: REPAIRED_TEXT,
                  startOffset: REPAIRED_TEXT.length + 2,
                  endOffset: (REPAIRED_TEXT.length * 2) + 2,
                  nodes: [],
                },
              },
            ];

            /**
             Accuracy outcomes corresponding to prepared input order.
             */
            const firstOutcome = settledOutcome({
              resolvedIssueIds: [],
              authorship: NO_MODEL_WROTE_THE_FIXTURE,
            },);
            const outcomes: readonly ChunkRepairOutcome[] = [
              firstOutcome,
              {
                ...firstOutcome,
                sliceIndex: 1,
              },
            ];

            /**
             Serial positive-control activity.
             */
            const serial: RefinerConcurrency = {
              now: 0,
              peak: 0,
              started: 0,
              finished: [],
            };
            await runRefinePhase({
              declaredNames: [],
              client: measuringRefiners({
                inner: scriptedPhase({ checkerVerdict: 'fixed', },),
                activity: serial,
              },),
              targetText: `${REPAIRED_TEXT}\n\n${REPAIRED_TEXT}`,
              slices: twoSlices,
              outcomes,
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              overlap: 1,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             Overlapped activity, gated so the second refiner answers first.
             */
            const overlapped: RefinerConcurrency = {
              now: 0,
              peak: 0,
              started: 0,
              finished: [],
              secondFinished: Promise.withResolvers<undefined>(),
            };
            const phase = await runRefinePhase({
              declaredNames: [],
              client: measuringRefiners({
                inner: scriptedPhase({ checkerVerdict: 'fixed', },),
                activity: overlapped,
              },),
              targetText: `${REPAIRED_TEXT}\n\n${REPAIRED_TEXT}`,
              slices: twoSlices,
              outcomes,
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(serial.peak,).toBe(1,);
            expect(overlapped.peak,).toBe(2,);
            // THE SECOND REFINER DID ANSWER FIRST: the call started first is not
            // the first to finish, so the check that outcomes come back in input
            // order is exercised.
            expect(overlapped.finished,).toContain(0,);
            expect(overlapped.finished[0],).not.toBe(0,);
            expect(phase.outcomes.map(function toIndex(outcome,) {
              return outcome.sliceIndex;
            },),).toEqual([
              0,
              1,
            ],);
          },
        },),

        it({
          name: 'reports rewriters asked when ANY overlapped slice asked, even when '
            + 'the last slice in document order was ineligible and asked nobody',
          fn: async () => {
            /**
             Eligible first outcome followed by one too short to refine.

             SHORT RATHER THAN STANDING AS NON-TRANSLATION, which this case used
             until ledger L15: standing votes no longer keep a slice from the
             rewriters, so they no longer make one ineligible.
             */
            const eligible = settledOutcome({
              resolvedIssueIds: [],
              authorship: NO_MODEL_WROTE_THE_FIXTURE,
            },);
            const ineligible: ChunkRepairOutcome = {
              ...eligible,
              sliceIndex: 1,
              repairedText: SHORT_TEXT,
            };

            /**
             Second prepared pair matching ineligible outcome index.
             */
            const second: ChunkPair = {
              source: {
                sliceIndex: 1,
                text: SOURCE_TEXT,
                startOffset: SOURCE_TEXT.length + 2,
                endOffset: (SOURCE_TEXT.length * 2) + 2,
                nodes: [],
              },
              target: {
                sliceIndex: 1,
                text: SHORT_TEXT,
                startOffset: REPAIRED_TEXT.length + 2,
                endOffset: REPAIRED_TEXT.length + 2 + SHORT_TEXT.length,
                nodes: [],
              },
            };
            const phase = await runRefinePhase({
              declaredNames: [],
              client: scriptedPhase({ checkerVerdict: 'fixed', },),
              targetText: `${REPAIRED_TEXT}\n\n${SHORT_TEXT}`,
              slices: [
                ...SLICES,
                second,
              ],
              outcomes: [
                eligible,
                ineligible,
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(phase.askedRewriters,).toBe(true,);
          },
        },),

        it({
          name: 'reports a refinement that lands back on the ARCHIVE wording as '
            + 'unchanged, since the rewriter is measured against the accuracy text '
            + 'and can move off it right back onto the words the archive already '
            + 'had. Stamped changed, that slice enters the shipped set carrying the '
            + 'archive wording, which assembly refuses, so a run the models got '
            + 'right would fail the whole document',
          fn: async () => {
            /**
             Slices whose archive wording is the SMOOTH text, so the accuracy
             stage moved off it and the refinement lands back on it.
             */
            const archiveSlices: readonly ChunkPair[] = [
              {
                source: {
                  sliceIndex: 0,
                  text: SOURCE_TEXT,
                  startOffset: 0,
                  endOffset: SOURCE_TEXT.length,
                  nodes: [],
                },
                target: {
                  sliceIndex: 0,
                  text: SMOOTH_TEXT,
                  startOffset: 0,
                  endOffset: SMOOTH_TEXT.length,
                  nodes: [],
                },
              },
            ];

            /**
             Accuracy outcome that changed the archive wording and had an issue
             confirmed resolved in the text it produced.
             */
            const accuracy: ChunkRepairOutcome = {
              ...settledOutcome({ resolvedIssueIds: ['issue-1',], authorship: NO_MODEL_WROTE_THE_FIXTURE, },),
              changed: true,
            };

            /**
             Phase over that outcome, whose rewriter returns the archive wording.
             */
            const phase = await runRefinePhase({
              declaredNames: [],
              client: scriptedPhase({ checkerVerdict: 'fixed', },),
              targetText: SMOOTH_TEXT,
              slices: archiveSlices,
              outcomes: [accuracy,],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(phase.outcomes[0]?.changed,).toBe(false,);
            // Nothing this slice returns differs from the archive, so nothing it
            // returns can have resolved anything: crediting the issue here would
            // count a repair no reader saw.
            expect(phase.outcomes[0]?.resolvedIssueIds,).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES an outcome naming a slice the preparation never produced before buying any call, '
            + 'where it used to refine against an empty original and be refused by the step afterwards',
          fn: async () => {
            /**
             Calls made, which must stay at zero.
             */
            const calls = { count: 0, };

            await expect(runRefinePhase({
              declaredNames: [],
              client: countingClient({
                inner: scriptedPhase({ checkerVerdict: 'fixed', },),
                calls,
              },),
              targetText: REPAIRED_TEXT,
              slices: SLICES,
              outcomes: [
                {
                  ...settledOutcome({ resolvedIssueIds: [], authorship: NO_MODEL_WROTE_THE_FIXTURE, },),
                  sliceIndex: SLICES.length + 2,
                },
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),).rejects.toThrow(UnpreparedSliceError,);
            expect(calls.count,).toBe(0,);
          },
        },),

        it({
          name: 'THROWS on an abort that lands during a slice and persists nothing for it, the check the '
            + 'accuracy pass makes before its own write',
          fn: async () => {
            /**
             Cache that must stay empty.
             */
            const stored = new Map<string, RefinedSliceSettlement>();

            /**
             Abort raised from inside the first call, after which the scripted
             client still answers, so the stage settles and the guard before the
             write is what has to refuse.
             */
            const controller = new AbortController();

            /**
             Scripted client whose first exchange aborts the run.
             */
            const inner = scriptedPhase({ checkerVerdict: 'fixed', },);

            await expect(runRefinePhase({
              declaredNames: [],
              client: {
                chatText: inner.chatText,
                chatJson: async (request) => {
                  controller.abort(new Error('the operator stopped the run',),);
                  return await inner.chatJson(request,);
                },
                quotas: inner.quotas,
              },
              targetText: REPAIRED_TEXT,
              slices: SLICES,
              outcomes: [settledOutcome({ resolvedIssueIds: [], authorship: NO_MODEL_WROTE_THE_FIXTURE, },),],
              models: MODELS,
              refineCache: memoryRefineCache({ stored, },),
              signal: controller.signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),).rejects.toThrow(Error,);
            expect(stored.size,).toBe(0,);
          },
        },),


        it({
          name: 'WRITES A LATER SLICE\'S OWN FAILURE UNDER THE CALLER\'S STOP, an outcome naming no prepared slice '
            + 'refused before the stop landed while the first slice\'s rewriters ended in the caller\'s reason, and '
            + 'refuses with that reason',
          fn: async () => {
            /**
             Every line the phase wrote.
             */
            const messages: string[] = [];
            /**
             The caller's steering.
             */
            const steering = new AbortController();
            /**
             Why the caller stopped the refinement.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Slice the later outcome names, which no preparation produced.
             */
            const unprepared = SLICES.length + 2;
            /**
             The phase, started and left waiting on the first slice's rewriters.
             */
            const phase = rejectionOf(async function refinesUnderTheStop(): Promise<unknown> {
              return await runRefinePhase({
                declaredNames: [],
                client: {
                  chatText: async () => {
                    throw new Error('chatText unused',);
                  },
                  chatJson: async (request) => {
                    await untilAborted({ signal: request.signal, },);
                    request.signal
                      .throwIfAborted();
                    throw new Error('unreachable: a rewriter that waited for its abort was handed a signal that had not aborted',);
                  },
                  quotas: async () => {
                    throw new Error('quotas unused',);
                  },
                },
                targetText: REPAIRED_TEXT,
                slices: SLICES,
                outcomes: [
                  settledOutcome({ resolvedIssueIds: [], authorship: NO_MODEL_WROTE_THE_FIXTURE, },),
                  {
                    ...settledOutcome({ resolvedIssueIds: [], authorship: NO_MODEL_WROTE_THE_FIXTURE, },),
                    sliceIndex: unprepared,
                  },
                ],
                models: MODELS,
                signal: steering.signal,
                perCallTimeoutMs: HANG_STOP_MS,
                overlap: 2,
                l: capturingLogger({ messages, },),
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);
            await turnOfTheLoop();
            steering.abort(reason,);
            expect(await phase,).toBe(reason,);
            /**
             What the refusal of the later outcome says of itself.
             */
            const { message: unpreparedText, } = new UnpreparedSliceError({ sliceIndex: unprepared, },);
            expect(messages,).toEqual([
              `[mapOverlapped] slice ${String(unprepared,)} failed (${unpreparedText}), and the caller stopped the work`,
            ],);
          },
        },),

        it({
          name: 'REFUSES persistence after caller abort even when model work returned a '
            + 'complete settlement, preserving the final defense against future stages that settle silence',
          fn: async () => {
            /**
             Exact caller abort reason whose identity must surface.
             */
            const stopped = new Error('caller stopped refinement',);
            const controller = new AbortController();
            controller.abort(stopped,);

            /**
             Cache that aborted settlement must not reach.
             */
            const stored = new Map<string, RefinedSliceSettlement>();
            await expect(persistRefinePhaseSlice({
              key: 'refine-persistence-guard-fixture',
              settled: {
                outcome: settledOutcome({
                  resolvedIssueIds: [],
                  authorship: NO_MODEL_WROTE_THE_FIXTURE,
                },),
                findings: [],
              },
              sliceIndex: 0,
              refineCache: memoryRefineCache({ stored, },),
              signal: controller.signal,
              l,
            },),)
              .rejects
              .toBe(stopped,);
            expect(stored.size,).toBe(0,);
          },
        },),

        it({
          name: 'DOES NOT CACHE a refinement whose settled outcome names a silent panel as data, because the reader '
            + 'refuses that record and a write it would discard must not count as progress',
          fn: async () => {
            /**
             Cache writes made by this run.
             */
            const stored = new Map<string, RefinedSliceSettlement>();

            /**
             Lines the phase logged.
             */
            const messages: string[] = [];
            await persistRefinePhaseSlice({
              key: 'refine-persistence-silent-panel-fixture',
              settled: {
                outcome: {
                  ...settledOutcome({
                    resolvedIssueIds: [],
                    authorship: NO_MODEL_WROTE_THE_FIXTURE,
                  },),
                  silentStages: ['panel 0/6 (cluster/abc)',],
                },
                findings: [],
              },
              sliceIndex: 85,
              refineCache: memoryRefineCache({ stored, },),
              signal: new AbortController().signal,
              l: capturingLogger({ messages, },),
            },);
            expect(stored.size,).toBe(0,);
            expect(messages.join('\n',),).toContain('panel 0/6 (cluster/abc)',);
          },
        },),

        it({
          name: 'DOES NOT CACHE a refinement whose rewriter stage heard nobody, so a '
            + 'provider outage is reconsidered rather than resumed as a decision',
          fn: async () => {
            /**
             Cache writes made by this run.
             */
            const stored = new Map<string, RefinedSliceSettlement>();

            /**
             Refiner calls proving eligible work was attempted.
             */
            const calls = { count: 0, };

            /**
             Scripted client with only rewriters unavailable.
             */
            const inner = scriptedPhase({ checkerVerdict: 'fixed', },);
            const phase = await runRefinePhase({
              declaredNames: [],
              client: {
                chatText: inner.chatText,
                chatJson: async (request) => {
                  if (request.responseFormat
                    ?.json_schema
                    .name
                    === 'refine_report') {
                    calls.count += 1;
                    throw new Error('refiner provider is unavailable',);
                  }
                  return await inner.chatJson(request,);
                },
                quotas: inner.quotas,
              },
              targetText: REPAIRED_TEXT,
              slices: SLICES,
              outcomes: [settledOutcome({
                resolvedIssueIds: [],
                authorship: NO_MODEL_WROTE_THE_FIXTURE,
              },),],
              models: MODELS,
              refineCache: memoryRefineCache({ stored, },),
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(calls.count,).toBeGreaterThan(0,);
            expect(phase.askedRewriters,).toBe(true,);
            expect(stored.size,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${runRefinePhase.name} resume`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPUBLISHES THE SAME TEXT WITHOUT BUYING ANYTHING on a second run '
            + 'over one cache, which is the whole defect: the accuracy pass persists '
            + 'before this phase runs, so a resumed entry replayed accuracy from disk '
            + 'and then rebought the rewrite, publishing different text at 7 of 18 '
            + 'repair-lane slices across two runs on identical inputs',
          fn: async () => {
            /**
             Cache both runs share, as one entry directory would be.
             */
            const stored = new Map<string, RefinedSliceSettlement>();

            const first = await runCachedPhase({ stored, },);
            const second = await runCachedPhase({ stored, },);

            // The positive control: the first run must have bought something, or a
            // second run buying nothing would prove only that the lane never ran.
            expect(first.calls,).toBeGreaterThan(0,);
            expect(second.calls,).toBe(0,);

            expect(first.phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(second.phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(second.phase.outcomes[0]?.refined,).toBe(true,);
            expect(second.phase.outcomes[0]?.changed,).toBe(true,);
          },
        },),

        it({
          name: 'REPORTS NO REWRITER ASKED on the resumed run, because that answer '
            + 'decides whether a run overtaken by an abort may call itself finished. '
            + 'A resumed slice asked nobody anything, and carrying the stored answer '
            + 'would report a previous run\'s purchase as this one\'s',
          fn: async () => {
            const stored = new Map<string, RefinedSliceSettlement>();

            const first = await runCachedPhase({ stored, },);
            const second = await runCachedPhase({ stored, },);

            expect(first.phase.askedRewriters,).toBe(true,);
            expect(second.phase.askedRewriters,).toBe(false,);
          },
        },),

        it({
          name: 'CARRIES THE FINDINGS BACK with the resumed slice, so a scorecard '
            + 'reads the same telemetry whether the entry was bought or resumed',
          fn: async () => {
            const stored = new Map<string, RefinedSliceSettlement>();

            const first = await runCachedPhase({ stored, },);
            const second = await runCachedPhase({ stored, },);

            expect(second.phase.findings,).toStrictEqual(first.phase.findings,);
          },
        },),
      ],
    },),

    describe({
      name: `${runRefinePhase.name} checker bench at the stage`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS THE RE-SEATED BENCH for the recheck and the rewrite probe, never the bench the chunk '
            + 'was seated with (class one hundred thirteen, mikaela16: ten rechecks after Synthetic dried '
            + 'out heard 1 of 3 on the chunk bench while the proof stage, re-seated, heard 2 of 3)',
          fn: async () => {
            const { phase, calls, } = await runReseatedPhase({
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: {
                  ...MODELS,
                  checkerModelIds: RESEATED_CHECKERS,
                },
              }),
            },);
            // The rewrite shipped, so both checking stages ran; a case where
            // neither ran would pass the `calls.some` membership checks vacuously.
            expect(phase.outcomes[0]?.repairedText,).toBe(SMOOTH_TEXT,);
            expect(calls.some(function isProbe(call,): boolean {
              return call.stage === 'introduced_defect_report';
            },),).toBe(true,);
            expect(calls.some(function isRecheck(call,): boolean {
              return call.stage !== 'introduced_defect_report';
            },),).toBe(true,);
            expect(calls.every(function onReseatedBench(call,): boolean {
              return RESEATED_CHECKERS.includes(call.modelId,);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS THE CHUNK BENCH when the reading seats nothing new, the control showing the "ASKS THE '
            + 'RE-SEATED BENCH" case reports the hook and not a fixture that never asks the chunk bench',
          fn: async () => {
            const { calls, } = await runReseatedPhase({
              reseat: async (): Promise<RepairSliceSeating> => ({}),
            },);
            expect(calls.length,).toBeGreaterThan(0,);
            expect(calls.every(function onChunkBench(call,): boolean {
              return MODELS.checkerModelIds.includes(call.modelId,);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a re-seated bench below the checker floor at the recheck rather than running a '
            + 'stage the contract refuses',
          fn: async () => {
            await expect(runReseatedPhase({
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: {
                  ...MODELS,
                  checkerModelIds: [SEAT_OPENROUTER_ONLY,],
                },
              }),
            },),).rejects
              .toThrow(CheckerQuorumError,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Checker bench at the refine stage
