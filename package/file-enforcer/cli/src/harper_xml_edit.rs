//! What:
//!  Entry edits for the Harper policy in the IDE's two settings files:
//! set an option's value in place,
//! append an entry,
//! replace a mismatching subtree.
//! Why:
//!  Existing entries are edited attribute by attribute (an option's `value`),
//! and a nested `mappings` option is replaced only when it differs from the policy,
//! so a comment inside a managed entry survives every run that has nothing to change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function upsertLanguageEntry(xml: string, key: string, config: string, schema: string): string
//! ```

/// Imports the shared descriptions and finders from `crate::harper_xml`.
use crate::harper_xml::{LANGUAGE_CONTAINER, USER_CONTAINER, edit_failure, entry_spec, find_entry, mappings_spec, option_spec, parse};
/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `XmlDocument`,
/// `XmlElementId`,
/// `XmlElementSpec` from `monochromatic_xml_edit`.
use monochromatic_xml_edit::{XmlDocument, XmlElementId, XmlElementSpec};

/// Append an entry to the last `map` element,
/// where the IDE keeps its server entries.
fn append_entry(path: &str, document: &XmlDocument, spec: &XmlElementSpec) -> Result<String, Diagnostic> {
    let maps = document.descendants_named(document.root(), "map");
    let Some(last) = maps.last() else {
        let message = format!("{path}: the settings file has no map element to hold a server entry");
        return Err(Diagnostic::new("harper-xml-invalid", &message).with_path(path));
    };
    let edited = document.append_child(*last, spec).map_err(|error| return edit_failure(path, &error))?;
    return Ok(edited.source().to_string());
}

/// Replace a whole entry with its description.
fn replace_entry(path: &str, document: &XmlDocument, entry: XmlElementId, spec: &XmlElementSpec) -> Result<String, Diagnostic> {
    let edited = document.replace_element(entry, spec).map_err(|error| return edit_failure(path, &error))?;
    return Ok(edited.source().to_string());
}

/// What:
///  Make one option of an entry hold a value:
/// change the `value` attribute in place,
/// or append the option when the entry lacks it.
/// Why:
///  An attribute edit keeps comments and layout around the option.
///  With `overwrite` false an existing option is left alone,
/// for options the policy only seeds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureOption(xml: string, key: string, container: string, name: string, value: string, overwrite: boolean): string
/// ```
fn ensure_option(path: &str, text: &str, key: &str, container: &str, name: &str, value: &str, overwrite: bool) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let missing = || return edit_failure(path, &format!("the entry {key} has no {container} element"));
    let entry = find_entry(&document, key).ok_or_else(missing)?;
    let holder = *document.descendants_named(entry, container).first().ok_or_else(missing)?;
    match document.find_child(holder, "option", "name", name) {
        Some(option) => {
            if !overwrite || document.attribute(option, "value") == Some(value) {
                return Ok(text.to_string());
            }
            let edited = document.set_attribute(option, "value", value).map_err(|error| return edit_failure(path, &error))?;
            return Ok(edited.source().to_string());
        }
        None => {
            let edited = document.append_child(holder, &option_spec(name, value)).map_err(|error| return edit_failure(path, &error))?;
            return Ok(edited.source().to_string());
        }
    }
}

/// What:
///  Write the policy's configuration and schema into the base server's entry,
/// or into a scoped server's entry,
/// in `LanguageServersSettings.xml`.
/// Why:
///  An absent entry is appended;
/// an existing one has its two JSON options set and its two seed options created when missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upsertLanguageEntry(path: string, xml: string, key: string, config: string, schema: string): string
/// ```
pub fn upsert_language_entry(path: &str, text: &str, key: &str, config: &str, schema: &str) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let Some(entry) = find_entry(&document, key) else {
        let options = vec![
            option_spec("configurationContent", config),
            option_spec("configurationSchemaContent", schema),
            option_spec("experimentalContent", "{}"),
            option_spec("initializationOptionsContent", "{}"),
        ];
        return append_entry(path, &document, &entry_spec(key, LANGUAGE_CONTAINER, options));
    };
    if document.descendants_named(entry, LANGUAGE_CONTAINER).is_empty() {
        let options = vec![option_spec("configurationContent", config), option_spec("configurationSchemaContent", schema)];
        return replace_entry(path, &document, entry, &entry_spec(key, LANGUAGE_CONTAINER, options));
    }
    let with_config = ensure_option(path, text, key, LANGUAGE_CONTAINER, "configurationContent", config, true)?;
    let with_schema = ensure_option(path, &with_config, key, LANGUAGE_CONTAINER, "configurationSchemaContent", schema, true)?;
    let with_experimental = ensure_option(path, &with_schema, key, LANGUAGE_CONTAINER, "experimentalContent", "{}", false)?;
    return ensure_option(path, &with_experimental, key, LANGUAGE_CONTAINER, "initializationOptionsContent", "{}", false);
}

