import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import { contextRoot, } from './log-context.ts';

//region Typography mask refusal
// A TYPOGRAPHY READER THAT GETS AN ALL-FALSE MASK FROM A REFUSAL SAYS SO. The prose mask of a text the page grammar refuses
// has no prose (`typography-prose-mask.ts`), the same all-false mask a slice of code alone gets, so a rule that curls nothing in
// it is silent either way. The page assembly guard withdraws what the lanes chose that breaks the parse, but it does not read
// an archive slice no lane replaced, and a bound of the grammar (more than a thousand emphasis delimiters, more than two
// hundred fifty-six levels of nesting) refuses a paragraph of ordinary prose. A reader that was handed a refusal logs one
// finding at the warning level, naming the slice, so the silence is a line a person can find and not a pass.

/**
 The root logger of the typography readers.
 */
const l = contextRoot({ tag: 'typography-prose-mask', },);

/**
 What to call a text that no slice number names: its first characters, so a finding can be matched with the text it is about.
 */
const NAME_LENGTH = 48;

/**
 Names a text for a finding when no slice number is at hand.

 @param text - text a reader was handed

 @returns First characters of the text between quotation marks

 @example
 ```ts
 const name = textNamed({ text: 'Oops {brace it\'s here.', },); // 'the text "Oops {brace it\'s here."'
 ```
 */
export function textNamed({ text, }: { readonly text: string; },): string {
  return `the text ${JSON.stringify(wholeOpening({
    text,
    units: NAME_LENGTH,
  },),)}`;
}

/**
 Logs that a typography reader was handed a refusal of the page grammar, so it writes nothing in the slice.

 @param reader - reader that got the all-false mask

 @param name - slice, by number where the caller has one, else the text (`textNamed`)

 @example
 ```ts
 noteRefusedMask({ reader: 'unifyQuoteStyle', name: 'slice 3', },);
 ```
 */
export function noteRefusedMask(
  {
    reader,
    name,
  }: {
    readonly reader: string;
    readonly name: string;
  },
): void {
  /**
   This function's lines, tagged with its name.
   */
  const dl = tagged({
    tag: noteRefusedMask.name,
    l,
  },);
  dl.warn(`${reader}: ${name} is refused by the page grammar, so it has no prose and no rule writes in it`,);
}

//endregion Typography mask refusal
