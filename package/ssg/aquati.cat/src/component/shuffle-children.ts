/**
 MDX wrapper that shuffles the visual order of its direct children on each render.
 
 Modern browsers achieve the shuffle via CSS `order: random(...)` on each direct
 child; browsers that do not yet support CSS `random()` fall back to the
 DOM-reordering script in `src/client/shuffle-children.ts`. The component itself
 emits no JavaScript: the static markup is correct in source order, and the
 shuffle is either a pure CSS effect (modern path) or a one-time DOM mutation
 at module-load time (legacy path).
 
 Use to randomise quiz question order, photo grids, testimonials, or anything
 whose siblings should appear in a different order each page load.
 
 @example
 ```mdx
 <ShuffleChildren>
   <QuestionRadio ... />
   <QuestionRadio ... />
   <QuestionRadio ... />
 </ShuffleChildren>
 ```
 */
import {
  cssRandom,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  jsx,
  type SafeHtml,
} from '../lib/jsx-to-html.ts';

/**
 Props for {@link ShuffleChildren}.
 */
type ShuffleChildrenProps = {
  /**
   Direct children whose visual order is randomised on each render.
   */
  readonly children: unknown;
};

/**
 Renders a `<shuffle-children>` wrapper whose direct children appear in a
 fresh random order each time the page is rendered.
 
 @param props - component props
 
 @returns rendered `<shuffle-children>` element
 
 @example
 ```ts
 ShuffleChildren({ children: [a, b, c] })
 ```
 */
export function ShuffleChildren(
  props: ShuffleChildrenProps,
): SafeHtml {
  return jsx(
    'shuffle-children',
    {
      'data-is': '',
      children: props.children,
    },
  );
}

//region CSS

/**
 Upper bound of the per-child `order` integer used by the CSS shuffle path.
 
 Wide enough that collisions between sibling children are statistically negligible
 (for example, about 0.1% chance of any two among five siblings drawing the same
 integer), so the rendered order is effectively a fresh shuffle per render in
 browsers that support CSS `random()`. Lower bound and step are the exempt
 literal `1`.
 */
const SHUFFLE_ORDER_MAX = 1_000;

/**
 Structural styles for the shuffle-children wrapper.
 
 Lays out direct children in a flex column so the `order` declaration is honoured,
 then assigns each child a fresh `random()` value per render. Browsers that do not
 yet support CSS `random()` drop the `order` declaration entirely and the
 `src/client/shuffle-children.ts` script reorders the DOM nodes instead.
 
 @returns CSS string for the shuffle-children component
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return $({
    rule: 'shuffle-children',
    decls: {
      display: 'flex',
      'flex-direction': 'column',
    },
    children: [
      $({
        rule: '> *',
        decls: {
          order: cssRandom({
            min: 1,
            max: SHUFFLE_ORDER_MAX,
            step: 1,
          },),
        },
      },),
    ],
  },);
}

//endregion CSS
