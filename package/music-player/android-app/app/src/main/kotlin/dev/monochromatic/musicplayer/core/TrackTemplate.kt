// Supporting-text template language for track rows and the playing track.
// Text outside `$...$` formulas is always shown. A formula reads track fields, formats durations,
// converts case or cuts text, and joins pieces with `+`. There is no conditional.
// This is a port of package/music-player/design/template-reference.mjs; that file stays the reference.

// What:     `package ...core` places this engine beside Queue, Session, and PlaybackMode.
// Why:      Settings, the list rows, and the playing line can share one platform-independent evaluator.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import java.math` reads and prints numbers; `kotlin.math` supplies abs and floor.
// Why:      Duration output must match JavaScript's number text digit for digit.
//
// In TS you'd write (pseudocode):
// ```ts
// // Numbers and Math are built in; no import is needed.
// ```
import java.math.BigDecimal
import java.math.BigInteger
import kotlin.math.abs
import kotlin.math.floor

//region Fields and default templates
// What:     `data class TemplateField(...)` is an immutable record of one field a template may read.
// Why:      Settings lists each field with its label, and the editor help shows the same description.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateField = { mode: string; label: string; description: string };
// ```
/** One field a template may read, named by the mode word used inside `mi(...)`. */
data class TemplateField(
    /** Mode word used inside `mi(...)`. */
    val mode: String,
    /** Label Settings shows for the field. */
    val label: String,
    /** Sentence the editor help shows for the field. */
    val description: String,
)

// What:     `val trackTemplateFields: List<TemplateField>` lists the fields a track row may read.
// Why:      Track rows never see the playing track's place or folder total, so they get a shorter list.
//
// In TS you'd write (pseudocode):
// ```ts
// const trackTemplateFields: TemplateField[] = [{ mode: "title", label: "Title", description: "..." }];
// ```
/** Fields available to the track rows template. */
val trackTemplateFields: List<TemplateField> = listOf(
    TemplateField(mode = "title", label = "Title", description = "The name the row shows as its title."),
    TemplateField(mode = "file", label = "File name", description = "The file name without its extension."),
    TemplateField(mode = "ext", label = "Extension", description = "The file extension."),
    TemplateField(mode = "folder", label = "Folder", description = "The folder that holds the file."),
    TemplateField(mode = "path", label = "Path", description = "The path from the library root."),
    TemplateField(
        mode = "len",
        label = "Duration",
        description = "The length of the track, shown here as minutes and seconds. In a template it is a number of " +
            "seconds, and tf formats it, as in tf(mi(len)).",
    ),
    TemplateField(
        mode = "peak",
        label = "True peak",
        description = "The true peak with its unit, such as −1.2 dBTP, or nothing before the file is analyzed.",
    ),
)

// What:     `val playingTemplateFields` is the track rows' list plus `track` and `total`.
// Why:      The playing track also shows where it sits in its folder and how many tracks the folder holds.
//
// In TS you'd write (pseudocode):
// ```ts
// const playingTemplateFields = [...trackTemplateFields, { mode: "track" }, { mode: "total" }];
// ```
/** Fields available to the playing track template. */
val playingTemplateFields: List<TemplateField> = trackTemplateFields + listOf(
    TemplateField(
        mode = "track",
        label = "Place in folder",
        description = "Where the track is in its folder, counting from 1.",
    ),
    TemplateField(mode = "total", label = "Tracks in folder", description = "How many tracks the folder holds."),
)

// What:     `const val DEFAULT_TRACK_TEMPLATE` is the supporting line a track row starts with.
// Why:      It shows the duration, a space, then the true peak, which is empty before analysis.
//
// In TS you'd write (pseudocode):
// ```ts
// const DEFAULT_TRACK_TEMPLATE = "$tf(mi(len), m:ss)$ $mi(peak)$";
// ```
/** Default text for the track rows template; the dollar signs are escaped for Kotlin. */
const val DEFAULT_TRACK_TEMPLATE: String = "\$tf(mi(len), m:ss)\$ \$mi(peak)\$"

// What:     `enum class TemplateKind` names the two templates Settings lists, each with its label,
//           default text, and allowed fields.
// Why:      The track rows and the playing track differ in default text and in the fields they may read.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateKind = "track" | "playing";
// ```
/** Each template Settings lists, with its heading, default text, and the fields it may use. */
enum class TemplateKind(
    /** Heading Settings shows for this template. */
    val label: String,
    /** Text the template starts from before the user edits it. */
    val defaultTemplate: String,
    /** Fields the template may read; a field outside this list is refused. */
    val fields: List<TemplateField>,
) {
    /** Supporting text for each track row in the library. */
    TRACK_ROWS("Track rows", DEFAULT_TRACK_TEMPLATE, trackTemplateFields),

    /** Supporting text for the track that is playing. */
    PLAYING_TRACK("Playing track", "\$mi(track)\$ of \$mi(total)\$ \$mi(peak)\$", playingTemplateFields),
}

// What:     `data class TemplateParameter(...)` records one argument of a template function.
// Why:      The editor help names each argument and says what it is for.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateParameter = { name: string; description: string };
// ```
/** One argument of a template function, as the editor help shows it. */
data class TemplateParameter(
    /** Argument name shown in the signature. */
    val name: String,
    /** Sentence the editor help shows for the argument. */
    val description: String,
)

// What:     `data class TemplateFunction(...)` describes one template function's signature and arity.
// Why:      Evaluation checks argument counts against it, and the editor help displays it.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateFunction = { signature: string; minimum: number; maximum: number; parameters: TemplateParameter[] };
// ```
/** Signature, arity bounds, and argument help for one template function. */
data class TemplateFunction(
    /** Call shape shown in the help, such as `mi(field)`. */
    val signature: String,
    /** Fewest arguments the call may take. */
    val minimum: Int,
    /** Most arguments the call may take. */
    val maximum: Int,
    /** Arguments in call order, each with its help text. */
    val parameters: List<TemplateParameter>,
)

// What:     `val templateFunctions: Map<String, TemplateFunction>` maps `mi`, `tf`, and `tc` to their help.
// Why:      The Map lookup means names such as `constructor` are unknown functions, not inherited members.
//
// In TS you'd write (pseudocode):
// ```ts
// const functions: Record<string, FunctionHelp> = { mi: { ... }, tf: { ... }, tc: { ... } };
// ```
/** The three template functions with their signatures and argument help. */
val templateFunctions: Map<String, TemplateFunction> = mapOf(
    "mi" to TemplateFunction(
        signature = "mi(field)",
        minimum = 1,
        maximum = 1,
        parameters = listOf(
            TemplateParameter(
                name = "field",
                description = "Field: one of " + trackTemplateFields.joinToString(", ") { field -> field.mode } + ".",
            ),
        ),
    ),
    "tf" to TemplateFunction(
        signature = "tf(seconds, [format])",
        minimum = 1,
        maximum = 2,
        parameters = listOf(
            TemplateParameter(name = "seconds", description = "Seconds: a duration, such as mi(len)."),
            TemplateParameter(
                name = "format",
                description = "Format: h, m and s for hours, minutes and seconds; " +
                    "a doubled letter pads with a zero; text between apostrophes is kept. Without it, m:ss.",
            ),
        ),
    ),
    "tc" to TemplateFunction(
        signature = "tc(mode, text, [length])",
        minimum = 2,
        maximum = 3,
        parameters = listOf(
            TemplateParameter(
                name = "mode",
                description = "Mode: low, up or cap to change case; cut to keep the first characters.",
            ),
            TemplateParameter(name = "text", description = "Text: the text to convert."),
            TemplateParameter(name = "length", description = "Length: how many characters cut keeps."),
        ),
    ),
)
//endregion

//region Track values
// What:     `sealed interface TemplateTrack` is what a template reads field values from.
// Why:      The track rows and the playing track expose different fields, and one evaluator serves both.
//
// In TS you'd write (pseudocode):
// ```ts
// interface TemplateTrack { fieldText(mode: string): string | undefined }
// ```
/** A source of field text for one track. */
sealed interface TemplateTrack {
    /** Text of one field, or null when the track has no value for it. */
    fun fieldText(mode: String): String?
}

