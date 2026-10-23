import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider';
import { buildBlockPairingMessages, } from '@monochromatic-dev/translation-repair-stage';
import { blockPairingProtocol, } from '../dist/final/node/index.mjs';

const expectedFormat: JsonSchemaResponseFormat = {
  type: 'json_schema', json_schema: { name: 'block_pairing', schema: {
    type: 'object', properties: { pairs: { type: 'array', items: {
      type: 'object', properties: { source: { type: 'integer' }, target: { type: 'integer' } },
      required: ['source', 'target'], additionalProperties: false,
    } } }, required: ['pairs'], additionalProperties: false,
  } },
};

await describe({ name: blockPairingProtocol.name, children: [
  it({ name: 'builds the pairing messages and the schema, and nothing else', fn: async () => {
    const sourceBlocks = [{ index: 0, text: '猫。' }, { index: 1, text: '盒子。' }];
    const targetBlocks = [{ index: 0, text: 'Cat.' }, { index: 1, text: 'Box.' }];
    const protocol = blockPairingProtocol({ sourceBlocks, targetBlocks });
    expect(protocol.messages).toEqual(buildBlockPairingMessages({ sourceBlocks, targetBlocks }));
    // WHAT THE MESSAGES SAY, not only that they equal the builder the wrapper
    // calls (ledger T6): every block under its own number, originals first.
    const asked = protocol.messages
      .filter((message) => message.role === 'user')
      .map((message) => message.content)
      .join('\n');
    expect(asked).toContain('[0]\n=====\n猫。\n=====');
    expect(asked).toContain('[1]\n=====\n盒子。\n=====');
    expect(asked).toContain('[0]\n=====\nCat.\n=====');
    expect(asked).toContain('[1]\n=====\nBox.\n=====');
    expect(asked.indexOf('ORIGINAL BLOCKS')).toBeLessThan(asked.indexOf('TRANSLATION BLOCKS'));
    expect(protocol.responseFormat).toEqual(expectedFormat);
    expect(Object.keys(protocol).toSorted()).toEqual(['messages', 'responseFormat']);
    expect('strict' in protocol.responseFormat.json_schema).toBe(false);
  } }),
  it({ name: 'retains content-sensitive fences, supplied numbering, escapes and Unicode text', fn: async () => {
    const sourceBlocks = [{ index: 7, text: '```\n“猫” \\[note]。\n````\nSecond line.' }] as const;
    const targetBlocks = [{ index: 2, text: 'A cat.\n```\nQuoted `markup`.' }];
    const protocol = blockPairingProtocol({ sourceBlocks, targetBlocks });
    expect(protocol.messages).toEqual(buildBlockPairingMessages({ sourceBlocks, targetBlocks }));
    expect(JSON.stringify(protocol.messages)).toContain('“猫”');
    expect(blockPairingProtocol({ sourceBlocks: [{ index: 8, text: sourceBlocks[0].text }], targetBlocks }).messages).not.toEqual(protocol.messages);
    expect(blockPairingProtocol({ sourceBlocks, targetBlocks: [{ index: 2, text: 'Changed cat.' }] }).messages).not.toEqual(protocol.messages);
  } }),
  it({ name: 'owns the response schema so one consumer cannot change later acquisition contracts', fn: async () => {
    const input = { sourceBlocks: [{ index: 0, text: '猫。' }], targetBlocks: [{ index: 0, text: 'Cat.' }] };
    const first = blockPairingProtocol(input);
    Object.defineProperty(first.responseFormat.json_schema, 'name', { value: 'altered-fixture-schema' });
    first.responseFormat.json_schema.schema.extraFixtureProperty = true;
    const later = blockPairingProtocol(input);
    expect(later.responseFormat).toEqual(expectedFormat);
    expect(later.responseFormat).not.toBe(first.responseFormat);
    expect(later.responseFormat.json_schema.schema).not.toBe(first.responseFormat.json_schema.schema);
  } }),
] });
