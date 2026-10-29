/**
 Runtime-validated operation summaries for restored tool and slash-command results. @module
 */
import * as v from 'valibot';
import type { AdvisorTheme, } from './ports.ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { firstAdvisoryLine, } from './rendering-summary.ts';

/**
 A legacy or malformed result must continue through the existing fallback renderer.
 */
export const NO_ADVISOR_OPERATION_SUMMARY: unique symbol = Symbol('advisor/no-operation-summary');

/**
 Validate only the fields consumed by this renderer, without asserting a complete operation type.
 */
const OperationViewSchema = v.object({
  operation: v.object({
    attempts: v.array(v.object({
      model: v.string(),
      attempt: v.number(),
      state: v.string(),
    },),),
    reviews: v.array(v.object({ model: v.string(), },),),
    usageIncomplete: v.boolean(),
    end: v.exactOptional(v.string(),),
  },),
  defaultSelectionReason: v.exactOptional(v.string(),),
},);

/**
 Render all attempt identities and the collected-review count from a validated stored projection.
 
 @param details - possibly absent or malformed persisted tool details
 
 @param text - original tool content
 
 @param expanded - whether full review text is visible
 
 @param theme - host styling capability
 
 @returns operation summary or a sentinel requesting the legacy renderer
 
 @mutates theme - host theme methods apply terminal styling
 
 @example
 ```ts
 const summary = renderAdvisorOperationSummary({ details, text, expanded: false, theme });
 ```
 */
export function renderAdvisorOperationSummary({
  details,
  text,
  expanded,
  theme,
}: {
  readonly details: unknown;
  readonly text: string;
  readonly expanded: boolean;
  readonly theme: ForeignHostCapability<AdvisorTheme>;
},): string | typeof NO_ADVISOR_OPERATION_SUMMARY {
  /**
   Parsing projects unknown session data before any nested access.
   */
  const parsed = v.safeParse(
    OperationViewSchema,
    details,
  );
  if (!parsed.success)
    return NO_ADVISOR_OPERATION_SUMMARY;
  /**
   Fields consumed by the operation summary only.
   */
  const { operation, } = parsed.output;
  /**
   Presence of usable results does not reinterpret caller cancellation as success.
   */
  const succeeded = (operation.reviews
    .length
    > 0) && (operation.end !== 'caller');
  /**
   Text labels and color jointly distinguish operation states.
   */
  const heading = theme.fg(
    succeeded ? 'success' : 'error',
    `advisor: ${String(operation.reviews
      .length,)} usable review${operation.reviews
        .length
        === 1 ? '' : 's'}; ${String(operation.attempts
          .length,)} attempt${operation.attempts
            .length
            === 1 ? '' : 's'}; ${operation.end ?? 'running'}`,
  );
  /**
   Every attempted endpoint remains visible after partial progress is replaced.
   */
  const attempts = operation.attempts
    .map(function attemptLabel(attempt: {
      readonly model: string;
      readonly attempt: number;
      readonly state: string
    }): string {
    /**
     Icon and text remain distinguishable without color.
     */
      const icon = attempt.state === 'succeeded' ? '✓' : attempt.state === 'failed' ? '×' : attempt.state === 'cancelled' ? '■' : attempt.state === 'empty' ? '?' : '…';
    /**
     Color is a second cue rather than the only indication of state.
     */
      const color = attempt.state === 'succeeded' ? 'success' : attempt.state === 'failed' ? 'error' : (attempt.state === 'cancelled') || (attempt.state === 'empty') ? 'warning' : attempt.state === 'running' ? 'accent' : 'dim';
      return theme.fg(
        color,
        `${icon} ${attempt.model} #${String(attempt.attempt,)} ${attempt.state}`,
      );
    },)
    .join('; ',);
  /**
   Missing remote settlement is disclosed rather than displayed as complete zero usage.
   */
  const accounting = operation.usageIncomplete ? '\nUsage may be incomplete; cancelled requests can still be billed.' : '';
  /**
   Selection rationale naming the configured-reviewer outcome for empty-param
   calls, so a silently replaced `defaultReviewerModel` stays visible.
   */
  const selectionReason = parsed.output
    .defaultSelectionReason;
  /**
   Selection line rendered only when the operation made a default selection.
   */
  const selection = selectionReason === undefined
    ? ''
    : `\n${selectionReason}`;
  return `${heading}\n${attempts}${selection}${accounting}\n${expanded ? text : firstAdvisoryLine(text,)}`;
}
