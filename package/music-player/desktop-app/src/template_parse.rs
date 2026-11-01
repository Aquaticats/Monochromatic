//! Builds formula trees from tokens,
//!  and splits a template into literal text and formulas.
//!
//! A join is a flat list of operands rather than a nested pair,
//!  so the tree is only
//! as deep as the calls nest (bounded by `DEEPEST_CALL`).
//!  That keeps both
//! evaluation and dropping from recursing once per `+` sign in user text.

/// What:
///  imports the mistake record.
/// Why:
///  Parser mistakes use the same record as every other template mistake.
use crate::template_model::TemplateError;
/// What:
///  imports the token type and the tokenizer helpers.
/// Why:
///  The parser reads the tokens one cursor step at a time.
use crate::template_tokens::{Token, TokenKind, find_unit, slice_units, tokenize};

/// What:
///  `const DEEPEST_CALL: usize = 12` is the deepest a call may nest.
/// Why:
///  A bound keeps the recursive descent from running out of stack on hostile input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deepest = 12;
/// ```
const DEEPEST_CALL: usize = 12;

/// What:
///  `const DOLLAR: u16` is the delimiter that opens and closes a formula.
/// Why:
///  Splitting the template and finding a formula's end both search for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dollar = "$";
/// ```
const DOLLAR: u16 = b'$' as u16;

/// What:
///  `pub enum FormulaNode` is one node of a parsed formula.
/// Why:
///  Evaluation and help walk the same tree,
///  so the tree is one shared immutable record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FormulaNode = TextNode | WordNode | JoinNode | CallNode;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaNode {
    /// Quoted text,
    ///  kept as written and never evaluated.
    Text(
        /// The text without its quotation marks.
        String,
    ),
    /// A bare word,
    ///  such as a field name or a number.
    Word(
        /// The word as written.
        String,
    ),
    /// Values joined with `+`,
    ///  in order;
    ///  always at least two operands.
    Join(
        /// The operands in order.
        Vec<FormulaNode>,
    ),
    /// A function call.
    Call(
        /// The call with its name,
        /// its brackets,
        /// and its arguments.
        CallNode,
    ),
}

/// What:
///  `pub struct CallNode` is one function call with its name,
///  bracket positions,
///  and arguments.
/// Why:
///  Help needs the bracket and comma positions to find the argument under the caret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CallNode = { kind: "call"; name: string; start: number; open: number; close?: number;
///   parameters: FormulaNode[]; separators: number[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallNode {
    /// The function name as written.
    pub name: String,
    /// UTF-16 offset of the name's first unit.
    pub start: usize,
    /// UTF-16 offset just past the opening bracket.
    pub open: usize,
    /// UTF-16 offset of the closing bracket,
    ///  or `None` when the bracket is missing.
    pub close: Option<usize>,
    /// Arguments in call order.
    pub parameters: Vec<FormulaNode>,
    /// UTF-16 offsets of each comma between arguments.
    pub separators: Vec<usize>,
}

/// What:
///  `pub struct FormulaPart` is one formula between dollar signs,
///  with its tree and span.
/// Why:
///  Help uses the offsets to find the formula under the caret,
///  and an unclosed formula is marked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FormulaPart = { kind: "formula"; tree: FormulaNode; start: number; end: number; closed: boolean };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaPart {
    /// The parsed formula.
    pub tree: FormulaNode,
    /// Offset of the opening dollar sign.
    pub start: usize,
    /// Offset just past the closing dollar sign,
    ///  or the template's end when unclosed.
    pub end: usize,
    /// Whether a closing dollar sign was found.
    pub closed: bool,
}

/// What:
///  `pub enum TemplatePart` is one piece of a template:
///  literal text or a formula.
/// Why:
///  Evaluation and help both walk the template as an ordered list of these pieces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TemplatePart = { kind: "literal"; value: string } | FormulaPart;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart {
    /// Text outside formulas,
    ///  shown as written.
    Literal(
        /// The text as written.
        String,
    ),
    /// One formula between dollar signs.
    Formula(
        /// The parsed formula with its span.
        FormulaPart,
    ),
}

/// What:
///  `pub struct ParsedTemplate` pairs a template's pieces with the mistakes found while splitting and parsing it.
/// Why:
///  A template with parse mistakes is not evaluated,
///  so the two results travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedTemplate = { parts: TemplatePart[]; errors: TemplateError[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTemplate {
    /// The pieces in template order.
    pub parts: Vec<TemplatePart>,
    /// Every parse mistake,
    ///  in the order found.
    pub errors: Vec<TemplateError>,
}

