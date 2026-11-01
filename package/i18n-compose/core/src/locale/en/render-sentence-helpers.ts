/**
 Shared dependency bundle and slot helpers for the English sentence renderers.
 
 @module
 */

import type {
  Adverbial,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import type { SubjectEntry, } from '../../entries.ts';
import { applyCapitalization, } from '../../render-helpers.ts';
import type { EnglishComplementForm, } from './render-vp.ts';
import {
  EN_CASE_INVARIANTS,
  type EnglishVerbEntry,
} from './types.ts';

/**
 Dependency bundle for the English sentence sub-renderers.
 */
export type SentenceDeps<S extends string, V extends string, N extends string,> = {
  readonly subjects: Readonly<Record<S, SubjectEntry>>;
  readonly verbs: Readonly<Record<V, EnglishVerbEntry>>;
  readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
  readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
  readonly renderAdverbials: (
    advs?: readonly Adverbial<S, N>[],
  ) => string;
};

/**
 Renders a predicate's optional object slot, returning empty string when absent.
 
 Reads `predicate.object` itself so the absent case never crosses the call
 boundary as `undefined`; {@link joinTokens} drops the empty-string result.
 
 @param predicate - verb phrase whose object slot is rendered
 
 @param renderNounPhrase - noun-phrase render function
 
 @returns rendered surface, or empty string when the object slot is absent
 
 @example
 ```ts
 renderOptionalObject({ predicate: sentence.predicate, renderNounPhrase, },);
 // -> 'the door' when present, '' when the object slot is absent
 ```
 */
export function renderOptionalObject<S extends string, V extends string, N extends string,>(
  {
    predicate,
    renderNounPhrase,
  }: {
    readonly predicate: VerbPhrase<S, V, N>;
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
  },
): string {
  if (predicate.object
    === undefined)
    return '';
  return renderNounPhrase(predicate.object,);
}

/**
 Renders a predicate's optional infinitive complement (`to + ...`) returning empty string when absent.
 
 Reads `predicate.complement` itself so the absent case never crosses the
 call boundary as `undefined`; {@link joinTokens} drops the empty-string result.
 
 @param predicate - verb phrase whose complement slot is rendered
 
 @param renderVerbPhrase - verb-phrase render function
 
 @param form - complement attachment mode for the predicate head
 
 @returns rendered surface with `to` prefix, bare surface, or empty string when absent
 
 @example
 ```ts
 renderOptionalComplement({ predicate: sentence.predicate, renderVerbPhrase, form: 'infinitive', },);
 // -> 'to leave' for the infinitive form, 'leave' for the bare form
 ```
 */
export function renderOptionalComplement<S extends string, V extends string, N extends string,>(
  {
    predicate,
    renderVerbPhrase,
    form,
  }: {
    readonly predicate: VerbPhrase<S, V, N>;
    readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
    readonly form: EnglishComplementForm;
  },
): string {
  if (predicate.complement
    === undefined)
    return '';
  /**
   Rendered nested verb phrase before complement marker selection.
   */
  const rendered = renderVerbPhrase(predicate.complement
    .phrase,);
  return form === 'bare' ? rendered : `to ${rendered}`;
}

/**
 Capitalizes a sentence body using English case-invariants.
 
 @param body - rendered body before sentence-case fixup
 
 @returns same body with the first character uppercased unless it is in the invariant set
 
 @example
 ```ts
 capitalizeBody('the cat sleeps',);
 // -> 'The cat sleeps'
 ```
 */
export function capitalizeBody(body: string,): string {
  return applyCapitalization({
    text: body,
    mode: 'firstLetter',
    caseInvariants: EN_CASE_INVARIANTS,
  },);
}

/**
 Capitalizes a wh-word for the head position.
 
 @param wh - wh-word lowercase form
 
 @returns wh-word with the first character uppercased
 
 @example
 ```ts
 capitalizeWh('where',);
 // -> 'Where'
 ```
 */
export function capitalizeWh(wh: string,): string {
  return wh.charAt(0,)
    .toUpperCase()
    + wh
      .slice(1,);
}
