/**
 English fragment renderer factory.
 
 @module
 */

import type {
  Adverbial,
  Fragment,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import type { VerbFragmentForm, } from '../../grammar-primitives.ts';
import {
  applyCapitalization,
  joinTokens,
} from '../../render-helpers.ts';
import { englishGerund, } from './morphology.ts';
import { complementFormForVerb, } from './render-vp.ts';
import {
  EN_CASE_INVARIANTS,
  type EnglishVerbEntry,
} from './types.ts';

/**
 Dependency bundle for {@link makeEnglishFragmentRenderer}.
 */
type FragmentDeps<L extends string, S extends string, V extends string,
  N extends string,> = {
    readonly labels: Readonly<Record<L, string>>;
    readonly verbs: Readonly<Record<V, EnglishVerbEntry>>;
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
    readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
    readonly renderAdverbials: (
      advs?: readonly Adverbial<S, N>[],
    ) => string;
  };

/**
 Picks the English non-finite verb surface for a verb-phrase fragment.
 
 @param entry - English verb entry
 
 @param form - requested non-finite form
 
 @returns non-finite surface
 */
function nonFiniteSurface(
  {
    entry,
    form,
  }: {
    readonly entry: EnglishVerbEntry;
    readonly form: VerbFragmentForm;
  },
): string {
  if (form === 'imperative')
    return entry.imperative
      ?? entry
        .base;
  if (form === 'infinitive')
    return `to ${entry.base}`;
  return entry.gerund
    ?? englishGerund({ base: entry.base, },);
}

/**
 Capitalizes a fragment surface using English case-invariants.
 
 @param text - rendered surface
 
 @param mode - capitalization mode
 
 @returns capitalized surface
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
    caseInvariants: EN_CASE_INVARIANTS,
  },);
}

/**
 Renders a nested fragment complement using the head verb's attachment strategy.
 
 @param entry - head verb entry controlling bare vs infinitive attachment
 
 @param phrase - nested verb phrase to render
 
 @param renderVerbPhrase - verb-phrase renderer
 
 @returns rendered complement surface
 */
function renderComplement<S extends string, V extends string, N extends string,>(
  {
    entry,
    phrase,
    renderVerbPhrase,
  }: {
    readonly entry: EnglishVerbEntry;
    readonly phrase: VerbPhrase<S, V, N>;
    readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
  },
): string {
  /**
   Rendered nested verb phrase before complement marker selection.
   */
  const rendered = renderVerbPhrase(phrase,);
  return complementFormForVerb({ entry, },)
    === 'bare' ? rendered : `to ${rendered}`;
}

/**
 Builds an English fragment renderer.
 
 @param deps - per-locale dependencies (labels, verbs, sub-renderers)
 
 @returns render function for fragments
 
 @example
 ```ts
 const renderFragment = makeEnglishFragmentRenderer({ labels, verbs, renderNounPhrase, renderVerbPhrase, renderAdverbials });
 ```
 */
export function makeEnglishFragmentRenderer<
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
   Renders a verb-phrase fragment using the requested non-finite form.
   
   @param fragment - verb-phrase fragment AST
   
   @returns rendered surface
   */
  function renderVerbPhraseFragment(
    fragment: Extract<Fragment<L, S, V, N>, { kind: 'fragment.verbPhrase'; }>,
  ): string {
    /**
     Verb entry used by every non-finite branch.
     */
    const entry = verbs[fragment.phrase
      .verb];
    /**
     Non-finite surface for the requested form.
     */
    const head = nonFiniteSurface({
      entry,
      form: fragment.form,
    },);
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
     Rendered infinitive or bare complement; empty string when absent.
     */
    const complement = fragment.phrase
      .complement
      === undefined
      ? ''
      : renderComplement({
        entry,
        phrase: fragment.phrase
          .complement
          .phrase,
        renderVerbPhrase,
      },);
    /**
     Rendered adverbial cluster; empty string when none.
     */
    const adverbials = renderAdverbials(fragment.phrase
      .adverbials,);
    /**
     Joined surface before capitalization fixup.
     */
    const body = joinTokens([
      head,
      object,
      complement,
      adverbials,
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
     Rendered sequence parts space-joined.
     */
    const joined = fragment
      .parts
      .map(function mapPart(part,): string {
        return renderPart(part,);
      },)
      .join(' ',);
    return capitalize({
      text: joined,
      mode: fragment.capitalization
        ?? 'preserve',
    },);
  }

  return renderFragment;
}
