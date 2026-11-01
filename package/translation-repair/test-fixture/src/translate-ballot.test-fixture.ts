//region Translate ballot
// HOW A SCRIPTED JUDGE FINDS THE CANDIDATE IT MEANS TO BACK, read off the
// sheet the way a judge reads it, since the stage rotates the slate per slice
// and a script assuming an order backs whatever lands there.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Unit test files that kept their own copy
// of this reading, as `pickCandidate` or as a wrapper with its needle fixed,
// now import this one. Readings that only look alike stay in their files:
// one matches any of several needles, and two walk a serialized request body,
// where a line break is the two characters `\n` and splitting on a newline
// finds no heading.

/**
 Finds the one-based candidate index whose rendered text carries a needle.

 The judge sheet numbers candidates under a `CANDIDATE n` heading and fences
 their text, so this splits at the headings and reads each number off its
 heading line.

 @param content - judge sheet as sent

 @param needle - text the wanted candidate contains

 @returns One-based index, or zero when no candidate carries it, which is the
 ballot value for declining every candidate

 @example
 ```ts
 const best = candidateCarrying({ content, needle: 'dozes', },);
 ```
 */
export function candidateCarrying(
  {
    content,
    needle,
  }: {
    readonly content: string;
    readonly needle: string;
  },
): number {
  /**
   Sheet split at each candidate heading; the first piece is the evidence.
   */
  const [, ...blocks] = content.split('CANDIDATE ',);
  for (const block of blocks) {
    /**
     Heading line carrying this candidate's number.
     */
    const [heading = '',] = block.split('\n',);

    /**
     Number the heading states.
     */
    const index = Math.trunc(Number(heading,),);
    if (Number.isInteger(index,) && block.includes(needle,))
      return index;
  }
  return 0;
}

//endregion Translate ballot
