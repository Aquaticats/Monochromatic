import type { Nodes, } from 'mdast';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { findDroppedDeclaredNames, } from './declared-name-survival.ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  parseMarkdownBody,
  requireMarkdownRefusal,
  withMarkdownExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

/**
 Logger root for the contributor name reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

//region Contributor name authority

/**
 English archive labels introducing target-authoritative contributor names.
 */
export const CONTRIBUTOR_LABELS = [
  'Contributor for this entry:',
  'Contributors for this entry:',
  'Contributor for this entry：',
  'Contributors for this entry：',
] as const;

/**
 Splits comma-delimited contributor forms without splitting Markdown links or
 parenthetical role notes.

 @param text - contributor suffix after archive label

 @returns Contributor tokens in source order

 @example
 ```ts
 const forms = splitContributorForms({ text: 'Mika, [Neko](https://example.test)', });
 ```
 */
function splitContributorForms(
  { text, }: { readonly text: string; },
): readonly string[] {
  /**
   Completed top-level forms.
   */
  const forms: string[] = [];
  /**
   Start offset of current form.
   */
  let start = 0;
  /**
   Markdown label nesting depth.
   */
  let squareDepth = 0;
  /**
   Link target or role-note nesting depth.
   */
  let roundDepth = 0;
  for (let at = 0; at < text.length; at += 1) {
    /**
     Character under cursor.
     */
    const character = text.charAt(at,);
    if (character === '[')
      squareDepth += 1;
    else if ((character === ']') && (squareDepth > 0))
      squareDepth -= 1;
    else if (character === '(')
      roundDepth += 1;
    else if ((character === ')') && (roundDepth > 0))
      roundDepth -= 1;
    else {
      /**
       Whether cursor is delimiter outside labels, links, or role notes.
       */
      const atTopLevelDelimiter = (character === ',')
        && (squareDepth === 0)
        && (roundDepth === 0);
      if (atTopLevelDelimiter) {
        forms.push(text.slice(
          start,
          at,
        ),);
        start = at + 1;
      }
    }
  }
  forms.push(text.slice(start,),);
  return forms;
}

/**
 The markup of every reference link a text defines, with the label it shows.

 ONLY THE PARSE CAN TELL whether `[Whisker][w]` shows as a link: the grammar
 makes a reference a link only where the document defines it, and leaves the
 whole markup as text otherwise. So the text is parsed once, the markup of
 each `linkReference` it holds is read off its own offsets and its label off
 the offsets of its link text; a token that is not among them reads as the
 text it shows.

 @param text - complete text the contributor tokens were read from, which
 holds the definitions

 @returns Label shown by each reference link's markup as written; empty when
 the text holds no `[` or cannot be parsed

 @example
 ```ts
 const labels = referenceLinkLabels({ text: 'Mika, [Neko][n]\n\n[n]: https://example.test', },);
 ```
 */
function referenceLinkLabels({ text, }: { readonly text: string; },): ReadonlyMap<string, string> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: referenceLinkLabels.name,
    l,
  },);
  /**
   Markup and label of each reference link found.
   */
  const labels = new Map<string, string>();
  if (!text.includes('[',))
    return labels;
  try {
    /**
     Nodes still to visit, so a deep tree is walked without recursion.
     */
    const pending: Nodes[] = [
      withMarkdownExhaustionRefused({
        parse: parseMarkdownBody,
        input: { body: text, },
      },),
    ];
    for (
      let node = pending.pop();
      node !== undefined;
      node = pending.pop()
    ) {
      if ('children' in node)
        pending.push(...node.children,);
      if (node.type !== 'linkReference')
        continue;
      /**
       Where the markup stands in the text.
       */
      const start = node.position
        ?.start
        .offset;
      /**
       Where the markup ends in the text.
       */
      const end = node.position
        ?.end
        .offset;
      if ((start === undefined) || (end === undefined))
        throw new Error(
          'unreachable: a parsed reference link carries no start or end offset, though the parser sets a position '
            + 'on every node it builds',
        );
      /**
       Markup as written.
       */
      const markup = text.slice(
        start,
        end,
      );
      /**
       Last inline node of the link text, absent for a link text of nothing.
       */
      const lastInline = node.children
        .at(-1,);
      // THE LINK TEXT ENDS WHERE ITS LAST INLINE NODE ENDS, read off the parse
      // like the markup. Counting back from the markup's end by the length of
      // `node.label` read `Whisker]` out of `[Whisker][a\]b]` and
      // `Whisker][w&` out of `[Whisker][w&amp;x]`: the parser reports a
      // reference label with its escapes and entities decoded, so its length
      // is not the length written.
      /**
       Where the link text ends in the text, just past the opening bracket for
       a link text of nothing.
       */
      const textEnd = (lastInline === undefined)
        ? start + 1
        : lastInline.position
          ?.end
          .offset;
      if (textEnd === undefined)
        throw new Error(
          'unreachable: an inline node of a parsed reference link carries no end offset, though the parser sets a '
            + 'position on every node it builds',
        );
      labels.set(
        markup,
        text
          .slice(
          start + 1,
          textEnd,
          )
          .trim(),
      );
    }
  }
  catch (error) {
    // A text nested past the parser's stack is read as defining no reference,
    // which is what a reader of the plain tokens saw before this one.
    /**
     Why the parser gave up.
     */
    const { message, } = requireMarkdownRefusal({ error, },);
    rl.debug(`reference links not read, the text did not parse: ${message}`,);
    return new Map<string, string>();
  }
  return labels;
}

