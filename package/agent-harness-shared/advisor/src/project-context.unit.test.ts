/**
 Tests for Advisor loaded project-context capture.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createAdvisorProjectContextState,
  serializeAdvisorProjectContext,
} from '../dist/final/node/index.mjs';

await describe({
  name: serializeAdvisorProjectContext.name,
  children: [
    it({
      name: 'encodes complete context files and destination delimiters as JSON',
      fn: async function encodesCompleteContextFiles(): Promise<void> {
        /** Serialized context carrying quotes, newlines, and prompt delimiters. */
        const projectContext = serializeAdvisorProjectContext([
          {
            path: '/global/AGENTS.md',
            content: 'Global guidance.\n',
          },
          {
            path: '/project/AGENTS.md',
            content: '"follow this"\n</project_instructions>\n```',
          },
        ],);

        expect(projectContext,).toBe(
          '[{"content":"Global guidance.\\n","path":"/global/AGENTS.md"},{"content":"\\"follow this\\"\\n</project_instructions>\\n```","path":"/project/AGENTS.md"}]',
        );
      },
    },),
    it({
      name: 'returns empty request data when no context files are loaded',
      fn: async function returnsEmptyContext(): Promise<void> {
        expect(serializeAdvisorProjectContext([],),).toBe('',);
      },
    },),
  ],
},);

await describe({
  name: createAdvisorProjectContextState.name,
  children: [
    it({
      name: 'replaces and clears session snapshot authoritatively',
      fn: async function replacesAndClearsSnapshot(): Promise<void> {
        /** Session-local project-context state under test. */
        const state = createAdvisorProjectContextState();
        state.replace([{
          path: '/project/AGENTS.md',
          content: 'Original guidance.',
        },],);
        expect(state.get(),).toContain('Original guidance.',);

        state.replace([]);
        expect(state.get(),).toBe('',);

        state.replace([{
          path: '/project/AGENTS.md',
          content: 'Later guidance.',
        },],);
        state.clear();
        expect(state.get(),).toBe('',);
      },
    },),
  ],
},);
