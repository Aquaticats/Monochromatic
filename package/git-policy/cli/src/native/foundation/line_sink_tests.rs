//! Controls for the line sink: a function of one text is a sink in tests, and lines reach it in order.

use super::*;

/// A function-like value of one text is a sink in tests, so a test that only counts or collects lines writes no struct.
///  Shipped code writes a named struct instead, so no closure is passed through here.
impl<Say: FnMut(&str)> LineSink for Say {
    fn say(&mut self, text: &str) {
        self(text);
    }
}

/// A line said through the trait reaches the function that stands for the sink, in the order it was said.
#[test]
fn a_line_said_through_the_trait_reaches_the_sink_in_order() {
    let mut lines: Vec<String> = Vec::new();
    let mut collect = |text: &str| { lines.push(text.to_string()); };
    let sink: &mut dyn LineSink = &mut collect;
    sink.say("first");
    sink.say("second");
    assert_eq!(lines, vec![String::from("first"), String::from("second")]);
}

/// A list of text keeps the lines it is told, in order.
#[test]
fn a_list_keeps_the_lines_it_is_told_in_order() {
    let mut lines: Vec<String> = Vec::new();
    lines.say("first");
    lines.say("second");
    assert_eq!(lines, vec![String::from("first"), String::from("second")]);
}
