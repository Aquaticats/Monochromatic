import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  rendersAsNothing,
  straightenQuotes,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  foldedLine,
  type RepeatedTitleSpan,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  buildPageTitleLexiconMessages,
  isPageTitleLexiconWire,
  PAGE_TITLE_LEXICON_RESPONSE_FORMAT,
  type PageTitleAnswers,
  type PageTitleLexiconWire,
  resolvePageTitleAnswers,
} from './page-title-lexicon-wire.ts';
import {
  type FanOutMode,
  gatherStageVoices,
  MIN_STAGE_VOICES,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Page title lexicon stage
// ONE RENDERING PER REPEATED TITLE, ONCE PER PAGE (ledger H16, 2026-09-28).
// The bench is asked once; each title keeps the rendering most voices gave,
// compared without case, spacing, apostrophe style or wrapping marks (ledger
// B24), the earliest seat on the
// roster breaking a tie so a resumed page reads the same answer. The lines
// are evidence beside the page-name block: the judges still decide.
//
// NO TITLE SETTLES ON FEWER THAN TWO HEARD VOICES (the lead's ruling of
// 2026-10-25, on the owner's rule that one judge alone never decides): a
// bench of seven that lost six seats for the round used to settle, and cache,
// the one voice's rendering as a line the translators and judges read. A
// round that heard one voice settles nothing and every repeated title keeps its
// current rendering. A tie among three heard voices is still broken by the
// earliest seat (the ledger records that path as outside this ruling).

/**
 Heading of the sheet block.
 */
const HEADING = 'TITLES THIS PAGE REPEATS THAT THE ARCHIVE DOES NOT RENDER '
  + '(the bench settled one English rendering for each, so every passage renders the same title the same way):';

/**
 Marks a rendering may arrive wrapped in, taken off before comparing: quotes
 of every style a voice answers in, and Markdown emphasis of either kind
 (ledger B24).
 */
const WRAPPERS: ReadonlySet<string> = new Set([
  '"',
  '\'',
  '*',
  '_',
  '“',
  '”',
  '‘',
  '’',
  '《',
  '》',
  '「',
  '」',
  '『',
  '』',
],);

/**
 One title's settled rendering.

 @example
 ```ts
 const settled: SettledPageTitle = { source: '猫之歌', occurrences: 3, rendering: 'Song of the Cat', voices: 2, heard: 3, };
 ```
 */
export type SettledPageTitle = {
  /**
   Title as the original writes it.
   */
  readonly source: string;
  /**
   Marked places it stands on the page.
   */
  readonly occurrences: number;
  /**
   Rendering most voices gave.
   */
  readonly rendering: string;
  /**
   Voices that gave it.
   */
  readonly voices: number;
  /**
   Voices heard.
   */
  readonly heard: number;
};

/**
 What the lexicon round settled.

 @example
 ```ts
 const lexicon: PageTitleLexicon = { titles: [], heard: 0, findings: [], };
 ```
 */
export type PageTitleLexicon = {
  /**
   Titles settled, in order of first appearance; a title no voice rendered is absent.
   */
  readonly titles: readonly SettledPageTitle[];
  /**
   Voices heard.
   */
  readonly heard: number;
  /**
   What the round reported.
   */
  readonly findings: readonly string[];
};

/**
 One voice's reply resolved against the sheet's numbering, under its name.

 @example
 ```ts
 const voice: ResolvedVoice = { modelId: 'glm-5.3', answers: { renderings: new Map(), findings: [], }, };
 ```
 */
type ResolvedVoice = {
  /**
   Seat that answered, which its findings are reported under.
   */
  readonly modelId: RosterModelId;
  /**
   Its renderings by title number, with what reading the reply found.
   */
  readonly answers: PageTitleAnswers;
};

/**
 A rendering with its wrapping marks and surrounding space off.

 @param rendering - rendering as a voice gave it

 @returns Rendering as the sheet shows it

 @example
 ```ts
 unwrapped({ rendering: ' “Song of the Cat” ', },); // 'Song of the Cat'
 ```
 */
function unwrapped({ rendering, }: { readonly rendering: string; },): string {
  /**
   Rendering without surrounding space; every wrapper is one UTF-16 unit, so
   an index walks them from either end.
   */
  const trimmed = rendering.trim();
  /**
   Bounds of the wrapped words: first unit that is no wrapper, and one past the last.
   */
  const bounds = {
    start: 0,
    end: trimmed.length,
  };
  while ((bounds.start < bounds.end) && WRAPPERS.has(trimmed.charAt(bounds.start,),))
    bounds.start += 1;
  while ((bounds.end > bounds.start) && WRAPPERS.has(trimmed.charAt(bounds.end - 1,),))
    bounds.end -= 1;
  return trimmed
    .slice(
    bounds.start,
    bounds.end,
    )
    .trim();
}

/**
 Form two renderings share when they differ only in case, spacing of any
 kind, or the style of their apostrophes and quotation marks: the typography
 fold, since the typography restoration makes such renderings one (ledger
 B24).

 @param rendering - unwrapped rendering

 @returns Comparison key

 @example
 ```ts
 comparisonKey({ rendering: 'The  Cat’s song', },); // "the cat's song"
 ```
 */
function comparisonKey({ rendering, }: { readonly rendering: string; },): string {
  return foldedLine({ text: straightenQuotes({ text: rendering, },), },)
    .toLowerCase();
}

/**
 The rendering most voices gave one title, the earliest seat breaking a tie.

 A RENDERING THAT SHOWS A READER NOTHING IS NONE, asked of `rendersAsNothing`
 (ledger B40): asked of an empty-string comparison after the trim, two seats
 answering a zero-width space outvoted the one seat answering in words, and
 the title settled on a rendering no reader sees.

 @param answers - each voice's rendering of the title, in roster order, empty
 for a voice that gave none

 @returns Rendering and how many voices gave it, none where no voice rendered it

 @example
 ```ts
 const chosen = mostGiven({ answers: ['Song of the Cat', 'song of the cat', 'Cat Song',], },);
 ```
 */
function mostGiven(
  { answers, }: { readonly answers: readonly string[]; },
): readonly {
  readonly rendering: string;
  readonly voices: number;
}[] {
  /**
   Voices per comparison key, with the first wording given under it.
   */
  const tally = answers
    .filter(function given(answer,): boolean {
      return !rendersAsNothing({ text: answer, },);
    },)
    .reduce(
    function count(
      seen,
      answer,
    ) {
        /**
         Key the answer counts under.
         */
      const key = comparisonKey({ rendering: answer, },);
        /**
         Count so far under that key.
         */
      const held = seen.get(key,);
      seen.set(
        key,
        {
          rendering: held?.rendering ?? answer,
          voices: (held?.voices ?? 0) + 1,
        },
      );
      return seen;
    },
    new Map<string, {
      readonly rendering: string;
      readonly voices: number
    }>(),
    );
  return [...tally.values(),]
    .reduce(
    function best(
      chosen: readonly {
        readonly rendering: string;
        readonly voices: number
      }[],
      candidate,
    ) {
        /**
         Leader so far, none before the first.
         */
      const [leader,] = chosen;
      return ((leader === undefined) || (candidate.voices > leader.voices)) ? [candidate,] : chosen;
    },
    [],
    );
}

/**
 Settles one English rendering for each title a page repeats, asking the bench once.
 A round that heard fewer than two voices settles nothing.

 @param client - provider client

 @param modelIds - bench asked, in roster order

 @param sourceText - the original document

 @param spans - titles the page repeats and the archive leaves unpaired

 @param identityContext - the page's declared identity without the lexicon's
 own lines, so a work's official English title a web lookup or a note
 establishes is on the sheet (ledger B28)

 @param signal - the entry's abort

 @param exchangeTimeoutMs - per-call timeout

 @param l - entry logger

 @param fanOut - seats the round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Settled titles, voices heard and findings

 @example
 ```ts
 const lexicon = await settlePageTitles({ client, modelIds, sourceText, spans, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function settlePageTitles(
  {
    client,
    modelIds,
    sourceText,
    spans,
    identityContext,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly spans: readonly RepeatedTitleSpan[];
    readonly identityContext?: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<PageTitleLexicon> {
  /**
   Logger pre-tagged with this function's name.
   */
  const tl = tagged({
    tag: settlePageTitles.name,
    l,
  },);
  if (spans.length === 0) {
    tl.debug('the page repeats no unpaired title, so nobody is asked',);
    return {
      titles: [],
      heard: 0,
      findings: [],
    };
  }
  /**
   Quorum-bounded replies.
   */
  const gather = await gatherStageVoices<PageTitleLexiconWire>({
    client,
    modelIds,
    messages: buildPageTitleLexiconMessages({
      sourceText,
      titles: spans.map(function sourceOf(span,): string {
        return span.source;
      },),
      ...((identityContext === undefined) ? {} : { identityContext, }),
    },),
    signal,
    exchangeTimeoutMs,
    responseFormat: PAGE_TITLE_LEXICON_RESPONSE_FORMAT,
    validate: isPageTitleLexiconWire,
    stage: 'page-title-lexicon',
    l: tl,
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);
  /**
   Voices in roster order, so the earliest seat breaks a tie.
   */
  const voices = gather.voices
    .toSorted(function byRoster(
      left,
      right,
    ): number {
      return modelIds.indexOf(left.modelId,) - modelIds.indexOf(right.modelId,);
    },);
  /**
   Voices heard.
   */
  const heard = voices.length;
  /**
   Each voice's renderings by title number, in roster order. Read once per
   voice rather than searched once per title, so a voice answering one title
   twice or numbering a title the sheet never listed is a finding under its
   name instead of an answer passed over.
   */
  const resolved = voices.map(function resolvedOf(voice,): ResolvedVoice {
    return {
      modelId: voice.modelId,
      answers: resolvePageTitleAnswers({
        wire: voice.value,
        asked: spans.length,
      },),
    };
  },);
  /**
   Whether the round heard too few voices for any title to settle on: one judge
   alone never decides (the lead's ruling of 2026-10-25), so a thin round
   settles nothing and every repeated title keeps its current rendering.
   */
  const tooFewVoices = heard < MIN_STAGE_VOICES;
  if (tooFewVoices) {
    tl.warn(
      `PAGE TITLES heard=${String(heard,)}, below the ${String(MIN_STAGE_VOICES,)} ${
        wordForCount({
          count: MIN_STAGE_VOICES,
          one: 'voice',
          many: 'voices',
        },)
      } a title settles on; nothing settles`,
    );
  }
  /**
   Each title's settled rendering, where some voice gave one and enough voices
   were heard.
   */
  const titles = tooFewVoices ? [] : spans.flatMap(function settle(
    span,
    at,
  ): readonly SettledPageTitle[] {
    /**
     Each voice's rendering of this title, empty where it gave none.
     */
    const answers = resolved.map(function answerOf(voice,): string {
      /**
       This voice's rendering of the title, if it gave one.
       */
      const rendering = voice.answers
        .renderings
        .get(at + 1,);
      return (rendering === undefined) ? '' : unwrapped({ rendering, },);
    },);
    return mostGiven({ answers, },)
      .map(function toSettled(chosen,): SettledPageTitle {
        return {
          source: span.source,
          occurrences: span.occurrences,
          rendering: chosen.rendering,
          voices: chosen.voices,
          heard,
        };
      },);
  },);
  tl.info(
    `PAGE TITLES heard=${String(heard,)} asked=${String(spans.length,)} settled=${String(titles.length,)}`,
  );
  return {
    titles,
    heard,
    findings: [
      ...gather.findings,
      ...resolved.flatMap(function findingsOf(voice,): readonly string[] {
        return voice.answers
          .findings
          .map(function underItsVoice(finding,): string {
            return `${voice.modelId}: ${finding}`;
          },);
      },),
      `page title lexicon settled ${String(titles.length,)} of ${String(spans.length,)} repeated ${
        wordForCount({
          count: spans.length,
          one: 'title',
          many: 'titles',
        },)
      } from ${
        String(heard,)
      } ${
        wordForCount({
          count: heard,
          one: 'voice',
          many: 'voices',
        },)
      }`,
      ...(tooFewVoices
        ? [
          `page title lexicon heard fewer than ${
            String(MIN_STAGE_VOICES,)
          } ${
            wordForCount({
              count: MIN_STAGE_VOICES,
              one: 'voice',
              many: 'voices',
            },)
          }, so no title settles and every repeated title keeps its current rendering`,
        ]
        : []),
    ],
  };
}

/**
 Identity-context lines carrying a page's settled titles: a heading and one
 line per title, empty when none settled.

 @param titles - settled titles

 @returns Sheet lines

 @example
 ```ts
 const lines = pageTitleLines({ titles, },);
 ```
 */
export function pageTitleLines(
  { titles, }: { readonly titles: readonly SettledPageTitle[]; },
): readonly string[] {
  if (titles.length === 0)
    return [];
  return [
    HEADING,
    ...titles.map(function toLine(title,): string {
      return `- ${title.source} (${String(title.occurrences,)} ${
        wordForCount({
          count: title.occurrences,
          one: 'place',
          many: 'places',
        },)
      } on the page): "${title.rendering}" (${
        String(title.voices,)
      } of ${String(title.heard,)} ${
        wordForCount({
          count: title.heard,
          one: 'voice',
          many: 'voices',
        },)
      })`;
    },),
  ];
}

//endregion Page title lexicon stage
