import type { JsoncComment, } from './comment.ts';

/**
 Merges two comments into one, the way stacked comments against a single node
 collapse during parsing. Bodies join with a newline and stay untrimmed so
 `//region` and indentation survive; the type is the shared type, or `mixed`
 when the two differ.
 
 @param first - Earlier comment.
 
 @param second - Later comment.
 
 @returns Combined comment.
 
 @example
 ```ts
 mergeComments({
   first: { type: 'block', text: 'region' },
   second: { type: 'inline', text: ' note' },
 });
 // => { type: 'mixed', text: 'region\n note' }
 ```
 */
export function mergeComments({
  first,
  second,
}: {
  readonly first: JsoncComment;
  readonly second: JsoncComment;
},): JsoncComment {
  /**
   Joined body keeping a newline between the two original comment bodies.
   */
  const text = `${first.text}\n${second.text}`;
  /**
   Shared type when both comments agree, otherwise `mixed`.
   */
  const type = (first.type === second.type)
    ? first.type
    : 'mixed';
  return {
    type,
    text,
  };
}

/**
 Reduces a non-empty list of comments into a single merged comment, in order.
 The reduce callback is a named function expression whose positional shape is
 supplied by `Array.reduce`.
 
 @param comments - Comments to merge; must contain at least one.
 
 @returns Single merged comment.
 
 @example
 ```ts
 mergeAllComments([
   { type: 'inline', text: 'a' },
   { type: 'inline', text: 'b' },
 ]);
 // => { type: 'inline', text: 'a\nb' }
 ```
 */
export function mergeAllComments(comments: readonly JsoncComment[],): JsoncComment {
  return comments.reduce(
    function mergeStep(
      accumulated: JsoncComment,
      next: JsoncComment,
    ): JsoncComment {
      return mergeComments({
        first: accumulated,
        second: next,
      },);
    },
  );
}