// What:     `data class TrackFields(...)` holds one track's own values for the track rows template.
// Why:      Each field has a fixed type, and a missing duration or peak is null rather than an absent key.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrackFields = { title: string; file: string; ext: string; folder: string; path: string;
//   len?: number; peak?: string };
// ```
/** One track's fields as the track rows template reads them. */
data class TrackFields(
    /** The name the row shows as its title. */
    val title: String,
    /** The file name without its extension. */
    val file: String,
    /** The file extension. */
    val ext: String,
    /** The folder that holds the file. */
    val folder: String,
    /** The path from the library root. */
    val path: String,
    /** The duration in seconds, or null when unknown. */
    val len: Double?,
    /** The true peak text with its unit, or null before analysis. */
    val peak: String?,
) : TemplateTrack {
    // What:     `override fun fieldText(mode: String): String?` maps a mode word to this track's text.
    // Why:      Each mode reads one property, and an unknown mode yields null so the caller can refuse it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fieldText(mode: string) { return this[mode as keyof TrackFields]; }
    // ```
    /** Text of one field, or null when the mode names no field or the value is missing. */
    override fun fieldText(mode: String): String? = when (mode) {
        "title" -> title
        "file" -> file
        "ext" -> ext
        "folder" -> folder
        "path" -> path
        "len" -> len?.let { length -> numberText(length) }
        "peak" -> peak
        else -> null
    }
}

// What:     `data class PlayingTrackFields(...)` adds the place and folder total to a track.
// Why:      The playing template reads the same seven fields plus `track` and `total`.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayingTrackFields = TrackFields & { track?: number; total?: number };
// ```
/** The playing track: its own fields, and where it sits in its folder. */
data class PlayingTrackFields(
    /** The track's own fields. */
    val track: TrackFields,
    /** Where the track is in its folder, counting from 1, or null when unknown. */
    val placeInFolder: Int?,
    /** How many tracks the folder holds, or null when unknown. */
    val tracksInFolder: Int?,
) : TemplateTrack {
    // What:     `override fun fieldText(mode: String): String?` answers `track` and `total` itself and
    //           delegates every other mode to the track.
    // Why:      The folder place is not part of the track's own record, so it is read from here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fieldText(mode: string) { return mode === "track" ? this.placeInFolder : ...; }
    // ```
    /** Text of one field, or null when the mode names no field or the value is missing. */
    override fun fieldText(mode: String): String? = when (mode) {
        "track" -> placeInFolder?.toString()
        "total" -> tracksInFolder?.toString()
        else -> track.fieldText(mode)
    }
}
//endregion

//region Mistakes and results
// What:     `data class TemplateError(...)` records one mistake, the name it concerns, and its position.
// Why:      Each mistake is named after the function or word at fault and placed by character offset.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateError = { subject: string; message: string; at: number };
// ```
/** One mistake in a template, with the function or word it concerns and its character offset. */
data class TemplateError(
    /** The function name, or `formula` or `text` for a structural mistake. */
    val subject: String,
    /** What is wrong, in plain words. */
    val message: String,
    /** Character offset in the whole template where the mistake starts. */
    val at: Int,
)

// What:     `sealed interface TemplateResult` is either the shown text or the list of mistakes.
// Why:      A template that does not apply shows no text at all, so the two outcomes must not overlap.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateResult = { valid: true; text: string } | { valid: false; errors: Err[] };
// ```
/** The outcome of evaluating a template for one track. */
sealed interface TemplateResult {
    /** The template applied; `text` is the line to show. */
    data class Shown(
        /** The line to show. */
        val text: String,
    ) : TemplateResult

    /** The template does not apply; `errors` lists each mistake in template order. */
    data class Refused(
        /** Every mistake found, in the order the evaluator found them. */
        val errors: List<TemplateError>,
    ) : TemplateResult
}

// What:     `fun errorLines(errors: List<TemplateError>): List<String>` formats each mistake as one line.
// Why:      The editor shows one line per mistake, ordered by where each sits in the template.
//
// In TS you'd write (pseudocode):
// ```ts
// const lines = [...errors].sort((a, b) => a.at - b.at).map(e => `${e.subject}: ${e.message}`);
// ```
/** One line per mistake, naming the function first, in the order the mistakes sit in the template. */
fun errorLines(errors: List<TemplateError>): List<String> =
    errors.sortedBy { error -> error.at }.map { error -> error.subject + ": " + error.message }

// What:     `fun errorSentence(error: TemplateError): String` words one mistake as a plain sentence that says what to
//           do.
// Why:      The editor shows the mistakes to a person who does not know the evaluator's vocabulary, so each one names
//           the
//           call at fault and the remedy. The wording is the desktop's (`template_plain.rs`), so both editors read
//           alike.
//
// In TS you'd write (pseudocode):
// ```ts
// const errorSentence = (e: TemplateError) => table[kindOf(e)](e);
// ```
/** Words one mistake as a sentence that names the call at fault and says what to change. */
fun errorSentence(error: TemplateError): String {
    /** The call or word at fault. */
    val subject: String = error.subject
    /** What is wrong, as the evaluator reports it. */
    val message: String = error.message
    return when {
        message.startsWith("unknown field ") ->
            subject + " has no field called " + message.removePrefix("unknown field ") +
                ". Pick a name from the Fields list."
        message.startsWith("unknown format letter ") ->
            subject + " does not know the format letter " + message.removePrefix("unknown format letter ") +
                ". Use h, m or s, and put any other words between apostrophes."
        message.startsWith("unknown mode ") ->
            subject + " has no mode called " + message.removePrefix("unknown mode ") + ". Use low, up, cap or cut."
        message.startsWith("needs a number of seconds, not ") ->
            subject + " needs a number of seconds, not " + message.removePrefix("needs a number of seconds, not ") +
                ". Use mi(len) for the duration."
        message.startsWith("takes ") -> subject + " " + message + "."
        message.startsWith("unexpected ") ->
            "Unexpected " + message.removePrefix("unexpected ") +
                " in a formula. Join values with + or put each one in its own pair of $ signs."
        message.startsWith("a value is missing before ") ->
            "A value is missing before " + message.removePrefix("a value is missing before ") +
                " in a formula. Put a field such as mi(title), or some text, there."
        message.startsWith("the $ at character ") ->
            "The $ at character " + message.removePrefix("the $ at character ").substringBefore(' ') +
                " has no closing $. Add a $ where the formula ends."
        message == "unknown function" -> subject + " is not a function. Use mi, tf or tc."
        message == "a closing bracket is missing" ->
            subject + " is missing its closing bracket. Add a ) after its last value."
        message == "calls are nested too deeply" -> subject + " sits inside too many calls. Take some of the calls out."
        message == "a value is missing" ->
            "A formula is missing a value. Put a field such as mi(title), or some text, in it."
        message == "a quotation mark is not closed" -> "A quotation mark is not closed. Add a closing quotation mark."
        message.startsWith("cut needs a length") -> "tc needs a length to cut to, as in tc(cut, text, 10)."
        else -> subject + ": " + message + "."
    }
}

// What:     `fun errorSentences(errors: List<TemplateError>, parsed: ParsedTemplate): List<String>` words the mistakes
//           of a
//           template, one sentence per formula, in template order.
// Why:      The editor lists every mistake at once instead of only the first. One formula with several slips gets one
//           sentence for its first slip, so a single typo is not listed three times (the desktop does the same).
//
// In TS you'd write (pseudocode):
// ```ts
// const errorSentences = (errors: TemplateError[], parsed: ParsedTemplate) =>
// onePerFormula(errors.sort(byOffset)).map(errorSentence);
// ```
/** One plain sentence per formula that holds a mistake, plus one per mistake outside any formula, in template order. */
fun errorSentences(errors: List<TemplateError>, parsed: ParsedTemplate): List<String> {
    /** Formulas of the template, whose spans own the mistakes inside them. */
    val formulas: List<FormulaPart> = parsed.parts.filterIsInstance<FormulaPart>()
    /** Indices of the formulas that already have a sentence. */
    val spoken = mutableSetOf<Int>()
    return errors.sortedBy { error -> error.at }.mapNotNull { error ->
        /** Index of the formula that holds the mistake, or -1 when none does. */
        val owner: Int = formulas.indexOfFirst { formula -> error.at >= formula.start && error.at <= formula.end }
        if (owner >= 0 && !spoken.add(owner)) null else errorSentence(error)
    }
}

