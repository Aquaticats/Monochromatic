// Recorded verdicts for every scan in markup-scan.mjs.
//
// Each group names the pattern it replaces, the scan that now answers it, and a corpus of inputs with
// the verdict that pattern returned for each one. The verdicts were captured before the replacement
// by compiling the group's own `pattern` and `flags` over every row's `input` and reading the result
// the way `kind` describes, so a row is evidence rather than an opinion:
//
// - `truth` reads a boolean pattern test;
// - `count` counts the pattern's global matches;
// - `elements` lists the matched text of each global match;
// - `rectangle` reads the four capture groups, or undefined when there is no match.
//
// No pattern here is compiled at run time. markup-scan-test.mjs replays the corpus against the scans
// and fails on any row that disagrees, which is how an equivalence break becomes visible.

//region Element start counts

/**
 `<input` followed by a word boundary, ignoring case.
 */
const COUNT_INPUT = {
  scan: 'countTagStarts',
  arguments: {
    name: 'input'
  },
  pattern: '<input\\b',
  flags: 'i',
  kind: 'count',
  rows: [
    {
      input: '<input type="text">',
      expected: 1
    },
    {
      input: '<INPUT>',
      expected: 1
    },
    {
      input: '<InPuT>',
      expected: 1
    },
    {
      input: '<inpu>',
      expected: 0
    },
    {
      input: '<inputs>',
      expected: 0
    },
    {
      input: '<input_x>',
      expected: 0
    },
    {
      input: '<input1>',
      expected: 0
    },
    {
      input: '<input-required>',
      expected: 1
    },
    {
      input: '<input\n>',
      expected: 1
    },
    {
      input: '<inputé>',
      expected: 1
    },
    {
      input: '<ınput>',
      expected: 0
    },
    {
      input: '<İnput>',
      expected: 0
    },
    {
      input: '<input><input>',
      expected: 2
    },
    {
      input: '<input<input>',
      expected: 2
    },
    {
      input: '',
      expected: 0
    },
    {
      input: '<input',
      expected: 1
    },
  ]
};

/**
 `<textarea` followed by a word boundary, ignoring case.
 */
const COUNT_TEXTAREA = {
  scan: 'countTagStarts',
  arguments: {
    name: 'textarea'
  },
  pattern: '<textarea\\b',
  flags: 'gi',
  kind: 'count',
  rows: [
    {
      input: '<textareatextarea>',
      expected: 0
    },
    {
      input: '<textarea></textarea>',
      expected: 1
    },
    {
      input: '<TEXTAREA a><textarea b><textarea c>',
      expected: 3
    },
  ]
};

/**
 `<select` followed by a word boundary, ignoring case.
 */
const COUNT_SELECT = {
  scan: 'countTagStarts',
  arguments: {
    name: 'select'
  },
  pattern: '<select\\b',
  flags: 'gi',
  kind: 'count',
  rows: [
    {
      input: '<select><select><select>',
      expected: 3
    },
    {
      input: '<selected>',
      expected: 0
    },
    {
      input: '<select_one>',
      expected: 0
    },
  ]
};

/**
 `<script` followed by a word boundary, ignoring case.
 */
const COUNT_SCRIPT = {
  scan: 'countTagStarts',
  arguments: {
    name: 'script'
  },
  pattern: '<script\\b',
  flags: 'gi',
  kind: 'count',
  rows: [
    {
      input: '<script>',
      expected: 1
    },
    {
      input: '<script></script>',
      expected: 1
    },
    {
      input: '<script><SCRIPT>',
      expected: 2
    },
    {
      input: '<scripts>',
      expected: 0
    },
    {
      input: 'no script tag',
      expected: 0
    },
  ]
};

//endregion Element start counts

//region Element extraction

/**
 `<node` followed by a word boundary, running to the next `>`.
 */
const NODE_ELEMENTS = {
  scan: 'tagElements',
  arguments: {
    name: 'node'
  },
  pattern: '<node\\b[^>]*>',
  flags: 'g',
  kind: 'elements',
  rows: [
    {
      input: '<node>',
      expected: ['<node>']
    },
    {
      input: '<node a="1" b="2">',
      expected: ['<node a="1" b="2">']
    },
    {
      input: 'lead <node text="a"> tail',
      expected: ['<node text="a">']
    },
    {
      input: '<nodes>',
      expected: []
    },
    {
      input: '</node>',
      expected: []
    },
    {
      input: '<Node>',
      expected: []
    },
    {
      input: '<node a="1"',
      expected: []
    },
    {
      input: '<node\na>',
      expected: ['<node\na>']
    },
    {
      input: '<node a=">">',
      expected: ['<node a=">']
    },
    {
      input: '<node><node>',
      expected: ['<node>', '<node>']
    },
    {
      input: '<nod>',
      expected: []
    },
  ]
};

