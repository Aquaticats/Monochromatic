/**
 HTML asset reference extraction.
 
 Parses an HTML document with rehype-parse and walks the resulting hast tree
 to collect every local asset URL the page would cause a browser to fetch.
 */
import type {
  Element,
  Node,
  Parent,
  Root,
  Text,
} from 'hast';
import rehypeParse from 'rehype-parse';
import { unified, } from 'unified';

import type { DeepReadonly, } from './types.ts';
import { startsWithUriScheme, } from './url-detect.ts';

/**
 Sentinel for "this element exposes no candidate asset URL". Shared by the
 one cohesive flow that produces a fetchable URL or nothing: {@link attr}
 (missing/non-string/empty attribute), `firstSrcsetUrl` (empty srcset), and
 {@link ownAssetUrl} (tag carries no own asset). A `unique symbol`; callers
 narrow with `=== NO_ASSET_URL`.
 */
const NO_ASSET_URL: unique symbol = Symbol('page-weight/no-asset-url',);

/**
 Sentinel returned by {@link localUrlOrAbsent} when a candidate URL is not a
 local reference: absent, external scheme, protocol-relative, data URI, or
 fragment-only. A `unique symbol`; callers narrow with `=== NON_LOCAL_REF`.
 */
const NON_LOCAL_REF: unique symbol = Symbol('page-weight reference is not local',);

/**
 Sentinel returned by {@link inlineStyleText} when a `<style>` block has no
 non-blank text content. A `unique symbol`; callers narrow with
 `=== BLANK_STYLE`.
 */
const BLANK_STYLE: unique symbol = Symbol('page-weight style block has no text',);

/**
 Reusable unified parser configured for full HTML documents.
 */
const parser = unified()
  .use(rehypeParse,);

/**
 Parses an HTML source string into a hast root node.
 
 @param source - raw HTML text
 
 @returns hast root node
 */
function parseHtml(source: string,): Root {
  return parser.parse(source,) as Root;
}

/**
 Type-guards a generic hast node as an `Element` (has `tagName` and children).
 
 @param node - hast node to test
 
 @returns whether the node is an element
 */
function isElement(node: DeepReadonly<Node>,): node is DeepReadonly<Element> {
  return node.type
    === 'element';
}

/**
 Type-guards a generic hast node as a `Text` node.
 
 @param node - hast node to test
 
 @returns whether the node is a text node
 */
function isText(node: DeepReadonly<Node>,): node is DeepReadonly<Text> {
  return node.type
    === 'text';
}

/**
 Type-guards a generic hast node as a `Parent` (carries a `children` array).
 
 Replaces a `Partial<Parent>` view: `Partial` reopens every property as
 optional, which `no-optional-escape` bans, whereas a guard narrows precisely
 to the nodes that have children to descend into.
 
 @param node - hast node to test
 
 @returns whether the node has children to descend into
 */
function isParent(node: DeepReadonly<Node>,): node is DeepReadonly<Parent> {
  return ('children' in node)
    && Array.isArray(node.children,);
}

/**
 Reads a string attribute from a hast element.
 
 Returns {@link NO_ASSET_URL} for missing attributes, non-string values, or
 the empty string, simplifying caller logic that would otherwise have to
 narrow the return type and separately reject empty strings. Every attribute
 this module reads is URL-bearing, so the shared "no asset URL" sentinel
 names the absence accurately.
 
 @param element - source element
 
 @param name - attribute name
 
 @returns trimmed string value, or {@link NO_ASSET_URL}
 */
function attr(
  {
    element,
    name,
  }: {
    readonly element: DeepReadonly<Element>;
    readonly name: string;
  },
): string | typeof NO_ASSET_URL {
  /**
   Attribute value as hast stores it; may be missing or non-string.
   */
  const raw = element.properties[name];
  if ((typeof raw) !== 'string')
    return NO_ASSET_URL;
  /**
   Whitespace-stripped value so empty-after-trim attributes report as `NO_ASSET_URL`.
   */
  const trimmed = raw.trim();
  return trimmed === '' ? NO_ASSET_URL : trimmed;
}

