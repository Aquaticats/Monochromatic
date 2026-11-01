import type { Root, } from 'mdast';

import { explicitBreakCounts, } from './source-only-breaks.ts';

import {
  type DeepReadonlyData,
  parseMarkdownBody,
  requireMarkdownRefusal,
  withMarkdownExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  blockDetail,
  quotedPassageCount,
  readSliceSkeleton,
  type SkeletonRead,
} from './translate-skeleton.ts';

//region Page skeleton
// The PAGE side of the structural floor, which is a different job from the
// candidate side and needs a different grammar policy.
//
// A CANDIDATE THE STRICT GRAMMAR REFUSES IS A CANDIDATE TO REFUSE: we asked for
// it, and text we cannot parse is text we should not splice into a page.
// `readSliceSkeleton` is right to answer `unparseable` there, and
// `inspect-paragraph.ts` says the same thing for the paragraph inspector.
//
// THE PAGE IS NOT A CANDIDATE. It is archive text nobody in this pipeline
// wrote, and it can be refused for reasons no producer can act on. The measured
// case is a slice boundary falling between an opening `<details>` and its
// closing tag: the span carries no closing tag, the strict grammar demands one,
// and every rendering of that slice fails no matter how faithful. Document
// parsing already downgrades a whole page to plain markdown for exactly this,
// so the page side of the floor does too.
//
// WHY THIS IS NOT COSMETIC. Before this existed, a page the grammar refused
// became NO PAGE, the floor had no block list to compare against, and a
// candidate was measured against the source alone. On `Zha_Ke#1` of the sixth
// consolidation bed that let a 164-character rendering pass against a 3875-
// character page, deleting a person's will from a memorial while both lanes had
// carried it whole. A check that cannot run must not answer yes.

/**
 Which grammar read the page behind a verdict.

 CARRIED OUT OF THE READ rather than inferred, because a pass resting on the
 relaxed grammar is weaker evidence than one resting on the strict grammar,
 and nothing downstream could tell them apart otherwise. The repo's parser
 policy is that a grammar downgrade never happens silently.

 @example
 ```ts
 const grammar: PageGrammar = 'relaxed';
 ```
 */
export type PageGrammar = 'strict' | 'relaxed' | 'absent';

/**
 One page reading, beside the grammar that produced it.

 @example
 ```ts
 const page: PageRead = readPageSkeleton({ text, },);
 ```
 */
export type PageRead = {
  /**
   Blocks and atoms, or the refusal.
   */
  readonly read: SkeletonRead;

  /**
   Grammar that read them.
   */
  readonly grammar: PageGrammar;
};

/**
 Reading of a page that failed the strict grammar, under plain markdown.

 SEPARATE FROM {@link readSliceSkeleton} rather than a flag on it, because the
 two callers want opposite things from the same refusal and a boolean would
 hide which one a call site had chosen.

 @param text - exact page source, as the archive has it

 @returns Blocks and atoms under the looser grammar, or the refusal when even
 plain markdown will not read it

 @example
 ```ts
 const relaxed = readRelaxed({ text: pageText, },);
 ```
 */
function readRelaxed({ text, }: { readonly text: string; },): SkeletonRead {
  try {
    /**
     Page under the grammar that tolerates an unbalanced tag.
     */
    const root: DeepReadonlyData<Root> = withMarkdownExhaustionRefused({
      parse: parseMarkdownBody,
      input: { body: text, },
    },);
    return {
      kind: 'read',
      skeleton: {
        explicitBreaks: explicitBreakCounts({ root, },),
        // A quote inside an html region is hidden in that one opaque node, as
        // the atoms are, so on this reading the count is a floor rather than
        // the whole: a quote guard reading it can miss a quote lost from inside
        // one, never refuse a replacement over a quote that is not there.
        quotedPassages: quotedPassageCount({ root, },),
        blocks: root.children
          .map(function toShape(node,) {
            return {
              kind: node.type,
              detail: blockDetail({ node, },),
            };
          },),
        // ATOMS ARE LEFT EMPTY on the relaxed reading. Plain markdown collapses
        // a whole html region into one opaque node, so the references and code
        // inside it are not visible as atoms, and reporting an empty atom list
        // as if it were complete would make the atom check pass by ignorance.
        // Only the block sequence and the quote count's floor are claimed here.
        atoms: [],
      },
    };
  }
  catch (error) {
    // Plain markdown accepts any text, so it refuses a page only when the
    // text nests past the parse bound (`nesting-bound.ts`) or reading it
    // exhausts the parser's stack (ledger T8, sixth batch). That is reported
    // rather than thrown, and the floor then treats the page as it treats an
    // original no grammar reads: only the floors that read text run, and a
    // candidate none of them refuses is left unvalidated. Anything else is an
    // unexpected state that must keep propagating (ledger B100).
    return {
      kind: 'unparseable',
      detail: `plain markdown also refused: ${String(requireMarkdownRefusal({ error, },),)}`,
    };
  }
}

/**
 Reads the page a candidate would replace into its block sequence.

 Tries the strict grammar first so a well-formed page is read exactly as the
 candidate side reads it, and downgrades to plain markdown only where the
 strict grammar refuses.

 @internal

 @param text - exact page source, empty where the slice has no page

 @returns Blocks and atoms, or the refusal when neither grammar reads it

 @example
 ```ts
 const page = readPageSkeleton({ text: pageText, },);
 ```
 */
export function readPageSkeleton(
  { text, }: { readonly text: string; },
): PageRead {
  if (text === '')
    return {
      read: readSliceSkeleton({ text, },),
      grammar: 'absent',
    };

  /**
   Strict reading, which is what a page written under this grammar gets.
   */
  const strict = readSliceSkeleton({ text, },);
  if (strict.kind === 'read')
    return {
      read: strict,
      grammar: 'strict',
    };

  return {
    read: readRelaxed({ text, },),
    grammar: 'relaxed',
  };
}

//endregion Page skeleton
