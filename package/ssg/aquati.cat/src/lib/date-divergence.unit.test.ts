/**
 Tests for frontmatter date divergence warnings.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type Logger,
  warnOnAuthoredDateDivergence,
} from '../../dist/final/node/index.mjs';

/**
 Git-derived publication date shared by warning fixtures.
 */
const GIT_PUBLISHED_DATE = new Date('2026-04-16T10:01:43.000Z',);

/**
 Git-derived update date shared by warning fixtures.
 */
const GIT_UPDATED_DATE = new Date('2026-05-14T08:31:54.000Z',);

/**
 Git author offset, in minutes, of the commit behind {@link AUTHOR_LOCAL_UPDATED_DATE}: +08:00.
 */
const AUTHOR_OFFSET_MINUTES = 480;

/**
 Update date whose UTC day is the day before the author's local day.
 
 `2026-05-14T06:00:00+08:00` is `2026-05-13T22:00:00Z`, so a UTC calendar-day
 comparison reads the 13th while the author wrote the 14th on their own clock.
 */
const AUTHOR_LOCAL_UPDATED_DATE = new Date('2026-05-13T22:00:00.000Z',);

/**
 MDX path included in divergence warning messages.
 */
const FIXTURE_FILE_PATH = 'src/content/en/about.mdx';

/**
 Logger capture returned by `createWarningCapture`.
 */
type WarningCapture = {
  /**
   Warning messages emitted through the fake logger.
   */
  readonly warnings: string[];
  /**
   Logger subset accepted by the divergence function.
   */
  readonly l: Pick<Logger, 'warn'>;
};

/**
 Creates a warning logger that records messages for assertions.
 
 @returns captured warning array and logger facade
 
 @example
 ```ts
 const { warnings, l } = createWarningCapture();
 ```
 */
function createWarningCapture(): WarningCapture {
  /**
   Mutable capture buffer populated by the fake logger.
   */
  const warnings: string[] = [];
  /**
   Logger facade that records warning messages.
   */
  const l: Pick<Logger, 'warn'> = {
    warn: function warn(message,): void {
      warnings.push(message,);
    },
  };

  return {
    warnings,
    l,
  };
}

await describe({
  name: warnOnAuthoredDateDivergence.name,
  children: [
    it({
      name: 'warns when authored dates differ from git calendar dates',
      fn: async function warnsForDivergentDates(): Promise<void> {
        /**
         Warning capture for this test case.
         */
        const capture = createWarningCapture();

        warnOnAuthoredDateDivergence({
          authoredDates: {
            date: new Date('2026-05-01T00:00:00.000Z',),
            published: new Date('2026-04-01T00:00:00.000Z',),
          },
          resolvedDates: {
            published: GIT_PUBLISHED_DATE,
            publishedOffsetMinutes: 0,
            updated: GIT_UPDATED_DATE,
            updatedOffsetMinutes: 0,
          },
          filePath: FIXTURE_FILE_PATH,
          l: capture.l,
        },);

        expect(capture.warnings.length,).toBe(2,);
        expect(capture.warnings[0],).toContain(
          'Frontmatter date date 2026-05-01 in src/content/en/about.mdx diverges from git-derived updated date 2026-05-14',
        );
        expect(capture.warnings[1],).toContain(
          'Frontmatter published date 2026-04-01 in src/content/en/about.mdx diverges from git-derived published date 2026-04-16',
        );
      },
    },),
    it({
      name: 'does not warn when authored dates share git calendar dates',
      fn: async function ignoresMatchingCalendarDates(): Promise<void> {
        /**
         Warning capture for this test case.
         */
        const capture = createWarningCapture();

        warnOnAuthoredDateDivergence({
          authoredDates: {
            updated: new Date('2026-05-14T00:00:00.000Z',),
          },
          resolvedDates: {
            published: GIT_PUBLISHED_DATE,
            publishedOffsetMinutes: 0,
            updated: GIT_UPDATED_DATE,
            updatedOffsetMinutes: 0,
          },
          filePath: FIXTURE_FILE_PATH,
          l: capture.l,
        },);

        expect(capture.warnings.length,).toBe(0,);
      },
    },),
    it({
      name: 'compares calendar days in the git author offset',
      fn: async function comparesInAuthorOffset(): Promise<void> {
        /**
         Warning capture for this test case.
         */
        const capture = createWarningCapture();

        warnOnAuthoredDateDivergence({
          authoredDates: {
            updated: new Date('2026-05-14T00:00:00.000Z',),
          },
          resolvedDates: {
            published: GIT_PUBLISHED_DATE,
            publishedOffsetMinutes: 0,
            updated: AUTHOR_LOCAL_UPDATED_DATE,
            updatedOffsetMinutes: AUTHOR_OFFSET_MINUTES,
          },
          filePath: FIXTURE_FILE_PATH,
          l: capture.l,
        },);

        expect(capture.warnings.length,).toBe(0,);
      },
    },),
    it({
      name: 'still warns when the authored day differs in that offset',
      fn: async function warnsForDifferentAuthorDay(): Promise<void> {
        /**
         Warning capture for this test case.
         */
        const capture = createWarningCapture();

        warnOnAuthoredDateDivergence({
          authoredDates: {
            updated: new Date('2026-05-01T00:00:00.000Z',),
          },
          resolvedDates: {
            published: GIT_PUBLISHED_DATE,
            publishedOffsetMinutes: 0,
            updated: AUTHOR_LOCAL_UPDATED_DATE,
            updatedOffsetMinutes: AUTHOR_OFFSET_MINUTES,
          },
          filePath: FIXTURE_FILE_PATH,
          l: capture.l,
        },);

        expect(capture.warnings.length,).toBe(1,);
      },
    },),
  ],
},);