//endregion

//region Tokens
// What:     `private enum class TokenKind` names the three kinds of formula token.
// Why:      The tokenizer and parser branch on text, operator, and word tokens.
//
// In TS you'd write (pseudocode):
// ```ts
// type TokenKind = "text" | "operator" | "word";
// ```
/** The kinds of token a formula splits into. */
private enum class TokenKind {
    /** A quoted text token, stored without its quotation marks. */
    TEXT,

    /** A single operator character: `+`, `(`, `)`, or `,`. */
    OPERATOR,

    /** A bare word that runs to the next break character. */
    WORD,
}

// What:     `private data class Token(...)` records one token with its absolute start and end offsets.
// Why:      Errors and call positions point back into the whole template, so offsets are absolute.
//
// In TS you'd write (pseudocode):
// ```ts
// type Token = { kind: TokenKind; value: string; start: number; end: number };
// ```
/** One token of a formula, with its absolute character span in the template. */
private data class Token(
    /** Which kind of token this is. */
    val kind: TokenKind,
    /** Token text, without quotation marks for a text token. */
    val value: String,
    /** Absolute offset of the first character. */
    val start: Int,
    /** Absolute offset just past the last character. */
    val end: Int,
)

// What:     `private const val OPERATOR_CHARACTERS = "+(),"` lists the characters that always stand alone.
// Why:      A bare word stops at these, so a call and a join are always separate tokens.
//
// In TS you'd write (pseudocode):
// ```ts
// const operators = ["+", "(", ")", ","];
// ```
/** Single characters that split formula text into operator tokens. */
private const val OPERATOR_CHARACTERS: String = "+(),"

// What:     `private fun isWordBreak(character: Char): Boolean` tells whether a character ends a bare word.
// Why:      A word runs to the next space, quotation mark, or operator character.
//
// In TS you'd write (pseudocode):
// ```ts
// const isWordBreak = (c: string) => " \t\n\"".includes(c) || "+(),".includes(c);
// ```
/** Tells whether a character ends a bare word. */
private fun isWordBreak(character: Char): Boolean =
    character == ' ' || character == '\t' || character == '\n' || character == '"' || character in OPERATOR_CHARACTERS

// What:     `private fun tokenize(text: String, offset: Int, errors: MutableList<TemplateError>): List<Token>`
//           splits one formula's text in a single left-to-right pass.
// Why:      Formulas are short, and one pass keeps each mistake's position exact.
//
// In TS you'd write (pseudocode):
// ```ts
// function tokenize(text: string, offset: number, errors: TemplateError[]): Token[] { ... }
// ```
/** Splits one formula's text; `offset` is the formula's first character in the whole template. */
private fun tokenize(text: String, offset: Int, errors: MutableList<TemplateError>): List<Token> {
    /** Tokens found so far, in order. */
    val tokens = mutableListOf<Token>()
    /** Index of the character being read. */
    var index = 0
    while (index < text.length) {
        /** The character being read. */
        val character = text[index]
        if (character == ' ' || character == '\t' || character == '\n') {
            index += 1
        } else if (character == '"') {
            /** Index of the closing quotation mark, or -1 when it is missing. */
            val close = text.indexOf('"', index + 1)
            if (close < 0) {
                errors.add(TemplateError("text", "a quotation mark is not closed", offset + index))
                tokens.add(Token(TokenKind.TEXT, text.substring(index + 1), offset + index, offset + text.length))
                break
            }
            tokens.add(Token(TokenKind.TEXT, text.substring(index + 1, close), offset + index, offset + close + 1))
            index = close + 1
        } else if (character in OPERATOR_CHARACTERS) {
            tokens.add(Token(TokenKind.OPERATOR, character.toString(), offset + index, offset + index + 1))
            index += 1
        } else {
            /** Index just past the bare word being read. */
            var end = index
            while (end < text.length && !isWordBreak(text[end])) {
                end += 1
            }
            tokens.add(Token(TokenKind.WORD, text.substring(index, end), offset + index, offset + end))
            index = end
        }
    }
    return tokens
}
//endregion

//region Tokens to a tree; nesting is bounded so the descent is too
// What:     `private const val DEEPEST_CALL = 12` is the deepest a call may nest.
// Why:      A bound keeps the recursive descent from running out of stack on hostile input.
//
// In TS you'd write (pseudocode):
// ```ts
// const deepest = 12;
// ```
/** Deepest nesting of calls the parser descends into. */
private const val DEEPEST_CALL: Int = 12

// What:     `sealed interface FormulaNode` is one node of a parsed formula.
// Why:      Evaluation and help walk the same tree, so the tree is a shared immutable record.
//
// In TS you'd write (pseudocode):
// ```ts
// type FormulaNode = TextNode | WordNode | JoinNode | CallNode;
// ```
/** One node of a parsed formula: text, a bare word, a join, or a call. */
sealed interface FormulaNode

// What:     `data class TextNode(val value: String)` is quoted or literal text inside a formula.
// Why:      Quoted text is kept as written and never evaluated as a field or function.
//
// In TS you'd write (pseudocode):
// ```ts
// type TextNode = { kind: "text"; value: string };
// ```
/** Quoted text inside a formula. */
data class TextNode(
    /** The text without its quotation marks. */
    val value: String,
) : FormulaNode

// What:     `data class WordNode(val value: String)` is a bare word such as a field name or number.
// Why:      Words are text until a function reads them, so `mi(title)` and `"title"` mean the same field.
//
// In TS you'd write (pseudocode):
// ```ts
// type WordNode = { kind: "word"; value: string };
// ```
/** A bare word inside a formula. */
data class WordNode(
    /** The word's text. */
    val value: String,
) : FormulaNode

// What:     `data class JoinNode(val left: FormulaNode, val right: FormulaNode)` joins two values with `+`.
// Why:      Joins nest left to right, so `a + b + c` is one tree of two joins.
//
// In TS you'd write (pseudocode):
// ```ts
// type JoinNode = { kind: "join"; left: FormulaNode; right: FormulaNode };
// ```
/** Two values joined with `+`. */
data class JoinNode(
    /** The value before the `+`. */
    val left: FormulaNode,
    /** The value after the `+`. */
    val right: FormulaNode,
) : FormulaNode

// What:     `data class CallNode(...)` is one function call with its name, bracket positions, and arguments.
// Why:      Help needs the bracket and comma positions to find the argument under the caret.
//
// In TS you'd write (pseudocode):
// ```ts
// type CallNode = { kind: "call"; name: string; start: number; open: number; close?: number;
//   parameters: FormulaNode[]; separators: number[] };
// ```
/** A function call with its name, the offsets of its brackets and commas, and its arguments. */
data class CallNode(
    /** The function name as written. */
    val name: String,
    /** Absolute offset of the name's first character. */
    val start: Int,
    /** Absolute offset just past the opening bracket. */
    val open: Int,
    /** Absolute offset of the closing bracket, or null when the bracket is missing. */
    val close: Int?,
    /** Arguments in call order. */
    val parameters: List<FormulaNode>,
    /** Absolute offsets of each comma between arguments. */
    val separators: List<Int>,
) : FormulaNode

