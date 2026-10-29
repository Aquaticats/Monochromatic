import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.object.regexp.from.string.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'converts simple string to regex',
      fn: async () => {
        const result = $({ str: 'hello', },);
        expect(result,).toBeInstanceOf(RegExp,);
        expect(result.test('hello',),).toBe(true,);
        expect(result.test('hello world',),).toBe(true,);
      },
    },),

    it({
      name: 'escapes period character',
      fn: async () => {
        const result = $({ str: 'I love you.', },);
        expect(result.test('I love you.',),).toBe(true,);
        expect(result.test('I love youX',),).toBe(false,);
        expect(result.source,).toBe(String.raw`I love you\.`,);
      },
    },),

    it({
      name: 'escapes asterisk character',
      fn: async () => {
        const result = $({ str: 'Hello*', },);
        expect(result.test('Hello*',),).toBe(true,);
        expect(result.test('Hellooo',),).toBe(false,);
        expect(result.source,).toBe(String.raw`Hello\*`,);
      },
    },),

    it({
      name: 'escapes plus character',
      fn: async () => {
        const result = $({ str: 'test+', },);
        expect(result.test('test+',),).toBe(true,);
        expect(result.test('testtt',),).toBe(false,);
        expect(result.source,).toBe(String.raw`test\+`,);
      },
    },),

    it({
      name: 'escapes question mark character',
      fn: async () => {
        const result = $({ str: 'what?', },);
        expect(result.test('what?',),).toBe(true,);
        expect(result.test('wha',),).toBe(false,);
        expect(result.source,).toBe(String.raw`what\?`,);
      },
    },),

    it({
      name: 'escapes caret character',
      fn: async () => {
        const result = $({ str: '^start', },);
        expect(result.test('^start',),).toBe(true,);
        expect(result.test('start',),).toBe(false,);
        expect(result.source,).toBe(String.raw`\^start`,);
      },
    },),

    it({
      name: 'escapes dollar character',
      fn: async () => {
        const result = $({ str: 'end$', },);
        expect(result.test('end$',),).toBe(true,);
        expect(result.test('end',),).toBe(false,);
        expect(result.source,).toBe(String.raw`end\$`,);
      },
    },),

    it({
      name: 'escapes curly braces',
      fn: async () => {
        const result = $({ str: '{count}', },);
        expect(result.test('{count}',),).toBe(true,);
        expect(result.source,).toBe(String.raw`\{count\}`,);
      },
    },),

    it({
      name: 'escapes square brackets',
      fn: async () => {
        const result = $({ str: '[array]', },);
        expect(result.test('[array]',),).toBe(true,);
        expect(result.source,).toBe(String.raw`\[array\]`,);
      },
    },),

    it({
      name: 'escapes parentheses',
      fn: async () => {
        const result = $({ str: '(group)', },);
        expect(result.test('(group)',),).toBe(true,);
        expect(result.source,).toBe(String.raw`\(group\)`,);
      },
    },),

    it({
      name: 'escapes pipe character',
      fn: async () => {
        const result = $({ str: 'this|that', },);
        expect(result.test('this|that',),).toBe(true,);
        expect(result.test('this',),).toBe(false,);
        expect(result.source,).toBe(String.raw`this\|that`,);
      },
    },),

    it({
      name: 'escapes backslash character',
      fn: async () => {
        const result = $({ str: 'path\\to', },);
        expect(result.test(String.raw`path\to`,),).toBe(true,);
        expect(result.source,).toBe(String.raw`path\\to`,);
      },
    },),

    it({
      name: 'escapes forward slash character',
      fn: async () => {
        const result = $({ str: 'path/to/file', },);
        expect(result.test('path/to/file',),).toBe(true,);
        expect(result.source,).toBe(String.raw`path\/to\/file`,);
      },
    },),

    it({
      name: 'escapes multiple special characters',
      fn: async () => {
        const result = $({ str: 'Hello.*', },);
        expect(result.test('Hello.*',),).toBe(true,);
        expect(result.test('Hellooo',),).toBe(false,);
        expect(result.source,).toBe(String.raw`Hello\.\*`,);
      },
    },),

    it({
      name: 'handles empty string',
      fn: async () => {
        const result = $({ str: '', },);
        expect(result.test('',),).toBe(true,);
        expect(result.test('anything',),).toBe(true,);
      },
    },),

    it({
      name: 'handles string with no special characters',
      fn: async () => {
        const result = $({ str: 'simple text', },);
        expect(result.test('simple text',),).toBe(true,);
        expect(result.test('simple',),).toBe(false,);
        expect(result.source,).toBe('simple text',);
      },
    },),

    it({
      name: 'handles complex pattern with multiple escapes',
      fn: async () => {
        const result = $({ str: 'regex: ^[a-z]+$', },);
        expect(result.test('regex: ^[a-z]+$',),).toBe(true,);
        expect(result.test('regex: abc',),).toBe(false,);
        expect(result.source,).toBe(String.raw`regex: \^\[a-z\]\+\$`,);
      },
    },),

    it({
      name: 'handles unicode characters',
      fn: async () => {
        const result = $({ str: 'Hello 世界', },);
        expect(result.test('Hello 世界',),).toBe(true,);
        expect(result.test('Hello',),).toBe(false,);
      },
    },),

    it({
      name: 'handles newlines and special whitespace',
      fn: async () => {
        const result = $({ str: 'line1\nline2', },);
        expect(result.test('line1\nline2',),).toBe(true,);
        expect(result.test('line1 line2',),).toBe(false,);
      },
    },),

    it({
      name: 'handles tab characters',
      fn: async () => {
        const result = $({ str: 'col1\tcol2', },);
        expect(result.test('col1\tcol2',),).toBe(true,);
        expect(result.test('col1 col2',),).toBe(false,);
      },
    },),

    it({
      name: 'real-world example: file path',
      fn: async () => {
        const result = $({ str: '/usr/local/bin/script.sh', },);
        expect(result.test('/usr/local/bin/script.sh',),).toBe(true,);
        expect(result.source,).toBe(String.raw`\/usr\/local\/bin\/script\.sh`,);
      },
    },),

    it({
      name: 'real-world example: email with special chars',
      fn: async () => {
        const result = $({ str: 'user.name+tag@example.com', },);
        expect(result.test('user.name+tag@example.com',),).toBe(true,);
        expect(result.source,).toBe(String.raw`user\.name\+tag@example\.com`,);
      },
    },),

    it({
      name: 'real-world example: URL',
      fn: async () => {
        const result = $({ str: 'https://example.com/path?query=value', },);
        expect(result.test('https://example.com/path?query=value',),).toBe(true,);
        expect(result.source,).toBe(
          String.raw`https:\/\/example\.com\/path\?query=value`,
        );
      },
    },),

    it({
      name: 'real-world example: regex pattern as string',
      fn: async () => {
        const result = $({ str: '^[A-Z]{3}-\\d{4}$', },);
        expect(result.test(String.raw`^[A-Z]{3}-\d{4}$`,),).toBe(true,);
        expect(result.source,).toBe(String.raw`\^\[A-Z\]\{3\}-\\d\{4\}\$`,);
      },
    },),

    it({
      name: 'function returns RegExp instance',
      fn: async () => {
        const result = $({ str: 'test', },);
        expect(result,).toBeInstanceOf(RegExp,);
        expect(typeof result.test,).toBe('function',);
        expect(typeof result.exec,).toBe('function',);
      },
    },),

    it({
      name: 'escaped pattern matches string',
      fn: async () => {
        const result = $({ str: 'special.chars*', },);

        // Should match the exact string
        expect(result.test('special.chars*',),).toBe(true,);
        // Should match something including exact string.
        expect(result.test('special.chars**',),).toBe(true,);

        // Should not match variations that would match unescaped regex
        expect(result.test('specialXcharsY',),).toBe(false,);
        expect(result.test('specialchars',),).toBe(false,);
        expect(result.test('special.chars',),).toBe(false,);
      },
    },),
  ],
},);
