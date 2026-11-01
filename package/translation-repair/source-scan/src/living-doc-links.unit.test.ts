/**
 Guards against relative links in the living docs that lead nowhere (ledger
 D32). A page's link markup quoted in prose without a code span, such as a
 title linked to a placeholder `url` or an ellipsis, renders as a live link to
 a file that does not exist; fifteen stood in the pass log, the snapshot and a
 planning record, and no linter reads a link's target. A doc that moves or a
 heading that is renamed leaves the same kind of dead link behind.

 WHAT IS READ is every link, image and link definition the package's own
 Markdown parser finds, so a code span or an escaped bracket, which renders no
 link, is never read. An absolute URL is left alone. A relative target must
 name a file that exists beside the linking doc, and a fragment into a
 Markdown doc must name one of its headings by the id GitHub gives it.

 THE FIXTURES COME FIRST, so the living-docs case is read against a check
 shown able to find each shape (ledger M21). Fixtures are cat-themed.

 @module
 */

import {
  existsSync,
  readFileSync,
} from 'node:fs';
import { readFile, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import type {
  Nodes,
  Root,
} from 'mdast';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseMarkdownBody, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  readLivingRepositoryDocs,
  readPackageDocPaths,
  REPOSITORY_ROOT,
} from './living-docs.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 What the check asks of the docs a link can reach, so fixtures need no disk.
 */
type DocReader = {
  /**
   Whether a path from the repository root names an existing file.
   */
  readonly exists: (path: string) => boolean;

  /**
   Heading ids of the Markdown doc at a path from the repository root.
   */
  readonly headingIds: (path: string) => ReadonlySet<string>;
};

/**
 Every node of a tree, parents before children.

 @param node - tree's top node

 @returns Nodes in document order

 @example
 ```ts
 const nodes = allNodes({ node: parseMarkdownBody({ body: '# Naps', },), },);
 ```
 */
function allNodes({ node: top, }: { readonly node: Nodes; },): readonly Nodes[] {
  /**
   Nodes met so far, which is what the function returns.
   */
  const met: Nodes[] = [];
  /**
   Nodes still to visit, the next on top.
   */
  const pending: Nodes[] = [top,];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const node = pending.pop();
    if (node === undefined)
      break;
    met.push(node,);
    if ('children' in node)
      pending.push(...(node.children as readonly Nodes[]).toReversed(),);
  }
  return met;
}

/**
 The id GitHub gives a heading: its text lowercased, every character that is
 not a letter, mark, digit, hyphen, underscore or space dropped, and each space
 turned into a hyphen.

 @param text - heading's visible text

 @returns Its id, before any suffix a repeated heading takes

 @example
 ```ts
 headingId({ text: 'Naps, at noon', },); // 'naps-at-noon'
 ```
 */
function headingId({ text, }: { readonly text: string; },): string {
  // By code point, not grapheme: GitHub's slugger drops each code point its
  // class does not keep, so an accent written as a combining mark stays.
  return Array.from(text.toLowerCase(),)
    .filter(function isKept(character,): boolean {
      // oxlint-disable-next-line no-restricted-syntax/no-regex -- Unicode letter, mark and number classes have no string-API equivalent, and the docs carry Han and Latin headings; input is ONE character and the class has no quantifier or alternation, so it cannot backtrack.
      return (character === ' ') || /[\p{L}\p{M}\p{N}_-]/u.test(character,);
    },)
    .join('',)
    .replaceAll(' ', '-',);
}

/**
 Heading ids of one parsed doc, a repeated heading's id suffixed `-1`, `-2`
 and on as GitHub does.

 @param root - parsed doc

 @returns Every id a fragment can name

 @example
 ```ts
 headingIdsOf({ root: parseMarkdownBody({ body: '# Naps\n\n## Naps\n', },), },); // naps, naps-1
 ```
 */
function headingIdsOf({ root, }: { readonly root: Root; },): ReadonlySet<string> {
  /**
   Times each plain id has been given, to suffix the next.
   */
  const seen = new Map<string, number>();
  return new Set(allNodes({ node: root, },)
    .filter(function isHeading(node,): boolean {
      return node.type === 'heading';
    },)
    .map(function idOf(heading,): string {
      /**
       Visible text: every text and code value under the heading.
       */
      const text = allNodes({ node: heading, },)
        .map(function valueOf(node,): string {
          return ((node.type === 'text') || (node.type === 'inlineCode')) ? node.value : '';
        },)
        .join('',);
      /**
       Id before any suffix.
       */
      const plain = headingId({ text, },);
      /**
       Times this id was given before.
       */
      const count = seen.get(plain,) ?? 0;
      seen.set(plain, count + 1,);
      return (count === 0) ? plain : `${plain}-${String(count,)}`;
    },),);
}

/**
 Every relative link in one doc that leads nowhere, each as
 `path:line target`.

 @param path - doc's path from the repository root

 @param text - doc's text

 @param reader - what the docs a link reaches hold

 @returns Dead links in document order

 @example
 ```ts
 const dead = deadLinks({ path: 'doc/cat.md', text, reader, },);
 ```
 */
