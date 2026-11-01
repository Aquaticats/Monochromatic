# Figma Kiwi Binary Format - Reverse Engineering Research

## File structure

All three Figma export formats (.
fig,
 .
deck,
 .
jam) are ZIP archives containing:

- `canvas.fig`:
   Binary blob with header + compressed schema + compressed document data
- `meta.json`:
   File metadata (background color,
   render bounds,
   export timestamp)
- `thumbnail.png`:
   Preview image
- `images/`:
   Referenced image assets (SHA-1 hash filenames)

## canvas.fig binary layout

1. **16-byte header**:
    8-byte ASCII prelude, little-endian uint32 format version, little-endian uint32 schema chunk length
   - bytes 0-7:
      prelude (`fig-kiwi`,
      `fig-deck`,
      or `fig-jam.`;
      the trailing `.` is part of the FigJam prelude)
   - bytes 8-11:
      format version, a little-endian uint32 (101 in older exports,
      106 in exports from October 2026)
   - bytes 12-15:
      byte length of the schema chunk that follows
2. **Deflate-compressed schema**:
    Raw deflate stream starting at byte 16
3. **4-byte LE uint32 size prefix**:
    Size of the zstd-compressed document section
4. **Zstd-compressed document data**:
    Starts with zstd frame magic `28 b5 2f fd`

### Format version is a number, not a letter

The trailing "letter" in `fig-kiwie` and `fig-kiwij` is the low byte of the version uint32:
 101 is ASCII `e` and 106 is ASCII `j`,
 and the three higher bytes are zero.
Reading the first nine bytes as a nine-character magic therefore broke the moment Figma raised the version.
`parseCanvasHeader` now matches only the 8-byte prelude and returns the version (also exposed as
`canvasVersion` on `FigmaFile`),
 accepting every uint32 value.
Two real exports from 2026-10-18 (276 MB and 301 MB) carry version 106 in their `canvas.fig` header.
`parseFigmaFile` decodes both through the package entry point:
 5624 nodes and 207 images (403 s),
 and 5702 nodes and 335 images (407 s),
 matching an independent scratch decoder.

Sources read for this finding:

- Grida `.fig` format notes (`https://canary.grida.co/docs/wg/feat-fig/`):
   "An 8-byte ASCII prelude (magic string) identifying the container,
   then a little-endian version `u32`";
   known preludes `fig-kiwi`,
   `fig-jam.`,
   `fig-deck`.
- `sunyui/figma-parser` README (`https://github.com/sunyui/figma-parser`):
   "Version number (uint32 LE, e.g., 106)";
   newer files (version at least about 100) use zstd for the message chunk,
   the schema chunk always raw deflate.
- `openfig-core` README (`https://cdn.jsdelivr.net/npm/openfig-core@0.4.1/README.md`):
   layout `[prelude 8B][version uint32 LE][len+chunk0][len+chunk1]...`,
   and a parsed example header `{ prelude: 'fig-kiwi', version: 52 }`,
   which shows that other version values exist and are not limited to the letters seen here.

No source states what changed between versions.
 Treat the version as an opaque revision number.

The zstd boundary is found by searching for the zstd frame magic bytes in the raw
canvas.
fig data.
 The 4 bytes immediately before the magic are the LE size prefix.

## Schema binary format (STOCK KIWI, VERIFIED)

Figma uses the stock Kiwi binary schema format from evanw/kiwi.
There is NO custom header:
 the first bytes `AF 04` are simply the
varuint encoding of the definition count (559).

```text
definitionCount (varuint) -- e.g. 0xAF 0x04 = 559
for each definition:
  name\0 (null-terminated string)
  kind (single byte: 0=ENUM, 1=STRUCT, 2=MESSAGE)
  fieldCount (varuint)
  for each field:
    fieldName\0 (null-terminated string)
    type (varint, zigzag encoded: negative = primitive, non-negative = def index)
    isArray (single byte, bit 0: 1=array, 0=single)
    value (varuint: field tag for messages/structs, enum value for enums)
```

### Type code encoding in schema (STOCK KIWI)

