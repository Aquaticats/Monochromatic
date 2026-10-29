/** Final operation rendering and metadata-only progress. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { type AdvisorTheme, createAdvisorOperationLedger, formatAdvisorProgress, NO_ADVISOR_OPERATION_SUMMARY, renderAdvisorOperationSummary, } from '../dist/final/node/index.mjs';

/** Theme boundary returns text verbatim so content assertions ignore ANSI styling. */
const theme = { fg: (_color: string, text: string): string => text, } as unknown as AdvisorTheme;

await describe({ name: '', children: [
  ...[undefined, {}, { operation: {} }, { operation: { attempts: [null], reviews: [], usageIncomplete: false } },].map(details =>
    it({ name: `rejects malformed stored operation ${JSON.stringify(details)}`, fn: async (): Promise<void> => {
      expect(renderAdvisorOperationSummary({ details, text: 'failure', expanded: false, theme, },),).toBe(NO_ADVISOR_OPERATION_SUMMARY,);
    }, },),
  ),
  it({ name: 'progress exposes no provider payload or premature review text', fn: async (): Promise<void> => {
    const ledger = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 100, },);
    ledger.record({ id: 1, model: 'p/a', provider: 'p', attempt: 1, state: 'failed', startedAtMs: 0,
      contextChars: 20, estimatedInputTokens: 5, truncated: false, usageIncomplete: true,
      diagnostic: 'PRIVATE_PROVIDER_PAYLOAD', },);
    ledger.blockProvider('p',);
    ledger.collect({ review: { attemptId: 2, model: 'q/b', text: 'PRIVATE_REVIEW_TEXT', lengthLimited: false, }, collectionEndsAtMs: 50, },);
    const progress = formatAdvisorProgress({ operation: ledger.snapshot(), now: 25, },);
    expect(progress,).toContain('p/a',);
    expect(progress,).toContain('Credit-exhausted providers',);
    expect(progress,).toContain('collection remaining 25ms',);
    expect(progress,).not.toContain('PRIVATE_PROVIDER_PAYLOAD',);
    expect(progress,).not.toContain('PRIVATE_REVIEW_TEXT',);
  }, },),
  it({ name: 'final collapsed summary retains all attempt identities and accounting uncertainty', fn: async (): Promise<void> => {
    const details = { operation: {
      attempts: [{ model: 'p/a', attempt: 1, state: 'cancelled' }, { model: 'q/b', attempt: 1, state: 'succeeded' }],
      reviews: [{ model: 'q/b' }], usageIncomplete: true, end: 'collection',
    } };
    const result = renderAdvisorOperationSummary({ details, text: 'Review first line\nReview second line', expanded: false, theme, },);
    expect(result,).toContain('1 usable review; 2 attempts; collection',);
    expect(result,).toContain('p/a #1 cancelled',);
    expect(result,).toContain('q/b #1 succeeded',);
    expect(result,).toContain('Usage may be incomplete',);
    expect(result,).not.toContain('Review second line',);
    const expanded = renderAdvisorOperationSummary({ details, text: 'Review first line\nReview second line', expanded: true, theme, },);
    expect(expanded,).toContain('Review second line',);
  }, },),
  it({ name: 'summary surfaces the configured-reviewer selection reason', fn: async (): Promise<void> => {
    const details = { operation: {
      attempts: [{ model: 'q/b', attempt: 1, state: 'succeeded' }],
      reviews: [{ model: 'q/b' }], usageIncomplete: false, end: 'stop',
    }, defaultSelectionReason: 'Initial selection: defaultReviewerModel "missing/reviewer" did not resolve among scoped models, so the default ranking applies', };
    const result = renderAdvisorOperationSummary({ details, text: 'Review text', expanded: false, theme, },);
    expect(result,).toContain('defaultReviewerModel "missing/reviewer" did not resolve among scoped models',);
  }, },),
  it({ name: 'summary omits the selection line without a default selection', fn: async (): Promise<void> => {
    const details = { operation: {
      attempts: [{ model: 'q/b', attempt: 1, state: 'succeeded' }],
      reviews: [{ model: 'q/b' }], usageIncomplete: false, end: 'stop',
    } };
    const result = renderAdvisorOperationSummary({ details, text: 'Review text', expanded: false, theme, },);
    expect(result,).not.toContain('Initial selection',);
  }, },),
], },);
