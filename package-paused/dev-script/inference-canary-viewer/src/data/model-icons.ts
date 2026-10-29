/**
 * Vendor brand SVG icons from `@lobehub/icons-static-svg`.
 *
 * Icons are defined once as `<symbol>` elements in an SVG sprite sheet
 * ({@link renderSvgSprite}), then referenced via `<use href>` throughout
 * the page. This avoids duplicating full SVG markup at every data point.
 */
import claudeSvg from '@lobehub/icons-static-svg/icons/claude-color.svg' with {
  type: 'text',
};
import geminiSvg from '@lobehub/icons-static-svg/icons/gemini-color.svg' with {
  type: 'text',
};
import kimiSvg from '@lobehub/icons-static-svg/icons/kimi-color.svg' with {
  type: 'text',
};
import minimaxSvg from '@lobehub/icons-static-svg/icons/minimax-color.svg' with {
  type: 'text',
};
import openaiSvg from '@lobehub/icons-static-svg/icons/openai.svg' with { type: 'text', };
import qwenSvg from '@lobehub/icons-static-svg/icons/qwen-color.svg' with {
  type: 'text',
};
import zhipuSvg from '@lobehub/icons-static-svg/icons/zhipu-color.svg' with {
  type: 'text',
};

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Kimi SVG has white (#fff) fills that are invisible on light backgrounds,
 * so a dark square is prepended behind its paths.
 */
const kimiFixed = kimiSvg.replace(
  '<path',
  '<rect width="24" height="24" rx="3" fill="#888"/><path',
);

/**
 * Default viewBox used when an SVG omits the attribute on its root element.
 */
const DEFAULT_VIEWBOX = '0 0 24 24';

/**
 * Extracts the `viewBox` attribute value from the first opening `<svg>` tag.
 * Mirrors `/viewBox="([^"]+)"/.exec(raw)?.[1]` with a linear `indexOf` walk:
 * locate the literal `viewBox="`, then read until the next `"`.
 *
 * @param raw - raw SVG document
 *
 * @returns viewBox value, or {@link DEFAULT_VIEWBOX} when the attribute is absent
 */
function extractViewBox(raw: string,): string {
  /**
   * Position of the opening of the attribute literal; `-1` ends the search.
   */
  const open = raw.indexOf('viewBox="',);
  if (open === (-1))
    return DEFAULT_VIEWBOX;
  /**
   * Position of the closing quote; `-1` means the attribute is unterminated.
   */
  const close = raw.indexOf(
    '"',
    open + 'viewBox="'
      .length,
  );
  if (close === (-1))
    return DEFAULT_VIEWBOX;
  return raw.slice(
    open + 'viewBox="'
      .length,
    close,
  );
}

/**
 * Strips the opening `<svg ...>` tag from the front of `raw`. Mirrors
 * `raw.replace(/^<svg[^>]*>/, '')`: requires the string to start with
 * `<svg`, then drops everything through (and including) the next `>`.
 *
 * @param raw - raw SVG document
 *
 * @returns SVG body without the opening `<svg>` tag
 */
function stripOpeningSvgTag(raw: string,): string {
  if (!raw.startsWith('<svg',))
    return raw;
  /**
   * Position of the `>` that closes the opening tag; `-1` means malformed input.
   */
  const gt = raw.indexOf(
    '>',
    '<svg'.length,
  );
  if (gt === (-1))
    return raw;
  return raw.slice(gt + 1,);
}

/**
 * Strips the trailing `</svg>` tag (plus any trailing whitespace) from
 * `s`. Mirrors `s.replace(/<\/svg>\s*$/, '')`: trims trailing whitespace,
 * then drops the final `</svg>` if present.
 *
 * @param s - SVG body that may end with a closing tag
 *
 * @returns body without the trailing `</svg>`
 */
function stripClosingSvgTag(s: string,): string {
  /**
   * Trailing whitespace stripped; matches `\s*$` semantics of the prior regex.
   */
  const trimmed = s.trimEnd();
  if (!trimmed.endsWith('</svg>',))
    return trimmed;
  return trimmed.slice(
    0,
    -('</svg>'.length),
  );
}

/**
 * Result of {@link extractAndStripDefs}: hoisted defs body and the
 * surrounding markup with each `<defs>` block removed.
 */
type DefsPartition = {
  /**
   * Concatenated bodies of every `<defs>` block discovered.
   */
  defs: string;
  /**
   * Source markup with the `<defs>` blocks removed.
   */
  content: string;
};

/**
 * Walks `s` and partitions it into the concatenated contents of every
 * `<defs>...</defs>` block plus the remainder (with the blocks removed).
 * Mirrors a single pass of both `matchAll(/<defs>([\s\S]*?)<\/defs>/g)`
 * and `replaceAll(/<defs>[\s\S]*?<\/defs>/g, '')` without regex; the
 * cursor never revisits any byte.
 *
 * @param s - SVG inner markup
 *
 * @returns hoisted defs content plus the markup with defs blocks removed
 *
 * @example
 * ```ts
 * const { defs, content } = extractAndStripDefs('<defs><stop/></defs><path/>');
 * // defs === '<stop/>', content === '<path/>'
 * ```
 */
export function extractAndStripDefs(s: string,): DefsPartition {
  return (function partition(): DefsPartition {
    /**
     * Hoisted `<defs>` block bodies in document order; joined once at the end so the build is O(n) time.
     */
    const defs: string[] = [];
    /**
     * Markup chunks outside every `<defs>` block; joined once at the end.
     */
    const content: string[] = [];
    /**
     * Scan cursor; advances past each `</defs>` so no byte is ever revisited (single linear pass, O(1) stack).
     */
    let from = 0;
    while (from <= s
      .length) {
      /**
       * Position of the next `<defs>` opener; `-1` ends the scan.
       */
      const open = s.indexOf(
        '<defs>',
        from,
      );
      if (open === (-1)) {
        content.push(s.slice(from,),);
        break;
      }
      /**
       * Position of the matching `</defs>`; `-1` means the block is unterminated.
       */
      const close = s.indexOf(
        '</defs>',
        open + '<defs>'
          .length,
      );
      if (close === (-1)) {
        content.push(s.slice(from,),);
        break;
      }
      content.push(s.slice(
        from,
        open,
      ),);
      defs.push(s.slice(
        open + '<defs>'
          .length,
        close,
      ),);
      from = close + '</defs>'
        .length;
    }
    return {
      defs: defs.join('',),
      content: content.join('',),
    };
  })();
}

/**
 * Raw SVG sources keyed by OpenRouter vendor prefix
 */
const RAW_SVGS: Record<string, string> = {
  anthropic: claudeSvg,
  google: geminiSvg,
  openai: openaiSvg,
  moonshotai: kimiFixed,
  minimax: minimaxSvg,
  'z-ai': zhipuSvg,
  qwen: qwenSvg,
};

/**
 * Extracts the inner content and viewBox from a raw SVG string.
 *
 * @param raw - full `<svg>...</svg>` string
 *
 * @returns viewBox attribute value and inner markup
 *
 * @example
 * ```ts
 * const { viewBox, inner } = parseSvg('<svg viewBox="0 0 24 24"><path d="M0 0"/></svg>');
 * // viewBox === '0 0 24 24', inner === '<path d="M0 0"/>'
 * ```
 */
function parseSvg(raw: string,): {
  viewBox: string;
  inner: string;
} {
  /**
   * Extracted viewBox value; `extractViewBox` falls back to the lobehub icon default.
   */
  const viewBox = extractViewBox(raw,);
  /**
   * SVG body with the outer `<svg>` opening and closing tags stripped.
   */
  const inner = stripClosingSvgTag(stripOpeningSvgTag(raw,),);
  return {
    viewBox,
    inner,
  };
}

/**
 * Separates `<defs>` blocks from SVG inner content so gradients
 * can be hoisted to the sprite root for reliable cross-reference
 * from `<use>` shadow trees.
 *
 * @param inner - SVG inner markup
 *
 * @returns extracted defs content and remaining markup
 *
 * @example
 * ```ts
 * const { defs, content } = extractDefs('<defs><linearGradient id="g"/></defs><path/>');
 * // defs === '<linearGradient id="g"/>', content === '<path/>'
 * ```
 */
function extractDefs(inner: string,): {
  defs: string;
  content: string;
} {
  return extractAndStripDefs(inner,);
}

/**
 * Parsed symbol data for a single vendor icon
 */
type VendorSymbol = {
  /**
   * Symbol element ID (e.g. `icon-anthropic`)
   */
  readonly id: string;
  /**
   * SVG viewBox attribute value
   */
  readonly viewBox: string;
  /**
   * Inner markup (paths, shapes) without `<defs>`
   */
  readonly inner: string;
  /**
   * Extracted `<defs>` content (gradients, filters) to hoist
   */
  readonly defs: string;
};

/**
 * Map of vendor prefix to parsed symbol data
 */
const VENDOR_SYMBOLS: ReadonlyMap<string, VendorSymbol> = new Map(
  Object.entries(RAW_SVGS,)
    .map(function parseEntry([vendor, raw,],) {
    /**
     * ViewBox and inner markup pulled from the raw SVG before defs are extracted.
     */
    const {
      viewBox,
      inner,
    } = parseSvg(raw,);
    /**
     * `<defs>` content and remaining markup, separated so gradients can be hoisted.
     */
    const {
      defs,
      content,
    } = extractDefs(inner,);
    return [
      vendor,
      {
        id: `icon-${vendor}`,
        viewBox,
        inner: content,
        defs,
      },
    ];
  },),
);

/**
 * Renders the SVG sprite sheet containing all vendor icon symbols.
 *
 * Gradient `<defs>` are hoisted to the sprite root so `url(#id)` references
 * resolve correctly when symbols are instantiated via `<use>`.
 * Must be included once in the document before any `<use>` references.
 *
 * @returns hidden `<svg>` element with `<defs>` and `<symbol>` definitions
 *
 * @example
 * ```ts
 * const sprite = renderSvgSprite();
 * // '<svg ...><defs>...</defs><symbol id="icon-anthropic" ...>...</symbol>...</svg>'
 * ```
 */
export function renderSvgSprite(): string {
  /**
   * Rendered `<symbol>` elements, one per vendor; concatenated inside the sprite root.
   */
  const symbols = [...VENDOR_SYMBOLS.values(),].map(
    function buildSymbol({
      id,
      viewBox,
      inner,
    },) {
      return `<symbol id="${id}" viewBox="${viewBox}">${inner}</symbol>`;
    },
  );
  /**
   * Concatenated `<defs>` content from every vendor; hoisted onto the sprite root.
   */
  const allDefs = [...VENDOR_SYMBOLS.values(),]
    .map(function pickDefs({ defs, },): string {
      return defs;
    },)
    .join('',);
  /**
   * Optional `<defs>` wrapper; omitted when no vendor contributed defs to avoid an empty tag.
   */
  const defsBlock = allDefs !== '' ? `<defs>${allDefs}</defs>` : '';
  return `<svg xmlns="http://www.w3.org/2000/svg" style="position:absolute;inline-size:0;block-size:0;overflow:hidden">${defsBlock}${
    symbols.join('',)
  }</svg>`;
}

/**
 * Builds an `<svg><use href>` reference to a vendor symbol in the sprite.
 *
 * @param symbolId - symbol ID to reference (e.g. `icon-anthropic`)
 *
 * @returns SVG element string with `<use>` reference
 *
 * @example
 * ```ts
 * useRef('icon-anthropic');
 * // '<svg height="1em" width="1em"><use href="#icon-anthropic"/></svg>'
 * ```
 */
function useRef(symbolId: string,): string {
  return `<svg height="1em" width="1em"><use href="#${symbolId}"/></svg>`;
}

/**
 * Extracts the vendor prefix (segment before the first `/`) from a model ID.
 *
 * Avoids indexing `split('/')[0]`, whose `string | undefined` element would need
 * an empty-string fallback; slicing on the separator position always yields a
 * string (the whole id when no separator is present).
 *
 * @param modelId - full OpenRouter model ID
 *
 * @returns vendor prefix, or whole id when it carries no `/`
 *
 * @example
 * ```ts
 * vendorPrefix('anthropic/claude-opus-4.6'); // 'anthropic'
 * vendorPrefix('localmodel'); // 'localmodel'
 * ```
 */
function vendorPrefix(modelId: string,): string {
  if (!modelId.includes('/',))
    return modelId;
  return modelId.slice(
    0,
    modelId.indexOf('/',),
  );
}

/**
 * Returns a `color-swatch` span containing a vendor icon `<use>` reference.
 * Falls back to a plain colored dot when no icon is available.
 *
 * @param modelId - full OpenRouter model ID
 *
 * @param color - CSS color for the dot background/fallback
 *
 * @returns HTML string for the icon dot
 *
 * @example
 * ```ts
 * const dot = iconDot({ modelId: 'anthropic/claude-opus-4.6', color: '#D97757', });
 * // '<span class="color-swatch" data-shape="icon" data-vendor="anthropic"><svg ...><use href="#icon-anthropic"/></svg></span>'
 * ```
 */
export function iconDot({
  modelId,
  color,
}: {
  readonly modelId: string;
  readonly color: string;
},): string {
  /**
   * Vendor prefix taken from the slash-delimited model ID.
   */
  const vendor = vendorPrefix(modelId,);
  /**
   * Parsed sprite symbol for the vendor; absent when the vendor has no icon.
   */
  const symbol = VENDOR_SYMBOLS.get(vendor,);
  if (symbol === undefined) {
    return h({
      tag: 'span',
      class: 'color-swatch',
      style: { '--point-color': color, },
    },);
  }
  return h({
    tag: 'span',
    class: 'color-swatch',
    attrs: {
      'data-shape': 'icon',
      'data-vendor': vendor,
    },
    html: useRef(symbol.id,),
  },);
}

/**
 * Builds a spreadable `{ icon }` fragment carrying an `<svg><use>` reference for
 * a vendor icon, or an empty fragment when the vendor has none.
 *
 * Returns an exact-optional property rather than an empty-string sentinel: a
 * vendor without an icon yields `{}`, so spreading the result simply omits the
 * `icon` key instead of populating it with a "no icon" marker.
 *
 * @param modelId - full OpenRouter model ID
 *
 * @returns `{ icon }` when a vendor symbol exists, otherwise an empty fragment
 *
 * @example
 * ```ts
 * const point = { runId, score, ...vendorIconEntry('anthropic/claude-opus-4.6'), };
 * // point.icon === '<svg height="1em" width="1em"><use href="#icon-anthropic"/></svg>'
 * vendorIconEntry('unknown/model'); // {} (icon omitted)
 * ```
 */
export function vendorIconEntry(modelId: string,): { readonly icon?: string; } {
  /**
   * Vendor prefix taken from the slash-delimited model ID.
   */
  const vendor = vendorPrefix(modelId,);
  /**
   * Parsed sprite symbol for the vendor; absent when the vendor has no icon.
   */
  const symbol = VENDOR_SYMBOLS.get(vendor,);
  if (symbol === undefined)
    return {};
  return { icon: useRef(symbol.id,), };
}
