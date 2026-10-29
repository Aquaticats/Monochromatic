/**
 Guards against a count printed before a fixed plural (ledger B98): a report
 line wrote `${String(count,)} slices` and printed "1 slices", and a test
 pinned one such line as correct. Widened (ledger B109) to also
 guard a count before one or two adjectives and then a fixed plural
 ("1 cached slices"), which the original scan could not see. The package
 chooses the noun a count takes with `wordForCount` (`count-word.ts`).

 WHAT THE SCAN READS, in the package's source (tests and their fixtures are
 left out): every template literal where a `String(...)` interpolation is
 followed, after one space, by up to three words (letters and hyphens), where
 the first, second or third is a counted noun: a word the package already
 counts, that is the `many` form of some `wordForCount` call in the source,
 so the noun list grows as nouns are counted, AND is itself shaped like a
 plain plural (one lowercase word, hyphens allowed, ending in "s"). That
 shape is what tells a true counted noun ("slices") apart from a `many` form
 written for subject-verb agreement ("are", "were", "go", "survive", "sit",
 "follow", "carry") or a full sentence fragment ("entries that are",
 "headings of slices"): neither ends a count-and-noun phrase, and excluding
 them by shape, rather than by naming them, is what keeps "cache records are"
 and "of them are" from reading as counted nouns. A candidate word that is a
 preposition or determiner ("of", "these", "them") ends the read right there,
 since what follows is not a modifier of the count but a reference elsewhere
 ("N of these artifacts"); a modifier is letters and hyphens, its first
 letter capital or not, so a proper adjective such as "Latin" in "Latin
 tokens" is read as one. A word
 followed by `=` is a log field's name and is passed over. Reads are keyed
 `path#site: word` for a direct count-then-noun, or `path#site: modifiers
 noun` when one or two words sit between (the words exactly as printed,
 space-joined), where the site is the nearest enclosing named function or
 class (so a count inside a class's constructor keys to the class name, not
 to the module, which keeps two constructors in one file from sharing an
 exemption key). Out of the scan's reach: a noun after another interpolation,
 a noun after more than two modifier words, a modifier capitalized past its
 first letter, a noun split from its count across a `+` between two literals, and
 a noun no `wordForCount` call names yet. The listed exemptions are counts
 that cannot be one, and verbs after an index ("slice 3 claims").

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isAsciiAlphanumeric,
  isAsciiLetter,
  isAsciiLowerLetter,
} from '@monochromatic-dev/translation-repair-markup';
import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Count nouns
// The scan itself: which nouns the package counts, and which template
// literals print one after a bare count.

/**
 Node kinds that open a function, which names the site of the reads inside.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Whether every character of a text is a lowercase ASCII letter or a hyphen.

 @param text - text read

 @returns True for an empty text too, which each caller rules out itself
 */
function isLowercaseRun({ text, }: { readonly text: string; },): boolean {
  for (const character of text) {
    if (!(isAsciiLowerLetter({ character, },) || (character === '-')))
      return false;
  }
  return true;
}

/**
 Whether a `many` form reads as a plain counted plural: one lowercase word,
 single hyphens allowed between its letters, ending in "s" after a letter.
 Separates a true counted noun ("slices", "self-votes") from a `many` form
 written for subject-verb agreement ("are", "were", "go") or a full sentence
 fragment ("entries that are"), neither of which ends a count-and-noun phrase.

 @param word - `many` form as written

 @returns True for a word shaped like a plural noun
 */
function isPluralShaped({ word, }: { readonly word: string; },): boolean {
  if ((word.length < 2) || (!word.endsWith('s',)) || word.startsWith('-',) || word.endsWith('-s',))
    return false;
  return (!word.includes('--',)) && isLowercaseRun({ text: word, },);
}

/**
 Words that end a modifier read because what follows is a preposition or
 determiner pointing elsewhere, not a word describing the count ("N of these
 artifacts" names artifacts counted elsewhere, not N of them).
 */
const MODIFIER_STOP: ReadonlySet<string> = new Set([
  'of',
  'these',
  'them',
],);

/**
 Whether a word reads as a modifier between a count and its noun: lowercase
 letters and hyphens, after an optional capital. The capital admits a proper
 adjective ("3 Latin tokens"), which agrees with its count as any adjective
 does; a word capitalized past its first letter (an acronym such as "MDX")
 ends the read.

 @param word - word printed between the count and a candidate noun

 @returns True for a word shaped like a modifier
 */
