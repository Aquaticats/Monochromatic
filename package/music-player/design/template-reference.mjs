//region Reference for the supporting-text template language, design evidence only
// D89 to D93 fix the scope: text with formulas between `$` signs, the track's fields, duration
// formatting, case and cut conversion, and `+` to join text. There is no conditional (D92): text
// outside formulas is always shown and a field with no value yields nothing (D93). The spelling
// follows KWGT where KWGT has one. This file is the design study's statement of what a template
// yields; it is not the player's parser and authorizes no production implementation.

/** Fields of a track, each a mode word of `mi`. */
export const fields = [
  {
    mode: 'title',
    label: 'Title',
    description: 'The name the row shows as its title.'
  },
  {
    mode: 'file',
    label: 'File name',
    description: 'The file name without its extension.'
  },
  {
    mode: 'ext',
    label: 'Extension',
    description: 'The file extension.'
  },
  {
    mode: 'folder',
    label: 'Folder',
    description: 'The folder that holds the file.'
  },
  {
    mode: 'path',
    label: 'Path',
    description: 'The path from the library root.'
  },
  {
    mode: 'len',
    label: 'Duration',
    description: 'The duration in seconds; tf formats it.'
  },
  {
    mode: 'peak',
    label: 'True peak',
    description: 'The true peak with its unit, such as \u22121.2 dBTP, or nothing before the file is analysed.'
  },
];

/** The playing track's fields: a track's own, and where it sits in its folder (the agent's version under D97). */
export const playingFields = [
  ...fields,
  {
    mode: 'track',
    label: 'Place in folder',
    description: 'Where the track is in its folder, counting from 1.'
  },
  {
    mode: 'total',
    label: 'Tracks in folder',
    description: 'How many tracks the folder holds.'
  },
];

/** Functions with the signature and argument help the editor shows while the caret is inside a call. */
export const functions = {
  mi: {
    signature: 'mi(field)',
    minimum: 1,
    maximum: 1,
    parameters: [
      {
        name: 'field',
        description: `Field: one of ${
       fields.map(field => field.mode)
         .join(', ')
       }.`
      }]
  },
  tf: {
    signature: 'tf(seconds, [format])',
    minimum: 1,
    maximum: 2,
    parameters: [
      {
        name: 'seconds',
        description: 'Seconds: a duration, such as mi(len).'
      },
      {
        name: 'format',
        description: "Format: h, m and s for hours, minutes and seconds; a doubled letter pads with a zero; text between apostrophes is kept. Without it, m:ss."
      }
    ]
  },
  tc: {
    signature: 'tc(mode, text, [length])',
    minimum: 2,
    maximum: 3,
    parameters: [
      {
        name: 'mode',
        description: 'Mode: low, up or cap to change case; cut to keep the first characters.'
      },
      {
        name: 'text',
        description: 'Text: the text to convert.'
      },
      {
        name: 'length',
        description: 'Length: how many characters cut keeps.'
      }
    ]
  },
};

/** The default supporting line (D93): the duration, a space, then the true peak, which is empty before analysis. */
export const defaultTemplate = '$tf(mi(len), m:ss)$ $mi(peak)$';

/** Each template Settings lists, with its default and the fields it may use (D97 version, for approval). */
export const templates = {
  track: {
    label: 'Track rows',
    defaultTemplate,
    fields
  },
  playing: {
    label: 'Playing track',
    defaultTemplate: '$mi(track)$ of $mi(total)$ $mi(peak)$',
    fields: playingFields
  },
};
//endregion

