/**
 The end of what one or more streams printed, kept as it arrives. A command that fails says why at the end of what it printed, and
 a batch of thirty runs would otherwise hold every byte each of them ever wrote.

 @module
 */

/**
 Keeps the end of what arrived, and answers with it when the command is over.
 */
export type TailKeeper = {
  /**
   Keeps what one stream printed, dropping what no longer fits.
   */
  readonly keep: (chunk: Buffer) => void;

  /**
   What was kept.
   */
  readonly text: () => string;
};

/**
 Builds a keeper of the end of what a command printed.

 @param characters - how many characters of the end to keep

 @returns the keeper

 @example
 ```ts
 const kept = tailKeeper({ characters: 400, });
 kept.keep(Buffer.from('a reason'));
 ```
 */
export function tailKeeper({ characters, }: { readonly characters: number },): TailKeeper {
  /**
   What arrived, of which only the end is kept.
   */
  const state: { tail: string } = { tail: '', };
  return {
    keep: function keep(chunk,) {
      state.tail = `${state.tail}${chunk.toString()}`.slice(-characters,);
    },
    text: function text() {
      return state.tail;
    },
  };
}
