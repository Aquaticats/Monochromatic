/**
 GitHub URL planning constants.
 
 @module
 */

//region Hosts

/**
 GitHub repository host served by gh.
 */
const GITHUB_HOST = 'github.com';

/**
 GitHub Gist host served by `gh gist view`.
 */
const GIST_GITHUB_HOST = 'gist.github.com';

/**
 GitHub REST API host forwarded to `gh api`.
 */
const API_GITHUB_HOST = 'api.github.com';

/**
 GitHub raw file host whose paths mirror blob URL paths.
 */
const RAW_GITHUB_CONTENT_HOST = 'raw.githubusercontent.com';

/**
 Hosts this package maps onto gh invocations.
 */
const MAPPED_GITHUB_HOSTS: readonly string[] = [
  GITHUB_HOST,
  GIST_GITHUB_HOST,
  API_GITHUB_HOST,
  RAW_GITHUB_CONTENT_HOST,
];

/**
 Optional www label removed before host matching.
 */
const WWW_HOST_PREFIX = 'www.';

/**
 Host label separator and optional trailing root dot.
 */
const HOST_DOT = '.';

//endregion Hosts

//region URL structure

/**
 Path separator used by GitHub URLs and repository paths.
 */
const URL_SLASH = '/';

/**
 Empty text used for absent paths and root directory listings.
 */
const EMPTY_SEGMENT = '';

/**
 HTTPS scheme accepted for GitHub URLs.
 */
const HTTPS_SCHEME = 'https:';

/**
 HTTP scheme accepted because GitHub redirects it to HTTPS.
 */
const HTTP_SCHEME = 'http:';

/**
 Leading dash rejected in positional gh arguments so they cannot parse as flags.
 */
const DASH_PREFIX = '-';

/**
 Characters accepted in owner, repository, and gist positional arguments.
 */
const TOKEN_ALLOWED_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-';

/**
 Highest character rejected as a control character or as space.
 */
const SPACE_CHARACTER = ' ';

/**
 Highest character accepted inside printable ASCII.
 */
const TILDE_CHARACTER = '~';

/**
 Longest accepted issue or pull request number.
 */
const MAXIMUM_REFERENCE_NUMBER_DIGITS = 9;

/**
 Lowest accepted decimal digit.
 */
const LOWEST_DIGIT = '0';

/**
 Highest accepted decimal digit.
 */
const HIGHEST_DIGIT = '9';

//endregion URL structure

//region GitHub URL sections

/**
 Repository file view section.
 */
const BLOB_SECTION = 'blob';

/**
 Raw file view section.
 */
const RAW_SECTION = 'raw';

/**
 Blame view section, whose file content matches blob output.
 */
const BLAME_SECTION = 'blame';

/**
 Repository directory view section.
 */
const TREE_SECTION = 'tree';

/**
 Issue detail section.
 */
const ISSUES_SECTION = 'issues';

/**
 Pull request section.
 */
const PULL_SECTION = 'pull';

/**
 Single commit section.
 */
const COMMIT_SECTION = 'commit';

/**
 Branch or tag comparison section.
 */
const COMPARE_SECTION = 'compare';

/**
 Release section.
 */
const RELEASES_SECTION = 'releases';

/**
 Release tag subsection.
 */
const RELEASE_TAG_SECTION = 'tag';

/**
 Pull request changed-files subsection.
 */
const PULL_FILES_SECTION = 'files';

/**
 Canonical issue and pull request URL prefix accepted by `gh issue view` and `gh pr view`.
 */
const GITHUB_URL_PREFIX = 'https://github.com/';

/**
 Path segments consumed by owner, repository, and section before section arguments start.
 */
const SECTION_ARGUMENT_OFFSET = 3;

//endregion GitHub URL sections

//region gh invocation fragments

/**
 gh subcommand calling the GitHub REST API.
 */
const GH_API_SUBCOMMAND = 'api';

/**
 gh flag adding one request header.
 */
const GH_HEADER_FLAG = '-H';

/**
 gh flag selecting one jq output projection.
 */
const GH_JQ_FLAG = '--jq';

/**
 gh flag selecting an explicit repository.
 */
const GH_REPO_FLAG = '--repo';

/**
 gh flag appending thread comments to issue and pull request output.
 */
const GH_COMMENTS_FLAG = '--comments';

/**
 gh flag selecting diff color behavior.
 */
const GH_COLOR_FLAG = '--color';

/**
 Diff color value keeping ANSI escapes out of piped output even when the environment forces color.
 */
const GH_COLOR_NEVER = 'never';

/**
 Request header returning repository file bytes without a JSON envelope.
 */
const GH_RAW_ACCEPT_HEADER = 'Accept: application/vnd.github.raw';

/**
 Request header returning one commit or comparison as a unified diff.
 */
const GH_DIFF_ACCEPT_HEADER = 'Accept: application/vnd.github.diff';

/**
 jq projection rendering one contents response as tab-separated listing lines.
 
 `String.raw` keeps jq's own backslash escapes intact for the child process.
 */
const GH_DIRECTORY_LISTING_JQ: string = String.raw`if type == "array" then .[] else . end | "\(.type)\t\(.name)\t\(.size)"`;

//endregion gh invocation fragments

export {
  BLOB_SECTION,
  BLAME_SECTION,
  COMMIT_SECTION,
  COMPARE_SECTION,
  DASH_PREFIX,
  EMPTY_SEGMENT,
  GIST_GITHUB_HOST,
  GITHUB_HOST,
  GITHUB_URL_PREFIX,
  GH_API_SUBCOMMAND,
  GH_COLOR_FLAG,
  GH_COLOR_NEVER,
  GH_COMMENTS_FLAG,
  GH_DIFF_ACCEPT_HEADER,
  GH_DIRECTORY_LISTING_JQ,
  GH_HEADER_FLAG,
  GH_JQ_FLAG,
  GH_RAW_ACCEPT_HEADER,
  GH_REPO_FLAG,
  HIGHEST_DIGIT,
  HOST_DOT,
  HTTP_SCHEME,
  HTTPS_SCHEME,
  ISSUES_SECTION,
  LOWEST_DIGIT,
  MAPPED_GITHUB_HOSTS,
  MAXIMUM_REFERENCE_NUMBER_DIGITS,
  PULL_FILES_SECTION,
  PULL_SECTION,
  RAW_GITHUB_CONTENT_HOST,
  RAW_SECTION,
  RELEASES_SECTION,
  RELEASE_TAG_SECTION,
  SECTION_ARGUMENT_OFFSET,
  SPACE_CHARACTER,
  TILDE_CHARACTER,
  TOKEN_ALLOWED_CHARACTERS,
  TREE_SECTION,
  URL_SLASH,
  WWW_HOST_PREFIX,
  API_GITHUB_HOST,
};