//region Formula text to tokens, one pass
// No comparison operators: with no conditional (D92) there is nothing to feed them, so `=`, `<` and `>` are ordinary characters.
const operators = [
  '+',
  '(',
  ')',
  ','
];
/** Splits one formula's text; `offset` is the formula's first character in the whole template. */
function tokenize({
  text,
  offset
}) {
  const tokens = [];
  const errors = [];
  let index = 0;
  while (index < text.length) {
    const character = text[index];
    if ((character === ' ') || (character === '\t')
      || (character === '\n')) { index += 1;
      continue; }
    if (character === '"') {
      const close = text.indexOf(
        '"',
        index + 1
      );
      if (close === (-1)) {
        errors.push({
          subject: 'text',
          message: 'a quotation mark is not closed',
          at: offset + index
        });
        tokens.push({
          kind: 'text',
          value: text.slice(index + 1),
          start: offset + index,
          end: offset + text.length
        });
        break;
      }
      tokens.push({
        kind: 'text',
        value: text.slice(
          index + 1,
          close
        ),
        start: offset + index,
        end: offset + close
          + 1
      });
      index = close + 1;
      continue;
    }
    const operator = operators.find(candidate => text.startsWith(
      candidate,
      index
    ));
    if (operator) {
      tokens.push({
        kind: 'operator',
        value: operator,
        start: offset + index,
        end: offset + index
          + operator.length
      });
      index += operator.length;
      continue;
    }
    // A bare word runs to the next space, quotation mark or operator.
    let end = index;
    while ((end < text.length) && (!' \t\n"'.includes(text[end]))
      && (!operators.some(candidate => text.startsWith(
        candidate,
        end
      )))) end += 1;
    tokens.push({
      kind: 'word',
      value: text.slice(
        index,
        end
      ),
      start: offset + index,
      end: offset + end
    });
    index = end;
  }
  return {
    tokens,
    errors
  };
}
//endregion

//region Tokens to a tree; nesting is bounded so the descent is too
const deepest = 12;
function parseFormula({
  text,
  offset
}) {
  const {
    tokens,
    errors
  } = tokenize({
    text,
    offset
  });
  let position = 0;
  // Set once nesting passes the bound: the rest of the formula is not read, and what is then missing is not reported again.
  let abandoned = false;
  function peek() { return tokens[position]; }
  function isOperator(value) { return (peek()
    ?.kind
    === 'operator') && (peek()
      .value
      === value); }
  function primary(depth) {
    const token = peek();
    if (!token) {
      if (!abandoned) errors.push({
        subject: 'formula',
        message: 'a value is missing',
        at: offset + text.length
      });
      return {
        kind: 'text',
        value: ''
      };
    }
    if (token.kind === 'operator') {
      errors.push({
        subject: 'formula',
        message: `a value is missing before ${  token.value}`,
        at: token.start
      });
      return {
        kind: 'text',
        value: ''
      };
    }
    position += 1;
    if ((token.kind === 'word') && isOperator('(')) {
      const open = peek();
      position += 1;
      const call = {
        kind: 'call',
        name: token.value,
        start: token.start,
        open: open.end,
        close: undefined,
        parameters: [],
        separators: []
      };
      if (depth >= deepest) {
        errors.push({
          subject: token.value,
          message: 'calls are nested too deeply',
          at: token.start
        });
        abandoned = true;
        position = tokens.length;
        return call;
      }
      if (isOperator(')')) { call.close = peek()
        .start;
        position += 1;
        return call; }
      for (;;) {
        call.parameters
          .push(joined(depth + 1));
        if (isOperator(',')) { call.separators
          .push(peek()
            .start);
          position += 1;
          continue; }
        if (isOperator(')')) { call.close = peek()
          .start;
          position += 1;
          return call; }
        if (!abandoned) errors.push({
          subject: token.value,
          message: 'a closing bracket is missing',
          at: token.start
        });
        return call;
      }
    }
    return token.kind === 'text' ? {
      kind: 'text',
      value: token.value
    } : {
      kind: 'word',
      value: token.value
    };
  }
  function joined(depth) {
    let left = primary(depth);
    while (isOperator('+')) { position += 1;
      left = {
        kind: 'join',
        left,
        right: primary(depth)
      }; }
    return left;
  }
  const tree = joined(0);
  if (position < tokens.length) errors.push({
    subject: 'formula',
    message: `unexpected ${  tokens[position]
      .value}`,
    at: tokens[position]
      .start
  });
  return {
    tree,
    errors
  };
}