//endregion Element extraction

//region Quoted attribute values

/**
 `text` with a quoted, non-empty value.
 */
const TEXT_VALUE = {
  scan: 'hasQuotedAttributeValue',
  arguments: {
    attribute: 'text'
  },
  pattern: 'text="[^"]+"',
  flags: '',
  kind: 'truth',
  rows: [
    {
      input: 'text="a"',
      expected: true
    },
    {
      input: 'text=""',
      expected: false
    },
    {
      input: 'text=" "',
      expected: true
    },
    {
      input: 'text="" text="b"',
      expected: true
    },
    {
      input: 'text="a',
      expected: false
    },
    {
      input: 'text="',
      expected: false
    },
    {
      input: 'content-desc="x"',
      expected: false
    },
    {
      input: 'data-text="x"',
      expected: true
    },
    {
      input: 'text="multi\nline"',
      expected: true
    },
    {
      input: 'Text="x"',
      expected: false
    },
  ]
};

/**
 `content-desc` with a quoted, non-empty value.
 */
const CONTENT_DESC_VALUE = {
  scan: 'hasQuotedAttributeValue',
  arguments: {
    attribute: 'content-desc'
  },
  pattern: 'content-desc="[^"]+"',
  flags: '',
  kind: 'truth',
  rows: [
    {
      input: 'content-desc="Pause"',
      expected: true
    },
    {
      input: 'content-desc=""',
      expected: false
    },
    {
      input: 'text="a" content-desc="b"',
      expected: true
    },
  ]
};

//endregion Quoted attribute values

//region Bounds rectangles

/**
 The four-group bounds extraction, read as numbers the way the caller reads them.
 */
const BOUNDS = {
  scan: 'boundsRectangle',
  arguments: {},
  pattern: 'bounds="\\[(\\d+),(\\d+)\\]\\[(\\d+),(\\d+)\\]"',
  flags: '',
  kind: 'rectangle',
  rows: [
    {
      input: 'bounds="[0,0][1080,2424]"',
      expected: {
        left: 0,
        top: 0,
        right: 1_080,
        bottom: 2_424
      }
    },
    {
      input: '<node bounds="[12,34][56,78]" clickable="true">',
      expected: {
        left: 12,
        top: 34,
        right: 56,
        bottom: 78
      }
    },
    {
      input: 'bounds="[007,2][3,4]"',
      expected: {
        left: 7,
        top: 2,
        right: 3,
        bottom: 4
      }
    },
    {
      input: 'bounds="[1,2][3,4]" bounds="[5,6][7,8]"',
      expected: {
        left: 1,
        top: 2,
        right: 3,
        bottom: 4
      }
    },
    {
      input: 'bounds="[1,2][3,4]',
      expected: undefined
    },
    {
      input: 'bounds="[1,2][3]"',
      expected: undefined
    },
    {
      input: 'bounds="[a,2][3,4]"',
      expected: undefined
    },
    {
      input: 'bounds="[,2][3,4]"',
      expected: undefined
    },
    {
      input: 'bounds="[-1,2][3,4]"',
      expected: undefined
    },
    {
      input: 'bounds="[1.5,2][3,4]"',
      expected: undefined
    },
    {
      input: 'bounds="[1,2](3,4]"',
      expected: undefined
    },
    {
      input: 'bounds="[1,2][3,4]"x',
      expected: {
        left: 1,
        top: 2,
        right: 3,
        bottom: 4
      }
    },
    {
      input: 'bounds="[9,9][9,9]"',
      expected: {
        left: 9,
        top: 9,
        right: 9,
        bottom: 9
      }
    },
    {
      input: 'no rectangle here',
      expected: undefined
    },
  ]
};

//endregion Bounds rectangles

//region Required control markers

/**
 A textarea or select element that marks itself required.
 */