// What:     `private class FormulaParser(...)` reads tokens with one position cursor.
// Why:      A class holds the cursor and the abandoned flag, so mutually recursive descent can share them.
//
// In TS you'd write (pseudocode):
// ```ts
// class FormulaParser { private position = 0; private abandoned = false; }
// ```
/** Builds a formula tree from tokens and records each mistake it finds. */
private class FormulaParser(
    /** The formula's text, used to place the missing-value mistake at its end. */
    private val text: String,
    /** The formula's first character offset in the whole template. */
    private val offset: Int,
    /** The tokens to read, in order. */
    private val tokens: List<Token>,
    /** The shared list that mistakes are added to. */
    private val errors: MutableList<TemplateError>,
) {
    // What:     `private var position = 0` is the index of the next unread token.
    // Why:      Reading advances this cursor; nothing else tracks where the parser is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let position = 0;
    // ```
    /** Index of the next unread token. */
    private var position: Int = 0

    // What:     `private var abandoned = false` records that nesting passed the bound.
    // Why:      After the bound, the rest of the formula is skipped and its missing values are not reported.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let abandoned = false;
    // ```
    /** Set once nesting passes the bound. */
    private var abandoned: Boolean = false

    // What:     `private fun peek(): Token?` returns the next unread token or null at the end.
    // Why:      Callers test for the end without indexing past the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const peek = (): Token | undefined => tokens[position];
    // ```
    /** The next unread token, or null when every token has been read. */
    private fun peek(): Token? = tokens.getOrNull(position)

    // What:     `private fun operatorToken(value: String): Token?` returns the next token when it is that operator.
    // Why:      Each check reads the same token once and gives the caller its position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const operatorToken = (value: string) =>
    //   peek()?.kind === "operator" && peek()?.value === value ? peek() : undefined;
    // ```
    /** The next token when it is the given operator, otherwise null. */
    private fun operatorToken(value: String): Token? {
        /** The next unread token, which the operator check reads once. */
        val token = peek() ?: return null
        return if (token.kind == TokenKind.OPERATOR && token.value == value) token else null
    }

    // What:     `private fun primary(depth: Int): FormulaNode` reads one value: text, a word, or a call.
    // Why:      Every value in a formula, including each argument, goes through this one function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function primary(depth: number): FormulaNode { ... }
    // ```
    /** Reads one value, reporting a missing or misplaced value and returning empty text for it. */
    private fun primary(depth: Int): FormulaNode {
        /** The next unread token, or null at the end of the formula. */
        val token = peek()
        if (token == null) {
            if (!abandoned) {
                errors.add(TemplateError("formula", "a value is missing", offset + text.length))
            }
            return TextNode("")
        }
        if (token.kind == TokenKind.OPERATOR) {
            errors.add(TemplateError("formula", "a value is missing before " + token.value, token.start))
            return TextNode("")
        }
        position += 1
        /** The opening bracket that follows the token, when the token is a call's name. */
        val open = operatorToken("(")
        if (token.kind == TokenKind.WORD && open != null) {
            return callNode(token, open, depth)
        }
        return if (token.kind == TokenKind.TEXT) TextNode(token.value) else WordNode(token.value)
    }

    // What:     `private fun callNode(name: Token, open: Token, depth: Int): FormulaNode` reads the arguments
    //           and the closing bracket of one call.
    // Why:      A call that passes the nesting bound is kept without arguments, and its mistake is reported once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function callNode(name: Token, open: Token, depth: number): CallNode { ... }
    // ```
    /** Reads one call's arguments; a missing closing bracket is reported unless nesting was abandoned. */
    private fun callNode(name: Token, open: Token, depth: Int): FormulaNode {
        position += 1
        if (depth >= DEEPEST_CALL) {
            errors.add(TemplateError(name.value, "calls are nested too deeply", name.start))
            abandoned = true
            position = tokens.size
            return buildCall(name, open, null, emptyList(), emptyList())
        }
        /** The closing bracket when the call has no arguments, otherwise null. */
        val immediateClose = operatorToken(")")
        if (immediateClose != null) {
            position += 1
            return buildCall(name, open, immediateClose.start, emptyList(), emptyList())
        }
        /** The argument values read so far, in call order. */
        val parameters = mutableListOf<FormulaNode>()
        /** The comma offsets read so far, in call order. */
        val separators = mutableListOf<Int>()
        parameters.add(joined(depth + 1))
        /** The comma that follows the latest argument, or null when the arguments end. */
        var separator = operatorToken(",")
        while (separator != null) {
            separators.add(separator.start)
            position += 1
            parameters.add(joined(depth + 1))
            separator = operatorToken(",")
        }
        /** The closing bracket that ends the arguments, or null when it is missing. */
        val close = operatorToken(")")
        if (close != null) {
            position += 1
            return buildCall(name, open, close.start, parameters, separators)
        }
        if (!abandoned) {
            errors.add(TemplateError(name.value, "a closing bracket is missing", name.start))
        }
        return buildCall(name, open, null, parameters, separators)
    }

    // What:     `private fun buildCall(...)` assembles one `CallNode` from the name, bracket, and arguments.
    // Why:      Every exit of `callNode` builds a call the same way, so the record is made in one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const buildCall = (name, open, close, parameters, separators): CallNode => ({ ... });
    // ```
    /** Builds a call record from its name token, opening bracket token, closing offset, and argument lists. */
    private fun buildCall(
        name: Token,
        open: Token,
        close: Int?,
        parameters: List<FormulaNode>,
        separators: List<Int>,
    ): CallNode = CallNode(
        name = name.value,
        start = name.start,
        open = open.end,
        close = close,
        parameters = parameters,
        separators = separators,
    )

    // What:     `private fun joined(depth: Int): FormulaNode` reads values separated by `+`.
    // Why:      `+` joins text left to right, so each join wraps everything read before it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function joined(depth: number): FormulaNode { ... }
    // ```
    /** Reads one value and every value joined to it with `+`. */
    private fun joined(depth: Int): FormulaNode {
        /** The value read so far, which the next `+` joins to. */
        var left: FormulaNode = primary(depth)
        while (operatorToken("+") != null) {
            position += 1
            left = JoinNode(left, primary(depth))
        }
        return left
    }

    // What:     `fun parse(): FormulaNode` reads the whole formula and reports any tokens left over.
    // Why:      A leftover token means the formula has two values side by side with no `+` between them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function parse(): FormulaNode { ... }
    // ```
    /** Parses the whole formula; anything left after the expression is reported as unexpected. */
    fun parse(): FormulaNode {
        /** The tree for the whole formula. */
        val tree = joined(0)
        /** The first token after the expression, or null when the formula was read completely. */
        val leftover = peek()
        if (leftover != null) {
            errors.add(TemplateError("formula", "unexpected " + leftover.value, leftover.start))
        }
        return tree
    }
}

// What:     `private fun parseFormula(text: String, offset: Int, errors: MutableList<TemplateError>): FormulaNode`
//           tokenizes and parses one formula.
// Why:      Tokenizing and parsing share the same mistake list, so their mistakes come out together.
//
// In TS you'd write (pseudocode):
// ```ts
// function parseFormula(text: string, offset: number, errors: TemplateError[]): FormulaNode { ... }
// ```
/** Parses one formula's text into a tree, adding its mistakes to `errors`. */
private fun parseFormula(text: String, offset: Int, errors: MutableList<TemplateError>): FormulaNode {
    /** The tokens of the formula, in order. */
    val tokens = tokenize(text, offset, errors)
    return FormulaParser(text, offset, tokens, errors).parse()
}

// What:     `sealed interface TemplatePart` is one piece of a template: literal text or a formula.
// Why:      Evaluation and help both walk the template as an ordered list of these pieces.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplatePart = LiteralPart | FormulaPart;
// ```
/** One piece of a template, in order. */
sealed interface TemplatePart

// What:     `data class LiteralPart(val value: String)` is text outside any formula.
// Why:      Literal text is always shown, so it needs no evaluation.
//
// In TS you'd write (pseudocode):
// ```ts
// type LiteralPart = { kind: "literal"; value: string };
// ```
/** Text outside formulas, shown as written. */
data class LiteralPart(
    /** The text as written. */
    val value: String,
) : TemplatePart

