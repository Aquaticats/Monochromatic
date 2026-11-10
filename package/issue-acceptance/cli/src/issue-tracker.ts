/**
 The GitHub boundary: reading issues and recording one verdict on each.

 Every call names the repository on its command line
 and disables the interactive prompt,
 so a session never waits on a question no one can see.

 @module
 */

import { runChild, } from './child-process.ts';
import { ChildProcessError, } from './errors.ts';
import {
  isRecord,
  type JsonObject,
} from './plan-value.ts';

/**
 Longest standard error echoed in a failure, because `gh` quotes the request back.
 */
const STDERR_ECHO_LIMIT = 400;

/**
 Open issue count this reader asks for.

 The repository's open issues stay in the low thousands,
 and `gh` pages internally above its own page size,
 so one call answers the whole list.
 */
const OPEN_ISSUE_LIMIT = 5_000;

/**
 Fields one issue search needs.
 */
const GH_LISTED_FIELDS = 'number,title,body';

/**
 Fields one full issue read needs: the body to check, and its earlier comments.
 */
const GH_ISSUE_FIELDS = 'number,title,state,body,comments';

/**
 One issue as the tracker reports it.
 */
export type IssueSnapshot = {
  readonly number: number;
  readonly title: string;
  readonly state: string;
  readonly body: string;
  readonly comments: readonly string[];
};

/**
 One open issue as a search reports it.
 */
export type IssueListing = {
  readonly number: number;
  readonly title: string;
  readonly body: string;
};

/**
 What a settlement does to the tracker.
 */
export type IssueTracker = {
  /**
   Reads one issue's state, as GitHub spells it.
   */
  readonly state: (number: number,) => Promise<string>;

  /**
   Closes one issue with the comment that records the evidence.
   */
  readonly close: (request: {
    readonly number: number;
    readonly comment: string;
  },) => Promise<void>;

  /**
   Comments on one issue without closing it.
   */
  readonly comment: (request: {
    readonly number: number;
    readonly body: string;
  },) => Promise<void>;

  /**
   Comments on one commit, which is how an inaccurate commit message is corrected.
   */
  readonly commentOnCommit: (request: {
    readonly commit: string;
    readonly body: string;
  },) => Promise<string>;

  /**
   Reads one issue in full.
   */
  readonly read: (number: number,) => Promise<IssueSnapshot>;

  /**
   Lists the open issues carrying every given label.
   */
  readonly listOpenWithLabels: (labels: readonly string[],) => Promise<readonly IssueListing[]>;
};

/**
 How to reach the tracker.
 */
export type IssueTrackerOptions = {
  readonly repository: string;
  readonly executable?: string;
};

/**
 Reads one field of a parsed issue as text.
 
 @param record - Parsed issue object.
 
 @param key - Field to read.
 
 @returns The field's text.
 
 @throws {@link ChildProcessError} when the field is not a string.
 
 @example
 ```ts
 const title = readTextField({ record: { title: 'x' }, key: 'title' });
 ```
 */
function readTextField({
  record,
  key,
}: {
  readonly record: JsonObject;
  readonly key: string;
},): string {
  /**
   Field as read, before it is required to be text.
   */
  const value = record[key];
  if ((typeof value) !== 'string') {
    throw new ChildProcessError({
      message: `gh answered an issue whose ${key} is not text`,
      stderr: '',
    },);
  }
  return value;
}

/**
 Reads one issue's number.
 
 @param record - Parsed issue object.
 
 @returns The issue number.
 
 @throws {@link ChildProcessError} when the number is not a safe integer.
 
 @example
 ```ts
 const number = readNumberField({ record: { number: 7 } });
 ```
 */
function readNumberField({ record, }: { readonly record: JsonObject; },): number {
  /**
   Field as read, before it is required to be an integer.
   */
  const value = record.number;
  if (((typeof value) !== 'number') || (!Number.isSafeInteger(value,))) {
    throw new ChildProcessError({
      message: 'gh answered an issue whose number is not an integer',
      stderr: '',
    },);
  }
  return value;
}

