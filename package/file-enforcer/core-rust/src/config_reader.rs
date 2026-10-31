//! What:
//!  Field-by-field readers over a `serde_json::Value`,
//! collecting every problem with its JSON pointer.
//! Why:
//!  The decoder reports all the mistakes in a config at once,
//! each naming the file and the pointer (`/rules/3/output`),
//!  instead of stopping at the first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Reader { errors: Diagnostic[]; text(object, key, pointer): string; }
//! ```

/// Imports `ValueExpr` from `crate::config_parts`.
use crate::config_parts::ValueExpr;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{Map, Value};

/// What:
///  The error collector and the readers that fill it.
///  `Vec<Diagnostic>` is a growable list (TS:
/// `Diagnostic[]`).
/// Why:
///  Readers return a harmless default after recording an error,
/// so decoding continues and the caller discards
///  the result when `errors` is not empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Reader { errors: Diagnostic[] = []; constructor(readonly displayPath: string) {} }
/// ```
pub struct Reader {
    /// Every problem found so far,
    /// in discovery order.
    pub errors: Vec<Diagnostic>,
    /// The config path shown in messages.
    pub display_path: String,
}

/// The `impl Reader` block.
impl Reader {
    /// What:
    ///  Start a reader for one config file.
    /// Why:
    ///  Each message names the file it came from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new Reader(displayPath)
    /// ```
    pub fn new(display_path: &str) -> Reader {
        return Reader { errors: Vec::new(), display_path: display_path.to_string() };
    }

    /// What:
    ///  Record one problem at a JSON pointer.
    /// Why:
    ///  A single place formats every decode message the same way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// error(pointer: string, message: string): void
    /// ```
    pub fn error(&mut self, pointer: &str, message: &str) {
        let shown = if pointer.is_empty() { "/" } else { pointer };
        let full = format!("{}: {shown}: {message}", self.display_path);
        let diagnostic = Diagnostic::new("config-invalid", &full).with_path(&self.display_path);
        self.errors.push(diagnostic);
    }