/**
 Returns the first candidate URL from a `srcset` attribute value.
 
 Browsers evaluate the full candidate set before picking one, but the
 exact pick depends on viewport and DPR; the first candidate is a
 reasonable proxy for the "canonical" reference.
 
 The pick is read out with {@link firstNonWhitespaceToken}.
 
 @param srcset - raw srcset value, e.g. `"a.jpg 1x, b.jpg 2x"`
 
 @returns first URL, or {@link NO_ASSET_URL} if the value is empty
 */
function firstSrcsetUrl(srcset: string,): string | typeof NO_ASSET_URL {
  /**
   First candidate descriptor in the srcset list; used as the canonical pick.
   */
  const first = srcset.split(',',)[0]
    ?.trim();
  if ((first === undefined) || (first === ''))
    return NO_ASSET_URL;
  // `first` is trimmed and non-empty, so it carries at least one non-whitespace
  // character; the leading-token scan therefore always yields a non-empty URL.
  return firstNonWhitespaceToken(first,);
}

/**
 Whether `c` is one of the ASCII whitespace characters that bound a token:
 space, tab, newline, carriage return, form feed, vertical tab. Matches the
 exact set the prior recursive walkers tested (ASCII subset of regex `\s`),
 so a non-breaking space and other Unicode spaces are deliberately not
 treated as boundaries.
 
 @param c - single character under inspection
 
 @returns whether `c` is ASCII whitespace
 */
function isWhitespace(c: string,): boolean {
  return (c === ' ')
    || (c === '\t')
    || (c === '\n')
    || (c === '\r')
    || (c === '\f')
    || (c === '\v');
}

/**
 Returns the leading non-whitespace token of `line`. Single linear pass:
 skips leading whitespace, then runs to the next whitespace. Empty input or a
 whitespace-only line returns an empty token. Exported so its edge-case
 behavior can be covered by unit tests directly.
 
 @param line - input line
 
 @returns first non-whitespace token (possibly empty)
 
 @example
 ```ts
 firstNonWhitespaceToken('  a.jpg 2x'); // 'a.jpg'
 firstNonWhitespaceToken('   ');        // ''
 ```
 */
export function firstNonWhitespaceToken(line: string,): string {
  // Single linear pass over `line`: skip the leading-whitespace run, then
  // collect characters until the next whitespace. Each character is visited at
  // most once (O(n) time, O(1) stack, no recursion). Fragments are pushed and
  // joined once, mirroring the codebase's other linear string scanners.
  return (function scan(): string {
    /**
     Token characters collected after the leading-whitespace run ends; joined once at the end.
     */
    const chars: string[] = [];
    /**
     Whether the leading-whitespace run has ended and token capture has begun.
     */
    let started = false;
    for (const c of line) {
      if (isWhitespace(c,)) {
        if (started)
          break;
        continue;
      }
      started = true;
      chars.push(c,);
    }
    return chars.join('',);
  })();
}

/**
 Returns the local reference carried by a candidate URL, or
 {@link NON_LOCAL_REF} when the candidate is missing, external, a data URI,
 or fragment-only.
 
 The candidate is an optional `string`: producers that found no URL convert
 their {@link NO_ASSET_URL} sentinel to `undefined` at this seam rather than
 threading a foreign sentinel into this function's narrowing. External
 schemes are detected via {@link startsWithUriScheme}.
 
 @param raw - candidate URL, or `undefined` when the source element had none
 
 @returns trimmed local reference, or {@link NON_LOCAL_REF}
 */
function localUrlOrAbsent(raw?: string,): string | typeof NON_LOCAL_REF {
  if (raw === undefined)
    return NON_LOCAL_REF;
  /**
   Whitespace-stripped form so empty and fragment-only references are filtered out.
   */
  const trimmed = raw.trim();
  if ((trimmed === '') || trimmed
    .startsWith('#',))
    return NON_LOCAL_REF;
  if (trimmed.startsWith('//',)
    || startsWithUriScheme(trimmed,))
    return NON_LOCAL_REF;
  return trimmed;
}

