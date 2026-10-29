/**
 Catalan wh-question sentence renderers (subject, object, adverbial).
 
 @module
 */

import {
  subjectAgreement,
  subjectSurface,
  WH_SUBJECT_AGREEMENT,
} from '../../agreement.ts';
import type { WhQuestion, } from '../../ast.ts';
import { joinTokens, } from '../../render-helpers.ts';
import { finiteVerbSurface, } from './render-vp.ts';
import {
  catalanWhWord,
  renderOptionalComplement,
  renderOptionalObject,
  type SentenceDeps,
} from './render-sentence-helpers.ts';

/**
 Renders a wh-subject question.
 
 @param sentence - wh-subject AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface fronted with `Qui`
 
 @example
 ```ts
 renderWhSubject({ sentence, deps, },);
 // -> 'Qui obre la porta?'
 ```
 */
export function renderWhSubject<S extends string, V extends string, N extends string,>(
  {
    sentence,
    deps,
  }: Readonly<{
    sentence: Extract<WhQuestion<S, V, N>, { kind: 'sentence.question.wh.subject'; }>;
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
   Sentence-level tense; defaults to present when omitted.
   */
  const tense = sentence.tense
    ?? 'present';
  /**
   Finite verb surface using wh-subject agreement (3s).
   */
  const verb = finiteVerbSurface({
    entry: verbs[sentence.predicate
      .verb],
    key: sentence.predicate
      .verb,
    tense,
    agreement: WH_SUBJECT_AGREEMENT,
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
   Sentence body with `Qui` at head.
   */
  const body = joinTokens([
    'Qui',
    verb,
    object,
    complement,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}

/**
 Renders a wh-object question.
 
 @param sentence - wh-object AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface fronted with `Què`
 
 @example
 ```ts
 renderWhObject({ sentence, deps, },);
 // -> 'Què vol el gat?'
 ```
 */
export function renderWhObject<S extends string, V extends string, N extends string,>(
  {
    sentence,
    deps,
  }: Readonly<{
    sentence: Extract<WhQuestion<S, V, N>, { kind: 'sentence.question.wh.object'; }>;
    deps: SentenceDeps<S, V, N>;
  }>,
): string {
  /**
   Locale dependencies captured for this render.
   */
  const {
    subjects,
    verbs,
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
   Subject surface after the wh-phrase.
   */
  const subj = subjectSurface({
    ref: sentence.subject,
    subjects,
  },);
  /**
   Finite verb surface.
   */
  const verb = finiteVerbSurface({
    entry: verbs[sentence.verb],
    key: sentence.verb,
    tense,
    agreement,
  },);
  /**
   Rendered adverbial cluster.
   */
  const adverbials = renderAdverbials(sentence.adverbials,);
  /**
   Sentence body with `Què` at head.
   */
  const body = joinTokens([
    'Què',
    verb,
    subj,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}

/**
 Renders a wh-adverbial question.
 
 @param sentence - wh-adverbial AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface fronted with the wh-word
 
 @example
 ```ts
 renderWhAdverbial({ sentence, deps, },);
 // -> 'On dorm el gat?'
 ```
 */
export function renderWhAdverbial<S extends string, V extends string, N extends string,>(
  {
    sentence,
    deps,
  }: Readonly<{
    sentence: Extract<WhQuestion<S, V, N>, { kind: 'sentence.question.wh.adverbial'; }>;
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
   Subject surface after the wh-phrase.
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
   Catalan wh-word matched to the AST relation.
   */
  const wh = catalanWhWord(sentence.wh,);
  /**
   Sentence body with wh-word at head.
   */
  const body = joinTokens([
    wh,
    verb,
    subj,
    object,
    complement,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}
