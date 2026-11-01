// Row templates are evaluated once per visible row on every composition, so the template text is parsed once and the
// parsed form is reused, the way the desktop's `evaluate_parsed_template` does.

// What:     `package ...core` places the cache beside the template language it serves.
// Why:      The cache is pure logic, so unit tests reach it without Android.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `private const val PARSED_TEMPLATE_CAPACITY: Int = 16` is how many distinct template texts stay parsed.
// Why:      The app has two stored templates, but the editor sends a new text on every keystroke; a small bound keeps
//           the
//           typing history from piling up while both stored templates stay resident.
//
// In TS you'd write (pseudocode):
// ```ts
// const PARSED_TEMPLATE_CAPACITY = 16;
// ```
/** Number of distinct template texts whose parsed form is kept. */
private const val PARSED_TEMPLATE_CAPACITY: Int = 16

// What:     `private const val LOAD_FACTOR: Float = 0.75f` is the standard hash-table load factor, passed only because
//           the access-ordered `LinkedHashMap` constructor wants it.
// Why:      Access order is what turns `LinkedHashMap` into a least-recently-used list.
//
// In TS you'd write (pseudocode):
// ```ts
// // A Map keeps insertion order; delete and re-set an entry to mark it recently used.
// ```
/** Load factor handed to the access-ordered map. */
private const val LOAD_FACTOR: Float = 0.75f

// What:     `object ParsedTemplateCache` is a Kotlin singleton holding the parsed form of recently used template texts.
// Why:      Parsing tokenizes and builds a syntax tree for the whole text; a screen of rows would repeat that work for
//           every row on every composition. One parse per distinct text removes the repetition (the ANR of S2).
//
// In TS you'd write (pseudocode):
// ```ts
// const cache = new Map<string, ParsedTemplate>();
// export function parsedTemplateFor(text: string): ParsedTemplate {
//   let parsed = cache.get(text);
//   if (!parsed) { parsed = parseTemplate(text); cache.set(text, parsed); evictOldest(); }
//   return parsed;
// }
// ```
/** Keeps the parsed form of the most recently used template texts. */
object ParsedTemplateCache {
    /** Parsed templates by text, least recently used first; guarded by its own monitor. */
    private val entries: LinkedHashMap<String, ParsedTemplate> =
        object : LinkedHashMap<String, ParsedTemplate>(PARSED_TEMPLATE_CAPACITY, LOAD_FACTOR, true) {
            /** Drops the least recently used text once the capacity is exceeded. */
            override fun removeEldestEntry(eldest: MutableMap.MutableEntry<String, ParsedTemplate>?): Boolean =
                size > PARSED_TEMPLATE_CAPACITY
        }

    /** Number of times a text was parsed because the cache did not hold it; read by tests. */
    @Volatile
    var parseCount: Int = 0
        private set

    /**
     * Returns the parsed form of [text], parsing it only when it is not already held.
     *
     * @param text template source exactly as typed or stored, because two texts that differ at all parse separately
     * @return the shared parse result, which is immutable and safe to reuse across rows and threads
     */
    fun parsedFor(text: String): ParsedTemplate {
        synchronized(entries) {
            /** Parse result already held for this text, if any. */
            val held: ParsedTemplate? = entries[text]
            if (held != null) {
                return held
            }
            /** Fresh parse of the text, which is stored for the next call. */
            val parsed: ParsedTemplate = parseTemplate(text)
            parseCount += 1
            entries[text] = parsed
            return parsed
        }
    }
}