// What:     `data class FormulaPart(...)` is one formula with the offsets of its dollar signs.
// Why:      Help uses the offsets to find the formula under the caret, and an unclosed formula is marked.
//
// In TS you'd write (pseudocode):
// ```ts
// type FormulaPart = { kind: "formula"; tree: FormulaNode; start: number; end: number; closed: boolean };
// ```
/** One formula between dollar signs, with its tree and span. */
data class FormulaPart(
    /** The parsed formula. */
    val tree: FormulaNode,
    /** Offset of the opening dollar sign. */
    val start: Int,
    /** Offset just past the closing dollar sign, or the template's end when unclosed. */
    val end: Int,
    /** Whether a closing dollar sign was found. */
    val closed: Boolean,
) : TemplatePart

// What:     `data class ParsedTemplate(...)` pairs a template's pieces with the mistakes found in parsing.
// Why:      A template with parse mistakes is not evaluated, so the two results travel together.
//
// In TS you'd write (pseudocode):
// ```ts
// type ParsedTemplate = { parts: TemplatePart[]; errors: TemplateError[] };
// ```
/** A template split into pieces, with the mistakes found while splitting it. */
data class ParsedTemplate(
    /** The pieces in template order. */
    val parts: List<TemplatePart>,
    /** Every parse mistake, in the order found. */
    val errors: List<TemplateError>,
)

// What:     `private fun appendFormulaPiece(...)` adds the literal text before one dollar sign and the formula
//           that follows it, then returns where the next piece starts.
// Why:      Moving the piece logic out of the loop keeps `parseTemplate` free of jump statements, and an
//           unclosed formula returns the template's end so the loop stops.
//
// In TS you'd write (pseudocode):
// ```ts
// function appendFormulaPiece(text: string, start: number, open: number, parts: TemplatePart[],
//   errors: TemplateError[]): number { ... }
// ```
/** Adds the text before a dollar sign and the formula after it; returns the offset where reading resumes. */
private fun appendFormulaPiece(
    text: String,
    start: Int,
    open: Int,
    parts: MutableList<TemplatePart>,
    errors: MutableList<TemplateError>,
): Int {
    if (open > start) {
        parts.add(LiteralPart(text.substring(start, open)))
    }
    /** Offset of the closing dollar sign, or -1 when it is missing. */
    val close = text.indexOf('$', open + 1)
    if (close < 0) {
        errors.add(TemplateError("formula", "the \$ at character " + (open + 1) + " has no closing \$", open))
    }
    /** Offset where the formula's source ends, which is the template's end when the dollar sign is missing. */
    val sourceEnd = if (close < 0) text.length else close
    /** The parsed formula between the dollar signs. */
    val tree = parseFormula(text.substring(open + 1, sourceEnd), open + 1, errors)
    parts.add(FormulaPart(tree, open, if (close < 0) text.length else close + 1, close >= 0))
    return if (close < 0) text.length else close + 1
}

// What:     `fun parseTemplate(text: String): ParsedTemplate` splits a template into literal text and formulas.
// Why:      A dollar sign with no partner is reported rather than shown, so a half-written formula never leaks.
//
// In TS you'd write (pseudocode):
// ```ts
// function parseTemplate(text: string): { parts: TemplatePart[]; errors: TemplateError[] } { ... }
// ```
/** Splits a template into literal text and formulas. A `$` with no partner is reported, not shown as text. */
fun parseTemplate(text: String): ParsedTemplate {
    /** The pieces found so far, in template order. */
    val parts = mutableListOf<TemplatePart>()
    /** The parse mistakes found so far. */
    val errors = mutableListOf<TemplateError>()
    /** Offset where the next piece starts. */
    var index = 0
    while (index < text.length) {
        /** Offset of the next dollar sign, or -1 when no dollar sign remains. */
        val open = text.indexOf('$', index)
        index = if (open < 0) {
            parts.add(LiteralPart(text.substring(index)))
            text.length
        } else {
            appendFormulaPiece(text, index, open, parts, errors)
        }
    }
    return ParsedTemplate(parts, errors)
}
//endregion

//region Numbers and text as JavaScript writes and reads them
// What:     `private const val ABSENT = ""` is the text a missing value yields.
// Why:      A missing field shows nothing and never a placeholder, as the reference does.
//
// In TS you'd write (pseudocode):
// ```ts
// const absent = "";
// ```
/** Text shown for a missing value. */
private const val ABSENT: String = ""

// What:     `private val jsWhitespaceCodePoints: Set<Int>` lists the code points JavaScript's trim removes.
// Why:      Kotlin's trim keeps some characters that JavaScript's trim removes, so the set is listed by code point.
//
// In TS you'd write (pseudocode):
// ```ts
// const jsWhitespace = new Set([0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x20, 0xa0, 0x1680,
//   0x2028, 0x2029, 0x202f, 0x205f, 0x3000, 0xfeff]);
// ```
/** Code points JavaScript's trim removes: ASCII white space, the Unicode spaces, and the line separators. */
private val jsWhitespaceCodePoints: Set<Int> =
    setOf(0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x0020, 0x00A0, 0x1680) +
        (0x2000..0x200A) +
        setOf(0x2028, 0x2029, 0x202F, 0x205F, 0x3000, 0xFEFF)

// What:     `private val hexadecimalPattern`, `octalPattern`, `binaryPattern`, and `decimalPattern` match
//           the number spellings JavaScript's Number() accepts.
// Why:      Kotlin's toDouble accepts suffixes such as `d` and `f` that JavaScript rejects, so text is matched first.
//
// In TS you'd write (pseudocode):
// ```ts
// const decimal = /^[+-]?(Infinity|(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?)$/;
// ```
/** Matches hexadecimal text such as `0x1F`. */
private val hexadecimalPattern: Regex = Regex("0[xX][0-9a-fA-F]+")

/** Matches octal text such as `0o17`. */
private val octalPattern: Regex = Regex("0[oO][0-7]+")

/** Matches binary text such as `0b101`. */
private val binaryPattern: Regex = Regex("0[bB][01]+")

/** Matches decimal text with an optional sign and exponent. */
private val decimalPattern: Regex = Regex("[+-]?(Infinity|(\\d+\\.?\\d*|\\.\\d+)([eE][+-]?\\d+)?)")

// What:     `private const val HEX_RADIX: Int = 16` is the radix that reads hexadecimal text.
// Why:      Naming the radix keeps the number-reading code free of unexplained literals.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEX_RADIX = 16;
// ```
/** Radix of hexadecimal number text. */
private const val HEX_RADIX: Int = 16

// What:     `private const val OCTAL_RADIX: Int = 8` is the radix that reads octal text.
// Why:      Naming the radix keeps the number-reading code free of unexplained literals.
//
// In TS you'd write (pseudocode):
// ```ts
// const OCTAL_RADIX = 8;
// ```
/** Radix of octal number text. */
private const val OCTAL_RADIX: Int = 8

// What:     `private const val BINARY_RADIX: Int = 2` is the radix that reads binary text.
// Why:      Naming the radix keeps the number-reading code free of unexplained literals.
//
// In TS you'd write (pseudocode):
// ```ts
// const BINARY_RADIX = 2;
// ```
/** Radix of binary number text. */
private const val BINARY_RADIX: Int = 2

// What:     `private fun isJsWhitespace(character: Char): Boolean` tells whether JavaScript's trim
//           removes a character.
// Why:      Trimming a formula's number text must follow JavaScript's whitespace set, not Kotlin's.
//
// In TS you'd write (pseudocode):
// ```ts
// const isSpace = (c: string) => jsWhitespace.has(c.charCodeAt(0));
// ```
/** Tells whether JavaScript's trim removes this character. */
private fun isJsWhitespace(character: Char): Boolean = character.code in jsWhitespaceCodePoints

// What:     `private fun numberFromText(text: String): Double?` reads text the way JavaScript's Number() does.
// Why:      A duration may be written in decimal, hexadecimal, octal, or binary, and nothing else reads as a number.
//
// In TS you'd write (pseudocode):
// ```ts
// const n = Number(text);
// ```
/** Reads a number from text, or null when JavaScript would give NaN. */
private fun numberFromText(text: String): Double? {
    if (hexadecimalPattern.matches(text)) {
        return BigInteger(text.substring(2), HEX_RADIX).toDouble()
    }
    if (octalPattern.matches(text)) {
        return BigInteger(text.substring(2), OCTAL_RADIX).toDouble()
    }
    if (binaryPattern.matches(text)) {
        return BigInteger(text.substring(2), BINARY_RADIX).toDouble()
    }
    if (decimalPattern.matches(text)) {
        return text.toDouble()
    }
    return null
}

