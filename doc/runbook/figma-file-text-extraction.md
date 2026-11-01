# Extract text styles from a Figma `.fig` file without Figma

Purpose: read every text node's font, size, line height and frame ancestry from a downloaded `.fig` file,
offline, to compare a built site against the design's type at each design width.
Verified 2026-10-18 on a 276 MB `.fig` (5,624 nodes, 1,113 text nodes) from the Xinyan Chen portfolio.

## File format

- A `.fig` download is a zip holding `canvas.fig`, `meta.json`, thumbnails and `images/` (the raster fills by hash).
- `canvas.fig` starts with the 8-byte magic `fig-kiwi`, then a little-endian `uint32` version,
  then chunks, each a little-endian `uint32` length followed by that many bytes.
- Chunk 0 is the compressed binary [kiwi](https://github.com/evanw/kiwi) schema; chunk 1 is the compressed message.
  Chunks are raw deflate in older files and zstd (magic `28 b5 2f fd`) in newer ones.
- The decoded message's `nodeChanges` is a flat list of nodes; `parentIndex.guid` links each to its parent,
  `guid` is `{ sessionID, localID }`.
  Canvases (pages such as Phone, Tablet, Desktop) and their top-level frames are ancestors of every node.
- Text nodes have `type: 'TEXT'`, `fontSize`, `fontName.family` and `.style`, `lineHeight`, `letterSpacing`,
  `textData.characters`, and `textData.styleOverrideTable` holding the sizes of mixed-style runs.

## Decoder

```bash
# Linux workstation, scratch under ~/temp/agent
mkdir --parents ~/temp/agent/fig-parse/in && chmod 700 ~/temp/agent/fig-parse
cd ~/temp/agent/fig-parse && unzip -o -q '/path/to/Design.fig' canvas.fig meta.json -d in
npm init -y && npm install kiwi-schema@0.5.0 fzstd@0.1.1 pako@2.1.0
node dump-text.mjs in/canvas.fig texts.json
```

```js
// dump-text.mjs: decode a Figma canvas.fig and print every text node with its frame ancestry and font size.
// Usage: node dump-text.mjs <canvas.fig> <out.json>
import { readFileSync, writeFileSync } from 'node:fs';
import { compileSchema, decodeBinarySchema } from 'kiwi-schema';
import { decompress as zstd } from 'fzstd';
import pako from 'pako';

const [input, output] = process.argv.slice(2);
const bytes = readFileSync(input);
const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
const chunks = [];
for (let offset = 12; offset < bytes.length;) {
  const length = view.getUint32(offset, true);
  chunks.push(bytes.subarray(offset + 4, offset + 4 + length));
  offset += 4 + length;
}
// Chunks are raw deflate in older files and zstd (magic 28 b5 2f fd) in newer ones.
const inflate = (chunk) => chunk[0] === 0x28 && chunk[1] === 0xb5 ? zstd(chunk) : pako.inflateRaw(chunk);
const schema = compileSchema(decodeBinarySchema(inflate(chunks[0])));
const message = schema.decodeMessage(inflate(chunks[1]));
const key = (guid) => guid ? `${guid.sessionID}:${guid.localID}` : '';
const nodes = new Map(message.nodeChanges.map((node) => [key(node.guid), node]));
const ancestry = (node) => {
  const names = [];
  for (let current = node; current; current = nodes.get(key(current.parentIndex?.guid))) {
    names.unshift(`${current.type}:${current.name}`);
  }
  return names;
};
const texts = message.nodeChanges.filter((node) => node.type === 'TEXT').map((node) => ({
  path: ancestry(node),
  text: node.textData?.characters?.slice(0, 80),
  fontSize: node.fontSize,
  font: node.fontName && `${node.fontName.family} ${node.fontName.style}`,
  lineHeight: node.lineHeight,
  letterSpacing: node.letterSpacing,
  styleOverrides: node.textData?.styleOverrideTable?.map((style) => ({ fontSize: style.fontSize, font: style.fontName && `${style.fontName.family} ${style.fontName.style}` })).filter((style) => style.fontSize || style.font),
  width: node.size?.x,
}));
writeFileSync(output, JSON.stringify(texts, null, 1));
console.log(`${message.nodeChanges.length} nodes, ${texts.length} text nodes`);
```

Each entry's `path` reads like `DOCUMENT:Document`, `CANVAS:Phone`, `FRAME:Home`, then nested groups.

## Comparing against a built site

- Record each text-owning element's computed `font-size` per page at the design widths (390, 744 and 1440 in the
  portfolio), then match it to Figma text nodes in the same canvas and frame.
- Match on normalized full text (letters and digits only, lowercased), allowing the site text to start a Figma node
  or one of its lines: designers often put a heading and its paragraph in one text node.
- Matching pitfalls that inflate mismatch counts:
  short strings (`Home`, `Work`) match inside unrelated nodes;
  the same words appear in a jump list and as a section heading;
  phone frames draw a hamburger, so their nav words belong to other nodes.
  Group by the CSS declaration that sets the size, not by element, before deciding on edits.
- Frame names in a hand-made file are inconsistent (the portfolio's Tablet Contact frame is `Contract`,
  its Desktop Tales frame is `Tales for Safer Trails`); map them to pages explicitly.
