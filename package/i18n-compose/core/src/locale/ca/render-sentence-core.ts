/**
 Catalan declarative, yes/no, and imperative sentence renderers.
 
 @module
 */

import {
  subjectAgreement,
  subjectSurface,
} from '../../agreement.ts';
import type { Sentence, } from '../../ast.ts';
import { joinTokens, } from '../../render-helpers.ts';
import { finiteVerbSurface, } from './render-vp.ts';
import {
  capitalizeBody,
  renderOptionalComplement,
  renderOptionalObject,
  type SentenceDeps,
} from './render-sentence-helpers.ts';

/**
 Renders a declarative sentence with sentence-case fixup.
 
 @param sentence - declarative AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface
 
 @example
 ```ts
 renderDeclarative({ sentence, deps, },);
 // -> 'El gat dorm.'
 ```
 */
export function renderDeclarative<S extends string, V extends string, N extends string,>(
  {
    sentence,
    deps,
  }: Readonly<{
    sentence: Extract<Sentence<S, V, N>, { kind: 'sentence.declarative'; }>;
    deps: SentenceDeps<S, V, N>;
  }>,
): string {
  /**
   Locale dependencies captured for this render.
   */
  const {
    subjects,
    verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  } = deps;
  /**
   Sentence-level tense; defaults to present when omitted.
   */
  const tense = sentence.tense
    ?? 'present';
  /**
   Agreement metadata.
   */
  const agreement = subjectAgreement({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Subject surface.
   */
  const subj = subjectSurface({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Finite verb surface.
   */
  const verb = finiteVerbSurface({
    entry: verbs[sentence.predicate
      .verb],
    key: sentence.predicate
      .verb,
    tense,
    agreement,
  },);
  /**
   Rendered object slot.
   */
  const object = renderOptionalObject({
    predicate: sentence.predicate,
    renderNounPhrase,
  },);
  /**
   Rendered infinitive complement.
   */
  const complement = renderOptionalComplement({
    predicate: sentence.predicate,
    renderVerbPhrase,
  },);
  /**
   Rendered adverbial cluster.
   */
  const adverbials = renderAdverbials(sentence.predicate
    .adverbials,);
  /**
   Sentence body before sentence-case fixup.
   */
  const body = joinTokens([
    subj,
    verb,
    object,
    complement,
    adverbials,
  ],);
  return `${capitalizeBody(body,)}${sentence.terminator
    ?? '.'}`;
}

/**
 Renders a yes/no question (Catalan v1 uses punctuation alone, no inversion).
 
 @param sentence - yes/no AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface
 
 @example
 ```ts
 renderYesNo({ sentence, deps, },);
 // -> 'el gat dorm?'
 ```
 */
export function renderYesNo<S extends string, V extends string, N extends string,>(
  {
    sentence,
    deps,
  }: Readonly<{
    sentence: Extract<Sentence<S, V, N>, { kind: 'sentence.question.yesNo'; }>;
    deps: SentenceDeps<S, V, N>;
  }>,
): string {
  /**
   Locale dependencies captured for this render.
   */
  const {
    subjects,
    verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  } = deps;
  /**
   Sentence-level tense; defaults to present when omitted.
   */
  const tense = sentence.tense
    ?? 'present';
  /**
   Agreement metadata.
   */
  const agreement = subjectAgreement({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Subject surface.
   */
  const subj = subjectSurface({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Finite verb surface.
   */
  const verb = finiteVerbSurface({
    entry: verbs[sentence.predicate
      .verb],
    key: sentence.predicate
      .verb,
    tense,
    agreement,
  },);
  /**
   Rendered object slot.
   */
  const object = renderOptionalObject({
    predicate: sentence.predicate,
    renderNounPhrase,
  },);
  /**
   Rendered infinitive complement.
   */
  const complement = renderOptionalComplement({
    predicate: sentence.predicate,
    renderVerbPhrase,
  },);
  /**
   Rendered adverbial cluster.
   */
  const adverbials = renderAdverbials(sentence.predicate
    .adverbials,);
  /**
   Sentence body before terminator.
   */
  const body = joinTokens([
    subj,
    verb,
    object,
    complement,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}

/**
 Renders an imperative sentence with sentence-case fixup.
 
 @param sentence - imperative AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface
 
 @example
 ```ts
 renderImperative({ sentence, deps, },);
 // -> 'Obre la porta.'
 ```
 */
export function renderImperative<S extends string, V extends string, N extends string,>(
  {
    sentence,
    deps,
  }: Readonly<{
    sentence: Extract<Sentence<S, V, N>, { kind: 'sentence.imperative'; }>;
    deps: SentenceDeps<S, V, N>;
  }>,
): string {
  /**
   Locale dependencies captured for this render.
   */
  const {
    verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  } = deps;
  /**
   Imperative surface; falls back to the infinitive when no dedicated form is supplied.
   */
  const verb = verbs[sentence.predicate
    .verb]
    .imperative
    ?? verbs[sentence.predicate
      .verb]
      .infinitive;
  /**
   Rendered object slot.
   */
  const object = renderOptionalObject({
    predicate: sentence.predicate,
    renderNounPhrase,
  },);
  /**
   Rendered infinitive complement.
   */
  const complement = renderOptionalComplement({
    predicate: sentence.predicate,
    renderVerbPhrase,
  },);
  /**
   Rendered adverbial cluster.
   */
  const adverbials = renderAdverbials(sentence.predicate
    .adverbials,);
  /**
   Sentence body before sentence-case fixup.
   */
  const body = joinTokens([
    verb,
    object,
    complement,
    adverbials,
  ],);
  return `${capitalizeBody(body,)}${sentence.terminator
    ?? '.'}`;
}
