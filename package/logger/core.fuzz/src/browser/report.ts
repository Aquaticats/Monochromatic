/**
 Report shape the browser property bundle hands back to the Playwright test
 through `page.evaluate`, so both sides type the same structure. Plain data
 only: it crosses the page boundary by structured clone.

 @module
 */

/**
 Outcome of one browser property.
 */
export type PropertyStatus = 'failed' | 'passed' | 'skipped';

/**
 One property's outcome, with fast-check's report text on failure or the
 reason a backend was unavailable on skip.
 */
export type PropertyResult = {
  readonly detail?: string;
  readonly name: string;
  readonly status: PropertyStatus;
};

/**
 Whole-run report.
 */
export type BrowserReport = {
  readonly results: readonly PropertyResult[];
  readonly userAgent: string;
};

/**
 Options the test passes into the page.
 */
export type BrowserRunOptions = {
  /**
   fast-check runs per property.
   */
  readonly numRuns: number;
};

/**
 Entry the bundle installs on the page's global.
 */
export type BrowserPropertyRunner = {
  readonly run: (options: BrowserRunOptions,) => Promise<BrowserReport>;
};