const REQUIRED_CONTROL = {
  scan: 'hasRequiredControlMarker',
  arguments: {},
  pattern: '<(?:textarea|select)\\b(?:[^"\'<>]|"[^"]*"|\'[^\']*\')*?\\srequired(?:\\s|=|>)',
  flags: 'i',
  kind: 'truth',
  rows: [
    {
      input: '<textarea required>',
      expected: true
    },
    {
      input: '<textarea  required>',
      expected: true
    },
    {
      input: '<textarea required="">',
      expected: true
    },
    {
      input: '<textarea required=>',
      expected: true
    },
    {
      input: '<textarea required\n>',
      expected: true
    },
    {
      input: '<textarea\nrequired>',
      expected: true
    },
    {
      input: '<textarea title="x" required>',
      expected: true
    },
    {
      input: '<textarea title=" required ">',
      expected: false
    },
    {
      input: "<textarea title=' required '>",
      expected: false
    },
    {
      input: '<textarea title="unterminated required>',
      expected: false
    },
    {
      input: '<textarearequired>',
      expected: false
    },
    {
      input: '<textarea>',
      expected: false
    },
    {
      input: '<textarea requiredx>',
      expected: false
    },
    {
      input: '<select required>',
      expected: true
    },
    {
      input: '<SELECT REQUIRED>',
      expected: true
    },
    {
      input: '<select required\n>',
      expected: true
    },
    {
      input: '<input required>',
      expected: false
    },
    {
      input: '<textarea><select required>',
      expected: true
    },
    {
      input: '<textarea required="no" title="yes">',
      expected: true
    },
    {
      input: '<textarea title="yes" required="no">',
      expected: true
    },
    {
      input: '<textarea required',
      expected: false
    },
    {
      input: '<textarea required}\n>',
      expected: false
    },
    {
      input: '<textarea title="a" data-required="b">',
      expected: false
    },
    {
      input: '<textarea required\r\n>',
      expected: true
    },
  ]
};

//endregion Required control markers

//region External subresources

/**
 An external script or stylesheet reference, ignoring case.
 */
const SUBRESOURCE = {
  scan: 'hasExternalSubresourceReference',
  arguments: {},
  pattern: '<script\\s+[^>]*src=|<link\\s+[^>]*href=',
  flags: 'i',
  kind: 'truth',
  rows: [
    {
      input: '<script src="a.js">',
      expected: true
    },
    {
      input: '<SCRIPT SRC="a.js">',
      expected: true
    },
    {
      input: '<script async src="a.js">',
      expected: true
    },
    {
      input: '<script\tsrc="a.js">',
      expected: true
    },
    {
      input: '<script\nsrc="a.js">',
      expected: true
    },
    {
      input: '<script\u00A0src="a.js">',
      expected: true
    },
    {
      input: '<script src="a.js"',
      expected: true
    },
    {
      input: '<script>',
      expected: false
    },
    {
      input: '<script src>',
      expected: false
    },
    {
      input: '<scriptsrc="a.js">',
      expected: false
    },
    {
      input: '<scripts src="a.js">',
      expected: false
    },
    {
      input: '<script async><img src="a.js">',
      expected: false
    },
    {
      input: '<link rel="stylesheet" href="a.css">',
      expected: true
    },
    {
      input: '<LINK HREF="a.css">',
      expected: true
    },
    {
      input: '<link href>',
      expected: false
    },
    {
      input: '<link\nhref="a.css">',
      expected: true
    },
    {
      input: '<link rel="icon">',
      expected: false
    },
    {
      input: '<script></script><link href="a.css">',
      expected: true
    },
    {
      input: '',
      expected: false
    },
  ]
};

//endregion External subresources

//region Clipboard use

/**
 A clipboard API name or a copy command.
 */
const CLIPBOARD = {
  scan: 'hasClipboardUse',
  arguments: {},
  pattern: 'navigator\\.clipboard|clipboardData|execCommand\\(\\s*[\'"]copy',
  flags: '',
  kind: 'truth',
  rows: [
    {
      input: 'navigator.clipboard.writeText(value)',
      expected: true
    },
    {
      input: 'navigatorXclipboard',
      expected: false
    },
    {
      input: 'clipboardData',
      expected: true
    },
    {
      input: "execCommand('copy')",
      expected: true
    },
    {
      input: 'execCommand( "copy")',
      expected: true
    },
    {
      input: "execCommand(\n\t'copy')",
      expected: true
    },
    {
      input: 'execCommand("copysomething")',
      expected: true
    },
    {
      input: 'execCommand("Copy")',
      expected: false
    },
    {
      input: 'execCommand(copy)',
      expected: false
    },
    {
      input: 'execCommand',
      expected: false
    },
    {
      input: 'document.execCommand("paste")',
      expected: false
    },
    {
      input: 'execCommand(,)',
      expected: false
    },
    {
      input: '',
      expected: false
    },
  ]
};

//endregion Clipboard use

/**
 Every recorded group, in the order the scans appear in markup-scan.mjs.
 */
const CORPUS = [
  COUNT_INPUT,
  COUNT_TEXTAREA,
  COUNT_SELECT,
  COUNT_SCRIPT,
  NODE_ELEMENTS,
  TEXT_VALUE,
  CONTENT_DESC_VALUE,
  BOUNDS,
  REQUIRED_CONTROL,
  SUBRESOURCE,
  CLIPBOARD,
];

export { CORPUS };
