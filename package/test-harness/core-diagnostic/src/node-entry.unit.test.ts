/** Node's public entry retains filesystem-backed workspace prefix discovery. @module */
import { fileURLToPath, } from 'node:url';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { formatErrorDeep, } from '@monochromatic-dev/test-harness-core-diagnostic';

await describe({
  name: formatErrorDeep.name,
  children: [
    it({
      name: 'uses the workspace root rather than the nested consumer working directory',
      fn: async () => {
        const root = fileURLToPath(new URL('../../../../', import.meta.url,),);
        const error = new Error('consumer frame',);
        error.stack = `Error: consumer frame\n    at userCall (${root}package/consumer/src/user.ts:1:1)`;
        const rendered = (await formatErrorDeep(error,)).join('\n',);
        expect(rendered,).toContain('package/consumer/src/user.ts:1:1',);
        expect(rendered,).not.toContain(root,);
      },
    },),
  ],
},);
