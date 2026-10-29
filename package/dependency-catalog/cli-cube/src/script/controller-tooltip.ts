/**
 Tooltip HTML renderer + pinned-tooltip DOM management.
 
 Two surfaces:
 - {@link formatTooltipHtml} returns a self-contained HTML string used by
   deck.gl's `getTooltip` callback for hover-time tooltips.
 - {@link pinTooltip} / {@link unpinTooltip} drive a separate
   `<aside id="pinned-tooltip">` element that the controller creates on
   first click; pinning lets the user rotate the scene while reading.
 
 Pure HTML formatting except for the pinned-tooltip DOM helpers, which
 are confined to this file so the main controller stays focused on
 deck.gl + event wiring.
 
 @example
 ```ts
 import { formatTooltipHtml, pinTooltip, unpinTooltip } from './controller-tooltip.ts';
 const html = formatTooltipHtml({ probe });
 pinTooltip({ probe });
 ```
 */

import type { PackageProbe, } from '../probe.ts';

//region Constants

/**
 Decimal places for continuous-dim values in tooltips.
 */
const TOOLTIP_DECIMALS = 2;

/**
 Pinned tooltip's outer element id.
 */
const PINNED_ID = 'pinned-tooltip';

/**
 Pinned tooltip's content wrapper class.
 */
const PINNED_CONTENT_CLASS = 'pinned-tooltip-content';

/**
 Pinned tooltip's close-button class.
 */
const PINNED_CLOSE_CLASS = 'pinned-tooltip-close';

//endregion Constants

//region HTML escape

/**
 Escapes a string for embedding in HTML text content. Used for every
 probe-derived value before splicing into the tooltip template, so
 malicious package names or licence strings can't inject markup.
 
 Order is critical: `&` must be replaced first or other replacements
 would double-escape it.
 
 @param raw - Untrusted source string.
 
 @returns Escaped string safe to embed inside an HTML node.
 */
function escapeHtml(raw: string,): string {
  return raw
    .replaceAll(
    '&',
    '&amp;',
    )
    .replaceAll(
    '<',
    '&lt;',
    )
    .replaceAll(
    '>',
    '&gt;',
    )
    .replaceAll(
    '"',
    '&quot;',
    );
}

//endregion HTML escape

//region Value formatting

/**
 Formats a number with fixed decimals, or returns "unknown" when the
 value is absent.
 
 @param value - Source value; absent renders as "unknown".
 
 @returns Formatted string.
 */
function formatNum(value?: number,): string {
  if (value === undefined)
    return 'unknown';
  return value.toFixed(TOOLTIP_DECIMALS,);
}

/**
 Formats a number with no decimals, or returns "unknown" when the
 value is absent. Used for counts.
 
 @param value - Source value; absent renders as "unknown".
 
 @returns Formatted string.
 */
function formatInt(value?: number,): string {
  if (value === undefined)
    return 'unknown';
  return Math.round(value,)
    .toString();
}

/**
 Formats a boolean as the conventional ✓/✗ pair, or "unknown" when
 absent.
 
 @param value - Source boolean; absent renders as "unknown".
 
 @returns Formatted string.
 */
function formatBool(value?: boolean,): string {
  if (value === undefined)
    return 'unknown';
  return value ? '✓' : '✗';
}

//endregion Value formatting

//region Tooltip HTML

/**
 Renders the per-probe tooltip HTML fragment.
 
 Covers every dim a user might want to see at a glance, regardless of
 which channel is currently mapped: source bytes, days stale, install
 size, downloads, TS ratio, dep counts, age, leaf flag, license,
 repository URL, and the unknown-reason if any.
 
 Every probe-derived value is escaped via {@link escapeHtml} before
 splicing.
 
 @param probe - Source probe.
 
 @returns HTML fragment ready for embedding in deck.gl's tooltip
   widget or the pinned-tooltip pane.
 
 @example
 ```ts
 const html = formatTooltipHtml({ probe });
 ```
 */
