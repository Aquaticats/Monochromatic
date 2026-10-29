/**
 Human and machine CLI output rendering.
 
 @module
 */

import type { AppliedResult, } from './cli-result.ts';
import type { GitHubRepository, } from './github-model.ts';
import type { RenderedIssue, } from './issue-model.ts';
import type { InputPosition, } from './model.ts';
import type { CreatedIssue, } from './publisher-model.ts';

/**
 Complete command help text with disclosure warnings.
 */
export const HELP_TEXT: string = [
  'Usage:',
  '  open-code-review-issue --interactive INPUT [--repo https://github.com/OWNER/NAME]',
  '  open-code-review-issue --non-interactive FILE [--repo https://github.com/OWNER/NAME]',
  '    [--apply [--non-security-only | --all]]',
  '',
  'Exactly one mode and one positional INPUT are required.',
  'Interactive INPUT is a named file or shell-quoted JSON; non-interactive INPUT is a named file.',
  'Piped stdin, redirected stdin, TTY paste, and `-` are never input sources.',
  '',
  'Security:',
  '  `--all` asserts every security finding is safe for public disclosure.',
  '  Unresolved suspected vulnerabilities must follow SECURITY.md private reporting.',
  '  OCR content remains active GitHub Markdown, including links, HTML, and mentions.',
  '  Code fallbacks can expose source or secrets in Issue titles and notifications.',
  '',
  'Non-interactive without --apply prints a redacted publication plan.',
].join('\n',);

/**
 Writes one JSON value and terminal newline.
 
 @param output - Destination standard output stream.
 
 @param value - Machine-readable object.
 
 @example
 ```ts
 writeJson({ output: process.stdout, value: {} });
 ```
 */
export function writeJson({
  output,
  value,
}: {
  readonly output: NodeJS.WritableStream;
  readonly value: unknown;
},): void {
  output.write(`${JSON.stringify(value,)}\n`,);
}

/**
 Formats one safe input position label.
 
 @param position - Record ordinal or JSONL line.
 
 @returns Human-readable position.
 */
function formatPosition(position: InputPosition,): string {
  return `${position.kind} ${String(position.value,)}`;
}

/**
 Writes final interactive batch summary before confirmation.
 
 @param output - Interactive TTY standard output.
 
 @param repository - Canonical destination identity.
 
 @param issues - Selected complete Issues.
 
 @param withheldCount - Security findings not authorized for publication.
 
 @example
 ```ts
 writeInteractiveSummary({ output, repository, issues, withheldCount: 0 });
 ```
 */
export function writeInteractiveSummary({
  output,
  repository,
  issues,
  withheldCount,
}: {
  readonly output: NodeJS.WritableStream;
  readonly repository: GitHubRepository;
  readonly issues: readonly RenderedIssue[];
  readonly withheldCount: number;
},): void {
  /**
   Selected title lines safe because user already reviewed security choices.
   */
  const titles = issues.map(function titleLine(issue,): string {
    /**
     Optional explicit ordinary classification marker.
     */
    const marker = issue.classificationMarker === undefined
      ? ''
      : `${issue.classificationMarker} `;
    return `- ${marker}${issue.title}`;
  },);
  output.write([
    '',
    `Destination: ${repository.url}`,
    'Issues to create:',
    ...titles,
    `Withheld SECURITY findings: ${String(withheldCount,)}`,
    '',
  ].join('\n',),);
}

/**
 Writes human-readable created Issue URLs.
 
 @param output - Interactive TTY standard output.
 
 @param created - Confirmed created or reconciled Issues.
 
 @example
 ```ts
 writeCreatedIssues({ output, created: [] });
 ```
 */
export function writeCreatedIssues({
  output,
  created,
}: {
  readonly output: NodeJS.WritableStream;
  readonly created: readonly CreatedIssue[];
},): void {
  output.write('Created GitHub Issues:\n',);
  created.forEach(function writeCreated(issue,): void {
    output.write(`- ${formatPosition(issue.position,)}: ${issue.url}\n`,);
  },);
}

/**
 Writes exact clean cancellation message.
 
 @param output - Interactive TTY standard output.
 
 @example
 ```ts
 writeCancellation(process.stdout);
 ```
 */
export function writeCancellation(output: NodeJS.WritableStream,): void {
  output.write('Issue creation canceled.\n',);
}

/**
 Writes final non-interactive applied result.
 
 @param output - Standard output reserved for one JSON object.
 
 @param result - Complete success or handled failure result.
 
 @example
 ```ts
 writeAppliedResult({ output, result });
 ```
 */
export function writeAppliedResult({
  output,
  result,
}: {
  readonly output: NodeJS.WritableStream;
  readonly result: AppliedResult;
},): void {
  writeJson({
    output,
    value: result,
  });
}