/**
 Tag names whose child `<source>` / `<img>` elements are picked by the browser
 rather than all loaded. Handled specially to avoid over-counting.
 */
const MEDIA_PARENTS = new Set([
  'picture',
  'video',
  'audio',
],);

/**
 Returns an inline `<style>` element's text content, or {@link BLANK_STYLE}
 when the block is blank so the caller's accumulator stays clean.
 
 @param element - `<style>` element
 
 @returns concatenated stylesheet text, or {@link BLANK_STYLE} when blank
 */
function inlineStyleText(element: DeepReadonly<Element>,): string | typeof BLANK_STYLE {
  /**
   Text-node fragments collected from the `<style>` children.
   */
  const parts: string[] = [];
  for (const child of element.children) {
    if (isText(child,))
      parts.push(child.value,);
  }
  /**
   Concatenated `<style>` content; only emitted when non-blank.
   */
  const text = parts.join('',);
  return text.trim() === '' ? BLANK_STYLE : text;
}

/**
 Collects the URL a media-parent element (`<picture>`, `<video>`, `<audio>`)
 would cause a browser to fetch.
 
 Heuristic: the first `<source>` child with a `src` or `srcset` is treated as
 the canonical pick (browsers evaluate sources in source order and stop at
 the first match). When no `<source>` is present, the fallback `<img>` `src`
 is used for `<picture>`.
 
 @param element - the `<picture>` / `<video>` / `<audio>` element
 
 @returns single URL the media element most likely fetches (`url` omitted
   when there is none) plus any `<style>` contents seen among its children
   before that pick
 */
function collectMedia(
  element: DeepReadonly<Element>,
): {
  readonly url?: string;
  readonly styles: readonly string[];
} {
  /**
   Inline `<style>` bodies found among the media children, in source order.
   */
  const styles: string[] = [];
  for (const child of element.children) {
    if (!isElement(child,))
      continue;
    if (child.tagName
      === 'source') {
      /**
       `srcset` on the current `<source>` child, if any; preferred over `src`.
       */
      const srcset = attr({
        element: child,
        name: 'srcset',
      },);
      if (srcset !== NO_ASSET_URL) {
        /**
         Canonical first candidate of the `<source>` `srcset`; omitted when empty.
         */
        const url = firstSrcsetUrl(srcset,);
        return url === NO_ASSET_URL ? { styles, } : {
          url,
          styles,
        };
      }
      /**
       Plain `src` fallback for the current `<source>` child when no `srcset` is set.
       */
      const src = attr({
        element: child,
        name: 'src',
      },);
      if (src !== NO_ASSET_URL)
        return {
          url: src,
          styles,
        };
    }
    if (child.tagName
      === 'style') {
      /**
       Text of the current `<style>` child, when non-blank.
       */
      const text = inlineStyleText(child,);
      if (text !== BLANK_STYLE)
        styles.push(text,);
    }
  }
  if (element.tagName
    === 'picture') {
    for (const child of element.children) {
      if (isElement(child,)
        && (child.tagName
          === 'img')) {
        /**
         Fallback `<img>` `src` used when no `<source>` child matched.
         */
        const src = attr({
          element: child,
          name: 'src',
        },);
        if (src !== NO_ASSET_URL)
          return {
            url: src,
            styles,
          };
      }
    }
  }
  return { styles, };
}

/**
 Collects a single element's own asset URL (not recursing into children).
 
 Returned URL maps directly to a browser fetch. Children are walked
 separately by the main walker, except for media parents where a single
 pick is applied instead.
 
 @param element - element to inspect
 
 @returns URL to fetch, or {@link NO_ASSET_URL} when the element has no own
   asset
 */
