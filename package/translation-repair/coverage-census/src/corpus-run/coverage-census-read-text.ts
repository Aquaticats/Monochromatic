import { readFile, } from 'node:fs/promises';

//region Coverage census read text
// How the census reads a file of text, named once so a reader of several
// files can be handed a scripted one and a case chooses which read ends first.

/**
 Reads one file as text.

 @example
 ```ts
 const readText: ReadText = readUtf8Text;
 ```
 */
export type ReadText = (input: { readonly path: string; },) => Promise<string>;

/**
 Reads one file as UTF-8 text.

 @param path - census input the caller names, a bundle of the build or a
 source file, read whole because the census reads its lines: a bundle's to
 refuse a minified build, a source's to count them

 @returns The file's text, decoded as UTF-8 since the build writes its
 bundles and the package keeps its sources in UTF-8

 @throws The filesystem's own failure, unchanged, where the file cannot be read

 @example
 ```ts
 const text = await readUtf8Text({ path: '/cats/nap.mjs', },);
 ```
 */
export async function readUtf8Text({ path, }: { readonly path: string; },): Promise<string> {
  return await readFile(
    path,
    'utf8',
  );
}

//endregion Coverage census read text
