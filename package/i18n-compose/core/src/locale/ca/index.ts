/**
 Catalan locale builder.
 
 Implements the Catalan-specific grammar strategy: gender/number article
 agreement, finite verb forms indexed by person/number/tense, infinitive
 complements via bare infinitive, question rendering via punctuation +
 intonation (no inversion rewrite in v1).
 
 @module
 */

import type { LocaleSpec, } from '../../locale-spec.ts';
import { makeCatalanAdverbialRenderer, } from './render-adverbial.ts';
import { makeCatalanFragmentRenderer, } from './render-fragment.ts';
import { makeCatalanNounPhraseRenderer, } from './render-np.ts';
import { makeCatalanSentenceRenderer, } from './render-sentence.ts';
import { makeCatalanVerbPhraseRenderer, } from './render-vp.ts';
import type { DefineCatalanLocaleInput, } from './types.ts';

/**
 Builds a Catalan {@link LocaleSpec} from a vocabulary bundle.
 
 @param input - labels, subjects, nouns, verbs keyed by the consumer's unions
 
 @returns spec ready to plug into {@link createI18n}
 
 @example
 ```ts
 const ca = defineCatalanLocale({ labels, subjects, nouns, verbs });
 ```
 */
export function defineCatalanLocale<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  >(
  input: DefineCatalanLocaleInput<Label, Subject, Verb, Noun>,
  ): LocaleSpec<Label, Subject, Verb, Noun> {
  /**
   Noun-phrase renderer closed over the noun + subject tables.
   */
  const renderNounPhrase = makeCatalanNounPhraseRenderer({
    nouns: input.nouns,
    subjects: input.subjects,
  },);
  /**
   Adverbial cluster renderer depending on noun-phrase rendering.
   */
  const renderAdverbials = makeCatalanAdverbialRenderer({ renderNounPhrase, },);
  /**
   Verb-phrase renderer with closure over verbs + sub-renderers.
   */
  const renderVerbPhrase = makeCatalanVerbPhraseRenderer({
    verbs: input.verbs,
    renderNounPhrase,
    renderAdverbials,
  },);
  /**
   Sentence renderer dispatching on AST kind.
   */
  const renderSentence = makeCatalanSentenceRenderer({
    subjects: input.subjects,
    verbs: input.verbs,
    renderNounPhrase,
    renderVerbPhrase,
    renderAdverbials,
  },);
  /**
   Fragment renderer dispatching on AST kind.
   */
  const renderFragment = makeCatalanFragmentRenderer({
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
  CatalanVerbEntry,
  DefineCatalanLocaleInput,
} from './types.ts';
