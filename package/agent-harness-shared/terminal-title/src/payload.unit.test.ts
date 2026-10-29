/**
 Tests for terminal title payload safety.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MAX_TERMINAL_TITLE_UTF8_BYTES,
  safeTerminalTitlePayload,
  sanitizeTerminalTitleText,
  terminalTitleUtf8ByteLength,
} from '@monochromatic-dev/agent-harness-shared-terminal-title';

await describe({
  name: safeTerminalTitlePayload.name,
  children: [
    it({
      name: 'replaces OSC-breaking controls with visible tokens',
      fn: async () => {
        expect(sanitizeTerminalTitleText('a\u001Bb\u0007c',),).toBe('a␛b␇c',);
      },
    },),
    it({
      name: 'caps unsafe ASCII payload by UTF-8 bytes',
      fn: async () => {
        /**
         Payload one byte over the terminal budget.
         */
        const unsafeTitle = 'a'.repeat(MAX_TERMINAL_TITLE_UTF8_BYTES + 1,);
        /**
         Safe payload after truncation.
         */
        const result = safeTerminalTitlePayload({ value: unsafeTitle, },);
        expect(terminalTitleUtf8ByteLength(result,) <= MAX_TERMINAL_TITLE_UTF8_BYTES,).toBe(true,);
        expect(result.endsWith('…',),).toBe(true,);
      },
    },),
    it({
      name: 'caps emoji without breaking surrogate pairs',
      fn: async () => {
        /**
         Emoji payload well over the terminal byte budget.
         */
        const unsafeTitle = '😀'.repeat(MAX_TERMINAL_TITLE_UTF8_BYTES,);
        /**
         Safe payload after byte truncation.
         */
        const result = safeTerminalTitlePayload({ value: unsafeTitle, },);
        expect(terminalTitleUtf8ByteLength(result,) <= MAX_TERMINAL_TITLE_UTF8_BYTES,).toBe(true,);
        expect(result.endsWith('…',),).toBe(true,);
      },
    },),
    it({
      name: 'applies sanitizing before byte truncation',
      fn: async () => {
        /**
         Payload with visible replacement that exceeds tiny budget.
         */
        const result = safeTerminalTitlePayload({
          value: '\u001Babcdef',
          maxBytes: 6,
        },);
        expect(result,).toBe('␛…',);
      },
    },),
  ],
},);
