/**
 English locale builder.
 
 Implements the English-specific grammar strategy: do-support for yes/no
 and wh questions, base verb after auxiliaries, future via `will + base`,
 infinitive complements via `to + base`, explicit article rendering.
 
 @module
 */

import type { LocaleSpec, } from '../../locale-spec.ts';
import { makeEnglishAdverbialRenderer, } from './render-adverbial.ts';
import { makeEnglishFragmentRenderer, } from './render-fragment.ts';
import { makeEnglishNounPhraseRenderer, } from './render-np.ts';
import { makeEnglishSentenceRenderer, } from './render-sentence.ts';
import { makeEnglishVerbPhraseRenderer, } from './render-vp.ts';
import type { DefineEnglishLocaleInput, } from './types.ts';

/**
 Builds an English {@link LocaleSpec} from a vocabulary bundle.
 
 @param input - labels, subjects, nouns, verbs keyed by the consumer's unions
 
 @returns spec ready to plug into {@link createI18n}
 
 @example
 ```ts
 const en = defineEnglishLocale({ labels, subjects, nouns, verbs });
 ```
 */
export function defineEnglishLocale<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  >(
  input: DefineEnglishLocaleInput<Label, Subject, Verb, Noun>,
  ): LocaleSpec<Label, Subject, Verb, Noun> {
  /**
   Noun-phrase renderer closed over the noun + subject tables.
   */
  const renderNounPhrase = makeEnglishNounPhraseRenderer({
    nouns: input.nouns,
    subjects: input.subjects,
  },);
  /**
   Adverbial cluster renderer that depends on noun-phrase rendering.
   */
  const renderAdverbials = makeEnglishAdverbialRenderer({ renderNounPhrase, },);
  /**
   Verb-phrase renderer with closure over verbs + sub-renderers.
   */
  const renderVerbPhrase = makeEnglishVerbPhraseRenderer({
    verbs: input.verbs,
    renderNounPhrase,
    renderAdverbials,
  },);
  /**
   Sentence renderer dispatching on AST kind.
   */
  const renderSentence = makeEnglishSentenceRenderer({
    subjects: input.subjects,
    verbs: input.verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  },);
  /**
   Fragment renderer dispatching on AST kind.
   */
  const renderFragment = makeEnglishFragmentRenderer({
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
  DefineEnglishLocaleInput,
  EnglishVerbEntry,
} from './types.ts';
