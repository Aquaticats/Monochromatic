import type { JsoncComment, } from './comment.ts';
import { mergeAllComments, } from './merge-comments.ts';
import {
  isJsonWhitespace,
  scanBlockComment,
  scanLineComment,
} from './scan.ts';

//region Leading trivia

/**
 Result of skipping leading trivia: the comments crossed (in order, possibly
 empty) and the offset of the next significant character.
 */
export type TriviaScan = {
  readonly comments: readonly JsoncComment[];
  readonly end: number;
};

/**
 Skips whitespace and comments before a token, collecting every comment it
 crosses. The collected comments later attach to the following key or value.
 
 @param source - Full JSONC source.
 
 @param index - Offset to start skipping from.
 
 @returns Collected leading comments and the next significant offset.
 
 @example
 ```ts
 skipTrivia({ source: ' // a\n{', index: 0 });
 // => { comments: [{ type: 'inline', text: ' a' }], end: 6 }
 ```
 */
export function skipTrivia({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): TriviaScan {
  /**
   Comments crossed so far, pushed in source order.
   */
  const comments: JsoncComment[] = [];
  for (let cursor = index; cursor < source.length; ) {
    /**
     Character under the cursor.
     */
    const char = source[cursor];
    if (char === undefined)
      return {
        comments,
        end: cursor,
      };
    if (isJsonWhitespace(char,)) {
      cursor += 1;
      continue;
    }
    if ((char === '/') && (source[cursor + 1] === '/')) {
      /**
       Scanned line comment body and end offset.
       */
      const scan = scanLineComment({
        source,
        index: cursor,
      },);
      comments.push({
        type: 'inline',
        text: scan.text,
      },);
      cursor = scan.end;
      continue;
    }
    if ((char === '/') && (source[cursor + 1] === '*')) {
      /**
       Scanned block comment body and end offset.
       */
      const scan = scanBlockComment({
        source,
        index: cursor,
      },);
      comments.push({
        type: 'block',
        text: scan.text,
      },);
      cursor = scan.end;
      continue;
    }
    return {
      comments,
      end: cursor,
    };
  }
  return {
    comments,
    end: source.length,
  };
}

//endregion Leading trivia

//region Trailing trivia

/**
 Whitespace characters that stay on the current line. A newline ends a
 trailing-comment capture, so it is excluded here.
 */
const INLINE_WHITESPACE = new Set([
  ' ',
  '\t',
  '\r',
],);

/**
 Result of capturing trailing trivia after a value: any same-line comments,
 whether a separating comma was consumed, and the next offset.
 */
export type TrailingScan = {
  readonly comments: readonly JsoncComment[];
  readonly commaSeen: boolean;
  readonly end: number;
};

/**
 Captures what follows a value on the same line: at most one separating comma
 and any same-line comments, in either order, stopping at a newline, a closing
 bracket, or the next value. Comments captured here are the value's trailing
 comments.
 
 @param source - Full JSONC source.
 
 @param index - Offset just past the parsed value.
 
 @returns Trailing comments, whether a comma was seen, and the next offset.
 
 @example
 ```ts
 captureTrailing({ source: ', // note\n', index: 0 });
 // => { comments: [{ type: 'inline', text: ' note' }], commaSeen: true, end: 9 }
 ```
 */
export function captureTrailing({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): TrailingScan {
  /**
   Same-line comments captured so far.
   */
  const comments: JsoncComment[] = [];
  /**
   Mutable flag tracking whether the single separating comma was consumed; a
   `const` object field avoids a function-root `let`.
   */
  const flags = { commaSeen: false, };
  for (let cursor = index; cursor < source.length; ) {
    /**
     Character under the cursor.
     */
    const char = source[cursor];
    if ((char === undefined) || (char === '\n'))
      return {
        comments,
        commaSeen: flags.commaSeen,
        end: cursor,
      };
    if (INLINE_WHITESPACE.has(char,)) {
      cursor += 1;
      continue;
    }
    if ((char === ',') && (!flags.commaSeen)) {
      flags.commaSeen = true;
      cursor += 1;
      continue;
    }
    if ((char === '/') && (source[cursor + 1] === '/')) {
      /**
       Scanned line comment body and end offset.
       */
      const scan = scanLineComment({
        source,
        index: cursor,
      },);
      comments.push({
        type: 'inline',
        text: scan.text,
      },);
      cursor = scan.end;
      continue;
    }
    if ((char === '/') && (source[cursor + 1] === '*')) {
      /**
       Scanned block comment body and end offset.
       */
      const scan = scanBlockComment({
        source,
        index: cursor,
      },);
      comments.push({
        type: 'block',
        text: scan.text,
      },);
      cursor = scan.end;
      continue;
    }
    return {
      comments,
      commaSeen: flags.commaSeen,
      end: cursor,
    };
  }
  return {
    comments,
    commaSeen: flags.commaSeen,
    end: source.length,
  };
}

//endregion Trailing trivia

//region Comment attachment

/**
 Returns a copy of a comment-bearing node with `comments` merged ahead of its
 existing comment, or the node unchanged when there are no comments.
 
 @param node - Node or key carrying an optional comment.
 
 @param comments - Comments to place ahead of the node's current comment.
 
 @returns Node with the leading comments merged in.
 
 @example
 ```ts
 prependComments({ node: { kind: 'null' }, comments: [{ type: 'inline', text: 'x' }] });
 // => { kind: 'null', comment: { type: 'inline', text: 'x' } }
 ```
 */
export function prependComments<const N extends { readonly comment?: JsoncComment; },>({
  node,
  comments,
}: {
  readonly node: N;
  readonly comments: readonly JsoncComment[];
},): N {
  if (comments.length === 0)
    return node;
  /**
   Node's current comment, if it already has one.
   */
  const existing = node.comment;
  /**
   Leading comments followed by the node's existing comment.
   */
  const all = (existing === undefined)
    ? [...comments,]
    : [
      ...comments,
      existing,
    ];
  return {
    ...node,
    comment: mergeAllComments(all,),
  };
}

/**
 Returns a copy of a comment-bearing node with `comments` merged after its
 existing comment, or the node unchanged when there are no comments.
 
 @param node - Node or key carrying an optional comment.
 
 @param comments - Comments to place after the node's current comment.
 
 @returns Node with the trailing comments merged in.
 
 @example
 ```ts
 appendComments({ node: { kind: 'null' }, comments: [{ type: 'inline', text: 'x' }] });
 // => { kind: 'null', comment: { type: 'inline', text: 'x' } }
 ```
 */
export function appendComments<const N extends { readonly comment?: JsoncComment; },>({
  node,
  comments,
}: {
  readonly node: N;
  readonly comments: readonly JsoncComment[];
},): N {
  if (comments.length === 0)
    return node;
  /**
   Node's current comment, if it already has one.
   */
  const existing = node.comment;
  /**
   Node's existing comment followed by the trailing comments.
   */
  const all = (existing === undefined)
    ? [...comments,]
    : [
      existing,
      ...comments,
    ];
  return {
    ...node,
    comment: mergeAllComments(all,),
  };
}

//endregion Comment attachment
