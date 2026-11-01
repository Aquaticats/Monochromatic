// What:     Unit tests for `folder_rail.rs`, pulled in by
//           `#[cfg(test)] #[path = "folder_rail_tests.rs"] mod tests;` at the bottom of
//           `folder_rail.rs`. Mirrors every `@Test` of `FolderRailTest.kt`. The expected values
//           are the ones the Kotlin tests assert, taken from running `design/candidates/buckets.js`
//           on the same inputs. Reaches the module items via `use super::*` because this file is
//           the `tests` CHILD of folder_rail.
// Why:      Keep the tests beside the code without inflating `folder_rail.rs` or its max-lines budget
//           (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need `rail_for`, `RailSection`, `RailCell`, and `RailScript`.
use super::*;

// What:     `fn single<T>(items: Vec<T>) -> T` test helper: returns the only item of a collection,
//           failing the test when there is not exactly one.
// Why:      Mirrors the Kotlin `single()` calls, so a wrong count fails loudly.
fn single<T>(mut items: Vec<T>) -> T {
    assert_eq!(items.len(), 1, "expected exactly one item");
    return items.remove(0);
}

// What:     `fn single_section<S: AsRef<str>>(names: &[S]) -> RailSection` test helper: returns the
//           only section a rail built from these names produces.
// Why:      Most cases use one writing system, so this states that expectation.
fn single_section<S: AsRef<str>>(names: &[S]) -> RailSection {
    return single(rail_for(names));
}

// What:     `fn cell_label_of(name: &str) -> String` test helper: returns the cell label that a
//           single folder name lands in.
// Why:      A one-name library has exactly one section and one cell, so the label reads directly.
fn cell_label_of(name: &str) -> String {
    return single(single_section(&[name]).cells).label;
}

