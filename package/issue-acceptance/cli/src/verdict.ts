/**
 Decides whether the branch holds what an issue's acceptance asked for.

 Each check yields one outcome with the words a failure needs:
 what was expected,
 where it was looked for,
 and what the branch held instead.

 @module
 */

/* oxlint-disable eslint/no-await-in-loop -- one plan entry at a time: a failure cannot interleave two verdicts, and GitHub receives one request in flight; see doc/lint/sequential-awaits.md */

import { PlanError, } from './errors.ts';
import type { GitReader } from './git-reader.ts';
import type { Check } from './plan-model.ts';

/**
 Matched paths a failure quotes before summarising the rest.
 */
const QUOTED_MATCHES = 5;

/**
 One check and what the branch answered.
 */
export type CheckOutcome = {
  readonly check: Check;
  readonly holds: boolean;
  readonly detail: string;
};

/**
 Every outcome of one issue's checks.
 */
export type Verdict = {
  readonly holds: boolean;
  readonly outcomes: readonly CheckOutcome[];
};

/**
 One issue's checks against one branch.
 */
export type VerdictRequest = {
  readonly checks: readonly Check[];
  readonly reader: GitReader;
};

/**
 One check to decide, with the reader that answers it.
 */
export type DecideCheckRequest = {
  readonly reader: GitReader;
  readonly check: Check;
};

/**
 Renders the paths a pattern matched, briefly.
 
 @param matches - Paths that matched.
 
 @returns Comma-separated paths, with a count when some were left out.
 
 @example
 ```ts
 const quoted = quoteMatches(['package/a/src/b.ts']);
 ```
 */
function quoteMatches(matches: readonly string[],): string {
  /**
   The first matches, quoted inside the failure.
   */
  const quoted = matches.slice(
    0,
    QUOTED_MATCHES,
  )
    .join(', ',);
  return matches.length > QUOTED_MATCHES
    ? `${quoted}, and ${String(matches.length - QUOTED_MATCHES,)} more`
    : quoted;
}

/**
 Compiles a path pattern, refusing one that is not a valid expression.
 
 @param pattern - Pattern as the plan wrote it.
 
 @returns The compiled expression.
 
 @throws {@link PlanError} when the pattern does not compile.
 
 @example
 ```ts
 const expression = compilePathPattern('^docs/');
 ```
 */
function compilePathPattern(pattern: string,): RegExp {
  try {
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- the plan supplies the pattern at run time so it cannot be a literal; the input is the branch's tracked path list and the pattern comes from a reviewed plan file. Justification recorded in this package's README.
    return new RegExp(
      pattern,
      'v',
    );
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new PlanError(`The plan's pathMatching pattern ${JSON.stringify(pattern,)} does not compile: ${error.message}`,);
  }
}

/**
 Decides one check.
 
 @param reader - Branch reader the check asks.
 
 @param check - One check.
 
 @returns The check's outcome.
 
 @throws {@link PlanError} when a `pathMatching` pattern does not compile or the kind is unknown.
 
 @example
 ```ts
 const outcome = await decideCheck({ reader, check: { kind: 'exists', path: 'README.md' } });
 ```
 */