function deadLinks(
  {
    path,
    text,
    reader,
  }: {
    readonly path: string;
    readonly text: string;
    readonly reader: DocReader;
  },
): readonly string[] {
  /**
   The doc parsed.
   */
  const root = parseMarkdownBody({ body: text, },);
  /**
   The doc's own heading ids, which a bare fragment names.
   */
  const ownIds = headingIdsOf({ root, },);
  return allNodes({ node: root, },).flatMap(function dead(node,): readonly string[] {
    if (((node.type !== 'link') && (node.type !== 'image') && (node.type !== 'definition')) || URL.canParse(node.url,))
      return [];
    /**
     Where the fragment opens, or the end.
     */
    const hash = node.url.includes('#',) ? node.url.indexOf('#',) : node.url.length;
    /**
     Fragment named, empty when none.
     */
    const fragment = decodeURIComponent(node.url.slice(hash + 1,),);
    /**
     Target file from the repository root, empty for a bare fragment.
     */
    const target = (hash === 0) ? '' : join(dirname(path,), decodeURIComponent(node.url.slice(0, hash,),),);
    /**
     Whether the link reaches what it names.
     */
    const reaches = (target === '')
      ? ownIds.has(fragment,)
      : (reader.exists(target,)
        && ((fragment === '') || (!target.endsWith('.md',)) || reader.headingIds(target,).has(fragment,)));
    return reaches ? [] : [`${path}:${String(node.position?.start.line ?? 0,)} ${node.url}`,];
  },);
}

/**
 A reader over the repository on disk, parsing each linked doc once.

 @returns Reader answering from the working tree

 @example
 ```ts
 const reader = diskReader();
 ```
 */
function diskReader(): DocReader {
  /**
   Heading ids of each doc parsed so far.
   */
  const parsed = new Map<string, ReadonlySet<string>>();
  return {
    exists(path,): boolean {
      return existsSync(join(REPOSITORY_ROOT, path,),);
    },
    headingIds(path,): ReadonlySet<string> {
      /**
       Ids parsed on an earlier call.
       */
      const known = parsed.get(path,);
      if (known !== undefined)
        return known;
      /**
       Ids of this doc.
       */
      const ids = headingIdsOf({
        root: parseMarkdownBody({ body: readFileSync(join(REPOSITORY_ROOT, path,), 'utf8',), },),
      },);
      parsed.set(path, ids,);
      return ids;
    },
  };
}

/**
 Every doc the guard reads: the living repository-level docs, the package's
 docs and its README.

 @returns Paths from the repository root

 @throws {@link Error} when the living docs cannot be located, since the guard
 would then read less than it claims

 @example
 ```ts
 const paths = await readLinkingDocs();
 ```
 */
async function readLinkingDocs(): Promise<readonly string[]> {
  /**
   Living repository-level docs.
   */
  const { decisionRecords, handover, currentPlanning, operations, } = await readLivingRepositoryDocs();
  return [
    ...decisionRecords,
    ...handover,
    ...currentPlanning,
    ...operations,
    ...await readPackageDocPaths(),
  ];
}

/**
 Fixture docs by path, standing in for the disk.
 */
const CAT_DOCS: Readonly<Record<string, string>> = {
  'doc/naps.md': '# Naps\n\n## Naps, at noon\n\n## Naps, at noon\n\n## The `purr` rule\n',
  'doc/bowl.json': '{}',
};

/**
 Reader over {@link CAT_DOCS}.
 */
const CAT_READER: DocReader = {
  exists(path,): boolean {
    return path in CAT_DOCS;
  },
  headingIds(path,): ReadonlySet<string> {
    return headingIdsOf({ root: parseMarkdownBody({ body: CAT_DOCS[path] ?? '', },), },);
  },
};

await describe({
  name: 'relative links in the living docs that lead nowhere',
  children: [
    it({
      name: 'FINDS a link, an image and a definition to a missing file and a fragment no heading carries, and '
        + 'passes a code span, an escaped bracket, an absolute URL, a repeated heading, a code heading and a JSON target',
      fn: async () => {
        expect(deadLinks({
          path: 'doc/cat.md',
          text: [
            'The bowl [refills](url) at noon, and ![a whisker](whisker.png) fell.',
            'Quoted as `《[a nap](url)》` or as \\[a nap](...), both render no link.',
            'See [the vet](https://example.com/vet) and [the naps](naps.md).',
            'Noon is [here](naps.md#naps-at-noon) and [again](naps.md#naps-at-noon-1),',
            'the rule [there](naps.md#the-purr-rule) and the bowl [as data](bowl.json#food);',
            '[no such nap](naps.md#naps-at-dusk) and [no such cat](#tabby), but [this cat](#cat).',
            '',
            '# Cat',
            '',
            '[kibble]: kibble.md',
          ].join('\n',),
          reader: CAT_READER,
        },),).toEqual([
          'doc/cat.md:1 url',
          'doc/cat.md:1 whisker.png',
          'doc/cat.md:6 naps.md#naps-at-dusk',
          'doc/cat.md:6 #tabby',
          'doc/cat.md:10 kibble.md',
        ],);
      },
    },),
    it({
      name: 'GIVES a heading GitHub\'s id: lowercase, punctuation dropped, Han kept, spaces as hyphens',
      fn: async () => {
        expect(headingId({ text: 'Naps, at noon (2026-09-29)', },),).toBe('naps-at-noon-2026-09-29',);
        expect(headingId({ text: '猫 naps', },),).toBe('猫-naps',);
        expect(headingId({ text: 'The `purr` rule', },),).toBe('the-purr-rule',);
      },
    },),
    it({
      name: 'FINDS NO DEAD RELATIVE LINK in the living repository-level docs, the package\'s docs or its README',
      fn: async () => {
        /**
         Every doc the guard reads.
         */
        const paths = await readLinkingDocs();
        expect(paths.some(function isPassLog(path,): boolean {
          return path.includes('translation-repair-openrouter-2026-09-03-proposal-',);
        },),).toBe(true,);
        /**
         Reader over the working tree.
         */
        const reader = diskReader();
        expectNoFindings({
          findings: (await Promise.all(paths.map(async function inDoc(path,): Promise<readonly string[]> {
            return deadLinks({ path, text: await readFile(join(REPOSITORY_ROOT, path,), 'utf8',), reader, },);
          },),)).flat(),
        },);
      },
    },),
  ],
},);
