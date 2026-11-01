//region Span rewrites
// One way to apply a set of span rewrites to a text, shared by the page passes
// that rewrite words in place: the name casing (`archive-name-casing.ts`), the
// casing restore (`archive-casing-restore.ts`), the Canadian forms
// (`canadian-forms.ts`), the pinyin tones (`pinyin-tone.ts`) and the tag
// attribute restore (`jsx-attribute-restore.ts`).
//
// EACH KEPT ITS OWN APPLIER, UNDER THREE CONTRACTS (audit area six,
// 2026-09-28). The name casing sorted its rewrites and dropped any that
// overlapped an earlier one; the Canadian forms sorted and trusted a comment
// that dates and spellings never overlap; the pinyin tones applied rewrites in
// the order the parentheses were read, sorted or not; the other two relied on
// their callers' order. An applier handed an unsorted or overlapping set
// splices a slice before its cursor, which repeats text on the page. Now every
// pass orders its rewrites the same way and withholds the same ones.

/**
 One span of a text and the wording that replaces it.

 @example
 ```ts
 const rewrite: SpanRewrite = { start: 3, end: 8, to: 'tabby', };
 ```
 */
export type SpanRewrite = {
  /**
   Offset of the span's first character.
   */
  readonly start: number;

  /**
   Offset just past the span.
   */
  readonly end: number;

  /**
   Wording that replaces the span.
   */
  readonly to: string;
};

/**
 Applies span rewrites to a text in offset order.

 ORDERED BY START, THE LONGER FIRST WHERE TWO START TOGETHER, and a rewrite
 that overlaps one kept before it is withheld: two rewrites cannot both hold
 one span, and the longer is the more specific reading of it.

 @param text - text the offsets address

 @param rewrites - rewrites in any order

 @returns Rewritten text, and the rewrites applied in offset order

 @example
 ```ts
 applySpanRewrites({ text: 'the cat', rewrites: [{ start: 4, end: 7, to: 'kit', },], },).text; // 'the kit'
 ```
 */
export function applySpanRewrites<const RewriteT extends SpanRewrite,>(
  {
    text,
    rewrites,
  }: {
    readonly text: string;
    readonly rewrites: readonly RewriteT[];
  },
): {
  readonly text: string;
  readonly applied: readonly RewriteT[];
} {
  /**
   Rewrites kept, in offset order, none overlapping an earlier one.
   */
  const applied: RewriteT[] = [];
  for (
    const rewrite of rewrites.toSorted(function byStart(
      left,
      right,
    ): number {
      return (left.start - right.start) || (right.end - left.end);
    },)
  ) {
    /**
     Last rewrite kept.
     */
    const previous = applied.at(-1,);
    if ((previous === undefined) || (rewrite.start >= previous.end))
      applied.push(rewrite,);
  }

  /**
   Text before each kept rewrite, then the rewrite's wording.
   */
  const parts = applied.flatMap(function around(
    rewrite,
    index,
  ): readonly string[] {
    return [
      text.slice(
        applied[index - 1]
          ?.end
          ?? 0,
        rewrite.start,
      ),
      rewrite.to,
    ];
  },);
  return {
    text: [
      ...parts,
      text.slice(applied.at(-1,)
        ?.end
        ?? 0,),
    ].join('',),
    applied,
  };
}

//endregion Span rewrites
