/**
 Distorted source-grounded content must not acquire a duplicate deletion diagnosis.

 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { buildAdjudicationMessages, buildCriticMessages, } from '../dist/final/node/index.mjs';

await describe({
  name: 'accuracy category scope',
  children: [
    it({
      name: 'DEFINES the distinction for the actual critic without changing the source or target',
      fn: async () => {
        const sourceText = '朋友问候了猫。';
        const targetText = 'The cat greeted her friend.';
        const messages = buildCriticMessages({ sourceText, targetText, });
        const system = messages.find(message => message.role === 'system')?.content ?? '';
        const user = messages.find(message => message.role === 'user')?.content ?? '';
        expect(system).toContain('use accuracy/mistranslation when a source-grounded event or statement');
        expect(system).toContain('Use accuracy/addition for independent extra information');
        expect(system).toContain('Do not additionally label the same distorted rendering as addition or omission');
        expect(system).toContain('Genuine independent added information remains an addition');
        expect(user).toContain(sourceText);
        expect(user).toContain(targetText);
      },
    }),
    it({
      name: 'REQUIRES the panel to check the diagnosis category rather than approve any error at that location',
      fn: async () => {
        const plan = buildAdjudicationMessages({ sourceText: '朋友问候了猫。', targetText: 'The cat greeted her friend.', clusters: [], });
        const system = plan.messages.find(message => message.role === 'system')?.content ?? '';
        expect(system).toContain('A supported defect must fit its claimed category');
        expect(system).toContain('vote unsupported on an addition diagnosis of that same distortion');
        expect(system).toContain('Genuine independent added information remains an addition');
        expect(plan.claimIds).toHaveLength(0);
        expect(plan.clusterIds).toHaveLength(0);
      },
    }),
  ],
});
