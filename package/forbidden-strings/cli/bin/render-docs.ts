#!/usr/bin/env node
/**
 Render only scanner documentation through the repository's installed CommonMark-to-HTML-tree pipeline.
 */
import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';
import { pathToFileURL } from 'node:url';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 A node of the rendered HTML tree, as far as this script reads it.
 */
type HastNode = {
  readonly type: string;
  readonly value?: string;
  readonly tagName?: string;
  readonly children?: readonly HastNode[];
};

/**
 A function exported by an installed renderer package.
 */
type Callable = (...args: readonly unknown[]) => unknown;

/**
 The unified processor, as far as this script uses it.
 */
type Processor = {
  readonly use: (plugin: unknown) => Processor;
  readonly parse: (source: string) => unknown;
  readonly run: (tree: unknown) => Promise<unknown>;
};

/**
 Documents rendered, relative to the repository root.
 */
const documents: readonly string[] = [
  'package/forbidden-strings/cli/README.md',
  'package/forbidden-strings/cli.fuzz/README.md',
  'doc/handover/scanner-native-verification.md',
];

/**
 Check that a value is a function.

 @param value - an export of an installed package

 @returns true when the value can be called

 @example
 ```ts
 isCallable(() => 1);
 ```
 */
function isCallable(value: unknown): value is Callable {
  return (typeof value) === 'function';
}

/**
 Check that a value is something that can hold properties.

 @param value - a module namespace, a processor or a tree node

 @returns true when the value is a non-null object or function (unified hangs its methods on a function)

 @example
 ```ts
 isHolder({});
 ```
 */
function isHolder(value: unknown): value is object {
  if (value === null) return false;
  return ((typeof value) === 'object') || ((typeof value) === 'function');
}

/**
 Check that a value is a unified processor.

 @param value - what the `unified` export returned

 @returns true when the value has the three methods this script calls

 @example
 ```ts
 isProcessor({});
 ```
 */
function isProcessor(value: unknown): value is Processor {
  if (!isHolder(value)) return false;
  if (!(('use' in value) && isCallable(value.use))) return false;
  if (!(('parse' in value) && isCallable(value.parse))) return false;
  return ('run' in value) && isCallable(value.run);
}

/**
 Check that a value is a node of the rendered tree.

 @param value - a node or a tree returned by the renderer

 @returns true when the value is an object with a node type

 @example
 ```ts
 isNode({ type: 'text', value: 'a' });
 ```
 */
function isNode(value: unknown): value is HastNode {
  if (!isHolder(value)) return false;
  return ('type' in value) && ((typeof value.type) === 'string');
}

/**
 Resolve the incumbent pnpm-installed renderer without introducing or selecting another dependency.

 @param name - package name of the renderer, which pnpm installs once as `name@version`

 @param exportName - the export to return

 @returns the named export

 @throws Error when no or several versions are installed, or the export is missing

 @example
 ```ts
 const unified = await installed({ name: 'unified', exportName: 'unified' });
 ```
 */
async function installed({
  name,
  exportName,
}: {
  readonly name: string;
  readonly exportName: string;
}): Promise<Callable> {
  /**
   The pnpm store of the repository.
   */
  const store = resolve(
    import.meta.dirname,
    '../../../../node_modules/.pnpm',
  );
  /**
   The installed versions of the renderer.
   */
  const entries = (await readdir(store)).filter(function isRenderer(entry) {
    return entry.startsWith(`${name}@`);
  });
  if (entries.length !== 1) throw new Error(`Expected one installed ${name} renderer, found ${entries.length}.`);
  /**
   The renderer's module namespace.
   */
  const renderer: unknown = await import(pathToFileURL(join(
    store,
    entries[0] ?? '',
    'node_modules',
    name,
    'index.js',
  ))
    .href);
  if ((!isHolder(renderer)) || (!(exportName in renderer)))
    throw new Error(`The installed ${name} renderer has no ${exportName} export.`);
  /**
   The requested export.
   */
  const exported: unknown = Reflect.get(
    renderer,
    exportName,
  );
  if (!isCallable(exported)) throw new Error(`The installed ${name} renderer has no ${exportName} function.`);
  return exported;
}

/**
 Build the Markdown-to-HTML-tree processor.

 @returns a processor that parses CommonMark and converts it to an HTML tree

 @throws Error when a renderer is missing or does not build a processor

 @example
 ```ts
 const processor = await createProcessor();
 ```
 */
async function createProcessor(): Promise<Processor> {
  /**
   The unified factory.
   */
  const unified = await installed({
    name: 'unified',
    exportName: 'unified',
  });
  /**
   The Markdown parser plugin.
   */
  const parse = await installed({
    name: 'remark-parse',
    exportName: 'default',
  });
  /**
   The Markdown-to-HTML-tree plugin.
   */
  const html = await installed({
    name: 'remark-rehype',
    exportName: 'default',
  });
  /**
   The bare processor.
   */
  const bare = unified();
  if (!isProcessor(bare)) throw new Error('The installed unified renderer did not build a processor.');
  return bare.use(parse)
    .use(html);
}

