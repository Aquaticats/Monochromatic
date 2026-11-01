/**
 Frontmatter date divergence diagnostics.
 
 Git history is the source of truth for rendered content dates. This module
 compares legacy hand-authored date fields against resolved git dates and
 emits warnings without letting frontmatter override git.
 */
import type {
  AuthoredDateFields,
  AuthoredDateFieldName,
} from './frontmatter-dates.ts';
import type { ResolvedDates, } from './content.ts';
import type { Logger, } from './types.ts';

/**
 Number of leading ISO characters that represent a calendar date.
 */
const ISO_DATE_LENGTH = 10;

/**
 {@link Logger} subset needed for divergence diagnostics.
 */
type WarningLogger = Pick<Logger, 'warn'>;

/**
 Pairing between one authored frontmatter date field and its git-derived source.
 */
type DateComparison = {
  /**
   Name of the frontmatter key supplied by a human.
   */
  readonly authoredKey: AuthoredDateFieldName;
  /**
   Date parsed from the authored frontmatter key.
   */
  readonly authoredDate: Date;
  /**
   Name of the git-derived date field used as source of truth.
   */
  readonly gitKey: keyof ResolvedDates;
  /**
   Date resolved from git history.
   */
  readonly gitDate: Date;
};

/**
 Formats a Date as the ISO calendar day used for divergence comparisons.
 
 @param date - date to normalize
 
 @returns ISO calendar date in `YYYY-MM-DD` form
 
 @example
 ```ts
 isoCalendarDate(new Date('2026-05-14T08:31:54.000Z'));
 // '2026-05-14'
 ```
 */
function isoCalendarDate(date: Date,): string {
  return date.toISOString()
    .slice(
    0,
    ISO_DATE_LENGTH,
    );
}

/**
 Emits a warning when one authored date field contradicts its git-derived date.
 
 @param comparison - authored and git date pair to inspect
 
 @param filePath - MDX file path included in diagnostics
 
 @param l - logger receiving warning lines
 
 @example
 ```ts
 warnIfDateDiverges({
   comparison: {
     authoredKey: 'updated',
     authoredDate: new Date('2026-05-01'),
     gitKey: 'updated',
     gitDate: new Date('2026-05-14'),
   },
   filePath: 'src/content/en/post.mdx',
   l,
 });
 ```
 */
function warnIfDateDiverges(
  {
    comparison,
    filePath,
    l,
  }: {
    readonly comparison: DateComparison;
    readonly filePath: string;
    readonly l: WarningLogger;
  },
): void {
  /**
   Calendar date from frontmatter, ignoring timestamp noise in hand-written dates.
   */
  const authoredIsoDate = isoCalendarDate(comparison.authoredDate,);
  /**
   Calendar date from git history, used as source of truth for rendered output.
   */
  const gitIsoDate = isoCalendarDate(comparison.gitDate,);

  if (authoredIsoDate === gitIsoDate)
    return;

  l.warn(
    `Frontmatter ${comparison.authoredKey} date ${authoredIsoDate} in ${filePath} diverges from git-derived ${comparison.gitKey} date ${gitIsoDate}; git history remains the source of truth.`,
  );
}

/**
 Warns when legacy hand-authored date fields disagree with git-derived dates.
 
 `date` is treated as a legacy alias for `updated`, because issue #170 asks
 for document modified dates and older frontmatter commonly used one date key.
 
 @param authoredDates - optional date fields parsed from MDX frontmatter
 
 @param resolvedDates - git-derived dates used by rendering and feeds
 
 @param filePath - MDX file path included in diagnostics
 
 @param l - logger receiving warning lines
 
 @example
 ```ts
 warnOnAuthoredDateDivergence({
   authoredDates: { updated: new Date('2026-05-01') },
   resolvedDates: {
     published: new Date('2026-04-01'),
     updated: new Date('2026-05-14'),
   },
   filePath: 'src/content/en/post.mdx',
   l,
 });
 ```
 */
export function warnOnAuthoredDateDivergence(
  {
    authoredDates,
    resolvedDates,
    filePath,
    l,
  }: {
    readonly authoredDates: AuthoredDateFields;
    readonly resolvedDates: ResolvedDates;
    readonly filePath: string;
    readonly l: WarningLogger;
  },
): void {
  if (authoredDates.date !== undefined) {
    warnIfDateDiverges({
      comparison: {
        authoredKey: 'date',
        authoredDate: authoredDates.date,
        gitKey: 'updated',
        gitDate: resolvedDates.updated,
      },
      filePath,
      l,
    },);
  }

  if (authoredDates.published !== undefined) {
    warnIfDateDiverges({
      comparison: {
        authoredKey: 'published',
        authoredDate: authoredDates.published,
        gitKey: 'published',
        gitDate: resolvedDates.published,
      },
      filePath,
      l,
    },);
  }

  if (authoredDates.updated !== undefined) {
    warnIfDateDiverges({
      comparison: {
        authoredKey: 'updated',
        authoredDate: authoredDates.updated,
        gitKey: 'updated',
        gitDate: resolvedDates.updated,
      },
      filePath,
      l,
    },);
  }
}