/**
 Parses a `gh` answer, refusing anything this reader cannot use.
 
 @param stdout - Standard output of the `gh` call.
 
 @param description - The call, for the failure message.
 
 @returns The parsed value.
 
 @throws {@link ChildProcessError} when the answer is not JSON.
 
 @example
 ```ts
 const parsed = parseAnswer({ stdout: '{"state":"OPEN"}', description: 'gh issue view' });
 ```
 */
function parseAnswer({
  stdout,
  description,
}: {
  readonly stdout: string;
  readonly description: string;
},): unknown {
  try {
    return JSON.parse(stdout,);
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new ChildProcessError({
      message: `${description} answered text that is not JSON: ${error.message}`,
      stderr: '',
    },);
  }
}

/**
 Reads the comments of one parsed issue.
 
 @param comments - Field as `gh` answered it.
 
 @param description - The call, for the failure message.
 
 @returns Each comment's body.
 
 @throws {@link ChildProcessError} when the field is not a list of comment objects.
 
 @example
 ```ts
 const bodies = readComments({ comments: [{ body: 'x' }], description: 'gh issue view 1' });
 ```
 */
function readComments({
  comments,
  description,
}: {
  readonly comments: unknown;
  readonly description: string;
},): readonly string[] {
  if (!Array.isArray(comments,)) {
    throw new ChildProcessError({
      message: `${description} answered an issue whose comments are not a list`,
      stderr: '',
    },);
  }
  return comments.map(function toCommentBody(item,): string {
    if (!isRecord(item,)) {
      throw new ChildProcessError({
        message: `${description} answered a comment that is not an object`,
        stderr: '',
      },);
    }
    return readTextField({
      record: item,
      key: 'body',
    },);
  },);
}

/**
 Turns one parsed issue object into a snapshot.
 
 @param parsed - Parsed issue object.
 
 @param description - The call, for the failure message.
 
 @returns The issue snapshot.
 
 @throws {@link ChildProcessError} when a field is not what this reader needs.
 
 @example
 ```ts
 const snapshot = toSnapshot({ parsed: { number: 1, title: 't', state: 'OPEN', body: '', comments: [] }, description: 'gh issue view 1' });
 ```
 */
function toSnapshot({
  parsed,
  description,
}: {
  readonly parsed: unknown;
  readonly description: string;
},): IssueSnapshot {
  if (!isRecord(parsed,)) {
    throw new ChildProcessError({
      message: `${description} answered something other than one issue object`,
      stderr: '',
    },);
  }
  return {
    number: readNumberField({ record: parsed, },),
    title: readTextField({
      record: parsed,
      key: 'title',
    },),
    state: readTextField({
      record: parsed,
      key: 'state',
    },),
    body: readTextField({
      record: parsed,
      key: 'body',
    },),
    comments: readComments({
      comments: parsed.comments,
      description,
    },),
  };
}

/**
 Turns one parsed search answer into listings.
 
 @param parsed - Parsed answer of the search.
 
 @returns One listing per open issue.
 
 @throws {@link ChildProcessError} when the answer is not a list of issue objects.
 
 @example
 ```ts
 const listings = toListings({ parsed: [{ number: 1, title: 't', body: '' }] });
 ```
 */
function toListings({ parsed, }: { readonly parsed: unknown; },): readonly IssueListing[] {
  if (!Array.isArray(parsed,)) {
    throw new ChildProcessError({
      message: 'gh issue list answered something other than a list of issues',
      stderr: '',
    },);
  }
  return parsed.map(function toListing(item,): IssueListing {
    if (!isRecord(item,)) {
      throw new ChildProcessError({
        message: 'gh issue list answered a list holding something other than an issue',
        stderr: '',
      },);
    }
    return {
      number: readNumberField({ record: item, },),
      title: readTextField({
        record: item,
        key: 'title',
      },),
      body: readTextField({
        record: item,
        key: 'body',
      },),
    };
  },);
}