// What:     `private fun asNumber(text: String): Double?` reads a finite number from trimmed text.
// Why:      `tf` needs seconds, and empty, NaN, or infinite text is not a duration.
//
// In TS you'd write (pseudocode):
// ```ts
// function asNumber(text: string): number | undefined {
//   const n = Number(text.trim()); return text.trim() === "" || !Number.isFinite(n) ? undefined : n;
// }
// ```
/** Reads a finite number from text with JavaScript's whitespace rules, or null when there is none. */
private fun asNumber(text: String): Double? {
    /** The text without JavaScript white space at either end. */
    val trimmed = text.trim { character -> isJsWhitespace(character) }
    if (trimmed.isEmpty()) {
        return null
    }
    /** The number the trimmed text spells, or null when it spells none. */
    val number = numberFromText(trimmed) ?: return null
    return if (number.isFinite()) number else null
}

// What:     `private fun numberText(value: Double): String` prints a number the way JavaScript's String() does.
// Why:      Track lengths and time parts must print with JavaScript's plain or exponent layout.
//
// In TS you'd write (pseudocode):
// ```ts
// const text = String(value);
// ```
/** Prints a number with JavaScript's rules: plain for ordinary sizes, an exponent for very large or small ones. */
private fun numberText(value: Double): String {
    if (value.isNaN()) {
        return "NaN"
    }
    if (value.isInfinite()) {
        return if (value > 0.0) "Infinity" else "-Infinity"
    }
    if (value == 0.0) {
        return "0"
    }
    /** The minus sign, or nothing for a positive value. */
    val sign = if (value < 0.0) "-" else ""
    /** The shortest decimal form of the value, without trailing zeros. */
    val decimal = BigDecimal(abs(value).toString()).stripTrailingZeros()
    /** The significant digits of the value. */
    val digits = decimal.unscaledValue().toString()
    /** Where the decimal point falls, counted from the start of the digits. */
    val pointPosition = digits.length - decimal.scale()
    return sign + layoutDigits(digits, pointPosition)
}

// What:     `private const val MAX_PLAIN_POINT: Int = 21` is the largest decimal point position printed without
//           an exponent.
// Why:      JavaScript switches to an exponent form beyond this position, so the threshold is named.
//
// In TS you'd write (pseudocode):
// ```ts
// const MAX_PLAIN_POINT = 21;
// ```
/** Largest decimal point position that still prints as plain digits. */
private const val MAX_PLAIN_POINT: Int = 21

// What:     `private const val MIN_PLAIN_POINT: Int = -6` is the point position below which an exponent is used.
// Why:      JavaScript prints very small numbers with an exponent once the point moves this far left.
//
// In TS you'd write (pseudocode):
// ```ts
// const MIN_PLAIN_POINT = -6;
// ```
/** Point position above which small numbers still print as plain digits. */
private const val MIN_PLAIN_POINT: Int = -6

// What:     `private fun layoutDigits(digits: String, pointPosition: Int): String` places the decimal point
//           using JavaScript's thresholds of 21 and minus six.
// Why:      Matching the thresholds makes exponent forms such as `1e+21` identical to the reference.
//
// In TS you'd write (pseudocode):
// ```ts
// function layout(digits: string, n: number): string { ... }
// ```
/** Lays out significant digits, where the value equals 0.digits times ten to the pointPosition. */
private fun layoutDigits(digits: String, pointPosition: Int): String {
    /** The number of significant digits. */
    val count = digits.length
    if (count <= pointPosition && pointPosition <= MAX_PLAIN_POINT) {
        return digits + "0".repeat(pointPosition - count)
    }
    if (0 < pointPosition && pointPosition <= MAX_PLAIN_POINT) {
        return digits.substring(0, pointPosition) + "." + digits.substring(pointPosition)
    }
    if (MIN_PLAIN_POINT < pointPosition && pointPosition <= 0) {
        return "0." + "0".repeat(-pointPosition) + digits
    }
    /** The power of ten of the first significant digit. */
    val exponent = pointPosition - 1
    /** The first digit, a point, and the rest of the digits, or the lone digit. */
    val mantissa = if (count == 1) digits else digits.substring(0, 1) + "." + digits.substring(1)
    /** The sign written before the exponent's digits. */
    val exponentSign = if (exponent < 0) "-" else "+"
    return mantissa + "e" + exponentSign + abs(exponent)
}

// What:     `private fun keepCodePoints(text: String, count: Double): String` keeps the first whole characters
//           counted by code point.
// Why:      The reference counts characters as code points, so a surrogate pair is one character.
//
// In TS you'd write (pseudocode):
// ```ts
// const kept = [...text].slice(0, Math.floor(count)).join("");
// ```
/** Keeps the first `count` code points of the text, or all of them when fewer exist. */
private fun keepCodePoints(text: String, count: Double): String {
    /** The number of code points in the text. */
    val total = text.codePointCount(0, text.length)
    /** The number of code points to keep, never more than the text holds. */
    val kept = if (count >= total.toDouble()) total else count.toInt()
    return text.substring(0, text.offsetByCodePoints(0, kept))
}

// What:     `private fun formatSeconds(seconds: Double, format: String): String` fills the format's letters
//           with hours, minutes, and seconds.
// Why:      Duration text follows the reference's letter, padding, and apostrophe rules exactly.
//
// In TS you'd write (pseudocode):
// ```ts
// function formatSeconds(seconds: number, format: string): string { ... }
// ```
/** Formats a duration; a doubled letter pads with a zero, and text between apostrophes is kept as written. */
private fun formatSeconds(seconds: Double, format: String): String {
    /** The whole seconds, never negative. */
    val whole = maxOf(0.0, floor(seconds))
    /** The whole hours in the duration. */
    val hours = floor(whole / 3600.0)
    // Minutes carry the hours when the format has no hour letter, so 4000 seconds reads 66:40 under m:ss.
    /** The minutes shown, counted from the start of the hour when the format has an hour letter. */
    val minutes = if (format.contains('h')) floor((whole % 3600.0) / 60.0) else floor(whole / 60.0)
    /** The value for each time letter. */
    val parts = mapOf('h' to hours, 'm' to minutes, 's' to whole % 60.0)
    /** The formatted text built so far. */
    val output = StringBuilder()
    /** Whether the reader is inside apostrophes, where letters are kept as written. */
    var kept = false
    /** Index of the format character being read. */
    var index = 0
    while (index < format.length) {
        /** The format character being read. */
        val letter = format[index]
        /** The time value for the letter, or null when the letter is not a time letter. */
        val value = parts[letter]
        if (letter == '\'') {
            kept = !kept
            index += 1
        } else if (kept || value == null) {
            output.append(letter)
            index += 1
        } else {
            /** Whether the letter is written twice, which pads the value to two digits. */
            val doubled = index + 1 < format.length && format[index + 1] == letter
            /** The value's plain text. */
            val text = numberText(value)
            output.append(if (doubled) text.padStart(2, '0') else text)
            index += if (doubled) 2 else 1
        }
    }
    return output.toString()
}
//endregion

