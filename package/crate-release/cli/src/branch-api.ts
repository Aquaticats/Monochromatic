import {
  compareFileLimit,
  comparisonStatuses,
} from './constants.ts';
import { isObjectName } from './commit-request.ts';
import { CrateReleaseError } from './error.ts';
import {
  graphqlErrorMessages,
  isRecord,
  parseJson,
} from './json-value.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import {
  parseBranchName,
  parseRepository,
} from './validate.ts';

/**
 How a branch compares with the commit a checkout started from.
 */
export type BranchComparison = {
  /**
   The status the compare endpoint reported: `ahead`, `behind`, `diverged` or `identical`.
   */
  readonly status: string;
  /**
   Every path the comparison lists as changed, a renamed file's old and new path both.
   */
  readonly changedPaths: readonly string[];
  /**
   True when the list is as long as the endpoint ever lists, so it may have been cut and cannot show a path unchanged.
   */
  readonly truncated: boolean;
};

/**
 Check that a text is a full object name, for a request that cannot name a short one.

 @param option - what the text is, for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value is not a full lower-case object name

 @example
 ```ts
 assertObjectName({ option: 'the base', value: 'a'.repeat(40) });
 ```
 */
function assertObjectName({
  option,
  value,
}: {
  readonly option: string;
  readonly value: string;
}): string {
  if (!isObjectName(value))
    throw new CrateReleaseError(`${option} must be a full lower-case object name, got ${JSON.stringify(value)}.`);
  return value;
}

/**
 Arguments of `gh api` that read the commit a branch points to.

 @param repo - `owner/name` of the repository

 @param branch - branch name

 @returns the arguments after `gh`; the answer is JSON

 @throws CrateReleaseError when the repository or the branch is malformed

 @example
 ```ts
 buildBranchHeadArguments({ repo: 'o/r', branch: 'main' });
 ```
 */
export function buildBranchHeadArguments({
  repo,
  branch,
}: {
  readonly repo: string;
  readonly branch: string;
}): readonly string[] {
  parseRepository({
    option: '--repo',
    value: repo
  });
  parseBranchName({
    option: '--branch',
    value: branch
  });
  return [
    'api',
    `repos/${repo}/git/ref/heads/${branch}`,
  ];
}

/**
 Read the commit a branch points to from the answer of the reference endpoint.

 @param body - JSON text printed by `gh api repos/<repo>/git/ref/heads/<branch>`

 @returns the full object name of the commit, or a miss when the answer is not JSON or names no commit

 @example
 ```ts
 parseBranchHead('{"object":{"type":"commit","sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}');
 ```
 */
export function parseBranchHead(body: string): Lookup<string> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  /**
   The object the reference points to.
   */
  const target: unknown = isRecord(answer.value) ? answer.value.object : undefined;
  if ((!isRecord(target)) || (target.type !== 'commit') || ((typeof target.sha) !== 'string')
    || (!isObjectName(target.sha as string)))
    return missingValue('The reference answer names no commit.');
  return foundValue(target.sha as string);
}

/**
 Arguments of `gh api` that compare a branch's new head with the commit a checkout started from.

 @param repo - `owner/name` of the repository

 @param base - full object name of the commit the checkout started from

 @param head - full object name of the commit the branch points to now

 @returns the arguments after `gh`; the answer is JSON

 @throws CrateReleaseError when the repository is malformed or a commit is not a full object name

 @example
 ```ts
 buildCompareArguments({ repo: 'o/r', base: 'a'.repeat(40), head: 'b'.repeat(40) });
 ```
 */
export function buildCompareArguments({
  repo,
  base,
  head,
}: {
  readonly repo: string;
  readonly base: string;
  readonly head: string;
}): readonly string[] {
  parseRepository({
    option: '--repo',
    value: repo
  });
  return [
    'api',
    `repos/${repo}/compare/${assertObjectName({
      option: 'the base',
      value: base
    })}...${assertObjectName({
      option: 'the head',
      value: head
    })}`,
  ];
}

/**
 Read a comparison from the answer of the compare endpoint.

 @param body - JSON text printed by `gh api repos/<repo>/compare/<base>...<head>`

 @returns the status, the changed paths and whether the list may have been cut, or a miss when the answer lacks them

 @example
 ```ts
 parseBranchComparison('{"status":"ahead","files":[{"filename":"a"}]}');
 ```
 */
export function parseBranchComparison(body: string): Lookup<BranchComparison> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if ((!isRecord(answer.value)) || ((typeof answer.value.status) !== 'string')
    || (!Array.isArray(answer.value.files)))
    return missingValue('The compare answer lacks its status or its file list.');
  /**
   The listed files.
   */
  const files = answer.value
    .files as readonly unknown[];
  if (!files.every(function hasName(file) {
    return isRecord(file) && ((typeof file.filename) === 'string');
  }))
    return missingValue('A file of the compare answer lacks its name.');
  return foundValue({
    status: answer.value
      .status as string,
    changedPaths: files.flatMap(function pathsOf(file): string[] {
      /**
       The file entry, already checked to be a record with a name.
       */
      const entry = file as Readonly<Record<string, unknown>>;
      return (typeof entry.previous_filename) === 'string' ? [
        entry.filename as string,
        entry.previous_filename as string
      ] : [entry.filename as string];
    }),
    truncated: files.length >= compareFileLimit,
  });
}