/**
 Creates the tracker for one repository.
 
 @param repository - Repository slug every call names.
 
 @param executable - `gh` executable, which tests replace to record the argument vectors.
 
 @returns The tracker.
 
 @example
 ```ts
 const tracker = createIssueTracker({ repository: 'OWNER/NAME' });
 ```
 */
export function createIssueTracker({
  repository,
  executable = 'gh',
}: IssueTrackerOptions,): IssueTracker {
  /**
   Runs one `gh` call and refuses a nonzero answer, because every call here has to have happened.
   
   @param childArguments - Arguments after the executable name.
   
   @returns The call's standard output.
   
   @throws {@link ChildProcessError} when the call failed.
   
   @example
   ```ts
   const state = await gh(['issue', 'view', '1', '--json', 'state']);
   ```
   */
  async function gh(childArguments: readonly string[],): Promise<string> {
    /**
     Result of the call.
     */
    const outcome = await runChild({
      file: executable,
      arguments: [...childArguments,],
      description: `gh ${childArguments.slice(
        0,
        2,
      )
        .join(' ',)}`,
      env: { GH_PROMPT_DISABLED: '1', },
    },);
    if (outcome.status !== 0) {
      throw new ChildProcessError({
        message: `gh ${childArguments.slice(
          0,
          2,
        )
          .join(' ',)} failed: ${outcome.stderr
            .slice(
              0,
              STDERR_ECHO_LIMIT,
            )}`,
        stderr: outcome.stderr,
        exitCode: outcome.status,
      },);
    }
    return outcome.stdout;
  }

  return {
    state: async function state(number,) {
      /**
       Answer of the state question, which is one bare word.
       */
      const answered = await gh([
        'issue',
        'view',
        String(number,),
        '--repo',
        repository,
        '--json',
        'state',
        '--jq',
        '.state',
      ],);
      return answered.trim();
    },

    close: async function close({
      number,
      comment,
    },) {
      await gh([
        'issue',
        'close',
        String(number,),
        '--repo',
        repository,
        '--comment',
        comment,
      ],);
    },

    comment: async function comment({
      number,
      body,
    },) {
      await gh([
        'issue',
        'comment',
        String(number,),
        '--repo',
        repository,
        '--body',
        body,
      ],);
    },

    commentOnCommit: async function commentOnCommit({
      commit,
      body,
    },) {
      /**
       Answer of the comment call, which is the new comment's id.
       */
      const answered = await gh([
        'api',
        '--method',
        'POST',
        `repos/${repository}/commits/${commit}/comments`,
        '--field',
        `body=${body}`,
        '--jq',
        '.id',
      ],);
      return answered.trim();
    },

    read: async function read(number,) {
      /**
       The call, for failure messages.
       */
      const description = `gh issue view ${String(number,)}`;
      return toSnapshot({
        parsed: parseAnswer({
          stdout: await gh([
            'issue',
            'view',
            String(number,),
            '--repo',
            repository,
            '--json',
            GH_ISSUE_FIELDS,
          ],),
          description,
        },),
        description,
      },);
    },

    listOpenWithLabels: async function listOpenWithLabels(labels,) {
      return toListings({
        parsed: parseAnswer({
          stdout: await gh([
            'issue',
            'list',
            '--repo',
            repository,
            '--state',
            'open',
            '--limit',
            String(OPEN_ISSUE_LIMIT,),
            ...labels.flatMap(function labelArguments(label,): readonly string[] {
              return [
                '--label',
                label,
              ];
            },),
            '--json',
            GH_LISTED_FIELDS,
          ],),
          description: 'gh issue list',
        },),
      },);
    },
  };
}