/// What:
///  `struct FormulaParser` reads tokens with one position cursor and records each mistake it finds.
/// Why:
///  A struct holds the cursor and the abandoned flag,
///  so the mutually recursive descent can share them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FormulaParser { private position = 0; private abandoned = false; }
/// ```
struct FormulaParser<'a> {
    /// The tokens to read,
    ///  in order.
    tokens: Vec<Token>,
    /// Index of the next unread token.
    position: usize,
    /// Set once nesting passes the bound;
    ///  the rest of the formula is then skipped unreported.
    abandoned: bool,
    /// The formula's first offset in the whole template.
    offset: usize,
    /// The formula's length in UTF-16 units,
    ///  used to place the missing-value mistake at its end.
    source_length: usize,
    /// The shared list that mistakes are added to.
    errors: &'a mut Vec<TemplateError>,
}

/// What:
///  the parser reading methods,
///  one per grammar rule.
/// Why:
///  They share the token cursor and the mistake list.
impl FormulaParser<'_> {
    /// What:
    ///  `fn operator_span(&self, value: &str) -> Option<(usize, usize)>` returns the span of the next token when it is that operator.
    /// Why:
    ///  Each check reads the next token once and gives the caller its position without holding a borrow.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const operatorSpan = (value: string) =>
    ///   peek()?.kind === "operator" && peek()?.value === value ? [peek().start, peek().end] : undefined;
    /// ```
    fn operator_span(&self, value: &str) -> Option<(usize, usize)> {
        let token = self.tokens.get(self.position)?;
        if token.kind == TokenKind::Operator && token.value == value {
            return Some((token.start, token.end));
        }
        return None;
    }

    /// What:
    ///  `fn primary(&mut self, depth: usize) -> FormulaNode` reads one value:
    ///  text,
    ///  a word,
    ///  or a call.
    /// Why:
    ///  Every value in a formula,
    ///  including each argument,
    ///  goes through this one function.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function primary(depth: number): FormulaNode { ... }
    /// ```
    fn primary(&mut self, depth: usize) -> FormulaNode {
        let Some(token) = self.tokens.get(self.position).cloned() else {
            if !self.abandoned {
                self.errors.push(TemplateError {
                    subject: String::from("formula"),
                    message: String::from("a value is missing"),
                    at: self.offset + self.source_length,
                });
            }
            return FormulaNode::Text(String::new());
        };
        if token.kind == TokenKind::Operator {
            self.errors.push(TemplateError {
                subject: String::from("formula"),
                message: format!("a value is missing before {}", token.value),
                at: token.start,
            });
            return FormulaNode::Text(String::new());
        }
        self.position += 1;
        let open = match token.kind {
            TokenKind::Word => self.operator_span("("),
            _ => None,
        };
        if let Some(open_span) = open {
            return self.call_node(token, open_span, depth);
        }
        return match token.kind {
            TokenKind::Text => FormulaNode::Text(token.value),
            _ => FormulaNode::Word(token.value),
        };
    }

    /// What:
    ///  `fn call_node(&mut self, name: Token, open: (usize, usize), depth: usize) -> FormulaNode` reads one call's arguments and closing bracket.
    /// Why:
    ///  A call past the nesting bound keeps no arguments,
    ///  and its mistake is reported once for the whole formula.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function callNode(name: Token, open: Token, depth: number): CallNode { ... }
    /// ```
    fn call_node(&mut self, name: Token, open: (usize, usize), depth: usize) -> FormulaNode {
        self.position += 1;
        if depth >= DEEPEST_CALL {
            self.errors.push(TemplateError {
                subject: name.value.clone(),
                message: String::from("calls are nested too deeply"),
                at: name.start,
            });
            self.abandoned = true;
            self.position = self.tokens.len();
            return FormulaNode::Call(build_call(name, open, None, Vec::new(), Vec::new()));
        }
        if let Some(close_span) = self.operator_span(")") {
            self.position += 1;
            return FormulaNode::Call(build_call(name, open, Some(close_span.0), Vec::new(), Vec::new()));
        }
        let mut parameters = vec![self.joined(depth + 1)];
        let mut separators = Vec::new();
        while let Some(comma) = self.operator_span(",") {
            separators.push(comma.0);
            self.position += 1;
            parameters.push(self.joined(depth + 1));
        }
        if let Some(close_span) = self.operator_span(")") {
            self.position += 1;
            return FormulaNode::Call(build_call(name, open, Some(close_span.0), parameters, separators));
        }
        if !self.abandoned {
            self.errors.push(TemplateError {
                subject: name.value.clone(),
                message: String::from("a closing bracket is missing"),
                at: name.start,
            });
        }
        return FormulaNode::Call(build_call(name, open, None, parameters, separators));
    }

    /// What:
    ///  `fn joined(&mut self, depth: usize) -> FormulaNode` reads values separated by `+`.
    /// Why:
    ///  `+` joins text left to right,
    ///  so the operands are kept as one flat list in order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function joined(depth: number): FormulaNode { ... }
    /// ```
    fn joined(&mut self, depth: usize) -> FormulaNode {
        let mut operands = vec![self.primary(depth)];
        while self.operator_span("+").is_some() {
            self.position += 1;
            operands.push(self.primary(depth));
        }
        if operands.len() == 1 {
            return operands.remove(0);
        }
        return FormulaNode::Join(operands);
    }

    /// What:
    ///  `fn parse(&mut self) -> FormulaNode` reads the whole formula and reports any tokens left over.
    /// Why:
    ///  A leftover token means two values sit side by side with no `+` between them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function parse(): FormulaNode { ... }
    /// ```
    fn parse(&mut self) -> FormulaNode {
        let tree = self.joined(0);
        if let Some(leftover) = self.tokens.get(self.position) {
            self.errors.push(TemplateError {
                subject: String::from("formula"),
                message: format!("unexpected {}", leftover.value),
                at: leftover.start,
            });
        }
        return tree;
    }
}

