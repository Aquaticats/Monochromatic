/**
 English declarative, yes/no, and imperative sentence renderers.
 
 @module
 */

import {
  subjectAgreement,
  subjectSurface,
} from '../../agreement.ts';
import type { Sentence, } from '../../ast.ts';
import { joinTokens, } from '../../render-helpers.ts';
import {
  complementFormForVerb,
  declarativeVerbSurface,
  questionVerbParts,
} from './render-vp.ts';
import {
  capitalizeBody,
  renderOptionalComplement,
  renderOptionalObject,
  type SentenceDeps,
} from './render-sentence-helpers.ts';

/**
 Renders a declarative sentence.
 
 @param sentence - declarative AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface with sentence-case fixup and terminator
 
 @example
 ```ts
 renderDeclarative({ sentence, deps, },);
 // -> 'The cat sleeps.'
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
   Agreement metadata extracted from the subject reference.
   */
  const agreement = subjectAgreement({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Subject surface used in the leading position.
   */
  const subj = subjectSurface({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Verb entry used for finite surface and complement attachment.
   */
  const entry = verbs[sentence.predicate
    .verb];
  /**
   Finite verb surface for this subject + tense.
   */
  const finite = declarativeVerbSurface({
    entry,
    tense,
    agreement,
  },);
  /**
   Future tense wraps the base in `will`.
   */
  const verb = tense === 'future' ? `will ${finite}` : finite;
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
    form: complementFormForVerb({ entry, },),
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
  return `${capitalizeBody(body,)}${sentence.terminator
    ?? '.'}`;
}

/**
 Renders a yes/no question with do-support.
 
 @param sentence - yes/no AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface with auxiliary, sentence-case fixup, and terminator
 
 @example
 ```ts
 renderYesNo({ sentence, deps, },);
 // -> 'Does the cat sleep?'
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
   Question verb parts chosen from the entry's auxiliary strategy.
   */
  const questionVerb = questionVerbParts({
    entry: verbs[sentence.predicate
      .verb],
    tense,
    agreement,
  },);
  /**
   Subject surface placed after the auxiliary.
   */
  const subj = subjectSurface({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Rendered object slot.
   */
  const object = renderOptionalObject({
    predicate: sentence.predicate,
    renderNounPhrase,
  },);
  /**
   Rendered infinitive or bare complement.
   */
  const complement = renderOptionalComplement({
    predicate: sentence.predicate,
    renderVerbPhrase,
    form: questionVerb.complementForm,
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
    questionVerb.auxiliary,
    subj,
    questionVerb.lexicalVerb,
    object,
    complement,
    adverbials,
  ],);
  return `${capitalizeBody(body,)}${sentence.terminator
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
 // -> 'Open the door.'
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
   Imperative surface, defaulting to `base` when no dedicated form is supplied.
   */
  const entry = verbs[sentence.predicate
    .verb];
  /**
   Verb surface used in the imperative head slot.
   */
  const verb = entry.imperative
    ?? entry
      .base;
  /**
   Rendered object slot.
   */
  const object = renderOptionalObject({
    predicate: sentence.predicate,
    renderNounPhrase,
  },);
  /**
   Rendered infinitive or bare complement.
   */
  const complement = renderOptionalComplement({
    predicate: sentence.predicate,
    renderVerbPhrase,
    form: complementFormForVerb({ entry, },),
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
