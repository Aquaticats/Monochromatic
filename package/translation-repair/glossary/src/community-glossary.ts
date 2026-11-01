import { FANDOM_GLOSSARY, } from './community-glossary-fandom.ts';
import { COMMUNITY_WORD_GLOSSARY, } from './community-glossary-words.ts';
import {
  foldForGlossary,
  formStarts,
  renderingSpans,
} from './glossary-match.ts';
import { withoutHtmlComments, } from './translate-address-drop.ts';

//region Community glossary
// THE COMMUNITY'S WORDS, beside the corpus pin (`corpus-source.ts`), by the
// owner's decision of 2026-09-09 (`doc/decision/translation-repair-community-glossary.md`).
// Two pages shipped the community's words wrong where the archive had them
// right: 自切 as "self-harmed by cutting" and "After she began cutting
// herself" where the archive has "attempted self-surgery"; 超天酱 as
// "Choco-chan" and "Chōten-chan" where the archive has "KAngel" of Needy
// Streamer Overload. Nothing in the pipeline told the bench, so nothing
// stopped it regressing a correct rendering twice.
//
// THREE USES. Every sheet that carries the declared names carries the terms
// an entry's source holds (`communityTermLines`, read into the identity
// context by `document-preparation.ts`), so writers and judges alike know the
// word. The sheets where judges compare candidates name each candidate
// lacking every accepted rendering where the source carries the term
// (`communityRenderingsBlock`): evidence to weigh, not a verdict, and how the
// archive's rendering joins every slate. And since class one hundred nineteen
// (2026-09-24) a floor refuses a candidate that keeps a term in Han or writes
// a form its entry refuses (`translate-community-term.ts`). No listed
// rendering is ever required, since a rendering inflects and the judges
// decide; until ledger C6 (2026-09-28) this comment said no candidate was
// barred, which the floor had made false.
//
// THE OWNER CURATES IT. An entry goes in wherever a run shipped the word
// wrong, whether or not the archive had it right: the archive wrote "tried
// coming out" for 炸柜 and "Alona and Atori" for 阿洛娜 and 亚托莉. The
// renderings are the archive's first where the archive renders the word
// well, then forms the community also uses. Where no archive renders the term
// well (药娘 by the owner's ruling, 逆子 where the only passage has no archive
// English), the renderings are the ones the entry's why states. Until ledger
// C6 this comment said a term the archive got wrong is not entered.

/**
 One community term and how the community renders it.

 @example
 ```ts
 const term: CommunityTerm = { term: '自切', renderings: ['self-surgery',], refusedForms: [], why: '...', };
 ```
 */
export type CommunityTerm = {
  /**
   Term as the original writes it.
   */
  readonly term: string;

  /**
   Renderings the community accepts, the archive's first where it renders
   the word well; a candidate carrying any of them, in any casing and
   inflected as English inflects it, carries the word.
   */
  readonly renderings: readonly string[];

  /**
   Forms a candidate may not write for the term, in any casing, refused by
   the source-carry floor (`translate-community-term.ts`) alongside the
   term left in Han; empty where no form is refused.
   */
  readonly refusedForms: readonly string[];

  /**
   One line of why, for the sheet; it is shown on every page whose original
   carries the term, so it says nothing that holds on one page alone.
   */
  readonly why: string;

  /**
   Source contexts in which the term stands inside an organization's proper
   name, where it is not the term (owner, 2026-09-27: "Allow it, because
   it's the proper name of an org."); an occurrence inside one is read as
   absent by the floor, the sheet lines and the departures.
   */
  readonly properNameContexts?: readonly string[];

  /**
   Longer words that write the term's characters without being the term
   (各自切, each cutting, holds 自切; ledger C10); an occurrence inside one is
   read as absent, as one inside a proper name is.
   */
  readonly enclosingWords?: readonly string[];
};

/**
 Community terms the pinned corpus carries, curated by the owner.
 */
export const COMMUNITY_GLOSSARY: readonly CommunityTerm[] = [
  ...COMMUNITY_WORD_GLOSSARY,
  ...FANDOM_GLOSSARY,
];

/**
 One text a judge is shown, by the label the sheet gives it.

 @example
 ```ts
 const candidate: RenderingCandidate = { label: 'CANDIDATE 1', text, };
 ```
 */
export type RenderingCandidate = {
  /**
   Label the sheet shows the text under.
   */
  readonly label: string;

  /**
   Text as the judge sees it.
   */
  readonly text: string;
};

/**
 Glossary terms a text carries outside its comments.

 AN INDEX SCAN PER TERM at word boundaries (`glossary-match.ts`), since each
 term is a fixed string and the glossary is short; a Latin term (OD, jk 裙)
 matches in any case and spacing, and never inside a longer word (MOD).

 @param text - original text to read

 @param glossary - terms to look for; defaults to the corpus glossary

 @returns Terms present, in glossary order

 @example
 ```ts
 communityTermsIn({ text: '在她自切后', },);
 ```
 */
export function communityTermsIn(
  {
    text,
    glossary = COMMUNITY_GLOSSARY,
  }: {
    readonly text: string;
    readonly glossary?: readonly CommunityTerm[];
  },
): readonly CommunityTerm[] {
  /**
   Original with its comments cut, read once for every term.
   */
  const uncommented = withoutHtmlComments({ text, },);
  return glossary.filter(function present(entry,): boolean {
    /**
     Original with every proper name and every longer word that carries the
     term cut out, so an organization's name (小药娘网络科技) or another word
     (各自切) is not read as the term.
     */
    const unnamed = [
      ...entry.properNameContexts ?? [],
      ...entry.enclosingWords ?? [],
    ].reduce(
      function cutName(
        remaining,
        context,
      ): string {
        return remaining.replaceAll(
          context,
          ' ',
        );
      },
      uncommented,
    );
    /**
     Bounded starts of the term in the original.
     */
    const starts = formStarts({
      folded: foldForGlossary({ text: unnamed, },),
      form: foldForGlossary({ text: entry.term, },),
      end: 'inflected',
    },);
    return starts.length > 0;
  },);
}