/// What:
///  `fn build_call(...) -> CallNode` assembles one call record from its name token,
///  bracket span,
///  and argument lists.
/// Why:
///  Every exit of the call reader builds a call the same way,
///  so the record is made in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const buildCall = (name, open, close, parameters, separators): CallNode => ({ ... });
/// ```
fn build_call(
    name: Token,
    open: (usize, usize),
    close: Option<usize>,
    parameters: Vec<FormulaNode>,
    separators: Vec<usize>,
) -> CallNode {
    return CallNode {
        name: name.value,
        start: name.start,
        open: open.1,
        close,
        parameters,
        separators,
    };
}

/// What:
///  `fn parse_formula(units: &[u16], offset: usize, errors: &mut Vec<TemplateError>) -> FormulaNode` tokenizes and parses one formula.
/// Why:
///  Tokenizing and parsing share one mistake list,
///  so their mistakes come out together in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseFormula(text: string, offset: number, errors: TemplateError[]): FormulaNode { ... }
/// ```
fn parse_formula(units: &[u16], offset: usize, errors: &mut Vec<TemplateError>) -> FormulaNode {
    let tokens = tokenize(units, offset, errors);
    let mut parser = FormulaParser {
        tokens,
        position: 0,
        abandoned: false,
        offset,
        source_length: units.len(),
        errors,
    };
    return parser.parse();
}

/// What:
///  `fn append_formula_piece(...) -> usize` adds the literal text before one dollar sign and the formula after it,
///  then returns where reading resumes.
/// Why:
///  An unclosed formula returns the template's end,
///  so the template loop stops after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendFormulaPiece(text: string, start: number, open: number, parts: TemplatePart[],
///   errors: TemplateError[]): number { ... }
/// ```
fn append_formula_piece(
    units: &[u16],
    start: usize,
    open: usize,
    parts: &mut Vec<TemplatePart>,
    errors: &mut Vec<TemplateError>,
) -> usize {
    if open > start {
        parts.push(TemplatePart::Literal(slice_units(units, start, open)));
    }
    let close = find_unit(units, open + 1, DOLLAR);
    if close.is_none() {
        errors.push(TemplateError {
            subject: String::from("formula"),
            message: format!("the $ at character {} has no closing $", open + 1),
            at: open,
        });
    }
    let source_end = close.unwrap_or(units.len());
    let tree = parse_formula(&units[open + 1..source_end], open + 1, errors);
    let end = close.map_or(units.len(), |found| return found + 1);
    parts.push(TemplatePart::Formula(FormulaPart {
        tree,
        start: open,
        end,
        closed: close.is_some(),
    }));
    return end;
}

/// What:
///  `pub fn formula_holds_caret(formula: &FormulaPart, caret: usize) -> bool` tells whether a caret offset sits inside a formula's span.
/// Why:
///  Help and field insertion both need the same rule for which formula the caret is in,
///  so the span rule is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inside = caret > part.start && caret <= (part.closed ? part.end - 1 : part.end);
/// ```
pub fn formula_holds_caret(formula: &FormulaPart, caret: usize) -> bool {
    let last = if formula.closed { formula.end.saturating_sub(1) } else { formula.end };
    return caret > formula.start && caret <= last;
}

/// What:
///  `pub fn parse_template(text: &str) -> ParsedTemplate` splits a template into literal text and formulas.
/// Why:
///  A dollar sign with no partner is reported rather than shown,
///  so a half-written formula never leaks into the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseTemplate(text: string): { parts: TemplatePart[]; errors: TemplateError[] } { ... }
/// ```
pub fn parse_template(text: &str) -> ParsedTemplate {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut parts = Vec::new();
    let mut errors = Vec::new();
    let mut index = 0;
    while index < units.len() {
        index = match find_unit(&units, index, DOLLAR) {
            None => {
                parts.push(TemplatePart::Literal(slice_units(&units, index, units.len())));
                units.len()
            }
            Some(open) => append_formula_piece(&units, index, open, &mut parts, &mut errors),
        };
    }
    return ParsedTemplate { parts, errors };
}

/// What:
///  `mod tests;` pulls in the unit tests from `template_parse_tests.rs`.
/// Why:
///  The tests sit beside the code without adding to the code-line budget.
#[cfg(test)]
#[path = "template_parse_tests.rs"]
mod tests;