/// What:
///  The file-name patterns a `mappings` option binds to its language id,
/// when it has the shape the policy writes.
/// Why:
///  Comparing decoded values decides whether the nested structure needs replacing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentMapping(doc: XmlDocument, mappings: ElementId): { languageId: string; fileNames: string[] } | undefined
/// ```
fn current_mapping(document: &XmlDocument, mappings: XmlElementId) -> Option<(String, Vec<String>)> {
    let settings = document.descendants_named(mappings, "ServerMappingSettings");
    if settings.len() != 1 {
        return None;
    }
    let language_id = document.attribute(settings[0], "languageId")?.to_string();
    let patterns = document.descendants_named(settings[0], "option").into_iter().find(|option| return document.attribute(*option, "name") == Some("patterns"))?;
    let mut names: Vec<String> = Vec::new();
    for child in document.children_named(patterns, "option") {
        names.push(document.attribute(child, "value")?.to_string());
    }
    return Some((language_id, names));
}

/// What:
///  Make a scoped server's `mappings` option bind the given files to the given language id.
/// Why:
///  A matching option is left byte for byte;
/// a different or absent one is replaced or appended as a whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureMappings(path: string, xml: string, key: string, languageId: string, fileNames: string[]): string
/// ```
fn ensure_mappings(path: &str, text: &str, key: &str, language_id: &str, file_names: &[String]) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let missing = || return edit_failure(path, &format!("the entry {key} has no {USER_CONTAINER} element"));
    let entry = find_entry(&document, key).ok_or_else(missing)?;
    let holder = *document.descendants_named(entry, USER_CONTAINER).first().ok_or_else(missing)?;
    let wanted = mappings_spec(language_id, file_names);
    let Some(existing) = document.find_child(holder, "option", "name", "mappings") else {
        let edited = document.append_child(holder, &wanted).map_err(|error| return edit_failure(path, &error))?;
        return Ok(edited.source().to_string());
    };
    if current_mapping(&document, existing).is_some_and(|(id, names)| return id == language_id && names == file_names) {
        return Ok(text.to_string());
    }
    let edited = document.replace_element(existing, &wanted).map_err(|error| return edit_failure(path, &error))?;
    return Ok(edited.source().to_string());
}

/// What:
///  What a scoped server entry in `UserDefinedLanguageServerSettings.xml` must say.
/// Why:
///  One value carries the many facts the entry's options hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScopedUserEntry = { key: string; name: string; languageId: string; fileNames: string[]; copied: [string, string][] }
/// ```
pub struct ScopedUserEntry<'a> {
    /// Server id used as the entry key and the `serverId` option.
    pub key: &'a str,
    /// Display name for the `serverName` option.
    pub name: &'a str,
    /// Language id the files map to.
    pub language_id: &'a str,
    /// File names mapped to the server.
    pub file_names: &'a [String],
    /// Options copied from the base server,
    /// in output order.
    pub copied: &'a [(String, String)],
}

/// What:
///  Write a scoped server entry into `UserDefinedLanguageServerSettings.xml`.
/// Why:
///  An absent entry is appended whole;
/// an existing one has each option set in place and its mapping compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upsertUserEntry(path: string, xml: string, entry: ScopedUserEntry): string
/// ```
pub fn upsert_user_entry(path: &str, text: &str, entry: &ScopedUserEntry) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let mut options: Vec<(String, String)> = entry.copied.to_vec();
    options.push(("serverId".to_string(), entry.key.to_string()));
    options.push(("serverName".to_string(), entry.name.to_string()));
    let Some(existing) = find_entry(&document, entry.key) else {
        let mut specs: Vec<XmlElementSpec> = options.iter().map(|(name, value)| return option_spec(name, value)).collect();
        specs.push(mappings_spec(entry.language_id, entry.file_names));
        return append_entry(path, &document, &entry_spec(entry.key, USER_CONTAINER, specs));
    };
    if document.descendants_named(existing, USER_CONTAINER).is_empty() {
        let mut specs: Vec<XmlElementSpec> = options.iter().map(|(name, value)| return option_spec(name, value)).collect();
        specs.push(mappings_spec(entry.language_id, entry.file_names));
        return replace_entry(path, &document, existing, &entry_spec(entry.key, USER_CONTAINER, specs));
    }
    let mut current = text.to_string();
    for (name, value) in &options {
        current = ensure_option(path, &current, entry.key, USER_CONTAINER, name, value, true)?;
    }
    return ensure_mappings(path, &current, entry.key, entry.language_id, entry.file_names);
}
