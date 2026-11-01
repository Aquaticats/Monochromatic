/**
 Custom message helper for manual Advisor command output.
 
 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { ADVISOR_MESSAGE_TYPE, } from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type { AdvisorRunResult, } from './types.ts';

//region Public API

/**
 Send manual Advisor output as a rendered custom message.
 
 @param pi - pi extension API
 
 @param result - Advisor review result
 
 @mutates pi - `pi.sendMessage` appends rendered Advisor output to Pi host state
 
 @example
 ```typescript
 sendAdvisorMessage({ pi, result });
 ```
 */
export function sendAdvisorMessage(
  {
    pi,
    result,
  }: {
    readonly pi: ForeignHostCapability<ExtensionAPI>;
    readonly result: AdvisorRunResult;
  },
): void {
  pi.sendMessage({
    customType: ADVISOR_MESSAGE_TYPE,
    content: result.text,
    display: true,
    details: result.details,
  },);
}

//endregion Public API
