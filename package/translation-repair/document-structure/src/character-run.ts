//region Character run
// The longest run of one character in a text, which both fence builders need:
// the Markdown fence (`markdown-fence.ts`, backticks) and the prompt fence
// (`prompt-fence.ts`, equals signs). Each kept its own copy of this pass
// (audit area six, 2026-09-28); a fence one character shorter than the
// longest run inside it closes early, so the two must count alike.

/**
 Longest unbroken run of one character anywhere in a text.

 Single linear pass, because the input is unbounded corpus prose. Read by
 code point, so a character outside the Basic Multilingual Plane counts once.

 @param text - content that will be fenced

 @param character - one code point to count runs of

 @returns Longest run length, zero when the character never appears

 @example
 ```ts
 longestRunOf({ text: 'a === b', character: '=', },); // 3
 ```
 */
export function longestRunOf(
  {
    text,
    character,
  }: {
    readonly text: string;
    readonly character: string;
  },
): number {
  /**
   Best and running run lengths across the pass.
   */
  const counters = {
    best: 0,
    current: 0,
  };
  for (const read of text) {
    if (read !== character) {
      counters.current = 0;
      continue;
    }
    counters.current += 1;
    counters.best = Math.max(
      counters.best,
      counters.current,
    );
  }
  return counters.best;
}

//endregion Character run
