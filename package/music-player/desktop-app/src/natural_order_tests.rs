// What:     Unit tests for `natural_order.rs`, pulled in by `#[cfg(test)] #[path = "natural_order_tests.rs"] mod tests;`.
// Why:      The walk order of every library depends on this comparator, so each rule gets a case, and the order must be total.

use super::*;

/// What: `sorted` sorts owned copies of `names` with `natural_cmp`.
/// Why: the tests read the whole order, as the folder walk does.
fn sorted(names: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = names.iter().map(|name| return (*name).to_string()).collect();
    owned.sort_by(|left, right| return natural_cmp(left, right));
    return owned;
}

#[test]
fn a_number_inside_a_name_counts_by_value() {
    assert_eq!(
        sorted(&["Loose 11.wav", "Loose 2.wav", "Loose 1.wav", "Loose 10.wav"]),
        vec!["Loose 1.wav", "Loose 2.wav", "Loose 10.wav", "Loose 11.wav"]
    );
}

#[test]
fn letters_compare_without_regard_to_case() {
    assert_eq!(sorted(&["beta", "Alpha", "alpha2", "Zed"]), vec!["Alpha", "alpha2", "beta", "Zed"]);
}

#[test]
fn a_number_longer_than_any_integer_type_still_compares() {
    let small: String = format!("t{}", "9".repeat(60));
    let large: String = format!("t1{}", "0".repeat(60));
    assert_eq!(natural_cmp(&small, &large), Ordering::Less);
}

#[test]
fn leading_zeros_do_not_change_the_value_but_the_bytes_break_the_tie() {
    assert_eq!(natural_cmp("a02", "a2"), Ordering::Less);
    assert_eq!(natural_cmp("a2", "a02"), Ordering::Greater);
    assert_eq!(natural_cmp("a02", "a3"), Ordering::Less);
}

#[test]
fn a_prefix_comes_before_a_longer_name() {
    assert_eq!(natural_cmp("Track", "Track 1"), Ordering::Less);
    assert_eq!(natural_cmp("Track 1", "Track"), Ordering::Greater);
}

#[test]
fn equal_names_are_equal_and_names_differing_only_in_case_are_not() {
    assert_eq!(natural_cmp("same", "same"), Ordering::Equal);
    assert_ne!(natural_cmp("Same", "same"), Ordering::Equal);
}

#[test]
fn digits_come_before_letters_and_non_latin_names_keep_a_stable_place() {
    assert_eq!(sorted(&["b", "9", "かめりあ 2", "かめりあ 10", "a"]), vec!["9", "a", "b", "かめりあ 2", "かめりあ 10"]);
}

#[test]
fn the_order_is_total_across_mixed_names() {
    let names: Vec<&str> = vec!["a", "A", "a1", "a01", "a2", "a10", "A10", "b", "1", "01", "", "a 1", "a-1", "ß", "SS"];
    for left in &names {
        for right in &names {
            let forward: Ordering = natural_cmp(left, right);
            assert_eq!(forward, natural_cmp(right, left).reverse(), "{left:?} against {right:?} is not antisymmetric");
            for third in &names {
                if forward != Ordering::Greater && natural_cmp(right, third) != Ordering::Greater {
                    assert_ne!(natural_cmp(left, third), Ordering::Greater, "{left:?} {right:?} {third:?} is not transitive");
                }
            }
        }
    }
}
