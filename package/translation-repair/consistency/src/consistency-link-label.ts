import {
  CLOSING_BRACKETS,
  CLOSING_CURLY_DOUBLE,
} from './consistency-markup-runs.ts';
import { continuesLatinWordOrCompatibilityForm, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { TreeNode, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency link label
// WHAT THE LABEL OF A LINK SHOWS A READER, read from the parse (n166 Z9, n168). The consistency pass reads words as a reader sees them,
// and a link shows its label and nothing of its destination or its title. A link whose label shows nothing (`[](u)`, `[](u "t")`,
// `[]()`, `[][r]` with its definition, an image `![](u)` with no alt text) neither ends a word nor starts one: the text on each side of
// it meets as if the link were not there. A link whose label shows marks or spaces and no Latin letter or digit (`[ ](u)`, `[-](u)`; a Han,
// Cyrillic or Greek label is marks here, since only the Latin letters and digits and the forms that fold to them make a word with
// an apostrophe) shows no word to the apostrophe after it, so that apostrophe starts a word, as it does after a space. Any other link shows its label, which
// the apostrophe after it joins as it joins a closing bracket (`[the cat](x)'s` is one word, `cat's`).
//
// THE LINK IS READ BY THE PARSE (`parseSliceBody`, micromark), so the rules of CommonMark for a destination (round brackets that
// balance, a backslash escape, a `<...>` form, a newline before the closing bracket), for a title (three quote forms, round brackets
// in it) and for a label (escaped square brackets, a link inside a label) are the parse's, and no second scanner of them is written.
// An English the page grammar refuses has no parse, so no link is read in it.
//
// THE LAST MARK OF A LABEL OF MARKS CLOSES OR DOES NOT. A label that shows a closing round or square bracket or a closing curly
// double quotation mark and no letter (`[\]](u)`, `[()](u)`) ends in what a closing bracket would be in plain text, so the apostrophe
// after it joins as after that bracket does. A label of other marks does not, which is the kind this file names `marks`. A label of
// closing marks and nothing else (`[\]](u)`, `[”](u)`, `[))](u)`) is told apart as `closing`, since the reading of closing marks before
// an apostrophe (`consistency-closing-marks.ts`) reads such a link as the closing marks it shows (n169).

/**
 What the label of a link shows a reader: nothing, marks and spaces that end in no closing mark and hold no Latin letter or digit
 (nor a form that folds to one; a Han, Cyrillic or Greek letter is a mark here), closing marks and nothing else, or anything else (a
 Latin letter or a digit, a closing mark after some other mark, or a shape the parse leaves unread such as raw markup).
 */
export type LinkLabelKind = 'closing' | 'empty' | 'marks' | 'other';

/**
 Whether a shown character closes a bracketed stretch or a quotation: a closing round or square bracket or a closing curly double
 quotation mark.

 @param character - one whole character the label shows

 @returns Whether it is one of the closing marks

 @example
 ```ts
 const closes = isClosingMark(']',); // true
 ```
 */
function isClosingMark(character: string,): boolean {
  return CLOSING_BRACKETS.has(character,) || (character === CLOSING_CURLY_DOUBLE);
}

/**
 The node types that are a link or an image, which show their label (the text of a link, the alt text of an image).
 */
const LINK_NODE_TYPES: ReadonlySet<string> = new Set([
  'link',
  'linkReference',
  'image',
  'imageReference',
],);

/**
 The node types inside a label whose shown text is their own `value` or their children's, so that the label's text is read from them.
 */
const TEXT_NODE_TYPES: ReadonlySet<string> = new Set([
  'text',
  'inlineCode',
  'emphasis',
  'strong',
  'delete',
  'break',
  'image',
  'imageReference',
],);

/**
 Whether a node of the tree is a link, a reference or an image, the nodes whose label a reader sees.

 @param node - a node of the tree

 @returns Whether the node is one of the four

 @example
 ```ts
 const isLink = isLinkNode({ node, },);
 ```
 */
export function isLinkNode({ node, }: { readonly node: TreeNode; },): boolean {
  return LINK_NODE_TYPES.has(node.type,);
}

/**
 The text a label shows a reader: the text and code inside it, the alt text of an image inside it, and a space for a line break.

 @param node - a link, a link reference, an image or an image reference

 @returns The shown text, absent when the label holds a node whose shown text the tree does not say (raw markup, an expression)

 @example
 ```ts
 const [shown,] = labelTextOf({ node, },); // ['x'] for `[x](u)`
 ```
 */
function labelTextOf({ node, }: { readonly node: TreeNode; },): readonly string[] {
  if ((node.type === 'image') || (node.type === 'imageReference'))
    return [node.alt ?? '',];
  /**
   The text met so far, which is what the function returns, mutated only inside this function.
   */
  const shown: string[] = [];
  /**
   Nodes still to read, the children of the node to begin with. The stack holds the children of each node last first, so that the
   next one taken is the one a reader sees first and the text is joined in the order it is shown.
   */
  const pending: TreeNode[] = ('children' in node ? (node.children as readonly TreeNode[]) : []).toReversed();
  for (let met = pending.pop(); met !== undefined; met = pending.pop()) {
    if (!TEXT_NODE_TYPES.has(met.type,))
      return [];
    if (met.type === 'break')
      shown.push(' ',);
    else if ((met.type === 'image') || (met.type === 'imageReference'))
      shown.push(met.alt ?? '',);
    else if ('value' in met)
      shown.push(met.value,);
    if ('children' in met)
      pending.push(...(met.children as readonly TreeNode[]).toReversed(),);
  }
  return [shown.join('',),];
}

/**
 Whether the label of a link ends in an open mark: its last shown character is neither a closing mark nor a Latin letter or digit
 (or a compatibility form of one). The `)` or `]` that closes such a link is no closing bracket to the apostrophe after it, as the
 label in plain text would be none (`kit!'s`, `Kit.'s`, `kit 's`); a label that ends in a closing mark (`[kit)](u)`) closes as
 the mark does, and one that ends in a letter or a digit makes the apostrophe after it a joint of the word.

 @param node - a link, a link reference, an image or an image reference

 @returns Whether the label shows something and its last character is an open mark; false for a label that shows nothing and for a
 label whose shown text the tree does not say

 @example
 ```ts
 const open = linkLabelEndsOpenOf({ node, },); // true for `[kit!](u)` and `[-](u)`, false for `[kit](u)` and `[kit)](u)`
 ```
 */
export function linkLabelEndsOpenOf({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The shown text of the label as a list of zero or one: none where the tree does not say what the label shows.
   */
  const [shown,] = labelTextOf({ node, },);
  /**
   The characters the label shows, none for a label that shows nothing or whose text the tree does not say.
   */
  const characters = Array.from(shown ?? '',);
  /**
   The last of them.
   */
  const last = characters.at(-1,);
  if (last === undefined)
    return false;
  if (isClosingMark(last,))
    return false;
  return !continuesLatinWordOrCompatibilityForm({ character: last, },);
}

/**
 What the label of a link, a reference or an image shows a reader.

 @param node - a link, a link reference, an image or an image reference

 @returns `empty` for a label that shows nothing, `closing` for one that shows closing round or square brackets and closing curly double
 quotation marks and nothing else, `marks` for one that shows marks and spaces, no Latin letter or digit, and no closing mark last, `other`
 for every other label

 @example
 ```ts
 const kind = linkLabelKindOf({ node, },); // 'empty' for `[](u)`, 'marks' for `[-](u)`, 'other' for `[the cat](u)`
 ```
 */
export function linkLabelKindOf({ node, }: { readonly node: TreeNode; },): LinkLabelKind {
  /**
   The shown text of the label as a list of zero or one: none where the tree does not say what the label shows.
   */
  const [shown,] = labelTextOf({ node, },);
  if (shown === undefined)
    return 'other';
  /**
   The characters the label shows.
   */
  const characters = Array.from(shown,);
  /**
   The last character the label shows, which the apostrophe after the link follows, none for a label that shows nothing.
   */
  const last = characters.at(-1,);
  if (last === undefined)
    return 'empty';
  if (characters.some(function continuesWord(character,): boolean {
    return continuesLatinWordOrCompatibilityForm({ character, },);
  },))
    return 'other';
  if (characters.every(isClosingMark,))
    return 'closing';
  return (isClosingMark(last,)) ? 'other' : 'marks';
}

//endregion Consistency link label