    /// What:
    ///  Require a JSON object,
    /// borrowing its members.
    /// Why:
    ///  Every rule and part is an object;
    /// anything else is a typed mistake.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// object(value: unknown, pointer: string): Record<string, unknown> | undefined
    /// ```
    pub fn object<'a>(&mut self, value: &'a Value, pointer: &str) -> Option<&'a Map<String, Value>> {
        if let Some(object) = value.as_object() {
            return Some(object);
        }
        self.error(pointer, "must be an object");
        return None;
    }

    /// What:
    ///  Report every key that is not in the allowed list.
    /// Why:
    ///  Unknown keys are almost always typos;
    /// the schema says `additionalProperties: false` too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rejectUnknown(object, pointer, allowed: string[]): void
    /// ```
    pub fn reject_unknown(&mut self, object: &Map<String, Value>, pointer: &str, allowed: &[&str]) {
        for key in object.keys() {
            if !allowed.contains(&key.as_str()) {
                let message = format!("unknown key {key:?}; accepted keys are {}", allowed.join(", "));
                self.error(pointer, &message);
            }
        }
    }

    /// What:
    ///  Read a required text member.
    /// Why:
    ///  Most rule fields are text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// text(object, key, pointer): string
    /// ```
    pub fn text(&mut self, object: &Map<String, Value>, key: &str, pointer: &str) -> String {
        match object.get(key) {
            Some(Value::String(text)) => return text.clone(),
            Some(_) => self.error(&format!("{pointer}/{key}"), "must be a string"),
            None => self.error(pointer, &format!("is missing the required key {key:?}")),
        }
        return String::new();
    }

    /// What:
    ///  Read an optional text member;
    /// `null` counts as absent,
    /// as the schema allows.
    /// Why:
    ///  `Option` fields in the schema accept `null`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// optionalText(object, key, pointer): string | undefined
    /// ```
    pub fn optional_text(&mut self, object: &Map<String, Value>, key: &str, pointer: &str) -> Option<String> {
        match object.get(key) {
            None | Some(Value::Null) => return None,
            Some(Value::String(text)) => return Some(text.clone()),
            Some(_) => self.error(&format!("{pointer}/{key}"), "must be a string"),
        }
        return None;
    }

    /// What:
    ///  Read a required array member,
    /// borrowing its elements.
    /// Why:
    ///  Callers decode each element with its own pointer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// array(object, key, pointer): unknown[]
    /// ```
    pub fn array<'a>(&mut self, object: &'a Map<String, Value>, key: &str, pointer: &str) -> &'a [Value] {
        match object.get(key) {
            Some(Value::Array(items)) => return items.as_slice(),
            Some(_) => self.error(&format!("{pointer}/{key}"), "must be an array"),
            None => self.error(pointer, &format!("is missing the required key {key:?}")),
        }
        return &[];
    }

    /// What:
    ///  Read a required array of text.
    /// Why:
    ///  Globs,
    /// key paths and ids are lists of strings.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// textList(object, key, pointer): string[]
    /// ```
    pub fn text_list(&mut self, object: &Map<String, Value>, key: &str, pointer: &str) -> Vec<String> {
        let items = self.array(object, key, pointer);
        return self.texts_of(items, &format!("{pointer}/{key}"));
    }

    /// What:
    ///  Read an optional array of text;
    /// `null` counts as absent.
    /// Why:
    ///  `onlyDirs` is optional.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// optionalTextList(object, key, pointer): string[] | undefined
    /// ```
    pub fn optional_text_list(&mut self, object: &Map<String, Value>, key: &str, pointer: &str) -> Option<Vec<String>> {
        match object.get(key) {
            None | Some(Value::Null) => return None,
            Some(_) => return Some(self.text_list(object, key, pointer)),
        }
    }

    /// What:
    ///  Convert every element of an array to text,
    /// recording a problem for each that is not.
    /// Why:
    ///  Shared by the list readers above.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// textsOf(items: unknown[], pointer: string): string[]
    /// ```
    pub fn texts_of(&mut self, items: &[Value], pointer: &str) -> Vec<String> {
        let mut texts: Vec<String> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            if let Some(text) = item.as_str() {
                texts.push(text.to_string());
            } else {
                self.error(&format!("{pointer}/{index}"), "must be a string");
            }
        }
        return texts;
    }

    /// What:
    ///  Decode a value expression:
    /// a literal,
    /// a list,
    /// or one of the tagged objects `file`,
    /// `var`,
    /// `concat`.
    /// Why:
    ///  The only expression language the config has.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// valueExpr(value: unknown, pointer: string): ValueExpr
    /// ```
    pub fn value_expr(&mut self, value: &Value, pointer: &str) -> ValueExpr {
        match value {
            Value::String(text) => return ValueExpr::Text(text.clone()),
            Value::Bool(flag) => return ValueExpr::Bool(*flag),
            Value::Number(number) => {
                if let Some(integer) = number.as_i64() {
                    return ValueExpr::Integer(integer);
                }
                self.error(pointer, "numbers must be integers");
            }
            Value::Array(items) => {
                let mut list: Vec<ValueExpr> = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    list.push(self.value_expr(item, &format!("{pointer}/{index}")));
                }
                return ValueExpr::List(list);
            }
            Value::Object(object) => return self.tagged_expr(object, pointer),
            Value::Null => self.error(pointer, "null is not a value expression"),
        }
        return ValueExpr::Text(String::new());
    }

    /// What:
    ///  Decode the tagged-object forms of a value expression.
    /// Why:
    ///  Exactly one of `file`,
    /// `var`,
    /// `concat` must be present.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// taggedExpr(object, pointer): ValueExpr
    /// ```
    fn tagged_expr(&mut self, object: &Map<String, Value>, pointer: &str) -> ValueExpr {
        let allowed = ["file", "var", "concat"];
        self.reject_unknown(object, pointer, &allowed);
        if object.len() != 1 {
            self.error(pointer, "must hold exactly one of the keys file, var, concat");
            return ValueExpr::Text(String::new());
        }
        if let Some(inner) = object.get("file") {
            let path = self.value_expr(inner, &format!("{pointer}/file"));
            return ValueExpr::File { file: Box::new(path) };
        }
        if let Some(inner) = object.get("var") {
            let name = self.text(object, "var", pointer);
            let _ = inner;
            return ValueExpr::Var { var: name };
        }
        let items = self.array(object, "concat", pointer);
        let mut parts: Vec<ValueExpr> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            parts.push(self.value_expr(item, &format!("{pointer}/concat/{index}")));
        }
        return ValueExpr::Concat { concat: parts };
    }
}