function ownAssetUrl(element: DeepReadonly<Element>,): string | typeof NO_ASSET_URL {
  if (element.tagName
    === 'link') {
    return attr({
      element,
      name: 'href',
    },);
  }
  if (
    (element.tagName
      === 'script')
      || (element.tagName
        === 'iframe')
      || (element.tagName
        === 'embed')
      || (element.tagName
        === 'audio')
      || (element.tagName
        === 'video')
  ) {
    return attr({
      element,
      name: 'src',
    },);
  }
  if (element.tagName
    === 'object') {
    return attr({
      element,
      name: 'data',
    },);
  }
  if (element.tagName
    === 'img') {
    /**
     `<img>` `srcset`, preferred over `src` when set.
     */
    const srcset = attr({
      element,
      name: 'srcset',
    },);
    if (srcset !== NO_ASSET_URL)
      return firstSrcsetUrl(srcset,);
    return attr({
      element,
      name: 'src',
    },);
  }
  if (element.tagName
    === 'use') {
    /**
     `<use>` `href`, preferred over the legacy `xlink:href`.
     */
    const href = attr({
      element,
      name: 'href',
    },);
    if (href !== NO_ASSET_URL)
      return href;
    return attr({
      element,
      name: 'xlink:href',
    },);
  }
  return NO_ASSET_URL;
}

/**
 Walk result: the local asset URLs referenced by a single HTML document
 plus inline stylesheet text blocks (which need their own `url()` scan).
 */
export type HtmlReferences = {
  /**
   Relative/absolute asset URLs as written in the source HTML.
   */
  readonly urls: readonly string[];
  /**
   Raw CSS text from `<style>` blocks, pending `url()` extraction.
   */
  readonly inlineStyles: readonly string[];
};

/**
 Extracts every local asset reference from an HTML document.
 
 @param source - raw HTML text
 
 @returns asset URLs + inline stylesheet text for further scanning
 
 @example
 ```ts
 const { urls, inlineStyles } = extractHtmlRefs(html);
 ```
 */
export function extractHtmlRefs(source: string,): HtmlReferences {
  /**
   Parsed hast tree the walker descends.
   */
  const tree = parseHtml(source,);
  /**
   Dedup set built up during the walk; converted to an array on return.
   */
  const urls = new Set<string>();
  /**
   `<style>` bodies pulled out for a separate `url()` scan by the caller.
   */
  const inlineStyles: string[] = [];

  /**
   Recursively descends a hast subtree, recording asset URLs and inline
   styles into the captured accumulators. Media-parent elements short-circuit
   child recursion to avoid over-counting `<source>` / fallback `<img>`
   combinations a browser picks exactly one of. Closes over `urls` and
   `inlineStyles` so neither accumulator is threaded through a parameter.
   
   @param node - current hast node
   */
  function walk(node: DeepReadonly<Node>,): void {
    if (isElement(node,)) {
      if (node.tagName
        === 'style') {
        /**
         Text of this `<style>` block, when non-blank.
         */
        const text = inlineStyleText(node,);
        if (text !== BLANK_STYLE)
          inlineStyles.push(text,);
        return;
      }
      if (MEDIA_PARENTS.has(node.tagName,)) {
        /**
         Canonical media pick plus any inline styles found among its children.
         */
        const media = collectMedia(node,);
        /**
         Local reference of the media pick, or `NON_LOCAL_REF` when external/absent.
         */
        const local = localUrlOrAbsent(media.url,);
        if (local !== NON_LOCAL_REF)
          urls.add(local,);
        for (const style of media.styles)
          inlineStyles.push(style,);
        return;
      }
      /**
       This element's own candidate asset URL, or `NO_ASSET_URL` when it has none.
       */
      const own = ownAssetUrl(node,);
      /**
       Local reference of this element's own asset, or `NON_LOCAL_REF`.
       `NO_ASSET_URL` is converted to `undefined` at this seam so the
       producer's sentinel never reaches {@link localUrlOrAbsent}'s narrowing.
       */
      const local = localUrlOrAbsent(own === NO_ASSET_URL ? undefined : own,);
      if (local !== NON_LOCAL_REF)
        urls.add(local,);
    }
    if (isParent(node,)) {
      for (const child of node.children)
        walk(child,);
    }
  }

  walk(tree,);

  return {
    urls: [...urls,],
    inlineStyles,
  };
}
