import type { SessionEntry, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildMorphInput,
  extractLatestQuery,
  wrapMorphOutput,
} from './formatting.ts';

/** Build a minimal message entry for query extraction tests. */
function makeMessageEntry({
  role,
  content,
}: {
  role: string;
  content: unknown;
},): SessionEntry {
  return {
    type: 'message',
    id: 'test',
    parentId: 'root',
    timestamp: new Date().toISOString(),
    message: {
      role,
      content,
      timestamp: new Date().toISOString(),
    },
  } as unknown as SessionEntry;
}

await describe({
  name: '',
  children: [
    describe({
      name: extractLatestQuery.name,
      children: [
        it({
          name: 'extracts last user message text',
          fn: async () => {
            const entries = [
              makeMessageEntry({ role: 'user',
                content: 'Tell me about European capitals', },),
            ];
            const query = extractLatestQuery({
              branchEntries: entries,            },);
            expect(query,).toBe('Tell me about European capitals',);
          },
        },),
        it({
          name: 'custom instructions take priority',
          fn: async () => {
            const entries = [
              makeMessageEntry({ role: 'user',
                content: 'Tell me about European capitals', },),
            ];
            const query = extractLatestQuery({
              branchEntries: entries,
              customInstructions: 'focus on geography',
            },);
            expect(query,).toBe('focus on geography',);
          },
        },),
        it({
          name: 'returns empty string for empty branch entries',
          fn: async () => {
            const query = extractLatestQuery({
              branchEntries: [],            },);
            expect(query,).toBe('',);
          },
        },),
        it({
          name: 'extracts from content array',
          fn: async () => {
            const entries = [
              {
                type: 'message',
                id: 'test',
                parentId: 'root',
                timestamp: new Date().toISOString(),
                message: {
                  role: 'user',
                  content: [
                    {
                      type: 'text',
                      text: 'What is the capital?',
                    },
                  ],
                  timestamp: new Date().toISOString(),
                },
              } as unknown as SessionEntry,
            ];
            const query = extractLatestQuery({
              branchEntries: entries,            },);
            expect(query,).toBe('What is the capital?',);
          },
        },),
      ],
    },),
    describe({
      name: buildMorphInput.name,
      children: [
        it({
          name: 'wraps previous summary in keepContext tags',
          fn: async () => {
            const input = buildMorphInput({
              serializedConversation: 'conversation text',
              previousSummary: 'Previous summary content',
            },);
            expect(input,).toContain('<keepContext>',);
            expect(input,).toContain('[Previous compacted context]',);
            expect(input,).toContain('Previous summary content',);
            expect(input,).toContain('</keepContext>',);
            expect(input,).toContain('conversation text',);
          },
        },),
        it({
          name: 'omits keepContext when no previous summary',
          fn: async () => {
            const input = buildMorphInput({
              serializedConversation: 'conversation text',
            },);
            expect(input,).not.toContain('<keepContext>',);
            expect(input,).toContain('conversation text',);
          },
        },),
        it({
          name: 'omits keepContext for empty previous summary',
          fn: async () => {
            const input = buildMorphInput({
              serializedConversation: 'conversation text',
              previousSummary: '   ',
            },);
            expect(input,).not.toContain('<keepContext>',);
          },
        },),
      ],
    },),
    describe({
      name: wrapMorphOutput.name,
      children: [
        it({
          name: 'wraps with header and XML tags',
          fn: async () => {
            const raw = '[User]: Paris is the capital';
            const wrapped = wrapMorphOutput(raw,);
            expect(wrapped,).toContain('<morph-compacted-history>',);
            expect(wrapped,).toContain('</morph-compacted-history>',);
            expect(wrapped,).toContain(
              'Morph Compact verbatim transcript',
            );
            expect(wrapped,).toContain('Paris is the capital',);
          },
        },),
        it({
          name: 'trims raw output',
          fn: async () => {
            const wrapped = wrapMorphOutput('  padded  \n',);
            expect(wrapped,).toContain('padded',);
            expect(wrapped,).not.toContain('  padded  \n',);
          },
        },),
      ],
    },),
  ],
},);