/** Splits a template into literal text and formulas. A `$` with no partner is reported, not shown as text. */
export function parseTemplate(text) {
  const parts = [];
  const errors = [];
  let index = 0;
  while (index < text.length) {
    const open = text.indexOf(
      '$',
      index
    );
    if (open === (-1)) { parts.push({
      kind: 'literal',
      value: text.slice(index)
    });
      break; }
    if (open > index) parts.push({
      kind: 'literal',
      value: text.slice(
        index,
        open
      )
    });
    const close = text.indexOf(
      '$',
      open + 1
    );
    const source = text.slice(
      open + 1,
      close === (-1) ? text.length : close
    );
    if (close === (-1)) errors.push({
      subject: 'formula',
      message: `the $ at character ${  open + 1
         } has no closing $`,
      at: open
    });
    const parsed = parseFormula({
      text: source,
      offset: open + 1
    });
    errors.push(...parsed.errors);
    parts.push({
      kind: 'formula',
      tree: parsed.tree,
      start: open,
      end: close === (-1) ? text.length : close + 1,
      closed: close !== (-1)
    });
    if (close === (-1)) break;
    index = close + 1;
  }
  return {
    parts,
    errors
  };
}
//endregion

//region Tree to text for one track
const absent = '';
function asText(value) {
  return String(value);
}
function asNumber(value) {
  // Only seconds and a cut length are read as numbers; nothing compares values any more (D92).
  const text = asText(value)
    .trim();
  const number = Number(text);
  return (text === '') || (!Number.isFinite(number)) ? undefined : number;
}
function formatSeconds({
  seconds,
  format
}) {
  const whole = Math.max(
    0,
    Math.floor(seconds)
  );
  const hours = Math.floor(whole / 3_600);
  // Minutes carry the hours when the format has no hour letter, so 4000 seconds reads 66:40 under m:ss.
  const minutes = format.includes('h') ? Math.floor((whole % 3_600) / 60) : Math.floor(whole / 60);
  const parts = {
    h: hours,
    m: minutes,
    s: whole % 60
  };
  let output = '';
  // Text between apostrophes is kept as written, as in KWGT's own example "h' hours' and m' minutes'".
  let kept = false;
  for (let index = 0; index < format.length; index += 1) {
    const letter = format[index];
    if (letter === "'") { kept = !kept;
      continue; }
    if (kept || (!Object.hasOwn(
      parts,
      letter
    ))) { output += letter;
      continue; }
    const doubled = format[index + 1] === letter;
    output += doubled ? String(parts[letter])
      .padStart(
      2,
      '0'
      ) : String(parts[letter]);
    if (doubled) index += 1;
  }
  return output;
}
function evaluate({
  node,
  track,
  errors,
  allowed
}) {
  if ((node.kind === 'text') || (node.kind === 'word')) return node.value;
  if (node.kind === 'join') return asText(evaluate({
    node: node.left,
    track,
    errors,
    allowed
  })) + asText(evaluate({
    node: node.right,
    track,
    errors,
    allowed
  }));
  const definition = Object.hasOwn(
    functions,
    node.name
  ) ? functions[node.name] : undefined;
  if (!definition) { errors.push({
    subject: node.name,
    message: 'unknown function',
    at: node.start
  });
    return absent; }
  if ((node.parameters
    .length
    < definition.minimum) || (node.parameters
      .length
      > definition.maximum)) {
    errors.push({
      subject: node.name,
      message: `takes ${  definition.minimum === definition.maximum ? definition.minimum : `${definition.minimum  } or ${
         definition.maximum}` 
      }${definition.maximum === 1 ? ' value' : ' values'  }, as in ${  definition.signature}`,
      at: node.start
    });
    return absent;
  }
  // Every argument is evaluated before the call, so a mistake in any of them is reported.
  const values = node.parameters
    .map(parameter => evaluate({
      node: parameter,
      track,
      errors,
      allowed
    }));
  if (node.name === 'mi') {
    const mode = asText(values[0]);
    // Which fields exist depends on the template: the playing track has two the track rows do not.
    if (!allowed.some(field => field.mode === mode)) { errors.push({
      subject: 'mi',
      message: `unknown field ${  mode}`,
      at: node.start
    });
      return absent; }
    const value = track[mode];
    return (value === undefined) || (value === null) ? absent : value;
  }
  if (node.name === 'tf') {
    if (asText(values[0]) === absent) return absent;
    const seconds = asNumber(values[0]);
    if (seconds === undefined) { errors.push({
      subject: 'tf',
      message: `needs a number of seconds, not ${  asText(values[0])}`,
      at: node.start
    });
      return absent; }
    return formatSeconds({
      seconds,
      format: values.length === 2 ? asText(values[1]) : 'm:ss'
    });
  }
  if (node.name === 'tc') {
    const mode = asText(values[0]);
    const text = asText(values[1]);
    if (mode === 'low') return text.toLowerCase();
    if (mode === 'up') return text.toUpperCase();
    if (mode === 'cap') return text.slice(
      0,
      1
    )
      .toUpperCase()
      + text.slice(1);
    if (mode === 'cut') {
      const length = values.length === 3 ? asNumber(values[2]) : undefined;
      if ((length === undefined) || (length < 0)) { errors.push({
        subject: 'tc',
        message: 'cut needs a length, as in tc(cut, text, 10)',
        at: node.start
      });
        return absent; }
      return [...text].slice(
        0,
        Math.floor(length)
      )
        .join('');
    }
    errors.push({
      subject: 'tc',
      message: `unknown mode ${  mode}`,
      at: node.start
    });
    return absent;
  }
  throw new Error(`A function is defined but not evaluated: ${  node.name}`);
}

