/**
 Deterministic GitHub Issue body and request rendering.
 
 @module
 */

import type {
  RenderedIssue,
  SourceLink,
} from './issue-model.ts';
import { renderIssueTitle, } from './issue-title.ts';
import {
  escapeMarkdownInline,
  indentCode,
} from './markdown.ts';
import type { NormalizedFinding, } from './model.ts';

/**
 Encodes source path segments for GitHub blob URL and Markdown destination.
 
 @param path - Untrusted repository-relative source path.
 
 @returns Slash-preserving URL path with Markdown parentheses encoded.
 
 @example
 ```ts
 encodeSourcePath('src/a b.ts'); // 'src/a%20b.ts'
 ```
 */
function encodeSourcePath(path: string,): string {
  return path.split('/',)
    .map(function encodeSegment(segment,): string {
      return encodeURIComponent(segment,)
        .replaceAll(
        '(',
        '%28',
        )
        .replaceAll(
        ')',
        '%29',
        );
    },)
    .join('/',);
}

/**
 Builds plain or commit-pinned source location line.
 
 @param finding - Validated finding carrying source range.
 
 @param sourceLink - Verified repository and commit coordinates.
 
 @returns Markdown source location item.
 
 @example
 ```ts
 renderSourceLocation({ finding });
 ```
 */

function renderSourceLocation({
  finding,
  sourceLink,
}: {
  readonly finding: NormalizedFinding;
  readonly sourceLink?: SourceLink;
},): string {
  /**
   Escaped source location text displayed in either branch.
   */
  const location = `${escapeMarkdownInline(finding.path,)}:${String(finding.startLine,)}-${String(finding.endLine,)}`;
  if (sourceLink === undefined) {
    return `- Location: ${location}`;
  }
  /**
   Commit-pinned URL whose path is encoded at final interpolation.
   */
  const url = `https://github.com/${sourceLink.repository}/blob/${sourceLink.commit}/${encodeSourcePath(finding.path,)}#L${String(finding.startLine,)}-L${String(finding.endLine,)}`;
  return `- Location: [${location}](${url})`;
}

/**
 Renders optional indented code section.
 
 @param heading - Stable Markdown section heading.
 
 @param code - OCR source field.
 
 @returns Complete section lines or empty list when field is absent.
 
 @example
 ```ts
 renderCodeSection({ heading: 'Existing code', code: 'x' });
 ```
 */
function renderCodeSection({
  heading,
  code,
}: {
  readonly heading: string;
  readonly code: string;
},): readonly string[] {
  return code === ''
    ? []
    : [
      `## ${heading}`,
      '',
      indentCode(code,),
      '',
    ];
}

/**
 Renders complete body while retaining OCR content as active Markdown.
 
 @param finding - Validated normalized finding.
 
 @param sourceLink - Verified repository and commit coordinates.
 
 @returns Deterministic GitHub Flavored Markdown body.
 
 @example
 ```ts
 renderIssueBody(finding);
 ```
 */
export function renderIssueBody({
  finding,
  sourceLink,
}: {
  readonly finding: NormalizedFinding;
  readonly sourceLink?: SourceLink;
},): string {
  /**
   Category metadata with explicit missing value.
   */
  const category = finding.category ?? 'uncategorized';
  /**
   Severity metadata with explicit missing value.
   */
  const severity = finding.severity ?? 'unspecified';
  return [
    '## Finding',
    '',
    finding.content,
    '',
    '## Source',
    '',
    renderSourceLocation({
      finding,
      ...(sourceLink === undefined ? {} : { sourceLink, }),
    },),
    `- Category: \`${category}\``,
    `- Severity: \`${severity}\``,
    '',
    ...renderCodeSection({
      heading: 'Existing code',
      code: finding.existingCode,
    },),
    ...renderCodeSection({
      heading: 'Suggested code',
      code: finding.suggestionCode,
    },),
    '## OpenCodeReview',
    '',
    'Generated from OpenCodeReview structured output.',
  ].join('\n',);
}

/**
 Selects explicit marker for ordinary exceptional category states.
 
 @param finding - Validated finding classification.
 
 @returns Optional OTHER or UNCATEGORIZED marker property.
 */
function classificationMarkerMetadata(
  finding: NormalizedFinding,
): Pick<RenderedIssue, 'classificationMarker'> {
  if (finding.category === 'other') {
    return { classificationMarker: 'OTHER', };
  }
  if (finding.category === undefined) {
    return { classificationMarker: 'UNCATEGORIZED', };
  }
  return {};
}

/**
 Renders one complete create-only Issue request.
 
 @param finding - Validated normalized finding.
 
 @param needsTriageLabel - Whether destination label exists.
 
 @param sourceLink - Verified repository and commit coordinates.
 
 @returns Deterministic title, body, labels, position, and security marker.
 
 @example
 ```ts
 renderIssue({ finding, needsTriageLabel: true });
 ```
 */
export function renderIssue({
  finding,
  needsTriageLabel,
  sourceLink,
}: {
  readonly finding: NormalizedFinding;
  readonly needsTriageLabel: boolean;
  readonly sourceLink?: SourceLink;
},): RenderedIssue {
  return {
    position: finding.position,
    security: finding.category === 'security',
    ...classificationMarkerMetadata(finding,),
    title: renderIssueTitle({
      finding,
      needsTriageLabel,
    }),
    body: renderIssueBody({
      finding,
      ...(sourceLink === undefined ? {} : { sourceLink, }),
    },),
    labels: needsTriageLabel ? ['needs-triage',] : [],
  };
}
