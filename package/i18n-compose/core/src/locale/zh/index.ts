/**
 Chinese locale builder.
 
 Implements the Chinese-specific grammar strategy: classifier-based
 counted phrases with ASCII space between digit and classifier, yes/no
 questions via the `吗` particle, wh-questions in-situ (no `吗`),
 Chinese sentence terminators (`。` `？` `！`).
 
 @module
 */

import type { LocaleSpec, } from '../../locale-spec.ts';
import { makeChineseAdverbialRenderer, } from './render-adverbial.ts';
import { makeChineseFragmentRenderer, } from './render-fragment.ts';
import { makeChineseNounPhraseRenderer, } from './render-np.ts';
import { makeChineseSentenceRenderer, } from './render-sentence.ts';
import { makeChineseVerbPhraseRenderer, } from './render-vp.ts';
import type { DefineChineseLocaleInput, } from './types.ts';

/**
 Builds a Chinese {@link LocaleSpec} from a vocabulary bundle.
 
 @param input - labels, subjects, nouns, verbs keyed by the consumer's unions
 
 @returns spec ready to plug into {@link createI18n}
 
 @example
 ```ts
 const zh = defineChineseLocale({ labels, subjects, nouns, verbs });
 ```
 */
export function defineChineseLocale<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  >(
  input: DefineChineseLocaleInput<Label, Subject, Verb, Noun>,
  ): LocaleSpec<Label, Subject, Verb, Noun> {
  /**
   Noun-phrase renderer closed over the noun + subject tables.
   */
  const renderNounPhrase = makeChineseNounPhraseRenderer({
    nouns: input.nouns,
    subjects: input.subjects,
  },);
  /**
   Adverbial cluster renderer depending on noun-phrase rendering.
   */
  const renderAdverbials = makeChineseAdverbialRenderer({ renderNounPhrase, },);
  /**
   Verb-phrase renderer with closure over verbs + sub-renderers.
   */
  const renderVerbPhrase = makeChineseVerbPhraseRenderer({
    verbs: input.verbs,
    renderNounPhrase,
    renderAdverbials,
  },);
  /**
   Sentence renderer dispatching on AST kind.
   */
  const renderSentence = makeChineseSentenceRenderer({
    subjects: input.subjects,
    verbs: input.verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  },);
  /**
   Fragment renderer dispatching on AST kind.
   */
  const renderFragment = makeChineseFragmentRenderer({
    labels: input.labels,
    verbs: input.verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  },);

  /**
   Resolves a static label key against the supplied label table.
   
   @param key - consumer label key
   
   @returns rendered surface
   */
  function renderLabel(key: Label,): string {
    return input.labels[key];
  }

  /**
   Resolves a bare noun key to its surface form.
   
   @param key - consumer noun key
   
   @returns rendered surface
   */
  function renderNoun(key: Noun,): string {
    return input.nouns[key]
      .surface;
  }

  return {
    renderLabel,
    renderNoun,
    renderNounPhrase,
    renderVerbPhrase,
    renderSentence,
    renderFragment,
  };
}

export type {
  ChineseVerbEntry,
  DefineChineseLocaleInput,
} from './types.ts';
