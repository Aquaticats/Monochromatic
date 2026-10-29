import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _neutralizeControlCharacters as neutralizeControlCharacters,
} from '@monochromatic-dev/logger-core';

/**
 Adversarial inputs at the terminal boundary paired with the exact output
 the neutralizer must produce. Each case names the attack or the malformed
 shape it pins.
 */
const BOUNDARY_CASES: readonly {
  readonly name: string;
  readonly input: string;
  readonly expected: string;
}[] = [
  {
    name: 'OSC title-set sequence',
    input: 'title:\u001B]0;PWNED\u0007 ok',
    expected: String.raw`title:\u001B]0;PWNED\u0007 ok`,
  },
  {
    name: 'CSI clear-screen sequence',
    input: 'clear:\u001B[2J',
    expected: String.raw`clear:\u001B[2J`,
  },
  {
    name: 'SGR color sequence (no allowlist)',
    input: '\u001B[31mred\u001B[0m',
    expected: String.raw`\u001B[31mred\u001B[0m`,
  },
  {
    name: 'trailing lone ESC',
    input: 'tail\u001B',
    expected: String.raw`tail\u001B`,
  },
  {
    name: 'ESC [ with no final byte',
    input: 'open\u001B[',
    expected: String.raw`open\u001B[`,
  },
  {
    name: 'unterminated OSC',
    input: '\u001B]2;never closed',
    expected: String.raw`\u001B]2;never closed`,
  },
  {
    name: 'nested ESC inside a sequence',
    input: '\u001B[\u001B[2J',
    expected: String.raw`\u001B[\u001B[2J`,
  },
  {
    name: '8-bit C1 CSI',
    input: 'c1:\u009B2J',
    expected: String.raw`c1:\u009B2J`,
  },
  {
    name: 'DEL',
    input: 'del:\u007F',
    expected: String.raw`del:\u007F`,
  },
  {
    name: 'NUL',
    input: 'nul:\u0000end',
    expected: String.raw`nul:\u0000end`,
  },
  {
    name: 'carriage return (line overwrite)',
    input: 'real\rfake',
    expected: String.raw`real\u000Dfake`,
  },
  {
    name: 'backspace',
    input: 'ab\bc',
    expected: String.raw`ab\u0008c`,
  },
];

/**
 Inputs the neutralizer must return unchanged.
 */
const PASSTHROUGH_CASES: readonly {
  readonly name: string;
  readonly input: string;
}[] = [
  {
    name: 'plain ASCII',
    input: 'server started on port 3000',
  },
  {
    name: 'empty string',
    input: '',
  },
  {
    name: 'newline and tab',
    input: 'line one\n\tindented two\n',
  },
  {
    name: 'non-ASCII text and emoji',
    input: 'héllo wörld 🚀 日本語',
  },
  {
    name: 'lone surrogate',
    input: 'x\uD83Dy',
  },
  {
    name: 'format specifiers',
    input: '%s %d %j %%',
  },
  {
    name: 'code point just above the C1 range',
    input: 'nbsp:\u00A0end',
  },
];

/**
 One past the last C1 code unit; the exhaustive sweep covers every code unit
 below it.
 */
const CONTROL_SWEEP_LENGTH = 0xA0;

/**
 Reports whether an output code unit is still a control the boundary forbids.
 
 @param codeUnit - UTF-16 code unit read from neutralizer output.
 
 @returns Whether the code unit should have been neutralized.
 */
function isForbiddenControl(codeUnit: number,): boolean {
  if (codeUnit < 0x20)
    return (codeUnit !== 0x0A) && (codeUnit !== 0x09);
  if (codeUnit === 0x7F)
    return true;
  return (codeUnit >= 0x80) && (codeUnit <= 0x9F);
}

await describe({
  name: neutralizeControlCharacters.name,
  children: [
    //region Neutralized controls

    ...BOUNDARY_CASES.map(function mapBoundaryCase(boundaryCase,) {
      return it({
        name: `neutralizes ${boundaryCase.name}`,
        fn: async () => {
          expect(neutralizeControlCharacters(boundaryCase.input,),)
            .toBe(boundaryCase.expected,);
        },
      },);
    },),

    it({
      name: 'output never contains a neutralized control after the pass',
      fn: async () => {
        /**
         Every code unit from NUL through U+009F, in one string.
         */
        const allControls = Array.from(
          { length: CONTROL_SWEEP_LENGTH, },
          function toChar(
            _unused,
            index,
          ) {
            return String.fromCodePoint(index,);
          },
        )
          .join('',);
        /**
         Neutralized sweep; only newline and tab may survive as controls.
         */
        const output = neutralizeControlCharacters(allControls,);
        /**
         Output code units that are still forbidden controls; must stay empty.
         */
        const leaked: number[] = [];
        for (const character of output) {
          // oxlint-disable-next-line unicorn/prefer-code-point -- The sweep classifies lead code units below U+00A0, which never sit inside a surrogate pair.
          const codeUnit = character.charCodeAt(0,);
          if (isForbiddenControl(codeUnit,))
            leaked.push(codeUnit,);
        }
        expect(leaked,)
          .toEqual([],);
      },
    },),

    //endregion Neutralized controls

    //region Passthrough

    ...PASSTHROUGH_CASES.map(function mapPassthroughCase(passthroughCase,) {
      return it({
        name: `leaves ${passthroughCase.name} untouched`,
        fn: async () => {
          expect(neutralizeControlCharacters(passthroughCase.input,),)
            .toBe(passthroughCase.input,);
        },
      },);
    },),

    //endregion Passthrough
  ],
},);