/**
 Quotes every accepted rendering for a sheet line.

 @param entry - term whose renderings are listed

 @returns Renderings quoted and comma-joined

 @example
 ```ts
 quotedRenderings({ entry, },);
 // => '"KAngel", "Needy Streamer Overload"'
 ```
 */
function quotedRenderings(
  { entry, }: { readonly entry: CommunityTerm; },
): string {
  return entry.renderings
    .map(function quoted(rendering,): string {
      return `"${rendering}"`;
    },)
    .join(', ',);
}

/**
 Identity-context lines for the terms an entry's source carries, so every
 sheet that carries the declared names carries the community's words.

 @param text - whole original document

 @param glossary - terms to look for; defaults to the corpus glossary

 @param heading - line naming what the terms are, so a glossary of ordinary
 words (`rendering-glossary.ts`) does not call them the community's

 @returns Heading and one line per term present, empty when none is

 @example
 ```ts
 const lines = communityTermLines({ text: sourceText, },);
 ```
 */
export function communityTermLines(
  {
    text,
    glossary = COMMUNITY_GLOSSARY,
    heading = 'COMMUNITY TERMS this entry carries, with the renderings the community uses (a rendering may inflect):',
  }: {
    readonly text: string;
    readonly glossary?: readonly CommunityTerm[];
    readonly heading?: string;
  },
): readonly string[] {
  /**
   Terms this entry carries.
   */
  const present = communityTermsIn({
    text,
    glossary,
  },);
  if (present.length === 0)
    return [];
  return [
    heading,
    ...present.map(function toLine(entry,): string {
      return `- ${entry.term}: ${quotedRenderings({ entry, },)} (${entry.why})`;
    },),
  ];
}

/**
 Whether a text carries any accepted rendering of a term outside its
 comments, in any casing, at word boundaries and inflected as English
 inflects it (`renderingSpans`): "cure" in "to cure" and "cured", never
 "cured" in "secured" nor "Atri" in "atrium" (ledger C3 and C4).

 @param text - candidate text

 @param entry - term whose renderings are looked for

 @returns Whether one rendering occurs

 @example
 ```ts
 carriesRendering({ text: 'She attempted self-surgery.', entry, },);
 // => true
 ```
 */
function carriesRendering(
  {
    text,
    entry,
  }: {
    readonly text: string;
    readonly entry: CommunityTerm;
  },
): boolean {
  /**
   Candidate folded once, its comments cut, for every rendering.
   */
  const folded = foldForGlossary({ text: withoutHtmlComments({ text, },), },);
  return entry.renderings
    .some(function occurs(rendering,): boolean {
      /**
       Spans at which the rendering stands in the candidate.
       */
      const spans = renderingSpans({
        folded,
        rendering,
      },);
      return spans.length > 0;
    },);
}

/**
 Names each candidate lacking every accepted rendering of a term the source
 carries.

 AN EMPTY CANDIDATE IS SKIPPED: an absent archive rendering carries no word
 and is not departing from one.

 @param sourceText - original the candidates render

 @param candidates - texts the judge is shown, by label

 @param glossary - terms to look for; defaults to the corpus glossary

 @returns One line per candidate and term, in candidate then glossary order

 @example
 ```ts
 const departures = communityRenderingDepartures({ sourceText, candidates, },);
 ```
 */
export function communityRenderingDepartures(
  {
    sourceText,
    candidates,
    glossary = COMMUNITY_GLOSSARY,
  }: {
    readonly sourceText: string;
    readonly candidates: readonly RenderingCandidate[];
    readonly glossary?: readonly CommunityTerm[];
  },
): readonly string[] {
  /**
   Terms the source carries.
   */
  const present = communityTermsIn({
    text: sourceText,
    glossary,
  },);
  return candidates.flatMap(function departuresOf(candidate,): readonly string[] {
    if (candidate.text === '')
      return [];
    return present
      .filter(function lacking(entry,): boolean {
        return !carriesRendering({
          text: candidate.text,
          entry,
        },);
      },)
      .map(function toLine(entry,): string {
        return `${candidate.label} carries none of the community's renderings of ${entry.term} (${
          quotedRenderings({ entry, },)
        }); the community glossary renders it so`;
      },);
  },);
}

/**
 Sheet block naming the departures, for the judge to weigh after the
 passages.

 @param sourceText - original the candidates render

 @param candidates - texts the judge is shown, by label

 @param glossary - terms to look for; defaults to the corpus glossary

 @returns Heading, one line per departure, the inflection note and a blank
 line; empty when no candidate departs

 @example
 ```ts
 const block = communityRenderingsBlock({ sourceText, candidates, },);
 ```
 */
export function communityRenderingsBlock(
  {
    sourceText,
    candidates,
    glossary = COMMUNITY_GLOSSARY,
  }: {
    readonly sourceText: string;
    readonly candidates: readonly RenderingCandidate[];
    readonly glossary?: readonly CommunityTerm[];
  },
): readonly string[] {
  /**
   Departures to name.
   */
  const departures = communityRenderingDepartures({
    sourceText,
    candidates,
    glossary,
  },);
  if (departures.length === 0)
    return [];
  return [
    'COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',
    ...departures.map(function toItem(line,): string {
      return `- ${line}`;
    },),
    'A rendering may inflect; a candidate that renders the term another way departs from the community\'s word.',
    '',
  ];
}

//endregion Community glossary
