import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  inspect,
  l,
} from '../../dist/final/node/index.mjs';

//region inspect

await describe({
  name: inspect.name,
  // Sequential execution required: tests spy on the shared module-level
  // logger, and sinon refuses to wrap an already-wrapped method.
  // Matches the convention in test-harness/core/src/sinon.unit.test.ts.
  concurrency: 1,
  children: [
    it({
      name: 'returns a string unchanged',
      fn: async () => {
        expect(inspect('hello world',),).toBe('hello world',);
      },
    },),
    it({
      name: 'returns a number unchanged',
      fn: async () => {
        /** Numeric input should pass through with its type preserved */
        const result = inspect(42,);
        expect(result,).toBe(42,);
      },
    },),
    it({
      name: 'returns an array unchanged',
      fn: async () => {
        /** Array input should be returned by reference */
        const arr = [1, 2, 3,];
        expect(inspect(arr,),).toBe(arr,);
      },
    },),
    it({
      name: 'returns an object unchanged',
      fn: async () => {
        /** Object input should be returned by reference */
        const obj = { key: 'value', };
        expect(inspect(obj,),).toBe(obj,);
      },
    },),
    it({
      name: 'returns boolean unchanged',
      fn: async () => {
        expect(inspect(true,),).toBe(true,);
      },
    },),
    it({
      name: 'returns null unchanged',
      fn: async () => {
        expect(inspect(null,),).toBeNull();
      },
    },),
    it({
      name: 'logs string content via tagged logger',
      fn: async ({ sinon, },) => {
        const infoSpy = sinon.spy(l, 'info',);
        inspect('test-content',);
        expect(infoSpy,).toHaveBeenCalledWith(
          expect.stringContaining('test-content',),
        );
      },
    },),
    it({
      name: 'logs non-string content as JSON',
      fn: async ({ sinon, },) => {
        const infoSpy = sinon.spy(l, 'info',);
        inspect({ a: 1, },);
        /** Non-string values should be JSON-stringified in the log */
        const loggedMessage = infoSpy.args[0]?.[0] as string;
        expect(loggedMessage,).toContain('"a": 1',);
      },
    },),
    it({
      name: 'truncates long string content in log output',
      fn: async ({ sinon, },) => {
        const infoSpy = sinon.spy(l, 'info',);
        /** Content longer than the 200-char preview limit */
        const longContent = 'x'.repeat(300,);
        /** Return value should still be the full content */
        const result = inspect(longContent,);
        expect(result,).toBe(longContent,);
        /** Logged preview should be truncated with ellipsis */
        const loggedMessage = infoSpy.args[0]?.[0] as string;
        expect(loggedMessage,).toContain('...',);
      },
    },),
    it({
      name: 'does not truncate content exactly at preview limit',
      fn: async ({ sinon, },) => {
        const infoSpy = sinon.spy(l, 'info',);
        /** Content exactly at the 200-char boundary */
        const exactContent = 'y'.repeat(200,);
        inspect(exactContent,);
        /** Should not contain ellipsis since it's not over the limit */
        const loggedMessage = infoSpy.args[0]?.[0] as string;
        expect(loggedMessage,).not.toContain('...',);
      },
    },),
    it({
      name: 'handles empty string',
      fn: async ({ sinon, },) => {
        const infoSpy = sinon.spy(l, 'info',);
        expect(inspect('',),).toBe('',);
        expect(infoSpy,).toHaveBeenCalled();
      },
    },),
    it({
      name: 'handles GlobResult-like array',
      fn: async () => {
        /** Simulates the shape returned by cat(string) */
        const globResults = [
          { path: '/a.ts', content: 'code', },
          { path: '/b.ts', content: 'more', },
        ];
        /** Should return the same array by reference */
        expect(inspect(globResults,),).toBe(globResults,);
      },
    },),
    it({
      name: 'truncates large JSON objects in log output',
      fn: async ({ sinon, },) => {
        const infoSpy = sinon.spy(l, 'info',);
        /** Large object whose JSON is over the preview limit */
        const largeObj = Object.fromEntries(
          Array.from({ length: 50, },
            (_, idx,) => [`key${String(idx,)}`, 'x'.repeat(20,),],),
        );
        inspect(largeObj,);
        const loggedMessage = infoSpy.args[0]?.[0] as string;
        expect(loggedMessage,).toContain('...',);
      },
    },),
  ],
},);

//endregion inspect
