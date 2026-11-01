/**
 Client-side fallback that reorders the direct children of every
 `<shuffle-children>` element in browsers that do not yet support CSS `random()`.
 
 The {@link ShuffleChildren} component sets `display: flex` on `<shuffle-children>`
 and `order: random(1, 1000, 1)` on each direct child to shuffle visually.
 Browsers that do not support CSS `random()` drop the `order` declaration and
 every child falls back to the default `order: 0`, leaving them in source order,
 which defeats the shuffle.
 
 This script detects the missing support via `CSS.supports` and reorders the
 DOM nodes directly so the shuffle still happens. In supporting browsers the
 script is a no-op; the CSS path takes effect and runs off the main thread
 without layout shift.
 
 The DOM-reorder happens at module-load time. The bundle entry that imports
 this file is loaded with `type="module"`, which is render-blocking but
 executed before first paint, so the reorder applies before the user sees
 any of the wrapped content.
 
 @example
 ```ts
 import './shuffle-children.ts';
 ```
 */

export {}; // module boundary marker

/**
 Probe used to detect whether the browser supports `random()` on `order`.

 Must stay spelled the way `cssRandom` emits, or this probe answers a different
 question than the one that matters. It previously read `random(1, 1000, by 1)`,
 the grammar from the stale W3C `/TR/` snapshot, which no engine accepts, so the
 probe reported false everywhere and this script always reordered the DOM.
 See `doc/troubleshooting/css-random.md`.
 */
const RANDOM_PROBE = 'random(1, 1000, 1)';

/**
 Reorders the direct children of each `<shuffle-children>` wrapper using
 `Math.random()` ranks. No-op when CSS `random()` is supported on `order`.
 
 Ranks attach via `map`, the array is sorted by rank, then ranks are dropped
 and the elements are reinserted via `replaceChildren`. For small sibling
 counts the worst-case bias of rank-then-sort is negligible.
 */
function shuffleChildren(): void {
  if (CSS.supports(
    'order',
    RANDOM_PROBE,
  )) {
    return;
  }

  /**
   Wrapper elements opting into per-render shuffling.
   */
  const wrappers = document.querySelectorAll<HTMLElement>(
    'shuffle-children',
  );

  for (const wrapper of wrappers) {
    /**
     Snapshot of current child order before shuffling.
     */
    const children = [...wrapper.children,];
    if (children.length
      === 0)
      continue;

    /**
     Child paired with immutable random rank for sorting.
     */
    type RankedChild = Readonly<{
      child: Element;
      rank: number;
    }>;
    /**
     Permutation produced by attaching a random rank then sorting by it.
     */
    const shuffled = children
      .map(function attachRank(child,): RankedChild {
        return {
          child,
          rank: Math.random(),
        };
      },)
      .toSorted(function byRank(
        left,
        right,
      ) {
        return left.rank
          - right
            .rank;
      },)
      .map(function dropRank({ child, },) {
        return child;
      },);

    wrapper.replaceChildren(...shuffled,);
  }
}

shuffleChildren();