//region Tree to text for one track
// What:     `private class TemplateEvaluation(...)` evaluates formula trees for one track and one allowed field set.
// Why:      The track, allowed fields, and mistake list travel together through every nested call.
//
// In TS you'd write (pseudocode):
// ```ts
// class Evaluation { constructor(track, allowed, errors) {} }
// ```
/** Evaluates formula trees for one track, collecting every mistake into one list. */
private class TemplateEvaluation(
    /** The track whose field text the template reads. */
    private val track: TemplateTrack,
    /** The fields the template is allowed to read. */
    private val allowed: List<TemplateField>,
    /** The shared list that mistakes are added to. */
    private val errors: MutableList<TemplateError>,
) {
    // What:     `fun evaluate(node: FormulaNode): String` returns the text one node yields.
    // Why:      Every node, including each argument, yields text, so one function covers the whole tree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function evaluate(node: FormulaNode): string { ... }
    // ```
    /** Text one node yields; a function with a mistake yields empty text. */
    fun evaluate(node: FormulaNode): String = when (node) {
        is TextNode -> node.value
        is WordNode -> node.value
        is JoinNode -> evaluate(node.left) + evaluate(node.right)
        is CallNode -> evaluateCall(node)
    }

    // What:     `private fun evaluateCall(node: CallNode): String` checks the name and arity, then evaluates the call.
    // Why:      Every argument is evaluated before the call, so a mistake in any of them is reported.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function evaluateCall(node: CallNode): string { ... }
    // ```
    /** Evaluates one call, refusing unknown names and wrong argument counts before reading any argument. */
    private fun evaluateCall(node: CallNode): String {
        /** The signature and arity of the named function, or null when the name is unknown. */
        val definition = templateFunctions[node.name]
        if (definition == null) {
            errors.add(TemplateError(node.name, "unknown function", node.start))
            return ABSENT
        }
        if (node.parameters.size < definition.minimum || node.parameters.size > definition.maximum) {
            errors.add(TemplateError(node.name, arityMessage(definition), node.start))
            return ABSENT
        }
        /** The text of each argument, evaluated in call order. */
        val values = node.parameters.map { parameter -> evaluate(parameter) }
        return when (node.name) {
            "mi" -> evaluateMi(node, values)
            "tf" -> evaluateTf(node, values)
            "tc" -> evaluateTc(node, values)
            else -> throw IllegalStateException("A function is defined but not evaluated: " + node.name)
        }
    }

    // What:     `private fun arityMessage(definition: TemplateFunction): String` writes the wrong-count mistake.
    // Why:      The message names the allowed count and the call shape, so the user can fix the call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const message = `takes ${range} ${unit}, as in ${signature}`;
    // ```
    /** Describes the allowed argument count and the call shape. */
    private fun arityMessage(definition: TemplateFunction): String {
        /** The allowed count as text: one number, or a range joined with `or`. */
        val range = if (definition.minimum == definition.maximum) {
            definition.minimum.toString()
        } else {
            definition.minimum.toString() + " or " + definition.maximum.toString()
        }
        /** Whether the count is written as `value` or `values`. */
        val unit = if (definition.maximum == 1) " value" else " values"
        return "takes " + range + unit + ", as in " + definition.signature
    }

    // What:     `private fun evaluateMi(node: CallNode, values: List<String>): String` reads one field.
    // Why:      A field outside the template's allowed list is refused, even when the track has a value for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function evaluateMi(node: CallNode, values: string[]): string { ... }
    // ```
    /** Reads a field the template may use, or refuses the mode name. */
    private fun evaluateMi(node: CallNode, values: List<String>): String {
        /** The field name the call asks for. */
        val mode = values[0]
        if (allowed.none { field -> field.mode == mode }) {
            errors.add(TemplateError("mi", "unknown field " + mode, node.start))
            return ABSENT
        }
        return track.fieldText(mode) ?: ABSENT
    }

    // What:     `private fun evaluateTf(node: CallNode, values: List<String>): String` formats a duration.
    // Why:      Missing seconds show nothing, but text that is not a number is a mistake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function evaluateTf(node: CallNode, values: string[]): string { ... }
    // ```
    /** Formats seconds with the given format, or the default `m:ss`. */
    private fun evaluateTf(node: CallNode, values: List<String>): String {
        /** The format text, or the default `m:ss` when the call gives none. */
        val format = if (values.size == 2) values[1] else "m:ss"
        // The format is checked even when the track has no duration, so a typo in it never hides behind an empty field.
        /** The first letter outside apostrophes that is not h, m or s, or null when the format is sound. */
        val unknown: Char? = unknownFormatLetter(format)
        if (unknown != null) {
            errors.add(TemplateError("tf", "unknown format letter " + unknown, node.start))
            return ABSENT
        }
        if (values[0] == ABSENT) {
            return ABSENT
        }
        /** The seconds read from the first argument, or null when it is not a number. */
        val seconds = asNumber(values[0])
        if (seconds == null) {
            errors.add(TemplateError("tf", "needs a number of seconds, not " + values[0], node.start))
            return ABSENT
        }
        return formatSeconds(seconds, format)
    }

    // What:     `private fun evaluateTc(node: CallNode, values: List<String>): String` converts case or cuts text.
    // Why:      Each mode has a fixed rule, and an unknown mode or a bad cut length is named as a mistake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function evaluateTc(node: CallNode, values: string[]): string { ... }
    // ```
    /** Converts text by mode: low, up, cap, or cut to a length. */
    private fun evaluateTc(node: CallNode, values: List<String>): String {
        /** The conversion mode named by the first argument. */
        val mode = values[0]
        /** The text the conversion applies to. */
        val text = values[1]
        if (mode == "low") {
            return text.lowercase()
        }
        if (mode == "up") {
            return text.uppercase()
        }
        if (mode == "cap") {
            return text.take(1).uppercase() + text.drop(1)
        }
        if (mode == "cut") {
            /** The cut length read from the third argument, or null when it is missing or not a number. */
            val length = if (values.size == 3) asNumber(values[2]) else null
            if (length == null || length < 0.0) {
                errors.add(TemplateError("tc", "cut needs a length, as in tc(cut, text, 10)", node.start))
                return ABSENT
            }
            return keepCodePoints(text, floor(length))
        }
        errors.add(TemplateError("tc", "unknown mode " + mode, node.start))
        return ABSENT
    }
}

// What:     `fun evaluateTemplate(...)` shows a template's text for one track, or the mistakes that refuse it.
// Why:      A template that does not parse is not evaluated, so its mistakes are not multiplied by follow-on errors.
//
// In TS you'd write (pseudocode):
// ```ts
// function evaluateTemplate(text: string, track: TemplateTrack, fields = trackFields): TemplateResult { ... }
// ```
/**
 * What a template shows for one track, or the mistakes that refuse it; `fields` are those the template may use.
 * The text is parsed through [ParsedTemplateCache], so many tracks sharing one template parse it once.
 */
fun evaluateTemplate(
    text: String,
    track: TemplateTrack,
    fields: List<TemplateField> = trackTemplateFields,
): TemplateResult = evaluateParsedTemplate(ParsedTemplateCache.parsedFor(text), track, fields)

// What:     `fun evaluateParsedTemplate(...)` evaluates a template that is already split into pieces.
// Why:      Rows share one template, so the caller parses it once and evaluates it per track, like the desktop's
//           `evaluate_parsed_template`.
//
// In TS you'd write (pseudocode):
// ```ts
// function evaluateParsedTemplate(parsed: ParsedTemplate, track: TemplateTrack, fields = trackFields): TemplateResult
// ```
// What:     `fun templateMistakes(...)` lists every mistake of a template for one track: the parse mistakes and the
//           evaluation mistakes of each formula that parsed.
// Why:      Row evaluation stops at the first parse mistake so one slip does not multiply into follow-on errors, but
//           the
//           editor owes the user the whole list: an unknown field in one formula is reported even when another formula
//           lacks a closing bracket. A formula that holds a parse mistake is not evaluated.
//
// In TS you'd write (pseudocode):
// ```ts
// function templateMistakes(text: string, track: TemplateTrack, fields = trackFields): TemplateError[] {
//   const parsed = parseCached(text);
//   const errors = [...parsed.errors];
//   for (const part of parsed.parts) if (part.isFormula && !parsed.errors.some((e) => part.contains(e.at)))
//   evaluate(part);
//   return errors;
// }
// ```
/** Every mistake the template has for [track]: parse mistakes plus the evaluation mistakes of the sound formulas. */
fun templateMistakes(
    text: String,
    track: TemplateTrack,
    fields: List<TemplateField> = trackTemplateFields,
): List<TemplateError> {
    /** The template's pieces and the mistakes found while splitting it. */
    val parsed: ParsedTemplate = ParsedTemplateCache.parsedFor(text)
    /** Mistakes found while evaluating the formulas that parsed cleanly. */
    val evaluationErrors = mutableListOf<TemplateError>()
    /** The evaluator for this track and its allowed fields. */
    val evaluation = TemplateEvaluation(track, fields, evaluationErrors)
    for (part in parsed.parts) {
        if (part is FormulaPart && parsed.errors.none { error -> error.at >= part.start && error.at <= part.end }) {
            evaluation.evaluate(part.tree)
        }
    }
    return parsed.errors + evaluationErrors
}

