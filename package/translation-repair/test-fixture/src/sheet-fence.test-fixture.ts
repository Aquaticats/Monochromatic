//region Sheet fence fixture
// Reads the fence a rendered sheet actually used, so a case can require it to
// be longer than every fence-character run the enclosed text writes without
// asking `selectFence` for the expected answer (ledger B28).

/**
 A run of the fence character longer than the shortest fence, written inside
 an enclosed line so the fence must outgrow it.
 */
export const LONG_FENCE_RUN = '============';

/**
 Fence the sheet opened one labelled block with: the run of fence characters
 before the first line that starts with one and names the label.

 @param content - one rendered message

 @param label - block label, as the sheet writes it between two fences

 @returns Fence, empty when no line opens that block

 @example
 ```ts
 const fence = fenceOpening({ content: user.content, label: 'PASSAGE', },);
 ```
 */
export function fenceOpening(
  {
    content,
    label,
  }: {
    readonly content: string;
    readonly label: string;
  },
): string {
  /**
   First line opening the block, fence first.
   */
  const opening = content
    .split('\n',)
    .find(function opensBlock(line,): boolean {
      return line.startsWith('=',) && line.includes(` ${label} `,);
    },);
  return (opening === undefined) ? '' : opening.slice(
    0,
    opening.indexOf(' ',),
  );
}

//endregion Sheet fence fixture
