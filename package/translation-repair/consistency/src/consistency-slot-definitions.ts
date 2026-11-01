import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ConsistencyVoice, } from './consistency-count-types.ts';
import type { QuotesByLabel, } from './consistency-majority.ts';
import { slotInSpan, } from './consistency-slot-in-span.ts';

//region Consistency slot definitions
// THE DEFINITIONS OF THE PAGE THAT MOVE A SLOT, which a recorded artifact keeps so that a reader holding no page reads the slot as the
// pass did. The pass reads a frame's slot inside its span with the link and footnote definitions of the whole page, so a shortcut
// reference in the span (`a[b]c`) whose definition stands in another slice is a link, `a[b]c` is one word and `b` is no stand. The file
// holds no page to rebuild the definitions from (the consolidated English is recorded nowhere), so the pass records them. THEY ARE
// RECORDED ONLY WHERE THEY CHANGE THE SLOT: a page's definitions are usually many (every footnote) and move nothing for the span, and a
// record that carried them all for every frame would hold the page's footnotes once for each place. Where they do change it, the
// smallest set of the page's definitions that gives the same slot is recorded, entry by entry in the order of the page, so a span that
// reads one reference records one definition and not the page's footnotes.

/**
 One reading of a slot: the slot words and where they start in the span.
 */
type SlotReading = ReturnType<typeof slotInSpan>;

/**
 Whether two readings of a slot are the same: both none, or the same slot at the same offset.

 @param left - a reading, a list of zero or one

 @param right - the other reading

 @returns Whether the two agree

 @example
 ```ts
 const same = sameSlot({ left: [], right: [], },); // true
 ```
 */
function sameSlot(
  {
    left,
    right,
  }: {
    readonly left: SlotReading;
    readonly right: SlotReading;
  },
): boolean {
  /**
   The first reading's slot, none when it has none.
   */
  const [one,] = left;
  /**
   The second reading's slot, none when it has none.
   */
  const [other,] = right;
  if ((one === undefined) || (other === undefined))
    return one === other;
  return (one.slot === other.slot) && (one.slotStart === other.slotStart);
}

/**
 Entries of a page's definitions, each starting at a line that opens with a bracket (a definition's label), so a footnote of
 several paragraphs, whose later lines are indented, stays one entry.

 @param definitions - the page's definitions in the canonical form

 @returns The entries in the order of the page

 @example
 ```ts
 const entries = entriesOf({ definitions: '[a]: u\n[b]: v', },); // ['[a]: u', '[b]: v']
 ```
 */
function entriesOf({ definitions, }: { readonly definitions: string; },): readonly string[] {
  return definitions
    .split('\n[',)
    .map(function restored(
      piece,
      at,
    ): string {
      return (at === 0) ? piece : `[${piece}`;
    },);
}

/**
 The page definitions that move the slot of a span: the smallest set of them, in the order of the page, that gives the slot the whole
 set gives, and none when the span's slot is the same with no definitions at all.

 @param voices - usable ballots

 @param label - the place's label

 @param words - the folded words the span was agreed by

 @param span - the English's own words of the span

 @param quotesByLabel - the quote entries of these ballots by label, read once for the group

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), empty for a place that carries no page

 @returns The definitions to record, or an empty string when they change nothing; a set that does not read back as the page's
 definitions is never returned, the whole set is returned in its place

 @example
 ```ts
 const moving = definitionsMovingSlotOf({
   voices,
   label: 'OCCURRENCE 1',
   words: 'a[b]c',
   span: 'a[b]c',
   quotesByLabel,
   definitions: '[b]: https://x.example/b\n[z]: https://x.example/z',
 },); // '[b]: https://x.example/b'
 ```
 */
export function definitionsMovingSlotOf(
  {
    voices,
    label,
    words,
    span,
    quotesByLabel,
    definitions,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly label: string;
    readonly words: string;
    readonly span: string;
    readonly quotesByLabel: QuotesByLabel;
    readonly definitions: string;
  },
): string {
  if (definitions === '')
    return '';
  /**
   The slot read with a set of definitions.

   @param set - the definitions to read with

   @returns The slot as a list of zero or one
   */
  function slotWith({ set, }: { readonly set: string; },): SlotReading {
    return slotInSpan({
      voices,
      label,
      words,
      span,
      quotesByLabel,
      definitions: set,
    },);
  }
  /**
   The slot the whole set gives.
   */
  const whole = slotWith({ set: definitions, },);
  if (sameSlot({
    left: whole,
    right: slotWith({ set: '', },),
  },))
    return '';
  /**
   The entries of the definitions, in the order of the page.
   */
  const entries = entriesOf({ definitions, },);
  /**
   Takes an entry out of the standing ones when the slot stays what the whole set gives.

   @param standing - the entries still needed

   @param entry - the entry to try to take out

   @returns The entries that stand after the attempt
   */
  function without(
    {
      standing,
      entry,
    }: {
      readonly standing: readonly string[];
      readonly entry: string;
    },
  ): readonly string[] {
    /**
     The entries that remain when this one is taken out.
     */
    const rest = standing.filter(function isOther(other,): boolean {
      return other !== entry;
    },);
    return ((rest.length > 0) && sameSlot({
      left: whole,
      right: slotWith({ set: rest.join('\n',), },),
    },)) ? rest : standing;
  }
  /**
   The entries still needed, taken out one at a time while the slot stays what the whole set gives.
   */
  const kept = entries.reduce(
    function keepIfNeeded(
      standing: readonly string[],
      entry: string,
    ): readonly string[] {
      return without({
        standing,
        entry,
      },);
    },
    entries,
  );
  /**
   The entries as one text.
   */
  const joined = kept.join('\n',);
  return (pageDefinitionsOf({ pageText: joined, },) === joined) ? joined : definitions;
}

//endregion Consistency slot definitions