// What:     `fun unknownFormatLetter(format: String): Char?` finds the first letter of a duration format that is no
//           time
//           letter.
// Why:      Consistency with the desktop (TE5): only h, m and s are time letters, apostrophes keep any other
//           words, and every other character is a separator. A letter such as the i in "m min" used to print as itself,
//           so
//           a typo went unnoticed.
//
// In TS you'd write (pseudocode):
// ```ts
// function unknownFormatLetter(format: string): string | null { /* scan outside apostrophes */ }
// ```
/** Returns the first letter outside apostrophes that is not h, m or s, or null when the format is sound. */
fun unknownFormatLetter(format: String): Char? {
    /** Whether the scan is inside apostrophes, where any text is kept. */
    var kept = false
    for (letter in format) {
        if (letter == '\'') {
            kept = !kept
        } else if (!kept && letter.isLetter() && letter != 'h' && letter != 'm' && letter != 's') {
            return letter
        }
    }
    return null
}

// What:     `fun durationText(seconds: Double): String` writes a duration as `m:ss`.
// Why:      The field list and the examples show a duration the way the rows do, not as a raw number of seconds.
//
// In TS you'd write (pseudocode):
// ```ts
// const durationText = (seconds: number) => formatSeconds(seconds, "m:ss");
// ```
/** Writes [seconds] as minutes and seconds, such as 3:20 for 200. */
fun durationText(seconds: Double): String = formatSeconds(seconds, "m:ss")

/** What an already parsed template shows for one track, or the mistakes that refuse it. */
fun evaluateParsedTemplate(
    parsed: ParsedTemplate,
    track: TemplateTrack,
    fields: List<TemplateField> = trackTemplateFields,
): TemplateResult {
    if (parsed.errors.isNotEmpty()) {
        return TemplateResult.Refused(parsed.errors)
    }
    /** The mistakes found while evaluating the formulas. */
    val errors = mutableListOf<TemplateError>()
    /** The evaluator for this track and its allowed fields. */
    val evaluation = TemplateEvaluation(track, fields, errors)
    /** The shown text built so far. */
    val output = StringBuilder()
    for (part in parsed.parts) {
        when (part) {
            is LiteralPart -> output.append(part.value)
            is FormulaPart -> output.append(evaluation.evaluate(part.tree))
        }
    }
    if (errors.isNotEmpty()) {
        return TemplateResult.Refused(errors.toList())
    }
    return TemplateResult.Shown(output.toString())
}
//endregion

//region Help for the call the caret is inside
// What:     `data class TemplateHelp(...)` is the signature and argument the caret is inside.
// Why:      The editor shows the call's shape and highlights the argument the caret is in.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateHelp = { name: string; signature: string; parameter: string; description: string };
// ```
/** Help for the innermost call around the caret. */
data class TemplateHelp(
    /** The function name. */
    val name: String,
    /** The call shape, such as `tf(seconds, [format])`. */
    val signature: String,
    /** The argument name the caret is in. */
    val parameter: String,
    /** The argument's help sentence for this template. */
    val description: String,
)

// What:     `private fun formulaTreesAround(...)` collects the trees of the formulas that contain the caret.
// Why:      Only formulas whose span holds the caret can contain the call the help should describe.
//
// In TS you'd write (pseudocode):
// ```ts
// const pending = parts.filter(p => p.kind === "formula" && caret > p.start && caret <= p.end - 1).map(p => p.tree);
// ```
/** The formula trees whose span holds the caret, ready for the work stack. */
private fun formulaTreesAround(parts: List<TemplatePart>, caret: Int): ArrayDeque<FormulaNode> {
    /** The work stack of trees still to walk. */
    val pending = ArrayDeque<FormulaNode>()
    for (part in parts) {
        if (part is FormulaPart && caret > part.start && caret <= (if (part.closed) part.end - 1 else part.end)) {
            pending.addLast(part.tree)
        }
    }
    return pending
}

// What:     `private fun callContains(node: CallNode, caret: Int): Boolean` tells whether the caret
//           lies inside a call.
// Why:      An unclosed call reaches the end of its formula, so a missing closing bracket counts as open.
//
// In TS you'd write (pseudocode):
// ```ts
// const inside = caret >= node.open && (node.close === undefined || caret <= node.close);
// ```
/** Tells whether the caret is inside the call's brackets, counting an unclosed call as open to its end. */
private fun callContains(node: CallNode, caret: Int): Boolean {
    /** The closing bracket offset, or null when the call is unclosed. */
    val close = node.close
    return caret >= node.open && (close == null || caret <= close)
}

// What:     `private fun innermostCall(...)` finds the call around the caret that starts last.
// Why:      The call that starts last is the innermost one, because nested calls start after their parents.
//
// In TS you'd write (pseudocode):
// ```ts
// function innermostCall(parts: TemplatePart[], caret: number): CallNode | undefined { ... }
// ```
/** The innermost call around the caret, or null when the caret is outside every call. */
private fun innermostCall(parts: List<TemplatePart>, caret: Int): CallNode? {
    /** The work stack of trees still to walk. */
    val pending = formulaTreesAround(parts, caret)
    /** The innermost call found so far. */
    var found: CallNode? = null
    while (pending.isNotEmpty()) {
        /** The tree node taken from the work stack. */
        val node = pending.removeLast()
        if (node is JoinNode) {
            pending.addLast(node.left)
            pending.addLast(node.right)
        } else if (node is CallNode) {
            pending.addAll(node.parameters)
            /** The innermost call found so far, read before the comparison. */
            val current = found
            if (callContains(node, caret) && (current == null || node.open > current.open)) {
                found = node
            }
        }
    }
    return found
}

// What:     `private fun helpForCall(...)` builds the help for one call and the argument the caret is in.
// Why:      The signature and argument help come from the function table, and `mi` lists this template's fields.
//
// In TS you'd write (pseudocode):
// ```ts
// function helpForCall(call: CallNode, caret: number, fields: Field[]): Help | undefined { ... }
// ```
/** Builds the help for a call, or null when the call names no template function. */
private fun helpForCall(call: CallNode, caret: Int, fields: List<TemplateField>): TemplateHelp? {
    /** The signature and argument help of the called function, or null when the name is unknown. */
    val definition = templateFunctions[call.name] ?: return null
    /** Position of the argument the caret is in, capped at the function's last argument. */
    val index = minOf(call.separators.count { separator -> separator < caret }, definition.parameters.size - 1)
    /** The argument the caret is in. */
    val parameter = definition.parameters[index]
    /** The help sentence; `mi` lists the fields of this template rather than the default list. */
    val description = if (call.name == "mi") {
        "Field: one of " + fields.joinToString(", ") { field -> field.mode } + "."
    } else {
        parameter.description
    }
    return TemplateHelp(
        name = call.name,
        signature = definition.signature,
        parameter = parameter.name,
        description = description,
    )
}

// What:     `fun helpAt(...)` finds the innermost call around the caret and the argument under it.
// Why:      A work stack walks the tree, so the search needs no recursion and its depth stays bounded.
//
// In TS you'd write (pseudocode):
// ```ts
// function helpAt(text: string, caret: number, fields = trackFields): Help | undefined { ... }
// ```
/** The innermost call around the caret and which of its arguments the caret is in, or null outside a call. */
fun helpAt(
    text: String,
    caret: Int,
    fields: List<TemplateField> = trackTemplateFields,
): TemplateHelp? {
    /** The innermost call around the caret, or null when there is none. */
    val innermost = innermostCall(parseTemplate(text).parts, caret) ?: return null
    return helpForCall(innermost, caret, fields)
}
//endregion
