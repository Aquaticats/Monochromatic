import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { evaluatePredicate, } from './evaluate-predicate.ts';

//region Predicates

await describe({
  name: evaluatePredicate.name,
  children: [
    it({
      name: 'returns true for a command that exits 0',
      fn: async () => {
        /** `true` binary always exits 0 */
        const result = await evaluatePredicate(['true',],);
        expect(result,).toBe(true,);
      },
    },),
    it({
      name: 'returns false for a command that exits non-zero',
      fn: async () => {
        /** `false` binary always exits 1 */
        const result = await evaluatePredicate(['false',],);
        expect(result,).toBe(false,);
      },
    },),
    it({
      name: 'passes arguments to the command',
      fn: async () => {
        /** `ls` with a known path exits 0 */
        const result = await evaluatePredicate(['ls', '/dev/null',],);
        expect(result,).toBe(true,);
      },
    },),
    it({
      name: 'returns false for a nonexistent command',
      fn: async () => {
        /** A command that does not exist should fail gracefully */
        const result = await evaluatePredicate(['nonexistent-command-abc123',],);
        expect(result,).toBe(false,);
      },
    },),
  ],
},);

//endregion Predicates