/**
 Tell whether a comparison shows the branch moved forward from the commit a checkout started from.

 @param comparison - the comparison

 @returns true when the head is ahead of the base or is the base, false for a branch that went back or diverged, whose
 changes a comparison would not describe

 @example
 ```ts
 movedForward({ status: 'ahead', changedPaths: [], truncated: false });
 ```
 */
export function movedForward(comparison: BranchComparison): boolean {
  return (comparison.status === comparisonStatuses.ahead) || (comparison.status === comparisonStatuses.identical);
}

/**
 Build the GraphQL request that reads the object name of each watched path at two commits.

 An object name of a directory covers everything below it, and equal names mean equal contents, so the answer shows
 whether a path changed without listing files and without a limit on how many changed. The paths travel as variables, so
 no path can change the query.

 @param repo - `owner/name` of the repository

 @param base - full object name of the commit the checkout started from

 @param head - full object name of the commit the branch points to now

 @param paths - repository-relative paths, files and directories

 @returns the JSON text of the request, for `gh api graphql --input`

 @throws CrateReleaseError when the repository is malformed, a commit is not a full object name, or no path is given

 @example
 ```ts
 buildTreeIdsRequest({ repo: 'o/r', base: 'a'.repeat(40), head: 'b'.repeat(40), paths: ['package/a'] });
 ```
 */
export function buildTreeIdsRequest({
  repo,
  base,
  head,
  paths,
}: {
  readonly repo: string;
  readonly base: string;
  readonly head: string;
  readonly paths: readonly string[];
}): string {
  parseRepository({
    option: '--repo',
    value: repo
  });
  assertObjectName({
    option: 'the base',
    value: base
  });
  assertObjectName({
    option: 'the head',
    value: head
  });
  if (paths.length === 0)
    throw new CrateReleaseError('At least one path must be compared.');
  /**
   Owner and name of the repository.
   */
  const [owner, name] = repo.split('/');
  /**
   Variable names, base then head, one pair per path.
   */
  const aliases = paths.flatMap(function aliasesOf(_path, index) {
    return [`b${String(index)}`, `h${String(index)}`];
  });
  return JSON.stringify({
    query: `query($owner: String!, $name: String!, ${aliases.map(function declare(alias) {
      return `$${alias}: String!`;
    })
      .join(', ')}) {
  repository(owner: $owner, name: $name) {
${aliases.map(function select(alias) {
      return `    ${alias}: object(expression: $${alias}) { oid }`;
    })
      .join('\n')}
  }
}`,
    variables: {
      owner,
      name,
      ...Object.fromEntries(paths.flatMap(function expressionsOf(path, index) {
        return [
          [`b${String(index)}`, `${base}:${path}`],
          [`h${String(index)}`, `${head}:${path}`],
        ];
      })),
    },
  });
}

/**
 Pick, from the answer to a tree id request, the paths whose object name differs between the two commits.

 A path that is missing at both commits is unchanged; a path that is missing at one and present at the other changed.

 @param body - JSON text printed by `gh api graphql`

 @param paths - the paths the request was built for, in its order

 @returns the paths whose names differ, or a miss when the answer lacks a path or holds errors

 @example
 ```ts
 parseChangedTreePaths({ body, paths: ['package/a'] });
 ```
 */
export function parseChangedTreePaths({
  body,
  paths,
}: {
  readonly body: string;
  readonly paths: readonly string[];
}): Lookup<readonly string[]> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if (isRecord(answer.value)) {
    /**
     The messages of the errors the answer lists.
     */
    const messages = graphqlErrorMessages(answer.value);
    if (messages.length > 0)
      return missingValue(messages.join('; '));
  }
  /**
   The repository object of the answer.
   */
  const repository: unknown = isRecord(answer.value) && isRecord(answer.value.data)
    ? answer.value
      .data
      .repository
    : undefined;
  if (!isRecord(repository))
    return missingValue('The tree id answer holds no repository.');
  /**
   The object name under an alias: text, or null for a missing path; undefined when the alias is absent.

   @param alias - alias of the object in the answer

   @returns the name, null for a missing path, undefined for an answer that lacks the alias
   */
  function nameAt(alias: string): string | null | undefined {
    if (!(alias in repository))
      return undefined;
    /**
     The aliased object.
     */
    const entry = repository[alias];
    if (entry === null)
      return null;
    return isRecord(entry) && ((typeof entry.oid) === 'string') ? entry.oid as string : undefined;
  }
  /**
   The names at both commits of each path.
   */
  const pairs = paths.map(function pairOf(path, index) {
    return {
      path,
      base: nameAt(`b${String(index)}`),
      head: nameAt(`h${String(index)}`)
    };
  });
  if (pairs.some(function lacksName(pair) {
    return (pair.base === undefined) || (pair.head === undefined);
  }))
    return missingValue('The tree id answer lacks the name of a path.');
  return foundValue(pairs.filter(function differs(pair) {
    return pair.base !== pair.head;
  })
    .map(function pathOf(pair) {
      return pair.path;
    }));
}