export function formatTooltipHtml(
  { probe, }: { readonly probe: PackageProbe; },
): string {
  /**
   Escaped npm name so the header is injection-safe.
   */
  const name = escapeHtml(probe.npmName,);
  /**
   Escaped resolved version paired with the name in the tooltip header.
   */
  const version = escapeHtml(probe.resolvedVersion,);
  /**
   Escaped license-class label rendered in the table.
   */
  const license = escapeHtml(probe.licenseClass,);
  /**
   Escaped repository URL, or the literal `"unknown"` when the probe lacks a repo.
   */
  const repo = probe.repositoryUrlOrNull
    === undefined
    ? 'unknown'
    : escapeHtml(probe.repositoryUrlOrNull,);
  /**
   Optional unknown-reason banner; empty string when the probe has no unknowns to report.
   */
  const unknown = probe.unknownReason
    === undefined
    ? ''
    : `<div class="tooltip-unknown">unknown: ${escapeHtml(probe.unknownReason,)}</div>`;
  return `<div class="tooltip">
      <div class="tooltip-name">${name}@${version}</div>
      <table class="tooltip-table">
        <tr><th>leaf</th><td>${formatBool(probe.isLeaf,)}</td></tr>
        <tr><th>TS ratio</th><td>${formatNum(probe.tsRatioOrNull,)}</td></tr>
        <tr><th>source bytes</th><td>${formatInt(probe.sourceBytesOrNull,)}</td></tr>
        <tr><th>install bytes</th><td>${formatInt(probe.installSizeBytes,)}</td></tr>
        <tr><th>downloads/week</th><td>${formatInt(probe.weeklyDownloads,)}</td></tr>
        <tr><th>days stale</th><td>${
    formatInt(probe.daysSinceLastCommitOrNull,)
  }</td></tr>
        <tr><th>age (days)</th><td>${formatInt(probe.packageAgeDays,)}</td></tr>
        <tr><th>runtime deps</th><td>${formatInt(probe.runtimeDepCount,)}</td></tr>
        <tr><th>transitive deps</th><td>${formatInt(probe.transitiveDepCount,)}</td></tr>
        <tr><th>license</th><td>${license}</td></tr>
        <tr><th>repo</th><td>${repo}</td></tr>
      </table>${unknown}
    </div>`;
}

//endregion Tooltip HTML

//region Pinned tooltip

/**
 Lazily creates the pinned-tooltip element on first use and inserts it
 at the end of `<body>`. Returns the existing element on subsequent
 calls. The element has a close-button child the controller wires to
 {@link unpinTooltip}.
 
 @returns The `<aside>` element used for pinned tooltips.
 */
function ensurePinElement(): HTMLElement {
  /**
   Existing pinned-tooltip element from a previous `ensurePinElement` call, or `null` on first use.
   */
  const existing = document.querySelector<HTMLElement>(`#${PINNED_ID}`,);
  if (existing !== null)
    return existing;
  /**
   Fresh outer `<aside>` element that owns the pinned-tooltip surface.
   */
  const aside = document.createElement('aside',);
  aside.id = PINNED_ID;
  aside.hidden = true;
  /**
   Close button child whose click handler hides the pinned tooltip.
   */
  const close = document.createElement('button',);
  close.type = 'button';
  close.className = PINNED_CLOSE_CLASS;
  close.textContent = '×';
  close.addEventListener(
    'click',
    function onCloseClick() {
      unpinTooltip();
    },
  );
  /**
   Inner content wrapper whose `innerHTML` is replaced on every pin.
   */
  const content = document.createElement('div',);
  content.className = PINNED_CONTENT_CLASS;
  aside.append(
    close,
    content,
  );
  document.body
    .append(aside,);
  return aside;
}

/**
 Pins a tooltip beside the canvas with full probe detail.
 
 @param probe - Probe to display.
 
 @example
 ```ts
 pinTooltip({ probe: info.object.probe });
 ```
 */
export function pinTooltip(
  { probe, }: { readonly probe: PackageProbe; },
): void {
  /**
   Pinned-tooltip `<aside>`, created on first use.
   */
  const aside = ensurePinElement();
  /**
   Content wrapper inside the aside; `null` only if the DOM was tampered with externally.
   */
  const content = aside.querySelector<HTMLDivElement>(`.${PINNED_CONTENT_CLASS}`,);
  if (content === null)
    return;
  content.innerHTML = formatTooltipHtml({
    probe,
  },);
  aside.hidden = false;
}

/**
 Hides the pinned tooltip (close-button click or controller reset).
 
 @example
 ```ts
 unpinTooltip();
 ```
 */
export function unpinTooltip(): void {
  /**
   Pinned-tooltip element, if it was ever created; missing element is a no-op.
   */
  const aside = document.querySelector<HTMLElement>(`#${PINNED_ID}`,);
  if (aside === null)
    return;
  aside.hidden = true;
}

//endregion Pinned tooltip
