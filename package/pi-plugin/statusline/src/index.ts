/**
 Pi statusline extension entry point.
 
 Adds one footer status segment for projected provider usage overflow warnings.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ExtensionEvent,
  SessionShutdownEvent,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  PLAIN_USAGE_WARNING_STYLE,
  type UsageWarningStyle,
} from './rate-limit-types.ts';
import { formatUsageWarningStatus, } from './usage-warning.ts';

/**
 Logger root for pi-statusline after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: statuslineLogger, },);
 ```
 */
const statuslineLogger = tagged({ tag: 'pi-statusline', },);

//region Constants

/**
 Footer status key owned by this extension.
 */
const STATUS_KEY = 'pi-statusline.usage';

//endregion Constants

//region UI helpers

/**
 Builds theme-backed style hooks for warning segments.
 
 @param ctx - {@link ExtensionContext} with UI theme access
 
 @returns style hooks using current Pi theme colors
 
 @example
 ```ts
 const style = styleFromContext(ctx);
 ```
 */
function styleFromContext(ctx: ExtensionContext,): UsageWarningStyle {
  /**
   Current Pi theme used to color footer status text.
   */
  const { theme, } = ctx.ui;

  return {
    green: function green(text: string,): string {
      return theme.fg(
        'success',
        text,
      );
    },
    yellow: function yellow(text: string,): string {
      return theme.fg(
        'warning',
        text,
      );
    },
    red: function red(text: string,): string {
      return theme.fg(
        'error',
        text,
      );
    },
  };
}

/**
 Applies or clears footer status when UI is available.
 
 @param ctx - {@link ExtensionContext}
 
 @param statusText - rendered status text, empty when clearing
 
 @example
 ```ts
 setUsageStatus({ ctx, statusText: 'codex 5h →180%' });
 ```
 */
function setUsageStatus({
  ctx,
  statusText,
}: Readonly<{
  ctx: ExtensionContext;
  statusText: string;
}>,): void {
  if (!ctx.hasUI)
    return;

  /**
   Whether a non-empty status should be shown.
   */
  const hasStatusText = statusText
    .length
    > 0;
  /**
   Next footer status value; `undefined` tells Pi to clear this key.
   */
  const nextStatus = hasStatusText ? statusText : undefined;

  ctx
    .ui
    .setStatus(
    STATUS_KEY,
    nextStatus,
    );
}

/**
 Clears footer status.
 
 @param ctx - {@link ExtensionContext}
 
 @param log - tagged logger for lifecycle notes
 
 @example
 ```ts
 clearStatus({ ctx, log });
 ```
 */
function clearStatus({
  ctx,
  log,
}: Readonly<{
  ctx: ExtensionContext;
  log: Logger;
}>,): void {
  setUsageStatus({
    ctx,
    statusText: '',
  },);
  log.debug('cleared usage warning status',);
}

//endregion UI helpers

//region Extension entry point

/**
 Pi statusline extension.
 
 Subscribes to provider responses, samples supported provider usage headers, and
 renders only the projected-overflow warning portion ported from the Claude Code statusline.
 
 @param pi - {@link ExtensionAPI}
 
 @example
 ```typescript
 // In ~/.pi/agent/settings.json:
 { "packages": ["./packages/pi-plugin/statusline"] }
 ```
 */
export default function statusline(pi: ForeignBorrowed<ExtensionAPI>,): void {
  /**
   Entry-point logger tagged by function name.
   */
  const log = tagged({
    tag: statusline.name,
    l: statuslineLogger,
  },);

  pi.on(
    'session_start',
    function handleSessionStart(
      _event: ForeignBorrowed<SessionStartEvent>,
      ctx,
    ) {
      clearStatus({
        ctx,
        log,
      },);
    },
  );

  pi.on(
    'session_shutdown',
    function handleSessionShutdown(
      _event: ForeignBorrowed<SessionShutdownEvent>,
      ctx,
    ) {
      clearStatus({
        ctx,
        log,
      },);
    },
  );

  pi.on(
    'after_provider_response',
    function handleAfterProviderResponse(
      event: ForeignBorrowed<Extract<ExtensionEvent, { type: 'after_provider_response'; }>>,
      ctx,
    ) {
      /**
       Warning style hooks. Plain style keeps non-UI modes from touching theme state.
       */
      const style = ctx.hasUI
        ? styleFromContext(ctx,)
        : PLAIN_USAGE_WARNING_STYLE;
      /**
       Formatted status from current provider response headers.
       */
      const result = formatUsageWarningStatus({
        headers: event.headers,
        nowMs: Date.now(),
        style,
      },);

      setUsageStatus({
        ctx,
        statusText: result.statusText,
      },);
      /**
       Whether formatted status contains at least one visible warning.
       */
      const hasWarningStatus = result.statusText
        .length
        > 0;
      log.debug(
        `processed provider response status=${event.status} warnings=${hasWarningStatus}`,
      );
    },
  );
}

//endregion Extension entry point

export { STATUS_KEY, };