/**
 Names of the heading elements the check counts.
 */
const headingTags: ReadonlySet<string> = new Set([
  'h1',
  'h2',
  'h3',
  'h4',
]);

/**
 List every node of a rendered tree in document order.

 @param rendered - the root of the rendered tree

 @returns the root, then each node after the node that contains it, children in order

 @example
 ```ts
 const nodes = flatten(rendered);
 ```
 */
function flatten(rendered: HastNode): readonly HastNode[] {
  /**
   Nodes still to visit; the last is visited next.
   */
  const pending: HastNode[] = [rendered];
  /**
   Nodes visited, in visiting order.
   */
  const visited: HastNode[] = [];
  while (pending.length > 0) {
    /**
     The node being visited.
     */
    const node = pending.pop();
    if (node === undefined) continue;
    visited.push(node);
    if (node.children) pending.push(...[...node.children].toReversed());
  }
  return visited;
}

/**
 Collect the text of a rendered tree and count its headings.

 @param rendered - the root of the rendered tree

 @param file - the document the tree came from, for messages

 @returns the concatenated text, and the number of headings h1 to h4

 @throws Error when a text node still holds a literal bold delimiter

 @example
 ```ts
 const { text, headings } = readTree({ rendered, file: 'README.md' });
 ```
 */
function readTree({
  rendered,
  file,
}: {
  readonly rendered: HastNode;
  readonly file: string;
}): {
  readonly text: string;
  readonly headings: number;
} {
  /**
   Every node, in document order.
   */
  const nodes = flatten(rendered);
  /**
   Text values in document order.
   */
  const text = nodes
    .filter(function isText(node) {
      return node.type === 'text';
    })
    .map(function valueOf(node) {
      return node.value ?? '';
    });
  /**
   The first text value that still holds a literal bold delimiter, if any.
   */
  const unrendered = text.find(function hasDelimiter(value) {
    return value.includes('**');
  });
  if (unrendered !== undefined) throw new Error(`Unrendered emphasis delimiter in ${file}: ${unrendered}`);
  /**
   The heading elements.
   */
  const headings = nodes.filter(function isHeading(node) {
    return (node.type === 'element') && headingTags.has(node.tagName ?? '');
  });
  return {
    text: text.join(''),
    headings: headings.length,
  };
}

/**
 Render one document and retain its HTML element tree plus readable rendered text.

 @param processor - the Markdown-to-HTML-tree processor

 @param repository - root of the repository

 @param evidence - directory that receives the tree and the text

 @param file - the document to render, relative to the repository root

 @throws Error when the rendered document has no headings or an unrendered bold delimiter

 @example
 ```ts
 await renderDocument({ processor, repository, evidence, file: 'README.md' });
 ```
 */
async function renderDocument({
  processor,
  repository,
  evidence,
  file,
}: {
  readonly processor: Processor;
  readonly repository: string;
  readonly evidence: string;
  readonly file: string;
}): Promise<void> {
  /**
   The Markdown source.
   */
  const source = await readFile(
    join(
      repository,
      file,
    ),
    'utf8',
  );
  /**
   The rendered HTML tree.
   */
  const rendered = await processor.run(processor.parse(source));
  if (!isNode(rendered)) throw new Error(`The renderer returned no tree for ${file}.`);
  /**
   The tree's text and heading count.
   */
  const read = readTree({
    rendered,
    file,
  });
  if (read.headings === 0) throw new Error(`No rendered headings in ${file}.`);
  /**
   The file name stem of the evidence files.
   */
  const filename = file.replaceAll(
    '/',
    '_',
  );
  await writeFile(
    join(
      evidence,
      `${filename}.hast.json`,
    ),
    `${JSON.stringify(
      rendered,
      null,
      2,
    )}\n`,
  );
  await writeFile(
    join(
      evidence,
      `${filename}.txt`,
    ),
    `${read.text}\n`,
  );
  console.log(`${file}: rendered ${read.headings} headings; no literal bold delimiters.`);
}

/**
 Render the documentation and retain the HTML element tree plus readable rendered text.

 @throws Error when a renderer is missing or a document renders without headings or with a literal bold delimiter

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   The Markdown-to-HTML-tree processor.
   */
  const processor = await createProcessor();
  /**
   Root of the repository.
   */
  const repository = resolve(
    import.meta.dirname,
    '../../../..',
  );
  /**
   Directory that receives the evidence.
   */
  const evidence = resolve(
    import.meta.dirname,
    '../target/verification/docs',
  );
  await mkdir(
    evidence,
    { recursive: true },
  );
  await mapInSequence({
    items: documents,
    run: function renderOf(file) {
      return renderDocument({
        processor,
        repository,
        evidence,
        file,
      });
    },
  });
}

await main();
