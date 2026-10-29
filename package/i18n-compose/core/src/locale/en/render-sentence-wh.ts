/**
 English wh-question sentence renderers (subject, object, adverbial).
 
 @module
 */

import {
  subjectAgreement,
  subjectSurface,
  WH_SUBJECT_AGREEMENT,
} from '../../agreement.ts';
import type { WhQuestion, } from '../../ast.ts';
import { joinTokens, } from '../../render-helpers.ts';
import {
  complementFormForVerb,
  questionVerbParts,
  subjectQuestionVerbSurface,
} from './render-vp.ts';
import {
  capitalizeWh,
  renderOptionalComplement,
  renderOptionalObject,
  type SentenceDeps,
} from './render-sentence-helpers.ts';

/**
 Renders a wh-subject question.
 
 @param sentence - wh-subject AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface fronted with `Who`
 
 @example
 ```ts
 renderWhSubject({ sentence, deps, },);
 // -> 'Who opened the door?'
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
   Verb entry used for the finite question head.
   */
  const entry = verbs[sentence.predicate
    .verb];
  /**
   Wh-subject treated as third-person singular for ordinary finite agreement.
   */
  const verb = subjectQuestionVerbSurface({
    entry,
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
    form: complementFormForVerb({ entry, },),
  },);
  /**
   Rendered adverbial cluster.
   */
  const adverbials = renderAdverbials(sentence.predicate
    .adverbials,);
  /**
   Sentence body with `Who` at head.
   */
  const body = joinTokens([
    'Who',
    verb,
    object,
    complement,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}

/**
 Renders a wh-object question with do-support.
 
 @param sentence - wh-object AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface fronted with `What`
 
 @example
 ```ts
 renderWhObject({ sentence, deps, },);
 // -> 'What does the cat want?'
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
   Question verb parts chosen from the entry's auxiliary strategy.
   */
  const questionVerb = questionVerbParts({
    entry: verbs[sentence.verb],
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
   Rendered adverbial cluster.
   */
  const adverbials = renderAdverbials(sentence.adverbials,);
  /**
   Sentence body with `What` at head.
   */
  const body = joinTokens([
    'What',
    questionVerb.auxiliary,
    subj,
    questionVerb.lexicalVerb,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}

/**
 Renders a wh-adverbial question (`Where/When/Why/How`).
 
 @param sentence - wh-adverbial AST
 
 @param deps - per-locale dependencies
 
 @returns rendered surface fronted with the wh-word
 
 @example
 ```ts
 renderWhAdverbial({ sentence, deps, },);
 // -> 'Where does the cat sleep?'
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
   Rendered infinitive complement.
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
   Wh-word capitalized in the head position.
   */
  const wh = capitalizeWh(sentence.wh,);
  /**
   Sentence body with wh-word at head.
   */
  const body = joinTokens([
    wh,
    questionVerb.auxiliary,
    subj,
    questionVerb.lexicalVerb,
    object,
    complement,
    adverbials,
  ],);
  return `${body}${sentence.terminator
    ?? '?'}`;
}
