const ESCAPE_LITERAL = new Map([
  [
    String.raw`\.`,
    '.'
  ],
  [
    String.raw`\/`,
    '/'
  ],
  [
    String.raw`\\`,
    '\\'
  ],
  [
    String.raw`\-`,
    '-'
  ],
  [
    String.raw`\[`,
    '['
  ],
  [
    String.raw`\]`,
    ']'
  ],
  [
    String.raw`\(`,
    '('
  ],
  [
    String.raw`\)`,
    ')'
  ],
  [
    String.raw`\{`,
    '{'
  ],
  [
    String.raw`\}`,
    '}'
  ],
  [
    String.raw`\+`,
    '+'
  ],
  [
    String.raw`\*`,
    '*'
  ],
  [
    String.raw`\?`,
    '?'
  ],
  [
    String.raw`\^`,
    '^'
  ],
  [
    String.raw`\$`,
    '$'
  ],
  [
    String.raw`\|`,
    '|'
  ],
  [
    String.raw`\#`,
    '#'
  ],
  [
    String.raw`\.`,
    '.'
  ],
  [
    String.raw`\ `,
    ' '
  ],
  [
    String.raw`\_`,
    '_'
  ],
  [
    String.raw`\:`,
    ':'
  ],
  [
    String.raw`\,`,
    ','
  ],
  [
    String.raw`\=`,
    '='
  ],
  [
    String.raw`\!`,
    '!'
  ],
  [
    String.raw`\@`,
    '@'
  ],
  [
    String.raw`\~`,
    '~'
  ],
  [
    '\\`',
    '`'
  ],
  [
    String.raw`\'`,
    "'"
  ],
  [
    String.raw`\"`,
    '"'
  ],
]);
const SHORTHAND = new Map([
  [
    String.raw`\d`,
    'digit'
  ],
  [
    String.raw`\D`,
    'non-digit'
  ],
  [
    String.raw`\w`,
    'word'
  ],
  [
    String.raw`\W`,
    'non-word'
  ],
  [
    String.raw`\s`,
    'whitespace'
  ],
  [
    String.raw`\S`,
    'non-whitespace'
  ],
]);

/**
 Split a literal RegExp source (or a `new RegExp('...')` argument) into body + flags.
 */
function splitPattern(pattern) {
  const text = pattern.replaceAll(
    '\n',
    ' '
  );
  if (text.startsWith('new RegExp') || text.startsWith('RegExp(')) {
    return {
      body: text,
      flags: '',
      dynamic: true,
    };
  }
  const first = text.indexOf('/');
  const last = text.lastIndexOf('/');
  if ((first === (-1)) || (last <= first)) return {
    body: text,
    flags: '',
    dynamic: true,
  };
  return {
    body: text.slice(
      first + 1,
      last
    ),
    flags: text.slice(last + 1),
    dynamic: false,
  };
}

/**
 Tokenize a pattern body into a flat atom list. Quantifiers attach to the previous atom.
 */
