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
 Milliseconds in one minute, converting a stored UTC offset into an instant shift.
 */
const MS_PER_MINUTE = 60_000;

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
  /**
   Minutes east of UTC the commit behind {@link gitDate} was authored in.
   */
  readonly gitOffsetMinutes: number;
};

/**
 Formats an instant as the calendar day it falls on in a given UTC offset.
 
 A hand-written frontmatter date is a day, not an instant, so it is read in UTC:
 `2026-05-14` parses to UTC midnight of the 14th and its day is the 14th. The
 git-derived instant is a real moment, so it is read in the offset its author
 wrote it at; reading it in UTC turned a commit at 06:00 in UTC+8 into the
 previous day and fired the warning for a correct frontmatter date.
 
 @param date - instant to format
 
 @param offsetMinutes - minutes east of UTC to read the instant in
 
 @returns ISO calendar date in `YYYY-MM-DD` form
 
 @example
 ```ts
 calendarDayIn({ date: new Date('2026-05-13T22:00:00.000Z'), offsetMinutes: 480 });
 // '2026-05-14'
 ```
 */
function calendarDayIn({
  date,
  offsetMinutes,
}: {
  readonly date: Date;
  readonly offsetMinutes: number;
},): string {
  return new Date(date.getTime() + (offsetMinutes * MS_PER_MINUTE),)
    .toISOString()
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
   Calendar date from frontmatter, read in UTC because a hand-written date is a day rather than an instant.
   */
  const authoredIsoDate = calendarDayIn({
    date: comparison.authoredDate,
    offsetMinutes: 0,
  },);
  /**
   Calendar date from git history in its author's offset, used as source of truth for rendered output.
   */
  const gitIsoDate = calendarDayIn({
    date: comparison.gitDate,
    offsetMinutes: comparison.gitOffsetMinutes,
  },);

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
   authoredDates: { updated: new Date('2026-05-14') },
   resolvedDates: {
     published: new Date('2026-04-16T10:01:43.000Z'),
     publishedOffsetMinutes: 0,
     updated: new Date('2026-05-13T22:00:00.000Z'),
     updatedOffsetMinutes: 480,
   },
   filePath: 'src/content/en/post.mdx',
   l,
 });
 // no warning: the 22:00 UTC instant is 06:00 on the 14th at +08:00
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
        gitOffsetMinutes: resolvedDates.updatedOffsetMinutes,
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
        gitOffsetMinutes: resolvedDates.publishedOffsetMinutes,
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
        gitOffsetMinutes: resolvedDates.updatedOffsetMinutes,
      },
      filePath,
      l,
    },);
  }
}
