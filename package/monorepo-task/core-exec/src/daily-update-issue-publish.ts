/**
 Opens the daily update's issues through `gh`, or comments on the open issue that already reports the same break.

 A `gh` failure never throws out of here: it is reported as a failed outcome with its reason, so the caller can keep the
 issue for the next run.

 @module
 */

import { oneAfterAnother, } from './daily-update-one-after-another.ts';
import { REPORT_SEARCH_PHRASE, } from './daily-update-issue-text.ts';
import type {
  GhRequest,
  GhResult,
  PublishOutcome,
  RenderedIssue,
} from './daily-update-package-check-types.ts';

export type { PublishOutcome, } from './daily-update-package-check-types.ts';

/**
 Most open reports one search returns.
 Measured 2026-11-07: the search answers in 0.5 seconds and no open issue carries the report phrase; a day's run opens
 at most one report per newly red package, so two hundred is far above any day's open reports.
 */
export const OPEN_REPORT_LIMIT = 200;

/**
 What a run of `gh` is given: a function that runs it.
 */
type RunGh = (request: GhRequest,) => Promise<GhResult>;

/**
 An open issue with its number and body.
 */
type OpenReport = {
  /**
   Issue number.
   */
  readonly number: number;
  /**
   Issue body, where the marker is.
   */
  readonly body: string;
};

/**
 Part of an address `gh issue create` prints, before the issue number.
 */
const ISSUE_ADDRESS_PART = '/issues/';

/**
 Runs `gh` and turns a failure to start into an answer, so callers handle one shape.

 @param gh - runs `gh`

 @param request - the call

 @returns what gh answered, or a nonzero answer that carries the error's text

 @example
 ```ts
 const answer = await askGh({ gh, request: { args: ['issue', 'list',], stdin: '', }, },);
 ```
 */
export async function askGh({
  gh,
  request,
}: {
  readonly gh: RunGh;
  readonly request: GhRequest;
},): Promise<GhResult> {
  try {
    return await gh(request,);
  }
  catch (error) {
    return {
      exitCode: -1,
      stdout: '',
      stderr: `gh could not run: ${String(error,)}`,
    };
  }
}

/**
 Writes why a `gh` call failed.

 @param request - the call

 @param answer - its answer

 @returns a sentence with the command, the status and the error text

 @example
 ```ts
 failureReason({ request, answer, },); // 'gh issue list exited with status 1: ...'
 ```
 */
export function failureReason({
  request,
  answer,
}: {
  readonly request: GhRequest;
  readonly answer: GhResult;
},): string {
  return `gh ${request.args
    .slice(
      0,
      2,
    )
    .join(' ',)} exited with status ${answer.exitCode}: ${answer.stderr
      .trim()}`;
}

/**
 Whether a value has the number and the body of an open issue.

 @param entry - a value from the parsed list

 @returns true when it has both

 @example
 ```ts
 isOpenReport({ number: 1, body: 'x', },); // true
 ```
 */
export function isOpenReport(entry: unknown,): entry is OpenReport {
  return ((typeof entry) === 'object') && (entry !== null)
    && ('number' in entry)
    && ((typeof entry.number) === 'number')
    && ('body' in entry)
    && ((typeof entry.body) === 'string');
}

/**
 Reads the open reports out of the `gh issue list` output.

 @param stdout - JSON list of objects with a number and a body

 @returns the reports

 @throws Error when the text is not a JSON list

 @example
 ```ts
 parseOpenReports('[{"number":1,"body":"x"}]',); // [{ number: 1, body: 'x' }]
 ```
 */
export function parseOpenReports(stdout: string,): readonly OpenReport[] {
  /**
   The parsed list.
   */
  const parsed: unknown = JSON.parse(stdout,);
  if (!Array.isArray(parsed,))
    throw new Error('gh issue list answered something other than a JSON list of issues',);
  return parsed.filter(isOpenReport,);
}

/**
 Reads the number out of the address `gh issue create` prints.

 @param stdout - what `gh issue create` printed

 @returns the issue number, or zero when no address is in the text

 @example
 ```ts
 issueNumberOf('https://github.com/o/r/issues/77\n',); // 77
 ```
 */
export function issueNumberOf(stdout: string,): number {
  /**
   Position of the number after the address part.
   */
  const start = stdout.indexOf(ISSUE_ADDRESS_PART,);
  if (start === (-1))
    return 0;
  /**
   The characters after the address part, up to the first one that is no digit.
   */
  const rest = stdout.slice(start + ISSUE_ADDRESS_PART.length,);
  /**
   Digits at the start of the rest.
   */
  const digits = rest.split('',)
    .findIndex(function isNoDigit(character,) {
      return (character < '0') || (character > '9');
    },);
  return Number(digits === (-1) ? rest : rest.slice(
    0,
    digits,
  ),);
}