// What:     `fn labels_of(cells: &[RailCell]) -> Vec<&str>` test helper: returns the cell labels in order.
// Why:      Most assertions compare label sequences.
fn labels_of(cells: &[RailCell]) -> Vec<&str> {
    return cells.iter().map(|cell| return cell.label.as_str()).collect();
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A Latin-only library gives one section whose cells run A to Z regardless of input order.
#[test]
fn latin_only_gives_one_section_with_ato_z_cells_in_order() {
    let names: Vec<String> = ('A'..='Z').rev().map(|letter| return format!("{letter}x")).collect();
    let section = single_section(&names);
    assert_eq!(section.script, RailScript::Latin);
    assert_eq!(section.label, "A");
    let letters: Vec<String> = ('A'..='Z').map(|letter| return letter.to_string()).collect();
    assert_eq!(labels_of(&section.cells), letters);
    assert_eq!(section.cells[0].names, vec!["Ax"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A Japanese-only library opens on the kana cell and places kanji last.
#[test]
fn japanese_only_opens_on_kana_cell() {
    let section = single_section(&["漢字", "あいさつ", "カラオケ"]);
    assert_eq!(section.script, RailScript::Japanese);
    assert_eq!(section.label, "あ");
    assert_eq!(labels_of(&section.cells), vec!["あ", "か", "漢"]);
    assert_eq!(section.cells[0].names, vec!["あいさつ"]);
    assert_eq!(section.cells[1].names, vec!["カラオケ"]);
    assert_eq!(section.cells[2].names, vec!["漢字"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The Latin section comes first in a library that also has Japanese names.
#[test]
fn mixed_latin_and_japanese_shows_latin_section_first() {
    let sections = rail_for(&["あいさつ", "Camellia"]);
    let scripts: Vec<RailScript> = sections.iter().map(|section| return section.script).collect();
    assert_eq!(scripts, vec![RailScript::Latin, RailScript::Japanese]);
    let labels: Vec<&str> = sections.iter().map(|section| return section.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "あ"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Small kana, voiced kana, and the prolonged sound mark map to their kana rows.
#[test]
fn kana_rows_include_small_and_voiced_forms() {
    let expected: [(&str, &str); 14] = [
        ("ぁい", "あ"),
        ("がっこう", "か"),
        ("ギター", "か"),
        ("ざっし", "さ"),
        ("だいこん", "た"),
        ("っ", "た"),
        ("ぱん", "は"),
        ("ばん", "は"),
        ("ゃく", "や"),
        ("ー", "わ"),
        ("ン", "わ"),
        ("らくだ", "ら"),
        ("まど", "ま"),
        ("なつ", "な"),
    ];
    for (name, label) in expected {
        assert_eq!(cell_label_of(name), label);
    }
}

// What:     `#[test]` marks the next function as a test case.
// Why:      One name per kana row and one kanji name produce cells in gojuon order with kanji last.
#[test]
fn kana_cells_sort_in_gojuon_order_before_kanji() {
    let names = ["わたし", "らくだ", "やさい", "まど", "なつ", "はる", "さくら", "たぬき", "かさ", "あさ", "漢字"];
    let section = single_section(&names);
    assert_eq!(
        labels_of(&section.cells),
        vec!["あ", "か", "さ", "た", "な", "は", "ま", "や", "ら", "わ", "漢"],
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Names that start with an ideograph land in the 漢 cell.
#[test]
fn kanji_falls_under_kanji_cell() {
    assert_eq!(cell_label_of("漢字"), "漢");
    assert_eq!(cell_label_of("中国"), "漢");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Unlisted katakana and halfwidth katakana follow the JS model into the kanji cell.
#[test]
fn katakana_outside_kana_rows_falls_to_kanji_cell() {
    let section = single_section(&["ヴィオラ", "ｱｲｳ"]);
    assert_eq!(labels_of(&section.cells), vec!["漢"]);
    assert_eq!(section.cells[0].names, vec!["ヴィオラ", "ｱｲｳ"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The Cyrillic section groups by uppercased first letter, with Ё sorted after Б.
#[test]
fn cyrillic_cells_sort_by_uppercased_letter() {
    let section = single_section(&["Бор", "Ёж", "Аня", "ёлка"]);
    assert_eq!(section.script, RailScript::Cyrillic);
    assert_eq!(section.label, "А");
    assert_eq!(labels_of(&section.cells), vec!["А", "Б", "Ё"]);
    assert_eq!(section.cells[2].names, vec!["Ёж", "ёлка"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The Greek section groups names by uppercased first letter.
#[test]
fn greek_cells_sort_by_uppercased_letter() {
    let section = single_section(&["Γάμμα", "Αλφα", "αβ"]);
    assert_eq!(section.script, RailScript::Greek);
    assert_eq!(section.label, "Α");
    assert_eq!(labels_of(&section.cells), vec!["Α", "Γ"]);
    assert_eq!(section.cells[0].names, vec!["Αλφα", "αβ"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Each Hangul first syllable gets its own cell, not one cell per block.
#[test]
fn hangul_cells_key_one_per_first_syllable() {
    let section = single_section(&["한국", "가나", "한두"]);
    assert_eq!(section.script, RailScript::Hangul);
    assert_eq!(section.label, "가");
    assert_eq!(labels_of(&section.cells), vec!["가", "한"]);
    assert_eq!(section.cells[1].names, vec!["한국", "한두"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Digit-led and symbol-led names land in the `#` cell of the Other section.
#[test]
fn digits_and_symbols_fall_to_other_hash_cell() {
    let section = single_section(&["123", "!wow"]);
    assert_eq!(section.script, RailScript::Other);
    assert_eq!(section.label, "#");
    assert_eq!(labels_of(&section.cells), vec!["#"]);
    assert_eq!(section.cells[0].names, vec!["123", "!wow"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The empty name lands in the `#` cell of the Other section without a crash.
#[test]
fn empty_name_falls_to_other_hash_cell() {
    let section = single_section(&[""]);
    assert_eq!(section.script, RailScript::Other);
    assert_eq!(section.cells[0].names, vec![""]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty list of names yields an empty rail, so the picker shows no rail.
#[test]
fn empty_library_gives_no_sections() {
    assert_eq!(rail_for::<&str>(&[]), Vec::<RailSection>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Lowercase and uppercase first letters share one cell and keep their input order.
#[test]
fn case_insensitive_grouping_puts_capsule_and_camellia_under_c() {
    let section = single_section(&["capsule", "Camellia"]);
    assert_eq!(labels_of(&section.cells), vec!["C"]);
    assert_eq!(section.cells[0].names, vec!["capsule", "Camellia"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An accent after the first letter does not move the name out of the C cell.
#[test]
fn accented_latin_stays_under_its_base_letter() {
    assert_eq!(cell_label_of("Cö shu Nie"), "C");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An accented leading letter gets its own cell, and tied labels keep first-seen order.
#[test]
fn accented_leading_letter_keeps_its_own_cell_as_the_js_model_does() {
    let section = single_section(&["Örjan", "Oscar"]);
    assert_eq!(labels_of(&section.cells), vec!["Ö", "O"]);
    assert_eq!(section.cells[0].names, vec!["Örjan"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A leading `ß` produces the two-letter `SS` cell, as the JS model does.
#[test]
fn eszett_uppercases_to_ss_like_the_js_model() {
    assert_eq!(cell_label_of("ßx"), "SS");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Every writing system appears once, in the fixed rail order.
#[test]
fn sections_follow_latin_japanese_cyrillic_greek_hangul_other() {
    let sections = rail_for(&["123", "한국", "Αλφα", "Бор", "中国", "あ", "Zed"]);
    let scripts: Vec<RailScript> = sections.iter().map(|section| return section.script).collect();
    assert_eq!(
        scripts,
        vec![
            RailScript::Latin,
            RailScript::Japanese,
            RailScript::Cyrillic,
            RailScript::Greek,
            RailScript::Hangul,
            RailScript::Other,
        ],
    );
    let labels: Vec<&str> = sections.iter().map(|section| return section.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "あ", "А", "Α", "가", "#"]);
}
