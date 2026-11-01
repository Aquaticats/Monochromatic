/**
 Shared dependency bundle and slot helpers for the Catalan sentence renderers.
 
 @module
 */

import type {
  Adverbial,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import type { SubjectEntry, } from '../../entries.ts';
import { applyCapitalization, } from '../../render-helpers.ts';
import {
  CA_CASE_INVARIANTS,
  type CatalanVerbEntry,
} from './types.ts';

/**
 Dependency bundle for the Catalan sentence sub-renderers.
 */
export type SentenceDeps<S extends string, V extends string, N extends string,> = {
  readonly subjects: Readonly<Record<S, SubjectEntry>>;
  readonly verbs: Readonly<Record<V, CatalanVerbEntry>>;
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
 // -> 'la porta' when present, '' when the object slot is absent
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
 Renders a predicate's optional infinitive complement, returning empty string when absent.
 
 Reads `predicate.complement` itself so the absent case never crosses the
 call boundary as `undefined`; {@link joinTokens} drops the empty-string result.
 
 @param predicate - verb phrase whose complement slot is rendered
 
 @param renderVerbPhrase - verb-phrase render function
 
 @returns rendered complement phrase, or empty string when absent
 
 @example
 ```ts
 renderOptionalComplement({ predicate: sentence.predicate, renderVerbPhrase, },);
 // -> 'marxar' when present, '' when the complement slot is absent
 ```
 */
export function renderOptionalComplement<S extends string, V extends string, N extends string,>(
  {
    predicate,
    renderVerbPhrase,
  }: {
    readonly predicate: VerbPhrase<S, V, N>;
    readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
  },
): string {
  if (predicate.complement
    === undefined)
    return '';
  return renderVerbPhrase(predicate.complement
    .phrase,);
}

/**
 Capitalizes a sentence body using Catalan case-invariants.
 
 @param body - rendered body before sentence-case fixup
 
 @returns same body with the first character uppercased
 
 @example
 ```ts
 capitalizeBody('el gat dorm',);
 // -> 'El gat dorm'
 ```
 */
export function capitalizeBody(body: string,): string {
  return applyCapitalization({
    text: body,
    mode: 'firstLetter',
    caseInvariants: CA_CASE_INVARIANTS,
  },);
}

/**
 Maps an English wh-word identifier to the Catalan surface.
 
 @param wh - wh-word from the AST
 
 @returns Catalan wh-surface
 
 @example
 ```ts
 catalanWhWord('where',);
 // -> 'On'
 ```
 */
export function catalanWhWord(wh: 'where' | 'when' | 'why' | 'how',): string {
  if (wh === 'where')
    return 'On';
  if (wh === 'when')
    return 'Quan';
  if (wh === 'why')
    return 'Per què';
  return 'Com';
}
