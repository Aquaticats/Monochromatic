/**
 Builds the refusal that names an `rm` whose paths are not written out.
 
 A refusal is only useful if the reader can act on it,
 so it names the command and the word that made the path unreadable,
 names the file and line when the command came from a script,
 and gives the spelling the rule asks for.
 
 @module
 */

import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';

//region Constants

/**
 How many findings one refusal lists before it summarizes the rest.
 
 A refusal is read by a model with a bounded context,
 and the first few findings already show the shape of the mistake.
 */
const LISTED_FINDING_LIMIT = 5;

/**
 The rule this refusal enforces, in the words a reader can act on.
 
 @example
 ```ts
 return LITERAL_RM_PATH_BAN_REASON;
 ```
 */
const LITERAL_RM_PATH_BAN_REASON: string = [
  'Blocked: this `rm` command names its paths at run time,',
  'and every `rm` here writes each path literally.',
  `The rule forbids environment variables such as \`$HOME\` and \`\${DIR}\` in an \`rm\` path,`,
  'because an unset or empty variable widens what gets deleted while the command still looks narrow.',
  'A path a substitution, backtick, or arithmetic expression computes is unreadable for the same reason.',
]
  .join(' ',);

//endregion Constants

//region Refusal text

/**
 Spells one finding so a reader can locate it.
 
 @param finding - finding to spell
 
 @returns the command, the offending word, and the file and line when known
 
 @example
 ```ts
 findingSentence({ command: 'rm -rf "$d"', word: '"$d"', }); // '`rm -rf "$d"` (word `"$d"`)'
 ```
 */
function findingSentence(finding: LiteralRmPathFinding,): string {
  /**
   Where the finding sits, which is the file and line for a script.
   */
  const location = (finding.file === undefined)
    ? ''
    : `${finding.file}${finding.line === undefined ? '' : ` line ${String(finding.line,)}`}: `;
  return `${location}\`${finding.command}\` (word \`${finding.word}\`)`;
}

/**
 Builds the refusal text for the findings one tool call produced.
 
 @param findings - what the call was found to run, never empty
 
 @returns refusal naming each finding and the spelling the rule asks for
 
 @example
 ```ts
 literalRmPathRefusal({ findings: [{ command: 'rm -rf "$d"', word: '"$d"', }] });
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
    'Write each path literally, in a single `rm` command,',
    'for example `rm --recursive /home/user/temp/agent/<folder>/scratch`.',
    'Do not re-express the deletion through a script, a heredoc, another tool,',
    'code mode, or another program: those forms delete the same paths,',
    'and the reader still cannot see which ones.',
  ]
    .join(' ',);
}

//endregion Refusal text

export {
  LITERAL_RM_PATH_BAN_REASON,
  literalRmPathRefusal,
};
