/** Failed-tool accounting is attached once and cleared at session teardown. @module */
import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { createAdvisorOperationLedger, registerAdvisorFailureAccounting, } from '../dist/final/node/index.mjs';

await describe({ name: '', children: [
  it({ name: 'restores matching failed usage once without changing the host error flag', fn: async (): Promise<void> => {
    const handlers = new Map<string, (event: { toolName: string; toolCallId: string }) => unknown>();
    const entries: { type: string; data: unknown }[] = [];
    const pi = {
      on: (event: string, handler: (event: { toolName: string; toolCallId: string }) => unknown) => { handlers.set(event, handler); },
      appendEntry: (type: string, data: unknown) => { entries.push({ type, data }); },
    } as unknown as ExtensionAPI;
    const record = registerAdvisorFailureAccounting(pi,);
    const ledger = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 100, },);
    ledger.record({ id: 1, model: 'p/a', provider: 'p', attempt: 1, state: 'failed', startedAtMs: 0,
      contextChars: 20, estimatedInputTokens: 5, truncated: false, usageIncomplete: true,
      usage: { input: 3, output: 2, cacheRead: 0, cacheWrite: 0, totalTokens: 5,
        cost: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0, total: 2 }, }, },);
    ledger.finish({ end: 'exhausted', now: 1, },);
    const operation = ledger.snapshot();
    record({ toolCallId: 'test-call', operation, },);
    expect(entries,).toEqual([{ type: 'pi-advisor.operation', data: operation, }],);
    const finalize = handlers.get('tool_result',);
    if (finalize === undefined) throw new Error('missing tool_result hook',);
    expect(finalize({ toolName: 'other-tool', toolCallId: 'test-call', },),).toBeUndefined();
    expect(finalize({ toolName: 'advisor', toolCallId: 'wrong-call', },),).toBeUndefined();
    const patch = finalize({ toolName: 'advisor', toolCallId: 'test-call', },);
    expect(patch,).toEqual({ details: { operation, }, usage: operation.usage, },);
    expect(patch,).not.toHaveProperty('isError',);
    expect(finalize({ toolName: 'advisor', toolCallId: 'test-call', },),).toBeUndefined();
    record({ toolCallId: 'later-call', operation, },);
    const shutdown = handlers.get('session_shutdown',);
    if (shutdown === undefined) throw new Error('missing shutdown hook',);
    shutdown({ toolName: '', toolCallId: '', },);
    expect(finalize({ toolName: 'advisor', toolCallId: 'later-call', },),).toBeUndefined();
  }, },),
], },);