/**
 Reads visible identity from one contributor token while retaining plain
 unlinked forms and role notes.

 @param token - one top-level contributor token

 @param references - label each reference link of the text shows, by its
 markup as written

 @returns Visible target-authoritative form without the spaces around it,
 empty for an empty token or a link label empty or only spaces, which shows a
 reader nothing

 @example
 ```ts
 const form = contributorForm({ token: '[Neko](https://example.test)', references: new Map(), });
 ```
 */
function contributorForm(
  {
    token,
    references,
  }: {
    readonly token: string;
    readonly references: ReadonlyMap<string, string>;
  },
): string {
  /**
   Token without delimiter-adjacent spacing or list marker.
   */
  const trimmed = token.trim();
  /**
   Form without optional Markdown unordered-list marker, nor the further
   spaces a marker may stand before its item with.
   */
  const unmarked = (trimmed.startsWith('- ',)
    || trimmed.startsWith('* ',)
    || trimmed.startsWith('+ ',))
    ? trimmed
      .slice(2,)
      .trimStart()
    : trimmed;
  if (!unmarked.startsWith('[',))
    return unmarked;
  // A reference link shows its label where the text defines it.
  /**
   Label the token shows as a reference link, when it is one.
   */
  const referenceLabel = references.get(unmarked,);
  if (referenceLabel !== undefined)
    return referenceLabel;
  /**
   Boundary between visible label and link destination.
   */
  const labelEnd = unmarked.indexOf('](',);
  if (labelEnd === (-1))
    return unmarked;
  // The label as a reader sees it: spaces inside the brackets show nothing.
  return unmarked
    .slice(
    1,
    labelEnd,
    )
    .trim();
}

/**
 One archive line the contributor reader takes as a declaration: a line that
 opens with a contributor label, or a line continuing one whose label stands
 alone.

 @example
 ```ts
 const declaration: ContributorDeclarationLine = { line: 0, names: 'Mika', };
 ```
 */
export type ContributorDeclarationLine = {
  /**
   Index of the line in the text split at line feeds.
   */
  readonly line: number;
  /**
   Names the line carries, trimmed: what follows the label on a label line,
   the whole line on a continuation; empty for a label line whose names
   follow on the lines after it.
   */
  readonly names: string;
};

/**
 Lines the contributor reader takes as declarations, in order.

 ONE READING FOR EVERY QUESTION ASKED OF A DECLARATION (T8's eighteenth
 batch, ledger B81): which names a block declares, and whether a block is
 declarations and nothing else, read the same lines. A label line declares
 the names after its label; a label standing alone declares the nonblank
 lines that follow it, up to the first blank one. Only a line that opens
 with a label counts, so the same words inside a sentence declare nothing.

 @param text - archive text, a page or one block

 @returns Each declaring line with the names it carries

 @example
 ```ts
 contributorDeclarationLines({ text: 'Contributor for this entry:\nMika', },); // [{ line: 0, names: '' }, { line: 1, names: 'Mika' }]
 ```
 */
