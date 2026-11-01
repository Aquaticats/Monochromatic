import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.string.from.string.trim.with.string.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'trims single character from both ends',
      fn: async () => {
        expect($({ str: 'abc', trimmer: 'a', },),).toBe('bc',);
      },
    },),

    it({
      name: 'trims multi-character string from both ends',
      fn: async () => {
        expect($({ str: 'TextSuffixText', trimmer: 'Text', },),).toBe('Suffix',);
      },
    },),

    it({
      name: 'trims only from start when pattern exists only at start',
      fn: async () => {
        expect($({ str: 'PrefixString', trimmer: 'Prefix', },),).toBe('String',);
      },
    },),

    it({
      name: 'trims only from end when pattern exists only at end',
      fn: async () => {
        expect($({ str: 'StringSuffix', trimmer: 'Suffix', },),).toBe('String',);
      },
    },),

    it({
      name: 'trims from both ends when pattern exists on both sides',
      fn: async () => {
        expect($({ str: 'PrefixStringSuffix', trimmer: 'Prefix', },),).toBe(
          'StringSuffix',
        );
      },
    },),

    it({
      name: 'removes multiple consecutive occurrences from start',
      fn: async () => {
        expect($({ str: 'xxxText', trimmer: 'x', },),).toBe('Text',);
      },
    },),

    it({
      name: 'removes multiple consecutive occurrences from end',
      fn: async () => {
        expect($({ str: 'Textxxx', trimmer: 'x', },),).toBe('Text',);
      },
    },),

    it({
      name: 'removes multiple consecutive occurrences from both ends',
      fn: async () => {
        expect($({ str: 'xxxTextxxx', trimmer: 'x', },),).toBe('Text',);
      },
    },),

    it({
      name: 'removes different amounts from both ends',
      fn: async () => {
        expect($({ str: 'xxTextxxx', trimmer: 'x', },),).toBe('Text',);
      },
    },),

    it({
      name: 'removes repeated multi-character patterns',
      fn: async () => {
        expect($({ str: 'file.txt.txt.txt', trimmer: '.txt', },),).toBe('file',);
      },
    },),

    it({
      name: 'case sensitive matching - different trimmer does not match',
      fn: async () => {
        expect($({ str: 'StringSUFFIX', trimmer: 'suffix', },),).toBe('StringSUFFIX',);
      },
    },),

    it({
      name: 'case sensitive matching - correct case is trimmed',
      fn: async () => {
        expect($({ str: 'StringSUFFIX', trimmer: 'SUFFIX', },),).toBe('String',);
      },
    },),

    it({
      name: 'handles empty string input',
      fn: async () => {
        expect($({ str: '', trimmer: 'anything', },),).toBe('',);
      },
    },),

    it({
      name: 'returns unchanged when trimmer not found',
      fn: async () => {
        expect($({ str: 'String', trimmer: 'prefix', },),).toBe('String',);
      },
    },),

    it({
      name: 'returns unchanged when trimmer is empty string',
      fn: async () => {
        expect($({ str: 'unchanged', trimmer: '', },),).toBe('unchanged',);
      },
    },),

    it({
      name: 'handles single character strings',
      fn: async () => {
        expect($({ str: 'a', trimmer: 'a', },),).toBe('',);
      },
    },),

    it({
      name: 'trimmer that matches entire string returns empty',
      fn: async () => {
        expect($({ str: 'match', trimmer: 'match', },),).toBe('',);
      },
    },),

    it({
      name: 'trimmer longer than input string returns unchanged',
      fn: async () => {
        expect($({ str: 'short', trimmer: 'longer', },),).toBe('short',);
      },
    },),

    it({
      name: 'trimmer with special regex characters is escaped',
      fn: async () => {
        expect($({ str: '.txt.file.txt', trimmer: '.txt', },),).toBe('.file',);
      },
    },),

    it({
      name: 'trimmer with spaces',
      fn: async () => {
        expect($({ str: '  Text  ', trimmer: ' ', },),).toBe('Text',);
      },
    },),

    it({
      name: 'trims pattern from start when it matches',
      fn: async () => {
        expect($({ str: 'abcaaabca', trimmer: 'abc', },),).toBe('aaabca',);
      },
    },),

    it({
      name:
        'removes repeated file extensions from end only when start has different prefix',
      fn: async () => {
        expect($({ str: '.backup.document.backup', trimmer: '.backup', },),).toBe(
          '.document',
        );
      },
    },),

    it({
      name: 'removes trailing slashes from URLs',
      fn: async () => {
        expect($({ str: 'https://example.com/', trimmer: '/', },),).toBe(
          'https://example.com',
        );
      },
    },),

    it({
      name: 'removes multiple trailing slashes from URLs',
      fn: async () => {
        expect($({ str: 'path/to/directory///', trimmer: '/', },),).toBe(
          'path/to/directory',
        );
      },
    },),

    it({
      name: 'removes repeated suffixes',
      fn: async () => {
        expect($({ str: 'testTestTest', trimmer: 'Test', },),).toBe('test',);
      },
    },),

    it({
      name: 'handles unicode characters',
      fn: async () => {
        expect($({ str: '世界Text世界', trimmer: '世界', },),).toBe('Text',);
      },
    },),

    it({
      name: 'handles characters with special regex meaning',
      fn: async () => {
        expect($({ str: '.file.', trimmer: '.', },),).toBe('file',);
      },
    },),

    it({
      name: 'handles dollar signs',
      fn: async () => {
        expect($({ str: '$$price$$', trimmer: '$', },),).toBe('price',);
      },
    },),

    it({
      name: 'handles question marks',
      fn: async () => {
        expect($({ str: '??test???', trimmer: '?', },),).toBe('test',);
      },
    },),

    it({
      name: 'handles asterisks',
      fn: async () => {
        expect($({ str: '***text***', trimmer: '*', },),).toBe('text',);
      },
    },),

    it({
      name: 'handles plus signs',
      fn: async () => {
        expect($({ str: '+++value+++', trimmer: '+', },),).toBe('value',);
      },
    },),

    it({
      name: 'handles pipe characters',
      fn: async () => {
        expect($({ str: '|data|', trimmer: '|', },),).toBe('data',);
      },
    },),

    it({
      name: 'handles backslash characters',
      fn: async () => {
        expect($({ str: '\\path\\', trimmer: '\\', },),).toBe('path',);
      },
    },),

    it({
      name: 'handles caret characters',
      fn: async () => {
        expect($({ str: '^content^', trimmer: '^', },),).toBe('content',);
      },
    },),

    it({
      name: 'handles square brackets',
      fn: async () => {
        expect($({ str: '[]value[]', trimmer: '[]', },),).toBe('value',);
      },
    },),

    it({
      name: 'handles curly braces',
      fn: async () => {
        expect($({ str: '{}data{}', trimmer: '{}', },),).toBe('data',);
      },
    },),

    it({
      name: 'handles parentheses',
      fn: async () => {
        expect($({ str: '(text)', trimmer: '(', },),).toBe('text)',);
      },
    },),

    it({
      name: 'trims newline characters',
      fn: async () => {
        expect($({ str: '\nText\n', trimmer: '\n', },),).toBe('Text',);
      },
    },),

    it({
      name: 'trims tab characters',
      fn: async () => {
        expect($({ str: '\tText\t', trimmer: '\t', },),).toBe('Text',);
      },
    },),

    it({
      name: 'trims carriage return characters',
      fn: async () => {
        expect($({ str: '\rContent\r', trimmer: '\r', },),).toBe('Content',);
      },
    },),

    it({
      name: 'handles emoji characters',
      fn: async () => {
        expect($({ str: '🚀test🚀', trimmer: '🚀', },),).toBe('test',);
      },
    },),

    it({
      name: 'handles mixed whitespace and text',
      fn: async () => {
        expect($({ str: '   Hello   ', trimmer: ' ', },),).toBe('Hello',);
      },
    },),

    it({
      name: 'removes hyphen patterns from both ends',
      fn: async () => {
        expect($({ str: '--text--', trimmer: '-', },),).toBe('text',);
      },
    },),

    it({
      name: 'removes underscore patterns from both ends',
      fn: async () => {
        expect($({ str: '__value__', trimmer: '_', },),).toBe('value',);
      },
    },),

    it({
      name: 'handles identical trimmer and input text pattern',
      fn: async () => {
        expect($({ str: 'aaa', trimmer: 'a', },),).toBe('',);
      },
    },),

    it({
      name: 'handles trimmer that appears only in middle',
      fn: async () => {
        expect($({ str: 'abcaaa', trimmer: 'abc', },),).toBe('aaa',);
      },
    },),

    it({
      name: 'handles mixed case trimmer with exact match',
      fn: async () => {
        expect($({ str: 'PREFIXStringPREFIX', trimmer: 'PREFIX', },),).toBe('String',);
      },
    },),

    it({
      name: 'removes leading and trailing zeros',
      fn: async () => {
        expect($({ str: '000123000', trimmer: '0', },),).toBe('123',);
      },
    },),

    it({
      name: 'handles nested patterns',
      fn: async () => {
        expect($({ str: '((text))', trimmer: '((', },),).toBe('text))',);
      },
    },),

    it({
      name: 'handles very long strings efficiently',
      fn: async () => {
        const longStr = `${'a'.repeat(1_000,)}text${'a'.repeat(1_000,)}`;
        expect($({ str: longStr, trimmer: 'a', },),).toBe('text',);
      },
    },),

    it({
      name: 'trimmer with special regex characters',
      fn: async () => {
        expect($({ str: '.*.', trimmer: '.*', },),).toBe('.',);
      },
    },),

    it({
      name: 'handles multiple consecutive multi-char patterns at start only',
      fn: async () => {
        expect($({ str: 'xxxyyyText', trimmer: 'xxx', },),).toBe('yyyText',);
      },
    },),

    it({
      name: 'handles multiple consecutive multi-char patterns at end only',
      fn: async () => {
        expect($({ str: 'Textxxxyyy', trimmer: 'yyy', },),).toBe('Textxxx',);
      },
    },),
  ],
},);
