/**
 Interactive ordinary and security finding selection flow.
 
 @module
 */

import type {
  InteractiveSelection,
  PromptStreams,
} from './interactive-model.ts';
import {
  promptForExplicitDecision,
  promptForIssues,
} from './interactive-prompts.ts';
import type { RenderedIssue, } from './issue-model.ts';
import type { PublicationPlan, } from './plan-model.ts';

/**
 Writes complete selected security preview before disclosure decision.
 
 @param title - Generated public Issue title.
 
 @param body - Generated public Issue Markdown body.
 
 @param output - Interactive TTY standard output.
 */
function writeSecurityPreview({
  title,
  body,
  output,
}: {
  readonly title: string;
  readonly body: string;
  readonly output: NodeJS.WritableStream;
},): void {
  output.write(`\nSECURITY finding preview\n\nTitle: ${title}\n\n${body}\n\n`,);
}

/**
 Runs separate picker stages and per-security disclosure confirmations.
 
 @param plan - Complete internal publication plan.
 
 @param streams - Explicit TTY streams.
 
 @returns Confirmed issues and every withheld security position.
 
 @example
 ```ts
 await selectInteractiveIssues({ plan, streams });
 ```
 */
export async function selectInteractiveIssues({
  plan,
  streams,
}: {
  readonly plan: PublicationPlan;
  readonly streams: PromptStreams;
},): Promise<InteractiveSelection> {
  /**
   Ordinary candidates shown selected by default.
   */
  const ordinaryCandidates = plan.issues
    .filter(function ordinary(issue,): boolean {
      return !issue.security;
    },);
  /**
   Security candidates shown unselected in separate red picker.
   */
  const securityCandidates = plan.issues
    .filter(function security(issue,): boolean {
      return issue.security;
    },);
  /**
   Ordinary picker result.
   */
  const ordinary = await promptForIssues({
    issues: ordinaryCandidates,
    security: false,
    required: securityCandidates.length === 0,
    streams,
  },);
  /**
   Security picker result.
   */
  const selectedSecurity = await promptForIssues({
    issues: securityCandidates,
    security: true,
    required: ordinary.length === 0,
    streams,
  },);
  /**
   Security confirmations retained for creation.
   */
  const confirmedSecurity: RenderedIssue[] = [];
  for (const issue of selectedSecurity) {
    writeSecurityPreview({
      title: issue.title,
      body: issue.body,
      output: streams.output,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- every selected security finding requires its own sequential disclosure decision.
    if (await promptForExplicitDecision({
      message: 'Publish this SECURITY finding publicly? Type yes or no',
      streams,
    },)) {
      confirmedSecurity.push(issue,);
    }
  }
  /**
   Confirmed security positions used to derive complete withheld set.
   */
  const confirmedPositions = new Set(confirmedSecurity.map(function positionKey(issue,) {
    return `${issue.position
      .kind}:${String(issue.position
        .value,)}`;
  },),);
  return {
    issues: [
      ...ordinary,
      ...confirmedSecurity,
    ],
    withheldPositions: securityCandidates
      .filter(function notConfirmed(issue,): boolean {
        return !confirmedPositions.has(`${issue.position
          .kind}:${String(issue.position
            .value,)}`,);
      },)
      .map(function position(issue,) {
        return issue.position;
      },),
  };
}