function isModifierWord({ word, }: { readonly word: string; },): boolean {
  /**
   First character, the only one that may be a capital.
   */
  const first = word.charAt(0,);
  /**
   Characters that must be lowercase letters or hyphens: the whole word, or
   all of it after a leading capital.
   */
  const body = (isAsciiLetter({ character: first, },) && (!isAsciiLowerLetter({ character: first, },)))
    ? word.slice(1,)
    : word;
  return (body.length > 0) && isLowercaseRun({ text: body, },);
}

/**
 Counts printed before a fixed plural that cannot be one, keyed
 `path#site: word` for a direct count-then-noun or `path#site: modifiers
 noun` for a count before one or two modifier words and then a noun, each
 with why.
 */
const EXEMPT_COUNTS: ReadonlyMap<string, string> = new Map([
  [
    'artifact/artifact-change-sets.ts#readArtifactChangeSets: records',
    'a verb after a schema version',
  ],
  [
    'assembly/assembly-contract-fault.ts#assemblySentence: claims',
    'a verb after a slice index',
  ],
  [
    'translate-stage/candidate-select.ts#decideBestCandidate: ballots',
    'MIN_SELECTION_BALLOTS, a constant above one',
  ],
  [
    'artifact/corpus-run/artifact-two-lane-verify.ts#assertLedgerDescribesPreparation: calls',
    'a verb after a slice index',
  ],
  [
    'run/corpus-run/command-line.ts#repeatRefusal: times',
    'only flags written more than once are listed',
  ],
  [
    'coverage-census/corpus-run/coverage-probe-args.ts#readCoverageProbeArguments: candidates',
    'DEFAULT_CANDIDATE_CAP, a constant above one',
  ],
  [
    'bench/corpus-run/window-trial-slice.ts#runSliceArms: arms',
    'the length of TRIAL_ARM_SET, a constant above one',
  ],
  [
    'typography/count-word.ts#howOften: times',
    'the form for every count but one',
  ],
  [
    'stage/pair-media-adjacency.ts#toFinding: claims',
    'a verb after a section index',
  ],
  [
    'archive/prepare-block-pairing.ts#prepareBlockPairing: pairs',
    'a verb after a section index',
  ],
  [
    'slice-reading/span-contiguity.ts#assertSpanContiguity: cuts',
    'a verb after a slice index',
  ],
  [
    'slice-reading/span-contiguity.ts#assertSpanContiguity: spans',
    'a verb after a slice index',
  ],
  [
    'artifact-pool/corpus-run/artifact-pool-refusal.ts#MixedGenerationError: pipeline generations',
    'generationCount, guarded above one before this error is constructed (artifact-eligible.ts, selectEligible)',
  ],
  [
    'artifact-pool/corpus-run/artifact-pool-refusal.ts#MixedGenerationError: settled entries',
    'census.total, at least generationCount (itself guarded above one) since every generation group holds at '
      + 'least one entry',
  ],
  [
    'glossary/corpus-run/pipeline-digest.ts#<module>: lowercase hex characters',
    'DIGEST_LENGTH, a constant above one',
  ],
  [
    'slice-cache/corpus-run/slice-census-widest.ts#sliceCensusWidestLines: target chars',
    'PROBE_TIMEOUT_CHARS, a constant above one',
  ],
  [
    'corpus-source/preparation-identity.ts#<module>: lowercase hex characters',
    'DIGEST_LENGTH, a constant above one',
  ],
],);

/**
 The text a string literal node holds.

 @param node - node read

 @returns Its text, empty for any other node
 */
function literalText({ node, }: { readonly node: unknown; },): string {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  return (isTreeNode(inner,) && (inner.type === 'Literal') && ((typeof inner.value) === 'string'))
    ? inner.value
    : '';
}

/**
 Whether a node calls a function by name.

 @param node - node read

 @param name - function name

 @returns Whether it is such a call, parentheses aside
 */
function callsNamed(
  {
    node,
    name,
  }: {
    readonly node: unknown;
    readonly name: string;
  },
): boolean {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  return isTreeNode(inner,) && (inner.type === 'CallExpression')
    && (identifierName({ node: inner.callee, },) === name);
}

/**
 Every `many` form the source's `wordForCount` calls name.

 @param files - files read; tests among them are passed over

 @returns Plural forms
 */
function countedNouns({ files, }: { readonly files: readonly SourceText[]; },): ReadonlySet<string> {
  /**
   Forms found so far.
   */
  const found = new Set<string>();
  for (const file of files) {
    if (file.isTest)
      continue;
    for (const node of nodesUnder({ root: parseSource({ file, },).program, },)) {
      if ((node.type !== 'Property') || (identifierName({ node: node.key, },) !== 'many'))
        continue;
      /**
       The form, where the value is a plain string shaped like a plural.
       */
      const form = literalText({ node: node.value, },);
      if (isPluralShaped({ word: form, },))
        found.add(form,);
    }
  }
  return found;
}

