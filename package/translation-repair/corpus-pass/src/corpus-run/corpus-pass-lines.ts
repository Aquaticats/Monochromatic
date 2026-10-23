import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  capOutlastsOneCall,
  capTooTightNote,
  corpusPinOverrideNote,
  type CorpusPinSetting,
  HARD_CAP_VAR,
  type ProviderOff,
  spendCeilingOverrideNote,
} from '@monochromatic-dev/translation-repair-run/ts';
import { PASS_MS_PER_MINUTE, } from './corpus-pass-limits.ts';
import type { IncompleteEntry, } from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Corpus pass lines
// The lines a corpus pass prints, each built from what the procedure already
// knows and returned rather than written, so a case reads the whole text of
// each without a run. Split out of the procedure that prints them.

/**
 Lines saying which entries a pass was restricted to.

 WHAT `--only` BYPASSES IS THE CHOICE, NOT THE ORDER. Unrestricted, the pass
 runs every pending pair at the pin; restricted, it runs the entries the
 operator named (`corpus-pass-select.ts`), and those still pending run in the
 pass's own order, cached progress first and then the size bands by rank
 (`corpus-pass-order.ts`). Until 2026-10-10 the line said the ordering was
 bypassed.

 A NAMED ENTRY FINISHED BEFORE DOES NOT RUN, so the line says the named
 entries run only if still pending; it is printed before the selection reads
 which of them are, and until 2026-10-10 it said every named entry runs.

 @param onlyIds - entry ids the command line named, empty when it named none

 @returns The one line, or none when the pass was not restricted

 @example
 ```ts
 const lines = passOnlyLines({ onlyIds: new Set(['tabby',],), },);
 ```
 */
export function passOnlyLines({ onlyIds, }: { readonly onlyIds: ReadonlySet<string>; },): readonly string[] {
  if (onlyIds.size === 0)
    return [];

  /**
   Chosen ids in a stable order, so two runs of one selection log alike.
   */
  const chosen = textsInCodePointOrder({ texts: [...onlyIds,], },)
    .join(',',);

  return [
    [
      `ONLY ${chosen} (chosen by hand in place of every pending pair at the pin; `,
      wordForCount({
        count: onlyIds.size,
        one: 'if still pending it runs',
        many: 'those still pending run',
      },),
      ' in the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a ',
      'hand-picked entry never enters a pool later draws treat as natural accumulation)',
    ].join('',),
  ];
}

/**
 Line saying that one entry has a page absent at the pin.

 @param gap - entry, the side that was absent and what the read said

 @returns The line

 @example
 ```ts
 const line = passIncompleteLine({ gap: { id: 'mittens', side: 'target', detail: 'missing-object', }, },);
 ```
 */
export function passIncompleteLine({ gap, }: { readonly gap: IncompleteEntry; },): string {
  return `INCOMPLETE ${gap.id}: ${gap.side} page absent at the pin (${gap.detail})`;
}

/**
 What the START line reports.

 @example
 ```ts
 const facts: PassStartFacts = {
   tip,
   pipelineDigest,
   fileCount: 189,
   pending: 2,
   done: 0,
   softBudgetMs: 259_200_000,
   hardCapMs: 25_200_000,
   perCallMs: 360_000,
   claudeCallMs: 460_000,
 };
 ```
 */
export type PassStartFacts = {
  /**
   Pipeline tip recorded into every artifact.
   */
  readonly tip: string;

  /**
   Identity of the built pipeline this invocation runs.
   */
  readonly pipelineDigest: string;

  /**
   Files the digest covers.
   */
  readonly fileCount: number;

  /**
   Entries the pass would run.
   */
  readonly pending: number;

  /**
   Entries already finished before this pass.
   */
  readonly done: number;

  /**
   Time after which no new entry starts.
   */
  readonly softBudgetMs: number;

  /**
   Ceiling one entry runs under.
   */
  readonly hardCapMs: number;

  /**
   Deadline one model exchange is allowed.
   */
  readonly perCallMs: number;

  /**
   Deadline a Claude exchange is allowed, which the Claude client arms in place of a shorter one.
   */
  readonly claudeCallMs: number;
};

/**
 The line that opens a pass's report.

 @param facts - what the line reports

 @returns The line

 @example
 ```ts
 const line = passStartLine({ facts, },);
 ```
 */
export function passStartLine({ facts, }: { readonly facts: PassStartFacts; },): string {
  return `START tip=${facts.tip} pipeline=${facts.pipelineDigest} files=${String(facts.fileCount,)} pending=${
    String(facts.pending,)
  } done=${String(facts.done,)} soft=${String(facts.softBudgetMs,)}ms hard=${String(facts.hardCapMs,)}ms call=${
    String(facts.perCallMs,)
  }ms claude-call=${String(facts.claudeCallMs,)}ms`;
}

