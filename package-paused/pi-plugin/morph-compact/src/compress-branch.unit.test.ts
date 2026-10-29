/**
 Tests for {@link compressBranch}.
 
 Uses sequential execution because all tests stub `globalThis.fetch`
 (the network seam behind the Morph client) and sinon refuses to wrap an
 already-wrapped method, so concurrent runs would conflict.
 
 @module
 */

import type {
  CompactionEntry,
  SessionEntry,
  SessionMessageEntry,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { compressBranch, } from './compress-branch.ts';

//region Test helpers

/** Build a minimal message entry for tests. */
function makeMessageEntry({
  role,
  content,
  id = 'msg-1',
}: {
  role: string;
  content: unknown;
  id?: string;
},): SessionEntry {
  return {
    type: 'message',
    id,
    parentId: 'root',
    timestamp: new Date().toISOString(),
    message: {
      role,
      content,
      timestamp: new Date().toISOString(),
    },
  } as unknown as SessionMessageEntry;
}

/** Build a minimal compaction entry for tests. */
function makeCompactionEntry({
  summary,
  id = 'compact-1',
}: {
  summary: string;
  id?: string;
},): SessionEntry {
  return {
    type: 'compaction',
    id,
    parentId: 'root',
    timestamp: new Date().toISOString(),
    summary,
    firstKeptEntryId: 'kept-1',
    tokensBefore: 50_000,
  } as CompactionEntry;
}

/** Minimal params without messages (nothing to compress). */
function emptyParams(): {
  branchEntries: SessionEntry[];
  apiKey: string;
} {
  return {
    branchEntries: [],
    apiKey: 'test-key',
  };
}

/** Simulated Morph Compact API response. */
const MOCK_COMPACT_OUTPUT = 'compressed output text';

/** Build a mock CompactResult for sinon stubs. */
function mockCompactResult(
  output: string,
): {
  id: string;
  output: string;
  messages: {
    role: string;
    content: string;
    compacted_line_ranges: {
      start: number;
      end: number;
    }[];
    kept_line_ranges: {
      start: number;
      end: number;
    }[];
  }[];
  usage: {
    input_tokens: number;
    output_tokens: number;
    compression_ratio: number;
    processing_time_ms: number;
  };
  model: string;
} {
  return {
    id: 'mock-id',
    output,
    messages: [
      {
        role: 'user',
        content: output,
        compacted_line_ranges: [],
        kept_line_ranges: [],
      },
    ],
    usage: {
      input_tokens: 1_000,
      output_tokens: 300,
      compression_ratio: 0.3,
      processing_time_ms: 500,
    },
    model: 'mock-model',
  };
}

//endregion

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: compressBranch.name,
      // Sequential execution required: tests stub globalThis.fetch and would
      // fail under concurrent execution because sinon refuses to wrap an
      // already-wrapped method.
      concurrency: 1,
      children: [
        // Thunks required for concurrency: 1 so execution
        // is deferred until the previous test settles
        it({
          name: 'throws when there are no messages and no previous compaction',
          fn: async () => {
            await expect(
              compressBranch(emptyParams(),),
            )
              .rejects
              .toThrow('Nothing to compress',);
          },
        },),
        it({
          name:
            'returns previous summary directly when no new messages exist after compaction',
          fn: async () => {
            const previousSummary = 'Previous compacted context here';
            const entries: SessionEntry[] = [
              makeCompactionEntry({ summary: previousSummary, },),
            ];
            const result = await compressBranch({
              ...emptyParams(),
              branchEntries: entries,
            },);
            expect(result,).toBe(previousSummary,);
          },
        },),
        it({
          name: 'calls Morph API and returns wrapped output for new messages',
          fn: async ({ sinon, },) => {
            const compactStub = sinon
              .stub(
              globalThis,
              'fetch',
              )
              .resolves(
              Response.json(mockCompactResult(MOCK_COMPACT_OUTPUT,),),
              );

            const entries: SessionEntry[] = [
              makeMessageEntry({
                role: 'user',
                content: 'What is the capital of France?',
                id: 'msg-1',
              },),
              makeMessageEntry({
                role: 'assistant',
                content: 'Paris is the capital of France.',
                id: 'msg-2',
              },),
            ];
            const result = await compressBranch({
              ...emptyParams(),
              branchEntries: entries,
            },);
            expect(result,).toContain(
              '<morph-compacted-history>',
            );
            expect(result,).toContain(
              '</morph-compacted-history>',
            );
            expect(result,).toContain(
              MOCK_COMPACT_OUTPUT,
            );
            expect(compactStub,).toHaveBeenCalled();
          },
        },),
        it({
          name: 'includes previous summary and new messages in API call',
          fn: async ({ sinon, },) => {
            const compactStub = sinon
              .stub(
              globalThis,
              'fetch',
              )
              .resolves(
              Response.json(mockCompactResult(MOCK_COMPACT_OUTPUT,),),
              );

            const previousSummary = 'Old context about Spain';
            const entries: SessionEntry[] = [
              makeCompactionEntry({ summary: previousSummary, },),
              makeMessageEntry({
                role: 'user',
                content: 'Now tell me about Italy',
                id: 'msg-1',
              },),
            ];
            const result = await compressBranch({
              ...emptyParams(),
              branchEntries: entries,
            },);
            // Should have called Morph API (not just returned
            // previous summary) and returned wrapped output
            expect(result,).toContain(
              '<morph-compacted-history>',
            );
            expect(result,).toContain(
              MOCK_COMPACT_OUTPUT,
            );
            expect(compactStub,).toHaveBeenCalled();
          },
        },),
        it({
          name: 'passes custom instructions to query extraction',
          fn: async ({ sinon, },) => {
            const compactStub = sinon
              .stub(
              globalThis,
              'fetch',
              )
              .resolves(
              Response.json(mockCompactResult(MOCK_COMPACT_OUTPUT,),),
              );

            const entries: SessionEntry[] = [
              makeMessageEntry({
                role: 'user',
                content: 'General question',
                id: 'msg-1',
              },),
            ];
            const result = await compressBranch({
              ...emptyParams(),
              branchEntries: entries,
              customInstructions: 'focus on authentication',
            },);
            expect(result,).toContain(
              '<morph-compacted-history>',
            );
            // Custom instructions should appear as the query
            // arg in the compact call
            expect(compactStub,).toHaveBeenCalled();
            const callInit = compactStub
              .firstCall
              .args[1] as { body: string; };
            const requestBody = JSON.parse(callInit.body,) as Record<string, unknown>;
            expect(requestBody.query,).toBe(
              'focus on authentication',
            );
          },
        },),
        it({
          name: 'throws when Morph API returns empty output',
          fn: async ({ sinon, },) => {
            sinon
              .stub(
              globalThis,
              'fetch',
              )
              .resolves(
              Response.json(mockCompactResult('',),),
              );

            const entries: SessionEntry[] = [
              makeMessageEntry({
                role: 'user',
                content: 'Some question',
                id: 'msg-1',
              },),
            ];
            await expect(
              compressBranch({
                ...emptyParams(),
                branchEntries: entries,
              },),
            )
              .rejects
              .toThrow('empty output',);
          },
        },),
        it({
          name: 'collects only messages after the last compaction entry',
          fn: async ({ sinon, },) => {
            const compactStub = sinon
              .stub(
              globalThis,
              'fetch',
              )
              .resolves(
              Response.json(mockCompactResult(MOCK_COMPACT_OUTPUT,),),
              );

            const previousSummary = 'Earlier conversation about Germany';
            const entries: SessionEntry[] = [
              makeMessageEntry({
                role: 'user',
                content: 'Old message before compaction',
                id: 'msg-old',
              },),
              makeCompactionEntry({
                summary: previousSummary,
                id: 'compact-1',
              },),
              makeMessageEntry({
                role: 'user',
                content: 'New message after compaction',
                id: 'msg-new',
              },),
            ];
            const result = await compressBranch({
              ...emptyParams(),
              branchEntries: entries,
            },);
            expect(result,).toContain(
              '<morph-compacted-history>',
            );
            // Verify compact was called (messages after
            // compaction triggered API call)
            expect(compactStub,).toHaveBeenCalled();
          },
        },),
        it({
          name: 'uses the last compaction when multiple exist',
          fn: async ({ sinon, },) => {
            const compactStub = sinon
              .stub(
              globalThis,
              'fetch',
              )
              .resolves(
              Response.json(mockCompactResult(MOCK_COMPACT_OUTPUT,),),
              );

            const entries: SessionEntry[] = [
              makeCompactionEntry({
                summary: 'First compaction',
                id: 'compact-1',
              },),
              makeMessageEntry({
                role: 'user',
                content: 'Message between compactions',
                id: 'msg-1',
              },),
              makeCompactionEntry({
                summary: 'Second compaction',
                id: 'compact-2',
              },),
              makeMessageEntry({
                role: 'user',
                content: 'Message after second compaction',
                id: 'msg-2',
              },),
            ];
            const result = await compressBranch({
              ...emptyParams(),
              branchEntries: entries,
            },);
            expect(result,).toContain(
              '<morph-compacted-history>',
            );
            // Verify compact was called with the input
            // containing "Second compaction" as
            // previousSummary
            expect(compactStub,).toHaveBeenCalled();
            const callInit = compactStub
              .firstCall
              .args[1] as { body: string; };
            const requestBody = JSON.parse(callInit.body,) as Record<string, unknown>;
            expect(
              requestBody.input as string,
            )
              .toContain('Second compaction',);
          },
        },),
      ],
    },),
  ],
},);