/**
 Reads the open reports out of the answer to the search, or says why they cannot be read.

 @param listing - the search that was asked

 @param listed - what `gh` answered

 @returns the reports and an empty problem, or no reports and the reason

 @example
 ```ts
 const { reports, problem, } = readOpenReports({ listing, listed, },);
 ```
 */
export function readOpenReports({
  listing,
  listed,
}: {
  readonly listing: GhRequest;
  readonly listed: GhResult;
},): {
  readonly reports: readonly OpenReport[];
  readonly problem: string
} {
  if (listed.exitCode !== 0) {
    return {
      reports: [],
      problem: failureReason({
        request: listing,
        answer: listed,
      },),
    };
  }
  try {
    return {
      reports: parseOpenReports(listed.stdout,),
      problem: '',
    };
  }
  catch (error) {
    return {
      reports: [],
      problem: String(error,),
    };
  }
}

/**
 Builds the call that comments on an open issue or creates a new one.

 @param issue - the issue to publish

 @param same - the open issue that carries the issue's marker, absent when there is none

 @returns the call

 @example
 ```ts
 const request = publishRequest({ issue, },);
 ```
 */
export function publishRequest({
  issue,
  same,
}: {
  readonly issue: RenderedIssue;
  readonly same?: OpenReport;
},): GhRequest {
  if (same !== undefined) {
    return {
      args: [
        'issue',
        'comment',
        String(same.number,),
        '--body-file',
        '-',
      ],
      stdin: issue.comment,
    };
  }
  return {
    args: [
      'issue',
      'create',
      '--title',
      issue.title,
      '--body-file',
      '-',
      ...issue.labels
        .flatMap(function labelArguments(label,) {
          return [
            '--label',
            label,
          ];
        },),
    ],
    stdin: issue.body,
  };
}

/**
 Opens each issue, or comments on the open issue with the same marker.
 One search finds the open reports; an issue created in this call joins them, so two findings with one marker make one
 issue and one comment.

 @param gh - runs `gh`; its credentials are its own, and no call carries a token or a body on the command line

 @param issues - issues to publish, in order

 @returns one outcome per issue, in order; nothing is asked of `gh` when there are no issues

 @example
 ```ts
 const outcomes = await publishIssues({ gh, issues: [issue,], },);
 ```
 */
export async function publishIssues({
  gh,
  issues,
}: {
  readonly gh: RunGh;
  readonly issues: readonly RenderedIssue[];
},): Promise<readonly PublishOutcome[]> {
  if (issues.length === 0)
    return [];
  /**
   The search for open reports.
   */
  const listing: GhRequest = {
    args: [
      'issue',
      'list',
      '--state',
      'open',
      '--search',
      `"${REPORT_SEARCH_PHRASE}" in:body`,
      '--limit',
      String(OPEN_REPORT_LIMIT,),
      '--json',
      'number,body',
    ],
    stdin: '',
  };
  /**
   What the search answered.
   */
  const listed = await askGh({
    gh,
    request: listing,
  },);
  /**
   The open reports, or why they could not be read.
   */
  const read = readOpenReports({
    listing,
    listed,
  },);
  if (read.problem !== '') {
    return issues.map(function failed(issue,): PublishOutcome {
      return {
        issue,
        action: 'failed',
        reason: read.problem,
      };
    },);
  }
  /**
   Open reports, plus the ones created here.
   */
  const known: OpenReport[] = [...read.reports,];
  return Array.fromAsync(oneAfterAnother({
    items: issues,
    run: async function publishOne(issue,): Promise<PublishOutcome> {
      /**
       The open issue that carries this marker.
       */
      const same = known.find(function hasMarker(report,) {
        return report.body
          .includes(issue.marker,);
      },);
      /**
       The call that comments or creates.
       */
      const request = publishRequest(same === undefined ? { issue, } : {
        issue,
        same,
      },);
      /**
       What gh answered.
       */
      const answer = await askGh({
        gh,
        request,
      },);
      if (answer.exitCode !== 0) {
        return {
          issue,
          action: 'failed',
          reason: failureReason({
            request,
            answer,
          },),
        };
      }
      if (same !== undefined) {
        return {
          issue,
          action: 'commented',
          issueNumber: same.number,
          reason: '',
        };
      }
      /**
       Number in the address `gh issue create` printed, zero when it printed none.
       */
      const issueNumber = issueNumberOf(answer.stdout,);
      if (issueNumber === 0) {
        return {
          issue,
          action: 'created',
          reason: '',
        };
      }
      known.push({
        number: issueNumber,
        body: issue.marker,
      },);
      return {
        issue,
        action: 'created',
        issueNumber,
        address: answer.stdout
          .trim()
          .split('\n',)[0]
          ?? '',
        reason: '',
      };
    },
  },),);
}