/** What a template shows for one track, or the mistakes that keep it from applying; `fields` are those its template may use. */
export function evaluateTemplate({
  text,
  track,
  fields: allowed = fields
}) {
  const {
    parts,
    errors
  } = parseTemplate(text);
  // A template that does not parse is not evaluated: what its pieces would yield only adds mistakes that follow from the first.
  if (errors.length > 0) return {
    valid: false,
    text: undefined,
    errors
  };
  const output = parts.map(part => part.kind === 'literal' ? part.value : asText(evaluate({
    node: part.tree,
    track,
    errors,
    allowed
  })))
    .join('');
  return errors.length === 0 ? {
    valid: true,
    text: output,
    errors: []
  } : {
    valid: false,
    text: undefined,
    errors
  };
}

/** One line per mistake, naming the function first, in the order the mistakes sit in the template. */
export function errorLines(errors) {
  return [...errors].sort((
    left,
    right
  ) => left.at - right.at)
    .map(error => `${error.subject  }: ${
       error.message}`);
}
//endregion

//region Help for the call the caret is inside
/** The innermost call around the caret and which of its arguments the caret is in, or nothing outside a call. */
export function helpAt({
  text,
  caret,
  fields: allowed = fields
}) {
  const { parts } = parseTemplate(text);
  let found;
  // A work stack walks the tree; depth is bounded by `deepest`, breadth by the template's own length.
  const pending = parts.filter(part => (part.kind === 'formula') && (caret > part.start)
    && (caret <= (part.closed ? part.end - 1 : part.end)))
    .map(part => part.tree);
  while (pending.length > 0) {
    const node = pending.pop();
    if (node.kind === 'join') { pending.push(
      node.left,
      node.right
    );
      continue; }
    if (node.kind !== 'call') continue;
    pending.push(...node.parameters);
    const inside = (caret >= node.open) && ((node.close === undefined) || (caret <= node.close));
    if (inside && ((!found) || (node.open > found.open))) found = node;
  }
  if ((!found) || (!Object.hasOwn(
    functions,
    found.name
  ))) return undefined;
  const definition = functions[found.name];
  const index = Math.min(
    found.separators
      .filter(separator => separator < caret)
      .length,
    definition.parameters
      .length
      - 1
  );
  // The field list in the help names the fields of this template.
  const description = found.name === 'mi' ? `Field: one of ${  allowed.map(field => field.mode)
    .join(', ')  }.` : definition.parameters[index]
      .description;
  return {
    name: found.name,
    signature: definition.signature,
    parameter: definition.parameters[index]
      .name,
    description
  };
}
//endregion