/**
 Lines naming every limit a launch changed or set too tight, in the order a
 pass prints them: a run must never hide which ceiling, window or corpus it
 ran under, since an artifact settled under one is not comparable with one
 settled under another.

 @param hardCapMs - ceiling one entry runs under, after any override

 @param builtInCapMinutes - ceiling the build ships, in minutes

 @param perCallMs - deadline one model exchange is allowed

 @param claudeCallMs - deadline a Claude exchange is allowed, which the ceiling must outlast too

 @param spendCeilingUsd - spend allowance, after any override

 @param graceNote - note naming the straggler window, empty for the built-in one

 @param writerNote - note naming the writer rounds' window, empty for the built-in one

 @param pinSetting - corpus pin beside where each half came from

 @returns The lines to print, none empty

 @example
 ```ts
 const lines = passLaunchLines({
   hardCapMs,
   builtInCapMinutes: 420,
   perCallMs,
   claudeCallMs,
   spendCeilingUsd,
   graceNote,
   writerNote,
   pinSetting,
 },);
 ```
 */
export function passLaunchLines(
  {
    hardCapMs,
    builtInCapMinutes,
    perCallMs,
    claudeCallMs,
    spendCeilingUsd,
    graceNote,
    writerNote,
    pinSetting,
  }: {
    readonly hardCapMs: number;
    readonly builtInCapMinutes: number;
    readonly perCallMs: number;
    readonly claudeCallMs: number;
    readonly spendCeilingUsd: number;
    readonly graceNote: string;
    readonly writerNote: string;
    readonly pinSetting: CorpusPinSetting;
  },
): readonly string[] {
  /**
   Ceiling in minutes, as a launch wrote it.
   */
  const capMinutes = hardCapMs / PASS_MS_PER_MINUTE;

  /**
   Cap line, empty for the built-in ceiling.
   */
  const capLine = (hardCapMs === (builtInCapMinutes * PASS_MS_PER_MINUTE))
    ? ''
    : `CAP OVERRIDDEN by ${HARD_CAP_VAR}: entries run under ${String(capMinutes,)} ${
      wordForCount({
        count: capMinutes,
        one: 'minute',
        many: 'minutes',
      },)
    } rather than the built-in ${String(builtInCapMinutes,)}`;

  /**
   Line saying the ceiling cannot outlast one exchange, empty when it can.
   */
  const tightLine = capOutlastsOneCall({
    capMs: hardCapMs,
    perCallMs,
    claudeCallMs,
  },)
    ? ''
    : capTooTightNote({
      capMs: hardCapMs,
      perCallMs,
      claudeCallMs,
    },);

  return [
    capLine,
    spendCeilingOverrideNote({ ceilingUsd: spendCeilingUsd, },),
    graceNote,
    writerNote,
    corpusPinOverrideNote({ setting: pinSetting, },),
    tightLine,
  ].filter(function isSaid(line,): boolean {
    return line !== '';
  },);
}

/**
 Lines saying which providers the launch required and found ready.

 @param providers - providers the command line required, none when it required none

 @returns The one line, or none when none was required

 @example
 ```ts
 const lines = passRequiredLines({ providers: ['hyper',], },);
 ```
 */
export function passRequiredLines({ providers, }: { readonly providers: readonly string[]; },): readonly string[] {
  if (providers.length === 0)
    return [];
  return [`REQUIRED-PROVIDERS ${providers.join(',',)} status=wet`,];
}

/**
 Lines saying which providers the run leaves off and why, so a launch states that Bedrock is not asked rather than
 a run finding out by a refusal (the owner's ruling of 2026-10-20).

 @param off - providers left off, none when nothing is

 @returns One line for each, in the order given, none when nothing is off

 @example
 ```ts
 const lines = passProvidersOffLines({ off: [{ provider: 'bedrock', reason: 'the owner left it off', },], },);
 // => ['PROVIDER OFF bedrock: the owner left it off',]
 ```
 */
export function passProvidersOffLines({ off, }: { readonly off: readonly ProviderOff[]; },): readonly string[] {
  return off.map(function lineOf({
    provider,
    reason,
  },): string {
    return `PROVIDER OFF ${provider}: ${reason}`;
  },);
}

/**
 The line a `--plan` run ends on.

 @param tip - pipeline tip recorded into every artifact

 @param pipelineDigest - identity of the built pipeline

 @param pendingIds - ids of the entries the pass would run, in run order

 @param previewCount - how many of them the line names

 @returns The line

 @example
 ```ts
 const line = passPlanLine({ tip, pipelineDigest, pendingIds: ['tabby',], previewCount: 5, },);
 ```
 */
