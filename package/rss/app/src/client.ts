// 104 lines: scroll observer + feed binding are tightly coupled; splitting loses the shared closure context
import {
  HALF,
  QUARTER,
  THREE_QUARTERS,
} from '@monochromatic-dev/const-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import * as v from 'valibot';

//region Scroll event observer: Tracks element visibility and dispatches custom scroll lifecycle events

/**
 Default visibility thresholds: both edges, quarters, half, and full coverage.
 */
const DEFAULT_THRESHOLD: readonly number[] = [
  0,
  QUARTER,
  HALF,
  THREE_QUARTERS,
  1,
];

/**
 Resolves a caller threshold into the writable shape `IntersectionObserverInit` requires.
 Copies array thresholds into a fresh mutable array and falls back to {@link DEFAULT_THRESHOLD}.
 
 @param threshold - Caller-supplied threshold, absent when the caller wants defaults
 
 @returns Writable threshold value accepted by the observer constructor
 
 @example
 ```ts
 const t = resolveThreshold(HALF);
 ```
 */
function resolveThreshold(
  threshold?: number | readonly number[],
): number | number[] {
  if (threshold === undefined)
    return [...DEFAULT_THRESHOLD,];
  if ((typeof threshold) === 'number')
    return threshold;
  return [...threshold,];
}

/**
 Attaches an IntersectionObserver to an element, dispatching custom events
 for scroll lifecycle transitions (enter, leave, half-visible, fully visible, scrolled out).
 Threshold configuration is resolved via {@link resolveThreshold}.
 
 @param scrollOptions - Element to observe and optional IntersectionObserver configuration
 
 @returns IntersectionObserver instance controlling the observation
 
 @mutates scrollOptions - `DOM commit 5796f716 dispatchEvent invokes listeners with event` through `scrollOptions.element`; `observer.observe` retains element observation.
 
 @example
 ```ts
 const observer = addScrollEvents({ element: myDiv });
 ```
 */
function addScrollEvents(scrollOptions: {
  readonly element: HTMLElement;
  readonly options?: {
    readonly root?: Element | Document;
    readonly rootMargin?: string;
    readonly threshold?: number | readonly number[];
  };
},): IntersectionObserver {
  /**
   Destructured inputs so the body reads without `scrollOptions.` prefix.
   */
  const {
    element,
    options,
  } = scrollOptions;
  /**
   Defaults merged with caller overrides so the observer sees a complete config.
   */
  const config: IntersectionObserverInit = {
    threshold: resolveThreshold(options?.threshold,),
    rootMargin: options?.rootMargin
      ?? '0px',
    ...((options?.root !== undefined) ? { root: options.root, } : {}),
  };

  /**
   Closure latch so `scrolledIn` fires exactly once per visibility cycle.
   */
  let wasFullyVisible = false;
  /**
   Closure cursor for ratio crossings so enter/leave events trigger on transition, not state.
   */
  let lastRatio = 0;

  /**
   IntersectionObserver bound to the closure state above so callbacks share lifecycle.
   */
  const observer = new IntersectionObserver(
    function onIntersect(entries: readonly IntersectionObserverEntry[],) {
      /**
       First entry per spec, used as the source of the ratio reading.
       */
      const [entry,] = entries;
      if (!entry) {
        console.error(`empty entries for observer; observed entry count: ${entries.length}`,);
        return;
      }
      /**
       Current intersection ratio used by every transition check below.
       */
      const ratio = entry.intersectionRatio;

      if ((ratio === 1) && (!wasFullyVisible)) {
        wasFullyVisible = true;
        element.dispatchEvent(new CustomEvent('scrolledIn',),);
      }

      if (wasFullyVisible && (ratio === 0)) {
        element.dispatchEvent(new CustomEvent('scrolledOut',),);
        wasFullyVisible = false;
      }

      if ((lastRatio === 0) && (ratio > 0))
        element.dispatchEvent(new CustomEvent('enterViewport',),);

      if ((lastRatio > 0) && (ratio === 0))
        element.dispatchEvent(new CustomEvent('leaveViewport',),);

      if ((ratio >= HALF) && (lastRatio < HALF))
        element.dispatchEvent(new CustomEvent('halfVisible',),);

      lastRatio = ratio;
    },
    config,
  );

  observer.observe(element,);
  return observer;
}

//endregion Scroll event observer

//region Feed element binding: Connects scroll events to the ignore API for auto-dismissal

/**
 All feed elements on the page, bound to scroll-based ignore behavior.
 
 @see {@link addScrollEvents} for the scroll lifecycle that triggers ignore calls
 */
const elements: NodeListOf<HTMLElement> = document.querySelectorAll<HTMLElement>(
  '.feed',
);
/**
 Binds one host-created feed element to observer and event-listener state.
 
 @param element - Feed element supplied by browser NodeList iteration.
 
 @mutates element - `DOM commit 5796f716 dispatchEvent invokes listeners with event`; `element.addEventListener` and `observer.observe` retain event or observation state.
 */
function bindScrollIgnore(element: HTMLElement,): void {
  addScrollEvents({ element, },);
  element.addEventListener(
    'scrolledOut',
    function onScrolledOut() {
      void (async function onScrolledOutAsync(): Promise<void> {
        try {
          console.error('scrolledOut',);
          /**
           Required metadata wrapper so a missing element fails loud, not silent.
           */
          const metadata = nonNullishOrThrow(
            element.querySelector<HTMLElement>('.feed__metadata',),
          );
          /**
           Required link anchor inside metadata so the href is guaranteed present.
           */
          const anchor: HTMLAnchorElement = nonNullishOrThrow(
            metadata.querySelector<HTMLAnchorElement>('.feed__link',),
          );

          /**
           Request body shape varies by whether the href is a valid URL.
           */
          const body: Record<string, string> = v
            .safeParse(
            v.pipe(
              v.string(),
              v.url(),
            ),
            anchor.href,
            )
            .success
            ? { link: anchor.href, }
            : { metadataOuterHtml: metadata.outerHTML, };

          /**
           Ignore-API response held so the ok check and text read share one Response.
           */
          const response = await fetch(
            `/api/ignore/new`,
            {
              method: 'POST',
              body: JSON.stringify(body,),
            },
          );
          if (!response.ok) {
            console.error(
              `ignore request failed`,
              response,
            );
            return;
          }
          /**
           Response text persisted for the success log line.
           */
          const text = await response.text();
          console.error(`ignored: ${text}`,);
          element.dataset
            .ignored = '';
        }
        catch (error: unknown) {
          console.error(
            `scrolledOut handler failed`,
            error,
          );
        }
      })();
    },
  );
}

elements.forEach(bindScrollIgnore,);

//endregion Feed element binding
