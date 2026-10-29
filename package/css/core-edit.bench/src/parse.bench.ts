/**
 Parse plus stringify benchmark: css-edit against postcss and css-tree on a
 synthetic stylesheet shaped like this repository's real CSS (custom
 properties, nesting, media queries, unknown at-rules). Run with the `bench`
 task; mitata reports timings.
 
 @module
 */

import {
  asCssSource,
  parseCss,
  stringifyCss,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  generate as cssTreeGenerate,
  parse as cssTreeParse,
} from 'css-tree';
import {
  bench,
  do_not_optimize,
  run,
  summary,
} from 'mitata';
import { parse as postcssParse, } from 'postcss';

//region Input

/**
 Number of component blocks in the benchmark input.
 */
const BLOCK_COUNT = 300;

/**
 Modulo spreading padding values across blocks.
 */
const PADDING_STEPS = 5;

/**
 Base rem width for generated media queries.
 */
const MEDIA_BASE_REM = 20;

/**
 Modulo spreading media-query widths across blocks.
 */
const MEDIA_SPREAD = 40;

/**
 Builds one component-shaped block: custom properties, declarations,
 `&` nesting, a nested media query, and an adversarial string.
 
 @param index - Block discriminator.
 
 @returns CSS text of one block.
 */
function buildBlock({
  index,
}: {
  readonly index: number;
},): string {
  return `
/* component ${String(index,)} */
.component-${String(index,)} {
  --local-accent: var(--accent, rebeccapurple);
  display: flex;
  padding: ${String(index % PADDING_STEPS,)}rem 1rem;
  background: url("assets/bg-${String(index,)}.png");
  &:hover { color: var(--local-accent); }
  .label { font-weight: bold; content: "{;}"; }
}
@media (width > ${String(MEDIA_BASE_REM + (index % MEDIA_SPREAD),)}rem) {
  .component-${String(index,)} { gap: 1px; }
}
`;
}

/**
 Full benchmark stylesheet.
 */
const INPUT = Array
  .from(
  { length: BLOCK_COUNT, },
  function blockAt(
    _unused,
    index,
  ) {
    return buildBlock({ index, },);
  },
  )
  .join('\n',);

console.log(`input: ${String(INPUT.length,)} chars, ${String(BLOCK_COUNT,)} blocks`,);

//endregion Input

//region Benchmarks

summary(function parseStringifyBenchmarks() {
  bench('css-edit parse+stringify', function runCssEdit() {
    do_not_optimize(
      stringifyCss({ state: parseCss({ source: asCssSource(INPUT,), },), },),
    );
  },);

  bench('postcss parse+toString', function runPostcss() {
    do_not_optimize(
      postcssParse(INPUT,)
        .toString(),
    );
  },);

  bench('css-tree parse+generate', function runCssTree() {
    do_not_optimize(
      cssTreeGenerate(cssTreeParse(INPUT,),),
    );
  },);
},);

await run();

//endregion Benchmarks
