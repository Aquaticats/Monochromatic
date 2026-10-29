/**
 Chinese sentence renderer factory: declarative, yes/no, wh, imperative.
 
 @module
 */

import { subjectSurface, } from '../../agreement.ts';
import type {
  Adverbial,
  NounPhrase,
  Sentence,
  VerbPhrase,
  WhQuestion,
} from '../../ast.ts';
import type { SubjectEntry, } from '../../entries.ts';
import { joinTokens, } from '../../render-helpers.ts';
import { verbSurfaceForTense, } from './render-vp.ts';
import type { ChineseVerbEntry, } from './types.ts';

/**
 Dependency bundle for {@link makeChineseSentenceRenderer}.
 */
type SentenceDeps<S extends string, V extends string, N extends string,> = {
  readonly subjects: Readonly<Record<S, SubjectEntry>>;
  readonly verbs: Readonly<Record<V, ChineseVerbEntry>>;
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
 */
function renderOptionalObject<S extends string, V extends string, N extends string,>(
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
 Renders a predicate's optional complement, returning empty string when absent.
 
 Reads `predicate.complement` itself so the absent case never crosses the
 call boundary as `undefined`; {@link joinTokens} drops the empty-string result.
 
 @param predicate - verb phrase whose complement slot is rendered
 
 @param renderVerbPhrase - verb-phrase render function
 
 @returns rendered surface, or empty string when absent
 */
function renderOptionalComplement<S extends string, V extends string, N extends string,>(
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
 Maps an English wh-word to the Chinese surface.
 
 @param wh - wh-word from the AST
 
 @returns Chinese wh-surface
 */
function chineseWhWord(wh: 'where' | 'when' | 'why' | 'how',): string {
  if (wh === 'where')
    return '在哪里';
  if (wh === 'when')
    return '什么时候';
  if (wh === 'why')
    return '为什么';
  return '怎么';
}

/**
 Builds a Chinese sentence renderer.
 
 @param deps - dependencies (subjects, verbs, sub-renderers)
 
 @returns render function for sentences
 
 @example
 ```ts
 const renderSentence = makeChineseSentenceRenderer({ subjects, verbs, renderNounPhrase, renderVerbPhrase, renderAdverbials });
 ```
 */
export function makeChineseSentenceRenderer<
  S extends string,
  V extends string,
  N extends string,
  >(
  deps: SentenceDeps<S, V, N>,
  ): (sentence: Sentence<S, V, N>,) => string {
  /**
   Destructured locale dependencies captured for use across every sub-renderer below.
   */
  const {
    subjects,
    verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  } = deps;

  /**
   Renders a declarative sentence with Chinese punctuation.
   
   @param sentence - declarative AST
   
   @returns rendered surface ending in `。`
   */
  function renderDeclarative(
    sentence: Extract<Sentence<S, V, N>, { kind: 'sentence.declarative'; }>,
  ): string {
    /**
     Sentence-level tense; defaults to present when omitted.
     */
    const tense = sentence.tense
      ?? 'present';
    /**
     Subject surface used in the leading position.
     */
    const subj = subjectSurface({
      ref: sentence.subject,
      subjects,
    },);
    /**
     Tense-marked verb surface.
     */
    const verb = verbSurfaceForTense({
      entry: verbs[sentence.predicate
        .verb],
      tense,
    },);
    /**
     Rendered object slot.
     */
    const object = renderOptionalObject({
      predicate: sentence.predicate,
      renderNounPhrase,
    },);
    /**
     Rendered complement.
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
      adverbials,
      verb,
      object,
      complement,
    ],);
    return `${body}。`;
  }

  /**
   Renders a yes/no question with the 吗 particle and Chinese question mark.
   
   @param sentence - yes/no AST
   
   @returns rendered surface ending in `吗？`
   */
  function renderYesNo(
    sentence: Extract<Sentence<S, V, N>, { kind: 'sentence.question.yesNo'; }>,
  ): string {
    /**
     Sentence-level tense; defaults to present when omitted.
     */
    const tense = sentence.tense
      ?? 'present';
    /**
     Subject surface.
     */
    const subj = subjectSurface({
      ref: sentence.subject,
      subjects,
    },);
    /**
     Tense-marked verb surface.
     */
    const verb = verbSurfaceForTense({
      entry: verbs[sentence.predicate
        .verb],
      tense,
    },);
    /**
     Rendered object slot.
     */
    const object = renderOptionalObject({
      predicate: sentence.predicate,
      renderNounPhrase,
    },);
    /**
     Rendered complement.
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
     Sentence body before particle and terminator.
     */
    const body = joinTokens([
      subj,
      adverbials,
      verb,
      object,
      complement,
    ],);
    return `${body}吗？`;
  }

  /**
   Renders a wh-subject question in head position.
   
   @param sentence - wh-subject AST
   
   @returns rendered surface fronted with `谁`
   */
  function renderWhSubject(
    sentence: Extract<WhQuestion<S, V, N>, { kind: 'sentence.question.wh.subject'; }>,
  ): string {
    /**
     Sentence-level tense; defaults to present when omitted.
     */
    const tense = sentence.tense
      ?? 'present';
    /**
     Tense-marked verb surface.
     */
    const verb = verbSurfaceForTense({
      entry: verbs[sentence.predicate
        .verb],
      tense,
    },);
    /**
     Rendered object slot.
     */
    const object = renderOptionalObject({
      predicate: sentence.predicate,
      renderNounPhrase,
    },);
    /**
     Rendered complement.
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
     Sentence body with `谁` at head.
     */
    const body = joinTokens([
      '谁',
      adverbials,
      verb,
      object,
      complement,
    ],);
    return `${body}？`;
  }

  /**
   Renders a wh-object question with in-situ 什么.
   
   @param sentence - wh-object AST
   
   @returns rendered surface with in-situ `什么`
   */
  function renderWhObject(
    sentence: Extract<WhQuestion<S, V, N>, { kind: 'sentence.question.wh.object'; }>,
  ): string {
    /**
     Sentence-level tense; defaults to present when omitted.
     */
    const tense = sentence.tense
      ?? 'present';
    /**
     Subject surface.
     */
    const subj = subjectSurface({
      ref: sentence.subject,
      subjects,
    },);
    /**
     Tense-marked verb surface.
     */
    const verb = verbSurfaceForTense({
      entry: verbs[sentence.verb],
      tense,
    },);
    /**
     Rendered adverbial cluster.
     */
    const adverbials = renderAdverbials(sentence.adverbials,);
    /**
     Sentence body with `什么` in the object slot.
     */
    const body = joinTokens([
      subj,
      adverbials,
      verb,
      '什么',
    ],);
    return `${body}？`;
  }

  /**
   Renders a wh-adverbial question with in-situ wh-word.
   
   @param sentence - wh-adverbial AST
   
   @returns rendered surface
   */
  function renderWhAdverbial(
    sentence: Extract<WhQuestion<S, V, N>, { kind: 'sentence.question.wh.adverbial'; }>,
  ): string {
    /**
     Sentence-level tense; defaults to present when omitted.
     */
    const tense = sentence.tense
      ?? 'present';
    /**
     Subject surface.
     */
    const subj = subjectSurface({
      ref: sentence.subject,
      subjects,
    },);
    /**
     Tense-marked verb surface.
     */
    const verb = verbSurfaceForTense({
      entry: verbs[sentence.predicate
        .verb],
      tense,
    },);
    /**
     Rendered object slot.
     */
    const object = renderOptionalObject({
      predicate: sentence.predicate,
      renderNounPhrase,
    },);
    /**
     Rendered complement.
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
     Chinese wh-word matched to the AST relation.
     */
    const wh = chineseWhWord(sentence.wh,);
    /**
     Sentence body with the wh-word in the adverbial slot.
     */
    const body = joinTokens([
      subj,
      wh,
      adverbials,
      verb,
      object,
      complement,
    ],);
    return `${body}？`;
  }

  /**
   Dispatches a wh-question by slot.
   
   @param sentence - wh AST
   
   @returns rendered surface
   */
  function renderWh(sentence: WhQuestion<S, V, N>,): string {
    if (sentence.kind
      === 'sentence.question.wh.subject')
      return renderWhSubject(sentence,);
    if (sentence.kind
      === 'sentence.question.wh.object')
      return renderWhObject(sentence,);
    return renderWhAdverbial(sentence,);
  }

  /**
   Renders an imperative sentence with the Chinese full stop or exclamation.
   
   @param sentence - imperative AST
   
   @returns rendered surface
   */
  function renderImperative(
    sentence: Extract<Sentence<S, V, N>, { kind: 'sentence.imperative'; }>,
  ): string {
    /**
     Imperative renders the bare verb surface.
     */
    const verb = verbs[sentence.predicate
      .verb]
      .surface;
    /**
     Rendered object slot.
     */
    const object = renderOptionalObject({
      predicate: sentence.predicate,
      renderNounPhrase,
    },);
    /**
     Rendered complement.
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
      adverbials,
      verb,
      object,
      complement,
    ],);
    /**
     Chinese imperative terminator: `！` for emphasis, otherwise `。`.
     */
    const terminator = sentence.terminator
      === '!' ? '！' : '。';
    return `${body}${terminator}`;
  }

  /**
   Dispatches a sentence AST to the correct sub-renderer.
   
   @param sentence - sentence AST
   
   @returns rendered surface
   */
  function renderSentence(sentence: Sentence<S, V, N>,): string {
    if (sentence.kind
      === 'sentence.declarative')
      return renderDeclarative(sentence,);
    if (sentence.kind
      === 'sentence.question.yesNo')
      return renderYesNo(sentence,);
    if (sentence.kind
      === 'sentence.imperative')
      return renderImperative(sentence,);
    return renderWh(sentence,);
  }

  return renderSentence;
}
