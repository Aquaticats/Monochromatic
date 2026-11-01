//region Quote line
// How a line reads past its Markdown quote markers, shared by the line
// structure guard (`line-structure-guard.ts`) and the bilingual pair bound
// (`bilingual-pair-bound.ts`). Each kept its own copy of both tests
// (audit area six, 2026-09-28), and two guards that disagree about what a
// line is would count one passage two ways.

/**
 Whether a line carries text, as against a blank line or a bare quote marker.

 A LINE OF NOTHING BUT `>` SEPARATES QUOTED BLOCKS the way a blank line
 separates plain ones (class forty-seven, shi_Yumiaoya2, 2026-09-17: the
 original wrote three of them inside one farewell quote and the archive one,
 and the count read that as two merged lines).

 @param line - one raw line

 @returns True on the first character that is neither a quote marker nor whitespace

 @example
 ```ts
 carriesContent({ line: '> 猫醒了。', },); // true
 carriesContent({ line: '>', },); // false
 ```
 */
export function carriesContent({ line, }: { readonly line: string; },): boolean {
  for (const character of line) {
    if ((character !== '>') && (character.trim() !== ''))
      return true;
  }
  return false;
}

/**
 Line past its leading quote markers and whitespace.

 @param line - one raw line

 @returns Rest of the line from its first content character, empty when
 there is none

 @example
 ```ts
 pastQuoteMarkers({ line: '> > cat', },); // 'cat'
 ```
 */
export function pastQuoteMarkers({ line, }: { readonly line: string; },): string {
  for (let at = 0; at < line.length; at += 1) {
    /**
     Character at the cursor.
     */
    const character = line.charAt(at,);
    if ((character !== '>') && (character.trim() !== ''))
      return line.slice(at,);
  }
  return '';
}

//endregion Quote line
