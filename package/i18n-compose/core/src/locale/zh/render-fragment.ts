/**
 Chinese fragment renderer factory.
 
 @module
 */

import type {
  Adverbial,
  Fragment,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import {
  applyCapitalization,
  joinTokens,
} from '../../render-helpers.ts';
import {
  type ChineseVerbEntry,
  ZH_CASE_INVARIANTS,
} from './types.ts';

/**
 Dependency bundle for {@link makeChineseFragmentRenderer}.
 */
type FragmentDeps<L extends string, S extends string, V extends string,
  N extends string,> = {
    readonly labels: Readonly<Record<L, string>>;
    readonly verbs: Readonly<Record<V, ChineseVerbEntry>>;
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
    readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
    readonly renderAdverbials: (
      advs?: readonly Adverbial<S, N>[],
    ) => string;
  };

/**
 Capitalizes a fragment surface (Chinese invariant set is empty, so this is a passthrough except for `firstLetter` mode).
 
 @param text - rendered surface
 
 @param mode - capitalization mode
 
 @returns same surface
 */
function capitalize(
  {
    text,
    mode,
  }: {
    readonly text: string;
    readonly mode: Parameters<typeof applyCapitalization>[0]['mode'];
  },
): string {
  return applyCapitalization({
    text,
    mode,
    caseInvariants: ZH_CASE_INVARIANTS,
  },);
}

/**
 Builds a Chinese fragment renderer.
 
 @param deps - dependencies (labels, verbs, sub-renderers)
 
 @returns render function for fragments
 
 @example
 ```ts
 const renderFragment = makeChineseFragmentRenderer({ labels, verbs, renderNounPhrase, renderVerbPhrase, renderAdverbials });
 ```
 */
export function makeChineseFragmentRenderer<
  L extends string,
  S extends string,
  V extends string,
  N extends string,
  >(
  deps: FragmentDeps<L, S, V, N>,
  ): (fragment: Fragment<L, S, V, N>,) => string {
  /**
   Destructured locale dependencies captured for use across every sub-renderer below.
   */
  const {
    labels,
    verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  } = deps;

  /**
   Renders a verb-phrase fragment using the bare verb surface.
   
   @param fragment - verb-phrase fragment AST
   
   @returns rendered surface
   */
  function renderVerbPhraseFragment(
    fragment: Extract<Fragment<L, S, V, N>, { kind: 'fragment.verbPhrase'; }>,
  ): string {
    /**
     Chinese non-finite forms reuse the surface; no morphological distinction.
     */
    const head = verbs[fragment.phrase
      .verb]
      .surface;
    /**
     Rendered object surface; empty string when absent.
     */
    const object = fragment.phrase
      .object
      === undefined
      ? ''
      : renderNounPhrase(fragment.phrase
        .object,);
    /**
     Rendered complement; empty string when absent.
     */
    const complement = fragment.phrase
      .complement
      === undefined
      ? ''
      : renderVerbPhrase(fragment.phrase
        .complement
        .phrase,);
    /**
     Rendered adverbial cluster; empty string when none.
     */
    const adverbials = renderAdverbials(fragment.phrase
      .adverbials,);
    /**
     Joined surface before capitalization fixup.
     */
    const body = joinTokens([
      adverbials,
      head,
      object,
      complement,
    ],);
    return capitalize({
      text: body,
      mode: fragment.capitalization
        ?? 'preserve',
    },);
  }

  /**
   Renders a single sequence part.
   
   @param part - part AST
   
   @returns rendered surface
   */
  function renderPart(
    part: Extract<Fragment<L, S, V, N>, { kind: 'fragment.sequence'; }>['parts'][number],
  ): string {
    if (part.kind
      === 'part.label')
      return labels[part.label];
    if (part.kind
      === 'part.nounPhrase')
      return renderNounPhrase(part.phrase,);
    return part.text;
  }

  /**
   Renders a fragment AST by dispatching on `kind`.
   
   @param fragment - fragment AST
   
   @returns rendered surface
   */
  function renderFragment(fragment: Fragment<L, S, V, N>,): string {
    if (fragment.kind
      === 'fragment.nounPhrase') {
      return capitalize({
        text: renderNounPhrase(fragment.phrase,),
        mode: fragment.capitalization
          ?? 'preserve',
      },);
    }
    if (fragment.kind
      === 'fragment.verbPhrase')
      return renderVerbPhraseFragment(fragment,);
    /**
     Rendered sequence parts concatenated (no space separator in Chinese).
     */
    const joined = fragment
      .parts
      .map(function mapPart(part,): string {
        return renderPart(part,);
      },)
      .join('',);
    return capitalize({
      text: joined,
      mode: fragment.capitalization
        ?? 'preserve',
    },);
  }

  return renderFragment;
}