function tokenize(body) {
  const atoms = [];
  let index = 0;
  while (index < body.length) {
    const char = body[index];
    if (char === '\\') {
      const pair = body.slice(
        index,
        index + 2
      );
      if (ESCAPE_LITERAL.has(pair)) {
        atoms.push({
          type: 'literal',
          text: ESCAPE_LITERAL.get(pair),
        });
        index += 2;
        continue;
      }
      if (SHORTHAND.has(pair)) {
        atoms.push({
          type: 'class',
          name: SHORTHAND.get(pair),
        });
        index += 2;
        continue;
      }
      if (pair === String.raw`\b`) {
        atoms.push({ type: 'boundary', });
        index += 2;
        continue;
      }
      atoms.push({
        type: 'escape',
        text: pair,
      });
      index += 2;
      continue;
    }
    if (char === '[') {
      let end = index + 1;
      if (body[end] === '^') end += 1;
      if (body[end] === ']') end += 1;
      while ((end < body.length) && (body[end] !== ']')) {
        end += body[end] === '\\' ? 2 : 1;
      }
      atoms.push({
        type: 'class',
        text: body.slice(
          index,
          end + 1
        ),
        negated: body[index + 1] === '^',
      });
      index = end + 1;
      continue;
    }
    if (char === '(') {
      let depth = 1;
      let end = index + 1;
      while ((end < body.length) && (depth > 0)) {
        if (body[end] === '\\') {
          end += 2;
          continue;
        }
        if (body[end] === '(') depth += 1;
        if (body[end] === ')') depth -= 1;
        end += 1;
      }
      const inner = body.slice(
        index + 1,
        end - 1
      );
      const nameMatch = /^\(\?<([A-Za-z0-9_$]+)>/.exec(body.slice(index));
      atoms.push({
        type: 'group',
        inner,
        named: nameMatch === null ? null : nameMatch[1],
        nonCapturing: inner.startsWith('?'),
        innerAtoms: tokenize(inner.replace(
          /^\?(?:<[^>]*>|:|=|!|<=|<!)/,
          ''
        )),
      });
      index = end;
      continue;
    }
    if ((char === '^') || (char === '$')) {
      atoms.push({ type: char === '^' ? 'anchor-start' : 'anchor-end', });
      index += 1;
      continue;
    }
    if (char === '|') {
      atoms.push({ type: 'alternation', });
      index += 1;
      continue;
    }
    if (char === '*') {
      const lazy = body[index + 1] === '?';
      applyQuantifier(
        atoms,
        lazy ? '*?' : '*'
      );
      index += lazy ? 2 : 1;
      continue;
    }
    if (char === '+') {
      const lazy = body[index + 1] === '?';
      applyQuantifier(
        atoms,
        lazy ? '+?' : '+'
      );
      index += lazy ? 2 : 1;
      continue;
    }
    if (char === '?') {
      applyQuantifier(
        atoms,
        '?'
      );
      index += 1;
      continue;
    }
    if (char === '{') {
      const close = body.indexOf(
        '}',
        index
      );
      if (close !== (-1)) {
        applyQuantifier(
          atoms,
          body.slice(
            index,
            close + 1
          )
        );
        index = close + 1;
        continue;
      }
    }
    if (char === '.') {
      atoms.push({
        type: 'class',
        name: 'any-char',
      });
      index += 1;
      continue;
    }
    atoms.push({
      type: 'literal',
      text: char,
    });
    index += 1;
  }
  return atoms;
}

function applyQuantifier(
  atoms,
  quantifier
) {
  const last = atoms.at(-1);
  if (last === undefined) {
    atoms.push({
      type: 'quantifier',
      quantifier,
    });
    return;
  }
  last.quantifier = (last.quantifier ?? '') + quantifier;
}

/**
 Flatten literal atoms into text, returning null when a non-literal atom appears.
 */
function literalText(atoms) {
  let text = '';
  for (const atom of atoms) {
    if ((atom.type !== 'literal') || ((atom.quantifier ?? '') !== '')) return null;
    text += atom.text;
  }
  return text;
}

/**
 Describe one atom in plain words for the issue body.
 */
function describeAtom(atom) {
  const quantifier = atom.quantifier ?? '';
  const quantifierText = quantifier === '' ? '' : ` repeated ${quantifier}`;
  if (atom.type === 'literal') return `"${atom.text}"${quantifierText}`;
  if (atom.type === 'class') {
    if (atom.name !== undefined) return `one ${atom.name} character${quantifierText}`;
    return `one character from ${atom.text}${quantifierText}`;
  }
  if (atom.type === 'group') return `a group (${atom.inner})${quantifierText}`;
  if (atom.type === 'boundary') return 'a word boundary';
  if (atom.type === 'anchor-start') return 'start of input';
  if (atom.type === 'anchor-end') return 'end of input';
  if (atom.type === 'alternation') return 'OR';
  return `${atom.type}${quantifierText}`;
}

/**
 Whether any group atom is a lookbehind assertion, which the tokenizer keeps as
 the raw group text.
 */
function bodyIncludesLookbehind(atoms) {
  for (const atom of atoms) {
    if (atom.type !== 'group') continue;
    const inner = atom.inner ?? '';
    if (inner.startsWith('?<=') || inner.startsWith('?<!')) return true;
    if (bodyIncludesLookbehind(atom.innerAtoms ?? [])) return true;
  }
  return false;
}

/**
 Merge adjacent literal atoms so a step sequence reads as prefix/class-run steps.
 */
function coalesceLiterals(atoms) {
  const merged = [];
  for (const atom of atoms) {
    const last = merged.at(-1);
    if ((atom.type === 'literal') && ((atom.quantifier ?? '') === '')
      && (last !== undefined)
      && (last.type === 'literal')
      && ((last.quantifier ?? '') === '')) {
      last.text += atom.text;
      continue;
    }
    merged.push({ ...atom, });
  }
  return merged;
}

/**
 Describe a literal-or-class step sequence as ordered scan steps.
 */
function sequenceSketch(middle) {
  const steps = [];
  for (const atom of middle) {
    if (atom.type === 'literal') {
      steps.push(`match the literal ${JSON.stringify(atom.text)}`);
      continue;
    }
    if (atom.type === 'group') {
      const candidates = literalAlternation(atom.innerAtoms ?? []);
      if (candidates !== null) {
        steps.push(`match one of the candidates ${JSON.stringify(candidates)}`);
        continue;
      }
      steps.push(`match the sub-pattern (${atom.inner}) with its own scan`);
      continue;
    }
    const classText = atom.name ?? atom.text
      ?? 'the class';
    const quantifier = atom.quantifier ?? '';
    const bound = /^\{(\d+)(?:,(\d*))?\}$/.exec(quantifier);
    if (bound !== null) {
      const minimum = bound[1];
      const maximum = bound[2] ?? '';
      steps.push(maximum === ''
        ? `walk characters satisfying ${classText}, requiring at least ${minimum}`
        : `walk ${minimum} to ${maximum} characters satisfying ${classText}`);
      continue;
    }
    if ((quantifier === '+') || (quantifier === '+?')) steps.push(`walk one or more characters satisfying ${classText}`);
    else if ((quantifier === '*') || (quantifier === '*?')) steps.push(`walk zero or more characters satisfying ${classText}`);
    else if (quantifier === '?') steps.push(`optionally consume one character satisfying ${classText}`);
    else steps.push(`consume one character satisfying ${classText}`);
  }
  return steps.join('; then ');
}

const CANDIDATE_CAP = 64;

/**
 Candidate strings a sequence of atoms can match, or null when the sequence is not
 built only from plain literals and groups whose bodies expand the same way.
 Nested groups expand by cartesian product, capped so a pathological pattern cannot
 blow up here.
 */
function sequenceTexts(atoms) {
  let combinations = [''];
  for (const atom of atoms) {
    let options = null;
    if ((atom.type === 'literal') && ((atom.quantifier ?? '') === '')) {
      options = [atom.text, ];
    } else if (atom.type === 'group') {
      options = alternativeTexts(atom.innerAtoms ?? []);
    }
    if (options === null) return null;
    const next = [];
    for (const prefix of combinations) {
      for (const option of options) next.push(prefix + option);
    }
    combinations = next;
    if (combinations.length > CANDIDATE_CAP) return null;
  }
  return combinations;
}

/**
 Candidate strings an atom list matches, splitting on top-level alternation and
 expanding each branch as a sequence. Null when any branch is not literal-shaped.
 */
function alternativeTexts(atoms) {
  const branches = [];
  let current = [];
  for (const atom of atoms) {
    if (atom.type === 'alternation') {
      branches.push(current);
      current = [];
      continue;
    }
    current.push(atom);
  }
  branches.push(current);
  const expanded = [];
  for (const branch of branches) {
    const texts = sequenceTexts(branch);
    if (texts === null) return null;
    expanded.push(...texts);
    if (expanded.length > CANDIDATE_CAP) return null;
  }
  if (expanded.length === 1) return null;
  if (expanded.some((text) => text === '')) return null;
  return expanded;
}

/**
 Candidate list for an atom list that is either a top-level alternation or a single
 group holding one, optionally wrapped in literals: `1 (?:test|passed)` becomes the
 candidates `1 test` and `1 passed`.
 */
function literalAlternation(atoms) {
  if (atoms.some((atom) => atom.type === 'alternation')) return alternativeTexts(atoms);
  const groupIndex = atoms.findIndex((atom) => atom.type === 'group');
  if (groupIndex === (-1)) return null;
  const inner = alternativeTexts(atoms[groupIndex]
    .innerAtoms
    ?? []);
  if (inner === null) return null;
  const before = literalText(atoms.slice(
    0,
    groupIndex
  ));
  const after = literalText(atoms.slice(groupIndex + 1));
  if ((before === null) || (after === null)) return null;
  return inner.map((candidate) => `${before}${candidate}${after}`);
}

/**
 Derive a replacement sketch from the atom list.
 */
function deriveSketch({
  atoms,
  flags,
  dynamic,
}) {
  if (dynamic) {
    return {
      shape: 'dynamic-construction',
      sketch: 'avoid constructing a RegExp: test the same condition with string APIs over the interpolated value',
      grammar: 'dynamically built pattern',
    };
  }
  const coalesced = coalesceLiterals(atoms);
  const hasBoundary = coalesced.some((atom) => atom.type === 'boundary');
  const significant = coalesced.filter((atom) => atom.type !== 'boundary');
  const anchoredStart = significant[0]
    ?.type
    === 'anchor-start';
  const anchoredEnd = significant.at(-1)
    ?.type
    === 'anchor-end';
  const middle = significant.slice(
    anchoredStart ? 1 : 0,
    anchoredEnd ? -1 : undefined
  );
  const grammar = coalesced.map(describeAtom)
    .join(', then ');

  // Case 0: a lookbehind encodes a backward condition, which is a backward walk.
  if (bodyIncludesLookbehind(coalesced)) {
    return {
      shape: 'lookbehind-scan',
      sketch: 'one forward pass that tracks escape parity: walk the characters, count the run of consecutive escape characters before each quote candidate, and treat the quote as effective when that count is even. The parity state is carried forward, so no backward rescan is needed and the whole input is visited once',
      grammar,
    };
  }

  // A multiline flag makes `^` and `$` line anchors, so the string form iterates lines.
  const perLine = flags.includes('m') && (anchoredStart || anchoredEnd);
  const linePrefix = perLine
    ? 'for each line (split on the newline), '
    : '';
  // A case-insensitive flag is a semantic requirement, not a detail: every string
  // comparison in the replacement has to fold case the same way.
  const caseNote = flags.includes('i')
    ? ' The pattern is case-insensitive, so every comparison in the replacement must fold case the same way (`toLowerCase()` on both sides for ASCII, and an explicit decision for non-ASCII, since the `i` flag with `u` applies Unicode simple case folding).'
    : '';
  const unanchoredNote = ((!anchoredStart) && (!anchoredEnd))
    ? ' The pattern is unanchored, so the replacement must search anywhere in the value rather than test its start or end.'
    : '';

  // Case 1: pure literal middle.
  const literal = literalText(middle);
  if ((literal !== null) && (literal !== '')) {
    if (hasBoundary) {
      return {
        shape: 'word-boundary-substring',
        sketch: `find candidate occurrences with indexOf(${JSON.stringify(literal)}) and keep only those whose flanking characters are not word characters (letters, digits, underscore); the scan stays linear because each candidate resumes after the previous one${caseNote}`,
        grammar,
      };
    }
    if (anchoredStart && anchoredEnd) return {
      shape: 'exact-equality',
      sketch: `${linePrefix}value === ${JSON.stringify(literal)}${caseNote}`,
      grammar,
    };
    if (anchoredStart) return {
      shape: 'prefix',
      sketch: `${linePrefix}value.startsWith(${JSON.stringify(literal)})${caseNote}`,
      grammar,
    };
    if (anchoredEnd) return {
      shape: 'suffix',
      sketch: `${linePrefix}value.endsWith(${JSON.stringify(literal)})${caseNote}`,
      grammar,
    };
    if (flags.includes('g')) return {
      shape: 'replace-all-literal',
      sketch: `value.replaceAll(${JSON.stringify(literal)}, replacement) with the string form, or a split/join`,
      grammar,
    };
    return {
      shape: 'substring',
      sketch: `value.includes(${JSON.stringify(literal)})`,
      grammar,
    };
  }

  // Case 2: a bounded run of one character class, optionally between literals.
  const classRuns = middle.filter((atom) => atom.type === 'class');
  if ((classRuns.length === 1) && middle.every((atom) => (atom.type === 'literal') || (atom.type === 'class'))) {
    const run = classRuns[0];
    const runText = run.name ?? run.text
      ?? 'class';
    const runIndex = middle.indexOf(run);
    const prefix = literalText(middle.slice(
      0,
      runIndex
    )) ?? '';
    const suffix = literalText(middle.slice(runIndex + 1)) ?? '';
    const bounded = /^\{(\d+)(?:,(\d*))?\}$/.exec(run.quantifier ?? '');
    if ((bounded !== null) && anchoredStart
      && anchoredEnd
      && middle.every((atom) => (atom.type === 'literal') || (atom === run))) {
      const exact = bounded[2] === undefined;
      return {
        shape: 'fixed-length-class-run',
        sketch: exact
          ? `${linePrefix}check the length is ${prefix.length + Number(bounded[1])
            + suffix.length}, ${prefix === '' ? '' : `check the literal ${JSON.stringify(prefix)} with startsWith, `}${suffix === '' ? '' : `check the literal ${JSON.stringify(suffix)} with endsWith, `}and check every remaining character satisfies ${runText}`
          : `${linePrefix}check the literal parts with startsWith/endsWith and walk ${bounded[1]} or more characters satisfying ${runText}`,
        grammar,
      };
    }
    return {
      shape: 'class-run-scan',
      sketch: `${linePrefix}linear scan: ${sequenceSketch(middle)}${caseNote}`,
      grammar,
    };
  }

  // Case 3: literal, class and literal-alternation steps only, in any count.
  function isPlainStep(atom) { return (atom.type === 'literal')
    || (atom.type === 'class')
    || ((atom.type === 'group') && (literalAlternation(atom.innerAtoms ?? []) !== null)); }
  if ((middle.length > 0) && middle.every(isPlainStep)) {
    return {
      shape: 'literal-class-sequence',
      sketch: `${linePrefix}single-pass linear scan: ${sequenceSketch(middle)}${caseNote}`,
      grammar,
    };
  }

  // Case 4: alternation of literals.
  if (middle.some((atom) => atom.type === 'alternation')) {
    return {
      shape: 'alternation',
      sketch: `a small array of literal candidates tested with some(startsWith/endsWith/includes), or one linear pass matching the longest candidate${caseNote}${unanchoredNote}`,
      grammar,
    };
  }

  // Case 5: a group whose body is only an alternation of literals is a candidate
  // list, not a field extraction.
  const candidateList = literalAlternation(middle);
  if (candidateList !== null) {
    return {
      shape: 'alternation',
      sketch: `${linePrefix}test the value against the fixed candidate list ${JSON.stringify(candidateList)} with some(startsWith/endsWith/includes), or one pass matching the longest candidate${caseNote}${unanchoredNote}`,
      grammar,
    };
  }

  // Case 6: captures between literal delimiters.
  if (middle.some((atom) => atom.type === 'group')) {
    const capturing = middle.filter((atom) => (atom.type === 'group') && (atom.nonCapturing === false));
    const capturedNames = capturing
      .map((atom) => atom.named ?? 'positional')
      .join(', ');
    return {
      shape: 'capture',
      sketch: `${linePrefix}indexOf the literal delimiters and slice between them; validate each slice with a character scan${capturing.length === 0 ? '' : `. The pattern captures ${capturing.length} field(s) (${capturedNames}), so the replacement must return the same field(s) in the same order`}${caseNote}${unanchoredNote}`,
      grammar,
    };
  }

  return {
    shape: 'scanner',
    sketch: `hand-written linear scanner over the same token grammar (indexOf/slice/charCodeAt walk, single pass, no backtracking)${caseNote}${unanchoredNote}`,
    grammar,
  };
}


export {
  splitPattern,
  tokenize,
  deriveSketch,
  coalesceLiterals,
  literalText,
  describeAtom
};
