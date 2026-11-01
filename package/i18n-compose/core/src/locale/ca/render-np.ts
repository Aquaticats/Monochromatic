/**
 Catalan noun-phrase renderer factory.
 
 @module
 */

import type {
  NounPhrase,
  Possessor,
} from '../../ast.ts';
import { assertCountableNoun, } from '../../countability.ts';
import type {
  NounEntry,
  SubjectEntry,
} from '../../entries.ts';
import type { GrammaticalNumber, } from '../../grammar-primitives.ts';

/**
 Returns the plural surface for a noun under a specific count.
 
 @param entry - resolved noun entry
 
 @param count - count requesting the form
 
 @returns plural surface; falls back to `surface + s`
 
 @example
 ```ts
 nounPlural({ entry: { surface: 'gat', plural: 'gats' }, count: 2 }); // 'gats'
 ```
 */
function nounPlural(
  {
    entry,
    count,
  }: {
    readonly entry: NounEntry;
    readonly count: number;
  },
): string {
  if (entry.plural
    === undefined)
    return `${entry.surface}s`;
  if ((typeof entry.plural) === 'string')
    return entry.plural;
  return entry.plural(count,);
}

/**
 Picks an article surface for a noun entry, definiteness, and number.
 
 @param entry - noun entry
 
 @param kind - definite or indefinite
 
 @param number - grammatical number
 
 @returns article surface; falls back to `el`/`un` when no entry-level form is supplied
 
 @example
 ```ts
 articleFor({ entry, kind: 'definite', number: 'singular' }); // 'el'
 ```
 */
function articleFor(
  {
    entry,
    kind,
    number,
  }: {
    readonly entry: NounEntry;
    readonly kind: 'definite' | 'indefinite';
    readonly number: GrammaticalNumber;
  },
): string {
  /**
   Article subtable for the requested definiteness.
   */
  const sub = entry.articles?.[kind];
  if (sub === undefined)
    return kind === 'definite' ? 'el' : 'un';
  return (number === 'singular' ? sub.singular : sub.plural)
    ?? (kind === 'definite' ? 'el' : 'un');
}

/**
 Joins an article and noun surface, attaching elided apostrophe-final articles.
 
 @param options - article and surface wrapped for named-parameter calls
 
 @returns article phrase with correct separator
 
 @example
 ```ts
 articlePhrase({ article: 'el', surface: 'gat' }); // 'el gat'
 articlePhrase({ article: "l'", surface: 'article' }); // "l'article"
 ```
 */
function articlePhrase(
  options: {
    readonly article: string;
    readonly surface: string;
  },
): string {
  /**
   Article surface and noun surface for elision-aware joining.
   */
  const {
    article,
    surface,
  } = options;
  if (article.endsWith("'",))
    return `${article}${surface}`;
  return `${article} ${surface}`;
}

/**
 Builds a Catalan noun-phrase renderer closed over the supplied vocab tables.
 
 @param nouns - noun vocabulary keyed by the consumer's `Noun` union
 
 @param subjects - subject vocabulary keyed by the consumer's `Subject` union
 
 @returns render function for noun phrases
 
 @example
 ```ts
 const renderNounPhrase = makeCatalanNounPhraseRenderer({ nouns, subjects });
 ```
 */
export function makeCatalanNounPhraseRenderer<S extends string, N extends string,>(
  {
    nouns,
    subjects,
  }: {
    readonly nouns: Readonly<Record<N, NounEntry>>;
    readonly subjects: Readonly<Record<S, SubjectEntry>>;
  },
): (phrase: NounPhrase<S, N>,) => string {
  /**
   Renders a possessor surface for `noun.possessed`.
   
   @param p - possessor AST node
   
   @returns possessive surface
   */
  function renderPossessor(p: Possessor<S>,): string {
    if (p.kind
      === 'possessor.subject')
      return subjects[p.subject]
        .possessive;
    return `de ${p.text}`;
  }

  /**
   Renders a noun-phrase AST in Catalan.
   
   @param phrase - noun-phrase AST
   
   @returns rendered surface
   */
  function renderNounPhrase(phrase: NounPhrase<S, N>,): string {
    if (phrase.kind
      === 'noun.bare')
      return nouns[phrase.noun]
        .surface;
    if (phrase.kind
      === 'noun.counted') {
      /**
       Resolved noun entry validated before numeric rendering.
       */
      const entry = nouns[phrase.noun];
      assertCountableNoun({
        entry,
        noun: phrase.noun,
      },);
      /**
       Plural surface when count is not 1, singular otherwise.
       */
      const surface = phrase.count
        === 1
        ? entry.surface
        : nounPlural({
          entry,
          count: phrase.count,
        },);
      return `${String(phrase.count,)} ${surface}`;
    }
    if (phrase.kind
      === 'noun.definite') {
      /**
       Resolved noun entry.
       */
      const entry = nouns[phrase.noun];
      return articlePhrase({
        article: articleFor({
          entry,
          kind: 'definite',
          number: 'singular',
        },),
        surface: entry.surface,
      },);
    }
    if (phrase.kind
      === 'noun.indefinite') {
      /**
       Resolved noun entry.
       */
      const entry = nouns[phrase.noun];
      return articlePhrase({
        article: articleFor({
          entry,
          kind: 'indefinite',
          number: 'singular',
        },),
        surface: entry.surface,
      },);
    }
    if (phrase.kind
      === 'noun.possessed')
      return `${nouns[phrase.noun]
        .surface} ${renderPossessor(phrase.possessor,)}`;
    return phrase.text;
  }

  return renderNounPhrase;
}
