import {
  dirname,
  resolve,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  asCssSource,
  parseCss,
  stringifyNodes,
} from '@monochromatic-dev/css-core-edit/ts';
import { readCssFile, } from './fs.ts';
import { inlineCssImports, } from './import.ts';
import {
  collectMixins,
  expandApplyRules,
  expandMixinRegistry,
} from './mixin.ts';

/**
 Build options for the CSS file pipeline.
 */
export type CssBuildOptions = {
  /**
   Input CSS file path.
   */
  readonly input: string;
  /**
   Output CSS file path.
   */
  readonly output: string;
};

/**
 Builds a CSS bundle from an entry file: inlines `\@import` rules with
 monorepo-aware resolution (relative paths, package.json `exports`, bare
 `node_modules` specifiers), expands `\@mixin`/`\@apply`, and writes the
 result.
 
 Untouched CSS survives byte-exactly, comments and author formatting
 included; spliced mixin bodies keep their definition-site formatting.
 
 @param options - Input and output file paths.
 
 @returns Processed CSS text, also written to `output`.
 
 @throws When an import cannot be resolved, a mixin reference is invalid,
 or an input file is not parseable CSS.
 
 @example
 ```ts
 const css = await buildCss({ input: 'src/main.css', output: 'dist/main.css' });
 ```
 */
export async function buildCss(options: CssBuildOptions,): Promise<string> {
  /**
   Destructured upfront so input and output paths are visible to the whole flow.
   */
  const {
    input,
    output,
  } = options;

  /**
   Absolute path to the CSS entry point.
   */
  const inputPath = resolve([input,],);

  /**
   Entry stylesheet with every import inlined.
   */
  const bundled = inlineCssImports({
    root: parseCss({
      source: asCssSource(await readCssFile(inputPath,),),
    },)
      .root,
    fromFile: inputPath,
    imported: new Set([inputPath,],),
  },);

  /**
   Bundled sheet with definitions stripped into the registry.
   */
  const collected = collectMixins({ root: bundled, },);

  /**
   Final CSS with all imports inlined and mixins expanded.
   */
  const result = stringifyNodes({
    nodes: expandApplyRules({
      root: collected.root,
      mixins: expandMixinRegistry({ mixins: collected.mixins, },),
    },)
      .children,
  },);

  // Write output: uses dynamic import so browser callers don't pull in node:fs
  /**
   Dynamic import keeps `node:fs/promises` out of browser bundles.
   */
  const {
    mkdir,
    writeFile,
  } = await import('node:fs/promises');
  /**
   Absolute path for the output file.
   */
  const outputPath = resolve([output,],);
  await mkdir(
    dirname(outputPath,),
    { recursive: true, },
  );
  await writeFile(
    outputPath,
    result,
  );

  return result;
}
