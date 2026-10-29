import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { RepeatedTitleSpan, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  mostGiven,
  unwrappedRendering,
} from './page-title-lexicon-count.ts';
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
// current rendering. A TITLE SETTLES ONLY ON A RENDERING TWO OR MORE VOICES
// GAVE (the lead's ruling of 2026-10-25, on the same rule, re-check 2 finding
// F1): a title one voice alone rendered, or that tied at one voice apiece,
// stays unsettled with a finding, however many voices were heard. A tie of two
// voices apiece or more is still broken by the earliest seat.

/**
 Heading of the sheet block.
 */
const HEADING = 'TITLES THIS PAGE REPEATS THAT THE ARCHIVE DOES NOT RENDER '
  + '(the bench settled one English rendering for each, so every passage renders the same title the same way):';

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
 Settles one English rendering for each title a page repeats, asking the bench once.
 A round that heard fewer than two voices settles nothing, and a title settles
 only on a rendering two or more voices gave.

 @param client - provider client

 @param modelIds - bench asked, in roster order

 @param sourceText - original document

 @param spans - titles the page repeats and the archive leaves unpaired

 @param identityContext - page's declared identity without the lexicon's
 own lines, so a work's official English title a web lookup or a note
 establishes is on the sheet (ledger B28)

 @param signal - entry's abort

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
   Each title's leading rendering, where some voice gave one and enough voices
   were heard.
   */
  const leaders = tooFewVoices ? [] : spans.flatMap(function settle(
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
      return (rendering === undefined) ? '' : unwrappedRendering({ rendering, },);
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
  /**
   Titles a rendering two or more voices gave settles: the leader of a title
   that only one voice rendered, or that tied at one voice apiece, is no
   decision, since one judge alone never decides.
   */
  const titles = leaders.filter(function namedByTwoVoices(title,): boolean {
    return title.voices >= MIN_STAGE_VOICES;
  },);
  /**
   Titles some voice rendered that no two voices agreed on.
   */
  const unsettled = leaders.length - titles.length;
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
      ...((unsettled === 0)
        ? []
        : [
          `page title lexicon left ${String(unsettled,)} repeated ${
            wordForCount({
              count: unsettled,
              one: 'title',
              many: 'titles',
            },)
          } unsettled, since no rendering was given by ${String(MIN_STAGE_VOICES,)} ${
            wordForCount({
              count: MIN_STAGE_VOICES,
              one: 'voice',
              many: 'voices',
            },)
          }`,
        ]),
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
