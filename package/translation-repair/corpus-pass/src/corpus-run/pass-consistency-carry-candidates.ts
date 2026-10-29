import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import {
  type CarryBlock,
  type CarryEdgeWords,
  standsOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wrapReplacementText, } from '@monochromatic-dev/translation-repair-typography/ts';
import { foldTranslation, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { crossingWordsLost, } from './pass-consistency-carry-crossing.ts';
import { consistencyFloorVerdict, } from './pass-consistency-floors.ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';
import type { ConsistencyWriterVoice, } from './pass-consistency-cache.ts';

//region Pass consistency carry candidates
// THE TEXTS A RECEIVING PLACE'S CARRY MAY CHOOSE AMONG, each made into the slice
// it would ship and put through every floor with the slice's current wording as
// the incumbent, before any judge sees it: a text the publication rule refuses
// cannot reach the page, so it is withheld from the bench (the translate stage's
// rule since class sixty-six) and kept as a finding. No model output reaches the
// page without the bench and the floors.

/**
 The slice with one block of its English replaced.

 @param english - slice's English

 @param block - block replaced

 @param paragraph - block's new text

 @returns Slice's English with the block's text replaced

 @example
 ```ts
 const candidate = composeSlice({ english, block, paragraph: 'The cat dozes.', },);
 ```
 */
export function composeSlice(
  {
    english,
    block,
    paragraph,
  }: {
    readonly english: string;
    readonly block: CarryBlock;
    readonly paragraph: string;
  },
): string {
  return `${english.slice(
    0,
    block.start,
  )}${paragraph}${english.slice(block.end,)}`;
}

/**
 What the floors make of a candidate slice, as one sentence.

 @param facts - slice

 @param currentText - wording the slice carries now

 @param candidateText - slice with a candidate paragraph in it

 @param declared - name pairs the front matter declares

 @param declaredNames - declared name forms

 @param declaredSubjectForms - declared forms among them that name the subject, which may stand in for one another

 @returns Floors' findings joined as a list of zero or one, none when every floor passed

 @example
 ```ts
 const [refusal,] = floorRefusalOf({ facts, currentText, candidateText, declared, declaredNames, },);
 ```
 */
export function floorRefusalOf(
  {
    facts,
    currentText,
    candidateText,
    declared,
    declaredNames,
    declaredSubjectForms,
  }: {
    readonly facts: ConsistencySliceFacts;
    readonly currentText: string;
    readonly candidateText: string;
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
  },
): readonly string[] {
  /**
   What the floors found.
   */
  const verdict = consistencyFloorVerdict({
    sliceIndex: facts.sliceIndex,
    sourceText: facts.sourceText,
    currentText,
    candidateText,
    lineStructured: facts.lineStructured,
    declared,
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    disputedWordings: facts.disputedWordings,
  },);
  return (verdict.kind === 'passes')
    ? []
    : [
      verdict.findings
        .join(' | ',),
    ];
}

/**
 A text without the line breaks that open and close it.

 @param text - text a writer returned

 @returns Text from its first character that is not a line break to its last

 @example
 ```ts
 const trimmed = withoutEdgeBreaks({ text: '\nThe cat naps.\n', },); // 'The cat naps.'
 ```
 */
function withoutEdgeBreaks({ text, }: { readonly text: string; },): string {
  /**
   The text one whole character at a time.
   */
  const points = Array.from(text,);
  /**
   Index of the first character that is not a line break, none when it is all line breaks.
   */
  const first = points.findIndex(function isWord(point,): boolean {
    return point !== '\n';
  },);
  if (first === (-1))
    return '';
  return points.slice(
    first,
    points.findLastIndex(function isWord(point,): boolean {
      return point !== '\n';
    },) + 1,
  )
    .join('',);
}

/**
 A writer's reply as the paragraph it would ship: folded of invisible variants
 (ledger B112), wrapped at its semantic boundaries unless the line-structure
 rule governs the slice, without the line breaks around it.

 @param seat - seat that wrote it, which each finding names

 @param translation - paragraph as the model wrote it

 @param lineStructured - whether the line-structure rule governs the slice

 @returns Paragraph as it would ship

 @throws Whatever the semantic wrap throws for text it refuses, which the caller contains

 @example
 ```ts
 const paragraph = shippingParagraphOf({ seat, translation, lineStructured: false, },);
 ```
 */
function shippingParagraphOf(
  {
    seat,
    translation,
    lineStructured,
  }: {
    readonly seat: string;
    readonly translation: string;
    readonly lineStructured: boolean;
  },
): string {
  /**
   The reply folded of invisible variants.
   */
  const folded = foldTranslation({
    modelId: seat,
    translation,
  },)
    .translation;
  /**
   The reply laid out as the page lays text out.
   */
  const laid = lineStructured
    ? folded
    : wrapReplacementText({ text: folded, },);
  return withoutEdgeBreaks({ text: laid, },);
}

/**
 The words of other places that a writer's paragraph no longer holds.

 @param paragraph - writer's paragraph as it would ship

 @param block - block it replaces

 @param keeping - English words that must stay as they are, one entry for each place

 @param definitions - page's link and footnote definitions (`pageDefinitionsOf`), which a reference in the paragraph or the
 block resolves against whichever slice holds the definition; none where the slice carries no page (n173)

 @returns Entries the paragraph holds fewer times than the block does, in order

 @example
 ```ts
 const lost = keptWordsLost({ paragraph, block, keeping: ['The cat naps',], definitions: '', },);
 ```
 */
function keptWordsLost(
  {
    paragraph,
    block,
    keeping,
    definitions,
  }: {
    readonly paragraph: string;
    readonly block: CarryBlock;
    readonly keeping: readonly string[];
    readonly definitions: string;
  },
): readonly string[] {
  return keeping.filter(function isLost(words,): boolean {
    return standsOf({
      haystack: paragraph,
      words,
      definitions,
    },) < standsOf({
      haystack: block.text,
      words,
      definitions,
    },);
  },);
}

/**
 The paragraphs the writers wrote that every floor passes, each as the slice
 would ship.

 THE GROUND IS READ ONCE, before any reply is judged: the floors run on the
 slice's own text standing in for a candidate, so a throw from the source or the
 current side (a fault of the pass, or of a page no reader reads) is raised once
 for the block and charged to no writer. What a reply raises after that is the
 reply's own, and is recorded for its seat alone.

 @param voices - writers' replies with their seats, in roster order

 @param english - receiving places' slice English

 @param block - block of it the writers rewrote

 @param keeping - English words of other places inside the block that must stay as they are

 @param crossing - stretches of words of places that must stay as they are and run over an edge of the block, each of which must stand at that edge of the paragraph

 @param facts - slice

 @param declared - name pairs the front matter declares

 @param declaredNames - declared name forms

 @param declaredSubjectForms - declared forms among them that name the subject, which may stand in for one another

 @param l - logger the stage writes to

 @returns Each surviving writer's paragraph, and a finding per reply withheld

 @throws Whatever the floors throw on the slice's own text, which the pass contains

 @example
 ```ts
 const { written, findings, } = writtenParagraphsOf({ voices, english, block, keeping: [], crossing: [], facts, declared, declaredNames, l, },);
 ```
 */
export function writtenParagraphsOf(
  {
    voices,
    english,
    block,
    keeping,
    crossing,
    facts,
    declared,
    declaredNames,
    declaredSubjectForms,
    l,
  }: {
    readonly voices: readonly ConsistencyWriterVoice[];
    readonly english: string;
    readonly block: CarryBlock;
    readonly keeping: readonly string[];
    readonly crossing: readonly CarryEdgeWords[];
    readonly facts: ConsistencySliceFacts;
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly l: Logger;
  },
): {
  readonly written: readonly {
    readonly seat: string;
    readonly text: string;
  }[];
  readonly findings: readonly string[];
} {
  /**
   Logger pre-tagged with this function's name.
   */
  const wl = tagged({
    tag: writtenParagraphsOf.name,
    l,
  },);
  // The slice's own text stands in for a candidate: the floors read the source
  // and the current side once, here, and what they throw on those is not a writer's.
  floorRefusalOf({
    facts,
    currentText: english,
    candidateText: english,
    declared,
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
  },);
  /**
   The English of the whole page the slice belongs to, absent for a slice read alone.
   */
  const { page, } = facts;
  /**
   The link and footnote definitions of the page, which a reference in a paragraph or in the block resolves against; none for a slice
   read alone.
   */
  const definitions = (page === undefined) ? '' : pageDefinitionsOf({ pageText: page.text, });
  /**
   Each reply beside what became of it: its paragraph, or why it was withheld.
   */
  const judged = voices.map(function judgeOne({
    seat,
    translation,
  },): {
    readonly kept: readonly {
      readonly seat: string;
      readonly text: string
    }[];
    readonly finding: readonly string[];
  } {
    try {
      /**
       The paragraph as it would ship.
       */
      const text = shippingParagraphOf({
        seat,
        translation,
        lineStructured: facts.lineStructured,
      },);
      /**
       The words of other places that this paragraph no longer holds as many times as the block did.
       */
      const lost = keptWordsLost({
        paragraph: text,
        block,
        keeping,
        definitions,
      },);
      if (lost.length > 0) {
        wl.warn(`withheld ${seat} from the bench: its paragraph no longer holds the words of another place that must stay`,);
        return {
          kept: [],
          finding: [`consistency-candidate-refused (${seat}): the paragraph no longer holds the words of another place that must stay as they are`,],
        };
      }
      /**
       The stretches of words of a place that must stay as it is and runs over an edge of the block, which this paragraph
       does not hold at that edge.
       */
      const edgeLost = crossingWordsLost({
        paragraph: text,
        crossing,
      },);
      if (edgeLost.length > 0) {
        wl.warn(`withheld ${seat} from the bench: its paragraph no longer holds, at the edge of the block, the words of a place that runs over it`,);
        return {
          kept: [],
          finding: [`consistency-candidate-refused (${seat}): the paragraph no longer holds, at the edge of the block, the words of a place that runs over it and must stay as they are`,],
        };
      }
      /**
       What the floors find against the slice carrying it, none when they pass it.
       */
      const [refusal,] = floorRefusalOf({
        facts,
        currentText: english,
        candidateText: composeSlice({
          english,
          block,
          paragraph: text,
        },),
        declared,
        declaredNames,
        ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
      },);
      if (refusal === undefined) {
        return {
          kept: [{
            seat,
            text,
          },],
          finding: [],
        };
      }
      wl.warn(`withheld ${seat} from the bench: the floors refuse its paragraph: ${refusal}`,);
      return {
        kept: [],
        finding: [`consistency-candidate-refused (${seat}): ${refusal}`,],
      };
    }
    catch (error) {
      wl.warn(`withheld ${seat} from the bench: its paragraph could not be read (${exchangeFailureLogText({ error, },)})`,);
      return {
        kept: [],
        finding: [`consistency-candidate-unreadable (${seat})`,],
      };
    }
  },);
  return {
    written: judged.flatMap(function keptOf({ kept, },) {
      return kept;
    },),
    findings: judged.flatMap(function findingOf({ finding, },) {
      return finding;
    },),
  };
}

//endregion Pass consistency carry candidates