- **Negative type values**:
   primitives.
   Index = `~typeRaw` (bitwise NOT).
  - `~0` = -1 = bool,
     `~1` = -2 = byte,
     `~2` = -3 = int,
    `~3` = -4 = uint,
     `~4` = -5 = float,
     `~5` = -6 = string,
    `~6` = -7 = int64,
     `~7` = -8 = uint64
- **Non-negative type values**:
   schema definition references (0-indexed)
  - e.g. type=50 -> defs[50] = GUID

### Primitives (8 types, same as stock Kiwi)

<table>
<thead>
<tr>
<th>~index</th>
<th>type_raw</th>
<th>Type</th>
</tr>
</thead>
<tbody>
<tr>
<td>0</td>
<td>-1</td>
<td>bool</td>
</tr>
<tr>
<td>1</td>
<td>-2</td>
<td>byte</td>
</tr>
<tr>
<td>2</td>
<td>-3</td>
<td>int</td>
</tr>
<tr>
<td>3</td>
<td>-4</td>
<td>uint</td>
</tr>
<tr>
<td>4</td>
<td>-5</td>
<td>float</td>
</tr>
<tr>
<td>5</td>
<td>-6</td>
<td>string</td>
</tr>
<tr>
<td>6</td>
<td>-7</td>
<td>int64</td>
</tr>
<tr>
<td>7</td>
<td>-8</td>
<td>uint64</td>
</tr>
</tbody>
</table>

### Difference from stock Kiwi schema

In stock Kiwi,
 enum fields have type=null which zigzag-encodes as varint(-1).
In Figma's schema,
 enum fields have type=0 (varint 0).
 This is a minor
difference (type is unused for enum fields anyway).
 The rest matches exactly.

## Document data encoding (FULLY DECODED, VERIFIED)

### Stock Kiwi encoding rules (verified against all 3 test files)

1. **MESSAGE**:
    Tagged fields with `tag(varuint)` + `value`,
    terminated by `tag=0`.
   Only present fields are encoded.
    Absent fields use their default value.
2. **STRUCT**:
    All fields present in order,
    no tags,
    no terminator.
3. **ENUM**:
    Single `varuint` value.
4. **bool**:
    Single byte (0=false,
    non-0=true).
5. **byte**:
    Single byte.
6. **int**:
    Zigzag varint (`readVarInt`).
7. **uint**:
    Varuint (`readVarUint`).
8. **float**:
    **VarFloat** (custom bit-rotated encoding,
    NOT raw IEEE 754):
   - If first byte = 0x00:
      value is 0.0 (1 byte total)
   - Otherwise:
      4 bytes.
      Read as little-endian uint32,
      then
     `bits = (bits << 23) | (bits >>> 9)`.
      Reinterpret as float32.
9. **string**:
    Null-terminated UTF-8.
10. **int64**:
     Zigzag varint (same as int but wider range).
11. **uint64**:
     Varuint (same as uint but wider range).
12. **Array**:
     `count(varuint)` followed by `count` values,
     each encoded per their type.
13. **Struct reference**:
     Fields inline (no length prefix,
     no delimiter).
14. **Message reference**:
     Fields inline with tag+value pairs,
     terminated by tag=0.

### VarFloat encoding detail

The varfloat format stores float32 values compactly:

- Zero uses 1 byte instead of 4.
- The exponent is moved to the first byte so small absolute values
  (common in UI coordinates) may compress better.
- Encoding:
   `float32 -> uint32 bits -> (bits >>> 23) | (bits << 9) -> LE 4 bytes`
  (with special case:
   if result & 0xFF == 0,
   store single byte 0x00)
- Decoding:
   `LE 4 bytes -> uint32 -> (bits << 23) | (bits >>> 9) -> float32`

### Top-level document structure

The document data is a `Message` struct with tagged fields.
For exported files,
 it typically contains:

- tag 1:
   type = MessageType.
  NODE_CHANGES (1)
- tag 2:
   sessionID = 0
- tag 3:
   ackID = 0
- tag 4:
   nodeChanges (repeated NodeChange)

### NodeChange encoding

