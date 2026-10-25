import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import { scoreEditedText, } from '../editor-fidelity-score.ts';
import type { EditorFidelityTrial, } from '../editor-fidelity-trials.ts';
import { runEditorTrial, } from './editor-fidelity-trial.ts';

//region Editor fidelity pool verdict
// THE SCOREABILITY CHECK a drawn trial passes before it joins the pool. Three questions, none of which asks a model.
// Does the damaged text read as not fixed, so that doing nothing cannot pass? Does the cheap reply that deletes the quoted
// region, or the one that gives the clean number alone, read as not clean, so that the instrument cannot be passed
// without reading? And does a scripted ideal editor, sent through the real stage path (the sheet, the parse and the apply
// gate), read as a clean fix with nothing else touched, so that a perfect edit can be scored perfect?

/**
 Says nothing, for a logger whose lines are not wanted.
 */
function say(): void {
  // A check over every drawn trial would otherwise log a round for each.
}

/**
 Logger that says nothing, since a check over every drawn trial would otherwise log a round for each.
 */
const QUIET: Logger = {
  debug: say,
  error: say,
  fatal: say,
  flush: function flush(): Promise<void> {
    return Promise.resolve();
  },
  info: say,
  trace: say,
  warn: say,
};

/**
 Longest an exchange of the scripted editor may take, in milliseconds.
 */
const SCRIPTED_DEADLINE_MS = 30_000;

/**
 The text the region becomes when the defect is mended and nothing else is touched.

 @param trial - trial whose region is mended

 @returns The region's text with the inserted sentence removed or the altered number restored

 @example
 ```ts
 const newText = idealRegionText({ trial, },);
 ```
 */
function idealRegionText({ trial, }: { readonly trial: EditorFidelityTrial; },): string {
  return trial.damagedText
    .slice(
    trial.region
      .startOffset,
    trial.region
      .endOffset,
    )
    .replace(
    trial.damagedDefect,
    trial.cleanDefect,
    );
}

/**
 The client's free-text ask, which the editor never makes, answered by a rejection saying the ask is unused.
 */
function unusedChatText(): Promise<never> {
  return Promise.reject(new Error('chatText unused by the editor',),);
}

/**
 The client's quota ask, which the editor never makes, answered by a rejection saying the ask is unused.
 */
function unusedQuotas(): Promise<never> {
  return Promise.reject(new Error('quotas unused by the editor',),);
}

/**
 A client that answers every ask with the ideal fix.

 @param trial - trial the client answers

 @returns The client

 @example
 ```ts
 const client = idealEditorClient({ trial, },);
 ```
 */
function idealEditorClient({ trial, }: { readonly trial: EditorFidelityTrial; },): ModelClient {
  /**
   The reply every ask gets.
   */
  const report = {
    edits: [
      {
        region: 1,
        newText: idealRegionText({ trial, },),
      },
    ],
  };
  return {
    chatText: unusedChatText,
    quotas: unusedQuotas,
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      if (!request.validate(report,)) {
        return Promise.resolve({
          kind: 'schema-mismatch',
          rawText: JSON.stringify(report,),
          detail: 'unreadable',
        },);
      }
      return Promise.resolve({
        kind: 'ok',
        value: report,
        rawText: JSON.stringify(report,),
      },);
    },
  };
}

/**
 The patched text when the quoted region becomes a given text.

 @param trial - trial patched

 @param replacement - what the region becomes

 @returns The damaged text with its region replaced

 @example
 ```ts
 const patched = patchedWith({ trial, replacement: '', },);
 ```
 */
export function patchedWith(
  {
    trial,
    replacement,
  }: {
    readonly trial: EditorFidelityTrial;
    readonly replacement: string;
  },
): string {
  return trial.damagedText
    .slice(
    0,
    trial.region
      .startOffset,
    )
    + replacement
    + trial.damagedText
      .slice(trial.region
        .endOffset,);
}

/**
 Whether a patched text reads as a clean fix.

 @param trial - trial read

 @param patchedText - text after the edit

 @returns Whether the reading is exact or equivalent

 @example
 ```ts
 const clean = readsClean({ trial, patchedText, },);
 ```
 */
function readsClean(
  {
    trial,
    patchedText,
  }: {
    readonly trial: EditorFidelityTrial;
    readonly patchedText: string;
  },
): boolean {
  /**
   The reading of the patched text.
   */
  const { fixed, } = scoreEditedText({
    trial,
    patchedText,
  },);
  return (fixed === 'exact') || (fixed === 'equivalent');
}

/**
 Says why a trial cannot join the pool, or that it can.

 @param trial - trial built from a drawn twin

 @returns The empty text when the trial passes, else the reason in scorecard-stable wording

 @example
 ```ts
 const verdict = await scoreableVerdict({ trial, },);
 ```
 */
export async function scoreableVerdict({ trial, }: { readonly trial: EditorFidelityTrial; },): Promise<string> {
  if (trial.envelopes
    .length
    !== 1)
    return `envelopes ${String(trial.envelopes
      .length,)}`;
  if (trial.region
    .endOffset
    <= trial.region
      .startOffset)
    return 'empty region';
  if (scoreEditedText({
    trial,
    patchedText: trial.damagedText,
  },)
    .fixed
    !== 'none')
    return 'damaged text scores fixed';
  if (readsClean({
    trial,
    patchedText: patchedWith({
      trial,
      replacement: '',
    },),
  },))
    return 'deleting the quoted region reads as a fix';
  if (readsClean({
    trial,
    patchedText: patchedWith({
      trial,
      replacement: trial.cleanDefect,
    },),
  },))
    return 'the clean defect alone reads as a fix';
  /**
   What the scripted ideal editor made of the trial through the real stage path.
   */
  const reading = await runEditorTrial({
    client: idealEditorClient({ trial, },),
    cardId: 'claude-sonnet-5-5-effort-low',
    trial,
    signal: new AbortController().signal,
    perCallTimeoutMs: SCRIPTED_DEADLINE_MS,
    l: QUIET,
    stageSettings: readStageSettings({ env: {}, },),
  },);
  if (reading.kind !== 'answered')
    return `scripted editor read ${reading.kind}`;
  if (((reading.score
    .fixed
    !== 'exact') && (reading.score
      .fixed
      !== 'equivalent'))
    || reading.score
      .collateral) {
    return `scripted editor scored ${reading.score
      .fixed} collateral ${String(reading.score
        .collateral,)}`;
  }
  return '';
}

//endregion Editor fidelity pool verdict
