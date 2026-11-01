/**
 English sentence renderer factory: declarative, yes/no, wh, imperative.
 
 @module
 */

import type {
  Sentence,
  WhQuestion,
} from '../../ast.ts';
import {
  renderDeclarative,
  renderImperative,
  renderYesNo,
} from './render-sentence-core.ts';
import type { SentenceDeps, } from './render-sentence-helpers.ts';
import {
  renderWhAdverbial,
  renderWhObject,
  renderWhSubject,
} from './render-sentence-wh.ts';

/**
 Builds an English sentence renderer.
 
 @param deps - per-locale dependencies (subjects, verbs, sub-renderers)
 
 @returns render function for sentences
 
 @example
 ```ts
 const renderSentence = makeEnglishSentenceRenderer({ subjects, verbs, renderNounPhrase, renderVerbPhrase, renderAdverbials });
 ```
 */
export function makeEnglishSentenceRenderer<
  S extends string,
  V extends string,
  N extends string,
  >(
  deps: SentenceDeps<S, V, N>,
  ): (sentence: Sentence<S, V, N>,) => string {
  /**
   Dispatches a wh-question by slot.
   
   @param sentence - wh-question AST
   
   @returns rendered surface
   */
  function renderWh(sentence: WhQuestion<S, V, N>,): string {
    if (sentence.kind
      === 'sentence.question.wh.subject')
      return renderWhSubject({
        sentence,
        deps,
      },);
    if (sentence.kind
      === 'sentence.question.wh.object')
      return renderWhObject({
        sentence,
        deps,
      },);
    return renderWhAdverbial({
      sentence,
      deps,
    },);
  }

  /**
   Dispatches a sentence AST to the correct sub-renderer.
   
   @param sentence - sentence AST
   
   @returns rendered surface
   */
  function renderSentence(sentence: Sentence<S, V, N>,): string {
    if (sentence.kind
      === 'sentence.declarative')
      return renderDeclarative({
        sentence,
        deps,
      },);
    if (sentence.kind
      === 'sentence.question.yesNo')
      return renderYesNo({
        sentence,
        deps,
      },);
    if (sentence.kind
      === 'sentence.imperative')
      return renderImperative({
        sentence,
        deps,
      },);
    return renderWh(sentence,);
  }

  return renderSentence;
}