export function contributorDeclarationLines(
  { text, }: { readonly text: string; },
): readonly ContributorDeclarationLine[] {
  /**
   Archive lines, kept whole because a label standing alone hands its names
   to the lines after it.
   */
  const lines = text.split('\n',);
  /**
   Declaring lines found so far.
   */
  const declarations: ContributorDeclarationLine[] = [];
  for (const [at, line,] of lines.entries()) {
    /**
     Contributor label this line begins with.
     */
    const label = CONTRIBUTOR_LABELS.find(function begins(candidate,): boolean {
      return line.startsWith(candidate,);
    },);
    if (label === undefined)
      continue;
    /**
     Names carried beside the label.
     */
    const sameLine = line
      .slice(label.length,)
      .trim();
    declarations.push({
      line: at,
      names: sameLine,
    },);
    if (sameLine !== '')
      continue;
    /**
     Index of the first line that may continue the label.
     */
    const start = at + 1;
    /**
     Lines after the label, which continue it up to the first blank one.
     */
    const following = lines.slice(start,);
    for (const [offset, next,] of following.entries()) {
      /**
       Names a continuation line carries.
       */
      const continuation = next.trim();
      if (continuation === '')
        break;
      declarations.push({
        line: start + offset,
        names: continuation,
      },);
    }
  }
  return declarations;
}

/**
 Reads target-authoritative contributor names from archive attribution lines.

 The source can identify same contributor under another script or handle.
 Existing English archive label is authority because it can carry chosen
 public handle unrelated to literal transliteration. Ordinary prose is not
 inspected, so matching words elsewhere never become protected identities.

 @param text - complete existing English archive page

 @returns Visible contributor forms, deduplicated, longest first

 @example
 ```ts
 const forms = archiveContributorNameForms({ text: 'Contributors for this entry: Mika, [Neko](https://example.test)', });
 ```
 */
export function archiveContributorNameForms(
  { text, }: { readonly text: string; },
): readonly string[] {
  /**
   Raw contributor suffixes the declaring lines carry, a label standing
   alone carrying none of its own.
   */
  const suffixes = contributorDeclarationLines({ text, },)
    .map(function namesOf(declaration,): string {
      return declaration.names;
    },)
    .filter(function carriesNames(names,): boolean {
      return names !== '';
    },);
  /**
   Reference links the text defines, which only a parse can tell from text.
   */
  const references = referenceLinkLabels({ text, },);
  /**
   Visible identities without repeated declarations.
   */
  const forms = new Set(suffixes
    .flatMap(function split(suffix,): readonly string[] {
      return splitContributorForms({ text: suffix, });
    },)
    .map(function visible(token,): string {
      return contributorForm({
        token,
        references,
      },);
    },)
    .filter(function nonempty(form,): boolean {
      return form !== '';
    },),);
  return [ ...forms, ].toSorted(function longestFirst(
    left,
    right,
  ): number {
    return right.length - left.length;
  },);
}

/**
 Reports whether two contributor spellings project to same complete identity.

 @param left - one visible contributor form

 @param right - other visible contributor form

 @returns Whether forms differ only by supported name separators or markup

 @example
 ```ts
 const same = contributorFormsMatch({ left: 'Snow_Cat', right: 'Snow Cat', });
 ```
 */
export function contributorFormsMatch(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): boolean {
  /**
   Left projected identity losses when compared into right.
   */
  const leftDropped = findDroppedDeclaredNames({
    forms: [left,],
    baseText: left,
    candidateText: right,
  },);
  /**
   Right projected identity losses when compared into left.
   */
  const rightDropped = findDroppedDeclaredNames({
    forms: [right,],
    baseText: right,
    candidateText: left,
  },);
  return (leftDropped.length === 0) && (rightDropped.length === 0);
}

/**
 Finds target-authoritative contributor forms missing or respelled in candidate.

 @param archiveText - existing English attribution authority

 @param candidateText - proposed wording

 @returns Missing target forms in archive order

 @example
 ```ts
 const dropped = droppedContributorNameForms({ archiveText, candidateText, });
 ```
 */
export function droppedContributorNameForms(
  {
    archiveText,
    candidateText,
  }: {
    readonly archiveText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Target-authoritative archive forms.
   */
  const forms = archiveContributorNameForms({ text: archiveText, },);
  /**
   Forms candidate attribution currently carries.
   */
  const candidateForms = archiveContributorNameForms({ text: candidateText, },);
  return forms.filter(function absent(form,): boolean {
    return !candidateForms.some(function matches(candidate,): boolean {
      return contributorFormsMatch({
        left: form,
        right: candidate,
      },);
    },);
  },);
}

//endregion Contributor name authority