export function passPlanLine(
  {
    tip,
    pipelineDigest,
    pendingIds,
    previewCount,
  }: {
    readonly tip: string;
    readonly pipelineDigest: string;
    readonly pendingIds: readonly string[];
    readonly previewCount: number;
  },
): string {
  /**
   Ids the line names, the first few of the run order.
   */
  const previewed = pendingIds.slice(
    0,
    previewCount,
  );
  return `PLAN ok tip=${tip} pipeline=${pipelineDigest} client=constructed pending=${
    String(pendingIds.length,)
  } first=${previewed.join(',',)}`;
}

/**
 The complete pairs the pass's own walk found, which its closing line counts
 the artifacts and the declined entries over.

 COUNTED, NEVER WRITTEN DOWN, AND ONE POPULATION. Until 2026-10-10 the closing
 line read against a literal 92, the pairs at the commit the milestone pinned,
 on any clone and any pin; then for a while it divided every artifact file in
 the runs directory, whatever pin it came from, by a count that took in
 declined entries, which never get an artifact, and finished entries whose
 English page it never read.

 @example
 ```ts
 const pairs: PassPairs = { walked: 'every-entry', ids: new Set(['tabby',],), };
 ```
 */
export type PassPairs = {
  /**
   Whether the walk read every entry at the pin, or only those `--only`
   named, which the line says since its figures are then about those alone.
   */
  readonly walked: 'every-entry' | 'named-entries';

  /**
   Entries whose original and English pages the pin both holds, those still
   to run and those finished before alike.
   */
  readonly ids: ReadonlySet<string>;
};

/**
 How many of a set's ids another set holds.

 @param ids - pairs the walk found, the population every count of the DONE
 line is over

 @param among - entries carrying an artifact or a decline record after the
 pass, whose share of the pairs the line reports

 @returns Count of the pairs the second set holds, which never exceeds the
 pairs counted

 @example
 ```ts
 const settled = countAmong({ ids: pairs.ids, among: settledIds, },);
 ```
 */
function countAmong(
  {
    ids,
    among,
  }: {
    readonly ids: Iterable<string>;
    readonly among: ReadonlySet<string>;
  },
): number {
  return [...ids,].filter(function held(id,): boolean {
    return among.has(id,);
  },)
    .length;
}

/**
 The line that closes a pass's report.

 THE ARTIFACTS ARE COUNTED AMONG THE PAIRS THE WALK FOUND, a pair declined and
 carrying no artifact is counted apart and out of the denominator, since it
 never gets one, and an artifact for an entry the walk found no complete pair
 for is counted apart too (`unpairedArtifacts`, printed where the walk read
 every entry: an artifact settled at an earlier pin for an entry this pin does
 not hold, and the artifact of a finished entry whose English page this pin
 lacks alike),
 so `artifacts=N/M` reads as the pairs this pin can still settle and N never
 passes M.

 @param processed - entries this pass finished, which the line names first
 since it is what this run did

 @param pending - entries it set out to run, which `processed` is read against

 @param pairs - complete pairs the walk found, the population every count
 after `processed` is over

 @param settledIds - entries carrying an artifact after the pass, read as the
 scheduler reads them, so a count of them cannot drift from what it skips

 @param declinedIds - entries carrying a decline record after the pass, which
 never get an artifact and so leave the denominator

 @param elapsedMs - time the processing loop took

 @returns The line

 @example
 ```ts
 const line = passDoneLine({ processed: 1, pending: 2, pairs, settledIds, declinedIds, elapsedMs: 5, },);
 ```
 */
export function passDoneLine(
  {
    processed,
    pending,
    pairs,
    settledIds,
    declinedIds,
    elapsedMs,
  }: {
    readonly processed: number;
    readonly pending: number;
    readonly pairs: PassPairs;
    readonly settledIds: ReadonlySet<string>;
    readonly declinedIds: ReadonlySet<string>;
    readonly elapsedMs: number;
  },
): string {
  /**
   Which entries the walk read, and the complete pairs it found among them.
   */
  const {
    walked,
    ids,
  } = pairs;

  /**
   Pairs carrying an artifact.
   */
  const settled = countAmong({
    ids,
    among: settledIds,
  },);

  /**
   Pairs declined and carrying no artifact, which no later run settles.
   */
  const declined = [...ids,].filter(function declinedOnly(id,): boolean {
    return declinedIds.has(id,) && (!settledIds.has(id,));
  },)
    .length;

  /**
   What the counts are over, as the walk read it.
   */
  const scope = (walked === 'every-entry')
    ? ` unpairedArtifacts=${String(settledIds.size - settled,)}`
    : ' (pairs counted over the entries --only named, not every pair at the pin)';
  return `DONE processed=${String(processed,)} of pending=${String(pending,)}; artifacts=${String(settled,)}/${
    String(ids.size - declined,)
  } declined=${String(declined,)}${scope} elapsed=${String(elapsedMs,)}ms`;
}

//endregion Corpus pass lines
