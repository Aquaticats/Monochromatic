/** Relocated source and artifact frames stay hidden without hiding consumer tests. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { formatErrorDeep, } from '@monochromatic-dev/test-harness-core-diagnostic';

await describe({
  name: formatErrorDeep.name,
  children: ['core', 'core-expect', 'core-sandbox', 'core-diagnostic',].flatMap(leaf => [
    `package/test-harness/${leaf}`,
    `node_modules/@monochromatic-dev/test-harness-${leaf}`,
  ].map(packagePath => it({
    name: `filters implementation but retains tests and similarly named packages for ${packagePath}`,
    fn: async () => {
      const error = new Error('relocated assertion',);
      error.stack = [
        'Error: relocated assertion',
        `    at hiddenSource (${packagePath}/src/internal.ts:1:1)`,
        `    at hiddenArtifact (${packagePath}/dist/final/neutral/index.mjs:1:1)`,
        `    at retainedTest (${packagePath}/src/example.unit.test.ts:1:1)`,
        `    at retainedNeighbor (${packagePath}-neighbor/src/consumer.ts:1:1)`,
        '    at retainedCaller (package/application/src/consumer.ts:1:1)',
      ].join('\n',);
      const rendered = (await formatErrorDeep(error,)).join('\n',);
      expect(rendered,).not.toContain('hiddenSource',);
      expect(rendered,).not.toContain('hiddenArtifact',);
      expect(rendered,).toContain('retainedTest',);
      expect(rendered,).toContain('retainedNeighbor',);
      expect(rendered,).toContain('retainedCaller',);
    },
  },),),),
},);