/**
 Where a word starting at the second character ends: the first character
 past it that is neither an ASCII letter nor a hyphen. Read by UTF-16 unit,
 since only ASCII is tested and a surrogate half is never ASCII.

 @param text - text whose first character is the space before the word

 @returns Index just past the word
 */
function wordEnd({ text, }: { readonly text: string; },): number {
  for (let index = 1; index < text.length; index += 1) {
    /**
     The character read.
     */
    const character = text.charAt(index,);
    if (!(isAsciiLetter({ character, },) || (character === '-')))
      return index;
  }
  return text.length;
}

/**
 Up to three words a template literal prints after its interpolation at a
 position, where a single space separates the interpolation from the first
 word and each word from the next. Stops before a word that does not stand
 alone: a digit, an underscore or `=` right after it makes it part of a name
 or a log field's name rather than a modifier or a noun.

 @param template - template literal

 @param index - interpolation's position among the template's expressions

 @returns Words in order, empty where none follows directly
 */
function wordsAfter(
  {
    template,
    index,
  }: {
    readonly template: TreeNode;
    readonly index: number;
  },
): readonly string[] {
  /**
   The template's literal stretches.
   */
  const quasis = Array.isArray(template.quasis,) ? template.quasis : [];
  /**
   The stretch after the interpolation.
   */
  const after: unknown = quasis[index + 1];
  if (!isTreeNode(after,))
    return [];
  /**
   Its text as the template writes it.
   */
  const { value, } = after;
  /**
   The raw text, starting from right after the interpolation.
   */
  const raw = ((typeof value) === 'object') && (value !== null) && ('raw' in value) ? String(value.raw,) : '';
  /**
   Words collected so far.
   */
  const words: string[] = [];
  /**
   Unread text, whose first character is always the space before the next
   candidate word, same contract `wordEnd` reads.
   */
  let rest = raw;
  while ((words.length < 3) && rest.startsWith(' ',)) {
    /**
     Where the word after the leading space ends.
     */
    const end = wordEnd({ text: rest, },);
    if (end <= 1)
      break;
    /**
     What follows the word: a digit, an underscore or `=` makes it part of a
     name or a log field's name rather than a modifier or a noun.
     */
    const next = rest.charAt(end,);
    if (isAsciiAlphanumeric({ character: next, },) || (next === '_') || (next === '='))
      break;
    words.push(rest.slice(1, end,),);
    rest = rest.slice(end,);
  }
  return words;
}

/**
 Every count printed before a word the package counts with `wordForCount`,
 directly or after one or two modifier words, keyed `path#site: word` or
 `path#site: modifiers noun`, one entry per count.

 @param files - files read; tests among them are passed over

 @param nouns - plural forms to look for

 @returns Keys sorted, repeated once per count
 */
function fixedPlurals(
  {
    files,
    nouns,
  }: {
    readonly files: readonly SourceText[];
    readonly nouns: ReadonlySet<string>;
  },
): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Nodes still to visit, each with its enclosing named function.
     */
    const pending: {
      readonly node: TreeNode;
      readonly site: string;
    }[] = [{
      node: parseSource({ file, },).program,
      site: '<module>',
    },];
    for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
      /**
       Node visited now, and the site it sits in.
       */
      const {
        node,
        site,
      } = next;
      /**
       Enclosing named function or class for the node and its children: a
       class keeps two of its constructors from sharing one exemption key
       with the module or with each other.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,))
        ? identifierName({ node: node.id, },)
        : ((node.type === 'ClassDeclaration') && isTreeNode(node.id,))
          ? identifierName({ node: node.id, },)
          : site;
      if (node.type === 'TemplateLiteral') {
        /**
         The template's interpolations.
         */
        const expressions = Array.isArray(node.expressions,) ? node.expressions : [];
        expressions.forEach(function readCount(expression: unknown, index: number,) {
          if (!callsNamed({
            node: expression,
            name: 'String',
          },))
            return;
          /**
           Up to three words printed after it: a direct noun, or one or two
           modifiers and then a noun.
           */
          const words = wordsAfter({
            template: node,
            index,
          },);
          /**
           Modifier words read so far, between the count and a noun.
           */
          const modifiers: string[] = [];
          for (const word of words) {
            if (nouns.has(word,)) {
              found.push(`${file.path}#${here}: ${[...modifiers, word,].join(' ',)}`,);
              return;
            }
            if (MODIFIER_STOP.has(word,) || (!isModifierWord({ word, },)))
              return;
            modifiers.push(word,);
          }
        },);
      }
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
        };
      },),);
    }
  }
  return found.toSorted();
}

//endregion Count nouns