async function decideCheck({
  reader,
  check,
}: DecideCheckRequest,): Promise<CheckOutcome> {
  if (check.kind === 'absent') {
    /**
     Matches the pattern still has under the pathspecs.
     */
    const found = await reader.countMatches({
      pattern: check.pattern,
      paths: check.paths,
    },);
    return {
      check,
      holds: found === 0,
      detail: `absent /${check.pattern}/ in ${check.paths
        .join(' ',)} found ${String(found,)}`,
    };
  }
  if (check.kind === 'present') {
    /**
     Matches the pattern has under the pathspecs.
     */
    const found = await reader.countMatches({
      pattern: check.pattern,
      paths: check.paths,
    },);
    return {
      check,
      holds: found > 0,
      detail: `present /${check.pattern}/ in ${check.paths
        .join(' ',)} found ${String(found,)}`,
    };
  }
  if (check.kind === 'exactly') {
    /**
     Matches the pattern has under the pathspecs.
     */
    const found = await reader.countMatches({
      pattern: check.pattern,
      paths: check.paths,
    },);
    return {
      check,
      holds: found === check.count,
      detail: `exactly ${String(check.count,)} of /${check.pattern}/ in ${check.paths
        .join(' ',)} found ${String(found,)}`,
    };
  }
  if (check.kind === 'atLeast') {
    /**
     Matches the pattern has under the pathspecs.
     */
    const found = await reader.countMatches({
      pattern: check.pattern,
      paths: check.paths,
    },);
    return {
      check,
      holds: found >= check.count,
      detail: `at least ${String(check.count,)} of /${check.pattern}/ in ${check.paths
        .join(' ',)} found ${String(found,)}`,
    };
  }
  if (check.kind === 'exists') {
    /**
     Whether the branch tracks the path.
     */
    const present = await reader.pathExists(check.path,);
    return {
      check,
      holds: present,
      detail: `exists ${check.path} answered ${String(present,)}`,
    };
  }
  if (check.kind === 'gone') {
    /**
     Whether the branch still tracks the path.
     */
    const present = await reader.pathExists(check.path,);
    return {
      check,
      holds: !present,
      detail: `gone ${check.path} answered ${String(present,)}`,
    };
  }
  if (check.kind === 'fileContains') {
    /**
     What the branch holds at the path.
     */
    const file = await reader.fileText(check.path,);
    /**
     Whether that text holds the wanted words.
     */
    const holds = (file.kind === 'text')
      && file.text
        .includes(check.text,);
    return {
      check,
      holds,
      detail: `file ${check.path} contains ${JSON.stringify(check.text,)} answered ${file.kind === 'absent' ? 'no such file' : String(holds,)}`,
    };
  }
  if (check.kind === 'fileLacks') {
    /**
     What the branch holds at the path.
     */
    const file = await reader.fileText(check.path,);
    /**
     Whether that text avoids the unwanted words, or the file is not there.
     */
    const holds = (file.kind === 'absent') || (!file.text
      .includes(check.text,));
    return {
      check,
      holds,
      detail: `file ${check.path} lacks ${JSON.stringify(check.text,)} answered ${file.kind === 'absent' ? 'no such file' : String(holds,)}`,
    };
  }
  if (check.kind === 'pathMatching') {
    /**
     Compiled path pattern, refused when it does not compile.
     */
    const expression = compilePathPattern(check.pattern,);
    /**
     Tracked paths the pattern matched.
     */
    const matches = (await reader.trackedPaths()).filter(function matchesPattern(path,): boolean {
      return expression.test(path,);
    },);
    return {
      check,
      holds: matches.length === 0,
      detail: `no tracked path matches /${check.pattern}/ found ${quoteMatches(matches,)}`,
    };
  }
  if (check.kind === 'onBranch') {
    /**
     Whether the commit is an ancestor of the branch.
     */
    const merged = await reader.isOnBranch(check.commit,);
    return {
      check,
      holds: merged,
      detail: `commit ${check.commit} is on the branch answered ${String(merged,)}`,
    };
  }
  throw new PlanError('A check reached the verdict with a kind this command does not implement.',);
}

/**
 Decides every check of one issue.
 
 @param checks - Checks the plan named, in order.
 
 @param reader - Branch reader every check asks.
 
 @returns The verdict and each check's outcome.
 
 @throws {@link PlanError} when a `pathMatching` pattern does not compile.
 
 @example
 ```ts
 const verdict = await evaluateChecks({ checks: [{ kind: 'onBranch', commit: '2b309f8f25' }], reader });
 ```
 */
export async function evaluateChecks({
  checks,
  reader,
}: VerdictRequest,): Promise<Verdict> {
  /**
   Outcomes collected so far, in the order the plan named the checks.
   */
  const outcomes: CheckOutcome[] = [];
  for (const check of checks) {
    /**
     What the branch answered for this check.
     */
    const outcome = await decideCheck({
      reader,
      check,
    },);
    outcomes.push(outcome,);
  }
  return {
    holds: outcomes.every(function holdsOutcome(outcome,): boolean {
      return outcome.holds;
    },),
    outcomes,
  };
}
