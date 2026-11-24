/**
 Builds the refusal that names why a deletion target is not written as the rule requires.
 
 A refusal is only useful if the reader can act on it,
 so it names the command and the word that broke the rule,
 says which check fired and why,
 names the file and line when the command came from a script,
 and gives the spelling the rule asks for.
 
 @module
 */

import type {
  LiteralRmPathCheck,
  LiteralRmPathFinding,
} from './literal-rm-path-finding.ts';

//region Constants

/**
 How many findings one refusal lists before it summarizes the rest.
 
 A refusal is read by a model with a bounded context,
 and the first few findings already show the shape of the mistake.
 */
const LISTED_FINDING_LIMIT = 5;

/**
 Phrases naming each check, so a reader knows which one fired.
 */
const CHECK_PHRASES: ReadonlyMap<LiteralRmPathCheck, string> = new Map([
  [
    'another-program-supplies-the-paths',
    'another program supplies the paths',
  ],
  [
    'find-deletes-the-paths-it-found',
    '`find -delete` deletes the paths it found, and none is written',
  ],
  [
    'shell-computes-the-path',
    'the shell computes this path at run time',
  ],
  [
    'target-collapses-to-a-broader-path',
    'the target collapses to a broader path than it looks like',
  ],
],);

/**
 The rule this refusal enforces, in the words a reader can act on.
 
 @example
 ```ts
 return LITERAL_RM_PATH_BAN_REASON;
 ```
 */
const LITERAL_RM_PATH_BAN_REASON: string = [
  'Blocked: this deletion command names its target in a way this repository forbids.',
  'The rule is that every deletion writes each path literally:',
  '`rm`, `rmdir`, `unlink`, `shred` and `find -delete` alike.',
  `The rule forbids environment variables such as \`$HOME\` and \`\${DIR}\` in a deletion path,`,
  'because an unset or empty value widens what gets deleted while the command still looks narrow;',
  'a substitution, a backtick, or an arithmetic expression is unreadable for the same reason.',
  'A target that is `/`, the account home folder, a top-level folder,',
  'a path holding `//`, or a path with a segment `undefined` or `null` is refused too,',
  'because that is the shape an empty value collapses to.',
]
  .join(' ',);

//endregion Constants

//region Refusal text

/**
 Spells why one finding fired.
 
 @param finding - finding to explain
 
 @returns the check's phrase, with the reading that fired it
 
 @example
 ```ts
 findingReason({ command: 'rmdir -p "$d"', word: '"$d"', check: 'shell-computes-the-path', });
 ```
 */
function findingReason(finding: LiteralRmPathFinding,): string {
  /**
   Phrase naming the check that fired.
   */
  const phrase = CHECK_PHRASES.get(finding.check,)
    ?? finding.check;
  return (finding.detail === undefined)
    ? phrase
    : `${phrase}: ${finding.detail}`;
}

/**
 Spells one finding so a reader can locate it.
 
 @param finding - finding to spell
 
 @returns the command, the offending word, the check, and the file and line when known
 
 @example
 ```ts
 findingSentence({ command: 'rm -rf "$d"', word: '"$d"', check: 'shell-computes-the-path', });
 ```
 */
function findingSentence(finding: LiteralRmPathFinding,): string {
  /**
   Where the finding sits, which is the file and line for a script.
   */
  const location = (finding.file === undefined)
    ? ''
    : `${finding.file}${finding.line === undefined ? '' : ` line ${String(finding.line,)}`}: `;
  return `${location}\`${finding.command}\` (word \`${finding.word}\`; ${findingReason(finding,)})`;
}

/**
 Builds the refusal text for the findings one tool call produced.
 
 @param findings - what the call was found to run, never empty
 
 @returns refusal naming each finding, its check, and the spelling the rule asks for
 
 @example
 ```ts
 literalRmPathRefusal({ findings: [{ command: 'rm -rf "$d"', word: '"$d"', check: 'shell-computes-the-path' }] });
 ```
 */
function literalRmPathRefusal({ findings, }: { readonly findings: readonly LiteralRmPathFinding[]; },): string {
  /**
   Findings this refusal spells out, which the limit keeps bounded.
   */
  const listed = findings.slice(
    0,
    LISTED_FINDING_LIMIT,
  );
  /**
   Findings the limit left out, summarized by count.
   */
  const omittedCount = findings.length - listed.length;

  return [
    LITERAL_RM_PATH_BAN_REASON,
    `Found: ${listed.map(findingSentence,)
      .join('; ',)}${omittedCount === 0 ? '' : `; and ${String(omittedCount,)} more`}.`,
    'Write each path literally, in a single deletion command,',
    'for example `rm --recursive /home/user/temp/agent/<folder>/scratch`.',
    'Do not re-express the deletion through a script, a heredoc, another tool,',
    'code mode, or another program: those forms delete the same paths,',
    'and the reader still cannot see which ones.',
  ]
    .join(' ',);
}

//endregion Refusal text

export {
  CHECK_PHRASES,
  findingReason,
  findingSentence,
  LITERAL_RM_PATH_BAN_REASON,
  literalRmPathRefusal,
};