await describe({
  name: 'count nouns (ledger B98, B109)',
  children: [
    it({
      name: 'FINDS a String interpolation followed by a noun some wordForCount call counts, directly or after one '
        + 'or two modifier words (a proper adjective among them), in a function, a class or at module level, and '
        + 'leaves other nouns, a noun after a preposition or determiner, a noun after an acronym, a many-form '
        + 'shaped like a verb rather than a noun, a noun after other text and tests',
      fn: async () => {
        /**
         A file counting naps once and printing them twice with a fixed plural, directly and after one or two
         modifiers, in a function, at module level and in two classes, plus the forms that must NOT be read as a
         counted noun: a stopped preposition, an acronym, and a many-form shaped like a verb.
         */
        const litter = {
          path: 'litter.ts',
          text: [
            'export function naps(count: number,): string {',
            `  return \`\${String(count,)} naps, \${String(count,)} \${wordForCount({ count, one: 'nap', many: 'naps', },)}\`;`,
            '}',
            `export const purrs = \`\${String(2,)} naps and \${String(3,)} purrs, deep naps \${String(4,)} naps=\${String(5,)}\`;`,
            // ONE MODIFIER, POSITIVE: a lowercase word between the count and the noun.
            `export const napping = \`\${String(6,)} napping naps\`;`,
            // ONE CAPITALIZED MODIFIER, POSITIVE: a proper adjective agrees with its count as any adjective does.
            `export const siamese = \`\${String(7,)} Siamese naps\`;`,
            // ONE MODIFIER, NEGATIVE: a word capitalized past its first letter (an acronym) stops the read.
            `export const acronym = \`\${String(11,)} MDX naps\`;`,
            // TWO MODIFIERS, POSITIVE: two lowercase words between the count and the noun.
            `export const soundAsleep = \`\${String(8,)} fast asleep naps\`;`,
            // STOPPED: a preposition right after the count ends the read, so the noun past it is never reached.
            `export const referred = \`\${String(9,)} of these naps\`;`,
            // A many-form shaped like a verb, declared so `recordsLine` can show it is never read as a noun.
            'export function status(count: number,): string {',
            `  return wordForCount({ count, one: 'is', many: 'are', },);`,
            '}',
            // TWO MODIFIERS, NEGATIVE: "are" is a many-form `status` declares, but it is not shaped like a plural,
            // so this is read as no match at all, the same as the package's own "cache records are" false positive.
            `export const recordsLine = \`\${String(10,)} cache records are\`;`,
            // CLASS SITES: two classes printing the same modified phrase key to their own class name, not to each
            // other or to the module, which is what keeps two constructors in one file from sharing an exemption.
            'export class Carrier {',
            '  constructor(count: number,) {',
            `    const message = \`\${String(count,)} elderly naps\`;`,
            '    void message;',
            '  }',
            '}',
            'export class Keeper {',
            '  constructor(count: number,) {',
            `    const message = \`\${String(count,)} elderly naps\`;`,
            '    void message;',
            '  }',
            '}',
          ].join('\n',),
          isTest: false,
        };
        /**
         A test printing the same plural.
         */
        const catnip = {
          path: 'catnip.unit.test.ts',
          text: `export const line = \`\${String(1,)} naps\`;`,
          isTest: true,
        };
        /**
         The nouns the files count.
         */
        const nouns = countedNouns({
          files: [
            litter,
            catnip,
          ],
        },);
        expect([...nouns,],).toEqual(['naps',],);
        expect(fixedPlurals({
          files: [
            litter,
            catnip,
          ],
          nouns,
        },),).toEqual([
          'litter.ts#<module>: Siamese naps',
          'litter.ts#<module>: fast asleep naps',
          'litter.ts#<module>: napping naps',
          'litter.ts#<module>: naps',
          'litter.ts#Carrier: elderly naps',
          'litter.ts#Keeper: elderly naps',
          'litter.ts#naps: naps',
        ],);
      },
    },),
    it({
      name: 'PRINTS NO BARE COUNT BEFORE A FIXED PLURAL the package counts elsewhere, outside the named exemptions, '
        + 'and every exemption still names one',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        /**
         The nouns the package counts.
         */
        const nouns = countedNouns({ files, },);
        expect(nouns.has('slices',),).toBe(true,);
        /**
         Every bare count before one of them.
         */
        const plurals = fixedPlurals({
          files,
          nouns,
        },);
        expectNoFindings({
          findings: plurals.filter(function unexempt(key,): boolean {
            return !EXEMPT_COUNTS.has(key,);
          },),
        },);
        expectNoFindings({
          findings: [...EXEMPT_COUNTS.keys(),].filter(function stale(key,): boolean {
            return !plurals.includes(key,);
          },),
        },);
      },
    },),
  ],
},);