NodeChange is a MESSAGE (tagged,
 terminated by 0).
 It has 565 fields
but most are absent in any given node.
 Only non-default fields are encoded.

Key fields (by tag):

- tag 1:
   guid (GUID struct;
   inline sessionID + localID varuints)
- tag 2:
   phase (NodePhase enum)
- tag 3:
   parentIndex (ParentIndex struct;
   inline GUID + position string)
- tag 4:
   type (NodeType enum)
- tag 5:
   name (string)
- tag 6:
   visible (bool)
- tag 8:
   opacity (float/varfloat)
- tag 12:
   transform (Matrix struct;
   6 varfloat values inline)
- etc.

### ParentIndex encoding

ParentIndex is a STRUCT (all fields present in order):

1. guid:
    GUID struct (sessionID varuint + localID varuint,
    inline)
2. position:
    string (null-terminated,
    encodes child ordinal within parent)

The position string uses a custom encoding where "!
" = first child,
'"' = second child,
 " ~" = some other position,
 etc.

### GUID encoding

GUID is a STRUCT (all fields present in order):

1. sessionID:
    uint (varuint)
2. localID:
    uint (varuint)

For nodes created in the same session,
 sessionID is typically 0
(and may be omitted from the output since varuint(0) = byte 0x00).

## Key schema definitions

### NodeType enum (62 values)

DOCUMENT=1,
 CANVAS=2,
 GROUP=3,
 FRAME=4,
 BOOLEAN_OPERATION=5,
VECTOR=6,
 STAR=7,
 LINE=8,
 ELLIPSE=9,
 RECTANGLE=10,
REGULAR_POLYGON=11,
 ROUNDED_RECTANGLE=12,
 TEXT=13,
 SLICE=14,
SYMBOL=15,
 INSTANCE=16,
 STICKY=17,
 SHAPE_WITH_TEXT=18,
CONNECTOR=19,
 CODE_BLOCK=20,
 WIDGET=21,
 STAMP=22,
 MEDIA=23,
HIGHLIGHT=24,
 SECTION=25,
 SECTION_OVERLAY=26,
 WASHI_TAPE=27,
VARIABLE=28,
 TABLE=29,
 TABLE_CELL=30,
 VARIABLE_SET=31,
 SLIDE=32,
ASSISTED_LAYOUT=33,
 INTERACTIVE_SLIDE_ELEMENT=34,
 ... (up to 61)

### NodeChange message (565 fields)

The main node change record containing all mutable node properties.
Fields with "Tag" suffix (guidTag,
 phaseTag,
 etc.) store the original
field tag from the CRDT operation log.

### Message struct (44 fields)

Top-level envelope.
 For exported files,
 only type,
 sessionID,
 ackID,
and nodeChanges are typically present.

## Verification results

All three file types decoded 100%:

<table>
<thead>
<tr>
<th>File</th>
<th>Type</th>
<th>Doc bytes</th>
<th>Nodes</th>
<th>Decoded</th>
</tr>
</thead>
<tbody>
<tr>
<td>Color palette - base.fig</td>
<td>fig</td>
<td>2,779</td>
<td>8</td>
<td>100%</td>
</tr>
<tr>
<td>ScholarCopilot.fig</td>
<td>fig</td>
<td>3,718,268</td>
<td>2,976</td>
<td>100%</td>
</tr>
<tr>
<td>MTM6162-040 participation 2 cover.deck</td>
<td>deck</td>
<td>114,358</td>
<td>59</td>
<td>100%</td>
</tr>
<tr>
<td>Todo app - Brainstorming.jam</td>
<td>jam</td>
<td>621,850</td>
<td>58</td>
<td>100%</td>
</tr>
</tbody>
</table>

## Research files

- `/tmp/figma-research/fig-extract/canvas.decompressed`:
   63746-byte decompressed schema
- `/tmp/figma-research/zstd_decompressed.bin`:
   2779-byte decompressed document (small file)
- `/tmp/figma-research/scholar_zstd_decompressed.bin`:
   3.7MB decompressed document (large file)
- `/tmp/kiwi-source/`:
   Cloned evanw/kiwi source for reference
