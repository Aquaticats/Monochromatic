import { isRecord, parseJson } from './json-value.ts';
import { CrateReleaseError } from './error.ts';
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
 Length of a full Git object name, which the commit mutation requires for the head it expects.
 */
const objectNameLength = 40;

/**
 One file the commit adds or replaces.
 */
export type CommitFile = {
  /**
   Repository-relative path.
   */
  readonly path: string;
  /**
   The file's whole contents, as bytes.
   */
  readonly contents: Uint8Array;
};

/**
 The commit GitHub created.
 */
export type CreatedCommit = {
  /**
   Full object name of the new commit.
   */
  readonly oid: string;
  /**
   Address of the commit on GitHub.
   */
  readonly url: string;
};

/**
 The GraphQL mutation that appends one commit to a branch.

 GitHub signs a commit made this way, which a branch that requires signed commits accepts and a plain `git push` of an
 unsigned commit does not.
 */
const commitMutation = `mutation($input: CreateCommitOnBranchInput!) {
  createCommitOnBranch(input: $input) {
    commit {
      oid
      url
    }
  }
}`;

/**
 Build the request body that appends a commit to a branch through the GraphQL API.

 @param repo - `owner/name` of the repository

 @param branch - branch to append to

 @param expectedHeadOid - full object name the branch must still point to; GitHub refuses the commit when the branch
 moved, so a release never lands on top of work it did not see

 @param headline - commit subject

 @param files - files to add or replace, with their whole contents

 @returns the JSON text of the request, for `gh api graphql --input`

 @throws CrateReleaseError when the repository, branch or head is malformed, or no file is given

 @example
 ```ts
 buildCommitRequest({ repo: 'o/r', branch: 'main', expectedHeadOid: 'a'.repeat(40), headline: 'h', files });
 ```
 */
export function buildCommitRequest({
  repo,
  branch,
  expectedHeadOid,
  headline,
  files,
}: {
  readonly repo: string;
  readonly branch: string;
  readonly expectedHeadOid: string;
  readonly headline: string;
  readonly files: readonly CommitFile[];
}): string {
  parseRepository({
    option: '--repo',
    value: repo
  });
  parseBranchName({
    option: '--branch',
    value: branch
  });
  if (!new RegExp(`^[0-9a-f]{${String(objectNameLength)}}$`, 'u').test(expectedHeadOid))
    throw new CrateReleaseError(
      `The expected head must be a full ${String(objectNameLength)}-digit lower-case hexadecimal object name, got ${JSON.stringify(expectedHeadOid)}.`,
    );
  if (headline.trim() === '' || headline.includes('\n'))
    throw new CrateReleaseError(`The commit headline must be one non-empty line, got ${JSON.stringify(headline)}.`);
  if (files.length === 0)
    throw new CrateReleaseError('A commit needs at least one file.');
  return JSON.stringify({
    query: commitMutation,
    variables: {
      input: {
        branch: {
          repositoryNameWithOwner: repo,
          branchName: branch
        },
        message: { headline },
        expectedHeadOid,
        fileChanges: {
          additions: files.map(function addition(file) {
            return {
              path: file.path,
              contents: Buffer.from(file.contents)
                .toString('base64'),
            };
          }),
        },
      },
    },
  });
}

/**
 Read the answer of the commit mutation.

 @param body - JSON text printed by `gh api graphql`

 @returns the created commit, or a miss that carries every error message GitHub gave, or says what was malformed

 @example
 ```ts
 parseCommitResponse('{"data":{"createCommitOnBranch":{"commit":{"oid":"a","url":"u"}}}}');
 ```
 */
export function parseCommitResponse(body: string): Lookup<CreatedCommit> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if (!isRecord(answer.value))
    return missingValue('The answer is not an object.');
  /**
   GraphQL errors, when the answer lists any.
   */
  const errors: unknown = answer.value.errors;
  if (Array.isArray(errors) && errors.length > 0)
    return missingValue((errors as readonly unknown[]).map(function messageOf(error) {
      return isRecord(error) && ((typeof error.message) === 'string') ? error.message as string : JSON.stringify(error);
    })
      .join('; '));
  /**
   The mutation's payload, when the answer carries data.
   */
  const data: unknown = isRecord(answer.value.data) ? answer.value.data.createCommitOnBranch : undefined;
  /**
   The commit in the payload.
   */
  const commit: unknown = isRecord(data) ? data.commit : undefined;
  if (!isRecord(commit) || ((typeof commit.oid) !== 'string') || ((typeof commit.url) !== 'string'))
    return missingValue('The answer holds no created commit.');
  return foundValue({
    oid: commit.oid as string,
    url: commit.url as string
  });
}
