import type { Root, } from 'mdast';

import {
  type LoneContainerTag,
  maskLoneContainerTags,
} from './mask-container-tags.ts';
import { maskHtmlComments, } from './mask-html-comments.ts';
import { maskInvisibleLines, } from './mask-invisible-lines.ts';
import {
  parseMdxBody,
  withMdxExhaustionRefused,
} from './parse-mdx.ts';
import type { DeepReadonlyData, } from './readonly-data.ts';

/**
 How the parse reads a line that holds only invisible or non-ASCII space characters, which CommonMark does not call blank.
 `blank` turns such a line into spaces first, as `parseDocument` does for the page, so the slice has the blocks the page has;
 `text` leaves it as it is, so the parser keeps it inside the paragraph as CommonMark and the site that renders the page do.
 */
export type InvisibleLinesReading = 'blank' | 'text';

/**
 Parses slice syntax with offset-preserving invisible-line, comment and lone-container masks.
 Structural admission and the model-facing break display share this grammar;
 neither treats literals inside expressions or code as prose line breaks.

 @param text - canonical slice bytes whose offsets the tree must retain

 @param invisibleLines - how a line of only invisible or non-ASCII space characters is read, `blank` as the page assembly
 reads it unless a caller reads the page as a renderer does

 @returns Positioned tree, container tags represented separately by atoms, and the text the parser read, which is the slice
 with every comment, lone container tag and (read as `blank`) invisible-only line turned into spaces of the same length

 THE ENGINE'S STACK EXHAUSTION IS READ AS THE REFUSAL HERE, AT THE PARSE CALL, and nowhere in the code that reads the
 tree this returns: a caller that reads the blocks of the result, or that walks it, is outside the parse, so an
 exhaustion it raises propagates (`withMdxExhaustionRefused`, `parse-mdx.ts`).

 @throws {@link import('./parse-mdx.ts').MdxParseError} when strict grammar cannot read the masked slice, or the parse
 exhausted the stack

 @example
 ```ts
 const { root, tags } = parseSliceBody({ text: sourceText });
 ```
 */
export function parseSliceBody(
  {
    text,
    invisibleLines = 'blank',
  }: {
    readonly text: string;
    readonly invisibleLines?: InvisibleLinesReading;
  },
): {
  readonly root: DeepReadonlyData<Root>;
  readonly tags: readonly LoneContainerTag[];
  readonly masked: string;
} {
  /**
   A line showing nothing yet not blank to CommonMark becomes spaces first, as
   `parseDocument` does, so the slice has the blocks the page has and no
   paragraph is welded to the next by a line only a reader cannot see; a caller
   that reads the page as a renderer does leaves it to the parser.
   */
  const unwelded = (invisibleLines === 'blank')
    ? maskInvisibleLines({ text, },)
      .masked
    : text;
  /**
   Comments become same-length whitespace, so no position moves.
   */
  const { masked: withoutComments, } = maskHtmlComments({ text: unwelded, },);
  /**
   A container half remains an atom without making the body unparseable.
   */
  const {
    masked,
    tags,
  } = maskLoneContainerTags({ text: withoutComments, },);
  return {
    root: withMdxExhaustionRefused({
      parse: parseMdxBody,
      input: { body: masked, },
    },),
    tags,
    masked,
  };
}
