import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type ConsistencyVoice,
  slotInSpan,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  artifactGenerationCutsContainmentAtJoints,
  artifactGenerationReadsLineEndingsAsCodePadding,
  artifactGenerationReadsSlotsBesideContainingQuotes,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';

//region Artifact consistency span slot
// THE SLOT A RECORDED PLACE OF A FRAME IS EXPECTED TO CARRY, recounted from its ballots inside the words it was located by. The
// pass reads the slot with the link and footnote definitions of the whole page, and the file holds no page, so from generation
// eighteen the place records the definitions that move its slot (`definitions`, ledger B665) and the recount reads the slot with
// them; a place of generation seventeen or earlier records none and is recounted over the span alone, the limit written at the head
// of `artifact-two-lane-read-consistency-span.ts`. Recorded definitions that move no slot are refused, since the pass records
// them only where they move one.
//
// TWO READINGS ARE GATED ON THE GENERATION, so a file is recounted under the rules its pass counted by: a joint of a longer word bars
// a quote from containing the located words, and a line ending pads inline code in the span, from generation nineteen
// (`artifactGenerationCutsContainmentAtJoints`, `artifactGenerationReadsLineEndingsAsCodePadding`). A file of eighteen or earlier is
// counted with only the letters beside a match barring containment and a literal space alone padding code.

/**
 The slot the ballots pin down inside a place's words, as a list of zero or one.
 */
type SlotReading = ReturnType<typeof slotInSpan>;

/**
 The slot the ballots of a refrain pin down inside the recorded words of a place, read with the definitions the place records,
 none for a run or when they pin none. A slot is counted beside the quotes the generation counted it beside: any quote that
 contains the located words from generation eighteen, and a quote that is the located words before it.

 @param voices - usable ballots of the group

 @param label - the place's label

 @param agreed - the folded words the place was located by

 @param span - the English's own words of the span, as the place records them

 @param refrain - whether the group repeats a refrain, the only repeat that has a slot

 @param definitions - the page definitions the place records, absent where it records none

 @param generation - generation the section was written under, which says how a slot is counted, whether definitions are read, whether a
 joint of a longer word bars a quote from containing the words, and whether line endings pad the inline code of the span

 @param path - where the place's rendering sits, for the parse error

 @returns The slot as the span holds it and its offset in the span, as a list of zero or one

 @throws {@link ArtifactParseError} when the place records definitions that change nothing about the slot the ballots give

 @example
 ```ts
 const [expected,] = expectedSlotOf({ voices, label, agreed, span, refrain: true, generation: 18, path, },);
 ```
 */
export function expectedSlotOf(
  {
    voices,
    label,
    agreed,
    span,
    refrain,
    definitions,
    generation,
    path,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly label: string;
    readonly agreed: string;
    readonly span: string;
    readonly refrain: boolean;
    readonly definitions?: string;
    readonly generation: TwoLaneArtifactGeneration;
    readonly path: string;
  },
): SlotReading {
  /**
   The slot read inside the span with the given definitions, or none of the page's.

   @param reading - the definitions to read with, none for the span alone

   @returns The slot as a list of zero or one
   */
  function slotRead({ reading, }: { readonly reading: { readonly definitions?: string; }; },): SlotReading {
    if (!refrain)
      return [];
    return slotInSpan({
      voices,
      label,
      words: agreed,
      span,
      slotAgreement: artifactGenerationReadsSlotsBesideContainingQuotes({ generation, },) ? 'containing-quote' : 'exact-quote',
      edges: artifactGenerationCutsContainmentAtJoints({ generation, },) ? 'locator' : 'letters',
      codePadding: artifactGenerationReadsLineEndingsAsCodePadding({ generation, },) ? 'spaces-and-line-endings' : 'spaces',
      ...reading,
    },);
  }
  /**
   The definitions the reading is given, none where the place records none.
   */
  const given = (definitions === undefined) ? {} : { definitions, };
  /**
   The slot the recount expects.
   */
  const expected = slotRead({ reading: given, },);
  if (definitions === undefined)
    return expected;
  /**
   The slot read with no definitions at all.
   */
  const [alone,] = slotRead({ reading: {}, },);
  /**
   The slot read with the recorded definitions.
   */
  const [read,] = expected;
  if ((alone?.slot === read?.slot) && (alone?.slotStart === read?.slotStart)) {
    throw new ArtifactParseError({
      path: `${path}.definitions`,
      reason: 'the definitions that change the slot the usable ballots give inside those words, and none where they change none',
    },);
  }
  return expected;
}

//endregion Artifact consistency span slot
