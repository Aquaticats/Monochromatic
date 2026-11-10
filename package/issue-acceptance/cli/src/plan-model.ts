/**
 The settlement plan: what each issue's acceptance checks,
 and what is said on the issue once they hold.

 A plan is data,
 so one command settles any round of issues,
 and the varying parts (numbers,
 patterns,
 paths,
 comments) never need new code.

 @module
 */

/**
 Names of the checks a plan may name.
 */
export type CheckKind =
  | 'absent'
    | 'present'
    | 'exactly'
    | 'atLeast'
    | 'exists'
    | 'gone'
    | 'fileContains'
    | 'fileLacks'
    | 'pathMatching'
    | 'onBranch';

/**
 One check of the branch,
 written so a failure names what was expected and what was found.
 */
export type Check =
  | {
    readonly kind: 'absent';
    readonly pattern: string;
    readonly paths: readonly string[];
  }
    | {
      readonly kind: 'present';
      readonly pattern: string;
      readonly paths: readonly string[];
    }
    | {
      readonly kind: 'exactly';
      readonly pattern: string;
      readonly paths: readonly string[];
      readonly count: number;
    }
    | {
      readonly kind: 'atLeast';
      readonly pattern: string;
      readonly paths: readonly string[];
      readonly count: number;
    }
    | {
      readonly kind: 'exists';
      readonly path: string;
    }
    | {
      readonly kind: 'gone';
      readonly path: string;
    }
    | {
      readonly kind: 'fileContains';
      readonly path: string;
      readonly text: string;
    }
    | {
      readonly kind: 'fileLacks';
      readonly path: string;
      readonly text: string;
    }
    | {
      readonly kind: 'pathMatching';
      readonly pattern: string;
    }
    | {
      readonly kind: 'onBranch';
      readonly commit: string;
    };

/**
 What happens on the issue once every check holds.

 `close` closes it with the comment;
 `comment` leaves it open and records the comment,
 which is how a partly done issue keeps its remaining work in view.
 */
export type IssueAction = 'close' | 'comment';

/**
 One issue,
 the checks that decide it,
 and the words the issue receives.
 */
export type IssueEntry = {
  readonly issue: number;
  readonly action: IssueAction;
  readonly checks: readonly Check[];
  readonly comment: string;
};

/**
 A commit whose message this run corrects,
 because amending it is refused.
 */
export type CommitCorrection = {
  readonly commit: string;
  readonly body: string;
};

/**
 A complete settlement run: where to look, which issues, and which commit messages to correct.
 */
export type Plan = {
  readonly repository: string;
  readonly checkout: string;
  readonly branch: string;
  readonly issues: readonly IssueEntry[];
  readonly commitCorrections: readonly CommitCorrection[];
};
