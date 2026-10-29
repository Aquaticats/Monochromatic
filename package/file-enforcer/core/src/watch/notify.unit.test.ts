import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  l,
  notifyWriteProtection,
} from '../../dist/final/node/index.mjs';

//region notifyWriteProtection

await describe({
  name: notifyWriteProtection.name,
  // Sequential execution required: tests spy on the shared module-level
  // logger, and sinon refuses to wrap an already-wrapped method.
  // Matches the convention in test-harness/core/src/sinon.unit.test.ts.
  concurrency: 1,
  children: [
    it({
      name: 'logs a warning via tagged logger',
      fn: async ({ sinon, },) => {
        const warnSpy = sinon.spy(l, 'warn',);
        await notifyWriteProtection('/repo/CLAUDE.md',);
        /** Should have logged with the PROTECTED prefix */
        expect(warnSpy,).toHaveBeenCalledWith(
          expect.stringContaining('PROTECTED',),
        );
      },
    },),
    it({
      name: 'includes the file path in the warning',
      fn: async ({ sinon, },) => {
        const warnSpy = sinon.spy(l, 'warn',);
        await notifyWriteProtection('/repo/output.txt',);
        expect(warnSpy,).toHaveBeenCalledWith(
          expect.stringContaining('/repo/output.txt',),
        );
      },
    },),
    it({
      name: 'does not throw when notify-send is unavailable',
      fn: async () => {
        // If notify-send fails or is missing, the function should still complete
        await expect(
          notifyWriteProtection('/nonexistent/path.txt',),
        )
          .resolves
          .toBeUndefined();
      },
    },),
  ],
},);

//endregion notifyWriteProtection
