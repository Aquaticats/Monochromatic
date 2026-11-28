//! Controls for the stream that hands whole lines to the thread which writes them:
//!  every line arrives once and whole,
//!  the order the bytes arrived in is the order the stream holds,
//!  a line still being built is said at its newline or at the end of the stream,
//!  a stretch with no end is handed over at its block,
//!  and a write answers an error when the line it completed did not reach the stream.
//! Most controls read a collector of their own rather than this process's standard error,
//! so they change no state another control of the run shares.
//! The two that hand this process's queue a stream of their own run their bodies in a child
//! (`run_the_body_in_a_child_of_its_own`),
//! because that queue and its thread are one for the whole process.

use super::{
    LONGEST_STRETCH_WITHOUT_AN_END,
    StandardErrorOffTheCallingThread,
};
use crate::lines_written_by_a_thread::HandedOver;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// What:
///  One collector of the lines a stream handed over,
///  and the stream that hands them there.
/// Why:
///  A control reads exactly the lines its own stream said,
///  in the order they arrived,
///  with nothing of this process's own standard error involved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = []; const stream = new StandardErrorOffTheCallingThread({ hand: (line) => said.push(line) });
/// ```
fn a_stream_that_keeps_what_it_said() -> (
    StandardErrorOffTheCallingThread<impl FnMut(&[u8]) -> HandedOver>,
    Rc<RefCell<Vec<Vec<u8>>>>,
) {
    let said: Rc<RefCell<Vec<Vec<u8>>>> = Rc::new(RefCell::new(Vec::new()));
    let kept: Rc<RefCell<Vec<Vec<u8>>>> = Rc::clone(&said);
    let stream: StandardErrorOffTheCallingThread<_> =
        StandardErrorOffTheCallingThread::handing_to(move |line: &[u8]| {
            kept.borrow_mut().push(line.to_vec());
            return HandedOver::Written;
        });
    return (stream, said);
}

/// What:
///  What the collector holds now,
///  as text.
/// Why:
///   A control compares the lines it expected with the lines the stream handed over,
///  and one joined text names every difference at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = (parts) => parts.map((bytes) => new TextDecoder().decode(bytes)).join('')
/// ```
fn as_text(parts: &Rc<RefCell<Vec<Vec<u8>>>>) -> String {
    let joined: Vec<u8> = parts
        .borrow()
        .iter()
        .flat_map(|line| {
            return line.clone();
        })
        .collect();
    return String::from_utf8_lossy(joined.as_slice()).into_owned();
}

/// What:
///  Two lines said in one write arrive as two lines,
///  and the line the write left unfinished stays here.
/// Why:
///  The queue takes one line whole,
///  so a write that carries two of them must not put them in the queue as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stream.write('one\ntwo\nthree'); expect(said).toEqual(['one\n', 'two\n']); expect(buffered).toBe('three')
/// ```
#[test]
fn whole_lines_are_handed_over_one_each_and_the_unfinished_line_stays() {
    let (mut stream, said) = a_stream_that_keeps_what_it_said();
    stream
        .write_all(b"one\ntwo\nthree")
        .expect("the stream takes the bytes");
    assert_eq!(
        said.borrow().len(),
        2,
        "two whole lines were said: {:?}",
        as_text(&said)
    );
    assert_eq!(
        as_text(&said),
        "one\ntwo\n",
        "the lines arrived whole and in order"
    );
    assert_eq!(
        stream.buffered,
        b"three".to_vec(),
        "the line with no end yet is kept here"
    );
}

/// What:
///  One line a formatting call hands over in three pieces is said once,
///  when its newline arrives.
/// Why:
///  `write!` and `writeln!` hand over their words,
///  their values and their newline as separate
///  writes,
///  and a reader must see one line rather than three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stream.write('the '); stream.write('whole '); expect(said).toEqual([]); stream.write('line\n'); expect(said).toEqual(['the whole line\n'])
/// ```
#[test]
fn a_line_said_in_pieces_is_handed_over_once_at_its_newline() {
    let (mut stream, said) = a_stream_that_keeps_what_it_said();
    stream.write_all(b"the ").expect("the first piece");
    stream.write_all(b"whole ").expect("the second piece");
    assert!(
        said.borrow().is_empty(),
        "no line is said before its newline arrives: {:?}",
        as_text(&said)
    );
    stream.write_all(b"line\n").expect("the last piece");
    assert_eq!(
        as_text(&said),
        "the whole line\n",
        "the pieces arrived as the one line they spell"
    );
    assert!(
        stream.buffered.is_empty(),
        "the line was said whole, so nothing is left here"
    );
}

/// What:
///  A flush says the line no newline ended,
///  and says nothing when there is none.
/// Why:
///  A note that ends without a newline is still a note the person must read,
///  and a flush after it must not say an empty line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stream.write('no end'); stream.flush(); expect(said).toEqual(['no end']); stream.flush(); expect(said.length).toBe(1)
/// ```
#[test]
fn a_flush_says_the_line_no_newline_ended_and_nothing_when_there_is_none() {
    let (mut stream, said) = a_stream_that_keeps_what_it_said();
    stream
        .write_all(b"no end")
        .expect("the line with no newline");
    stream.flush().expect("the stream flushes");
    assert_eq!(as_text(&said), "no end", "the line was said at the flush");
    stream.flush().expect("a second flush");
    assert_eq!(
        said.borrow().len(),
        1,
        "a flush with nothing to say says nothing"
    );
}

/// What:
///  A stretch with no newline is handed over once it reaches the block,
///  so a program below the commit cannot grow this buffer without end.
/// Why:
///  Git's own standard error can hold bytes with no newline at all,
///  and the commit's memory is not the place to keep an endless line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stream.write('x'.repeat(LONGEST_STRETCH_WITHOUT_AN_END - 1)); expect(said).toEqual([]); stream.write('x'); expect(said.length).toBe(1)
/// ```
#[test]
fn a_stretch_with_no_end_is_handed_over_at_the_block() {
    let (mut stream, said) = a_stream_that_keeps_what_it_said();
    stream
        .write_all(vec![b'x'; LONGEST_STRETCH_WITHOUT_AN_END - 1].as_slice())
        .expect("the bytes before the block");
    assert!(
        said.borrow().is_empty(),
        "a stretch short of the block stays here: {:?}",
        said.borrow().len()
    );
    stream
        .write_all(b"x")
        .expect("the byte that reaches the block");
    assert_eq!(
        said.borrow().len(),
        1,
        "the stretch was handed over once it reached the block"
    );
    assert_eq!(
        said.borrow()[0].len(),
        LONGEST_STRETCH_WITHOUT_AN_END,
        "every byte of the stretch went over"
    );
    assert!(
        stream.buffered.is_empty(),
        "the stretch is no longer held here"
    );
}

/// What:
///  A line the hand took is answered as a write that succeeded.
/// Why:
///  The answer is the whole of what a caller that must not lose its line reads,
///  and it comes from the thread that wrote it rather than from the write.
#[test]
fn a_line_the_stream_took_is_answered_as_a_write_that_succeeded() {
    let mut stream = StandardErrorOffTheCallingThread::handing_to(|_line: &[u8]| {
        return HandedOver::Written;
    });
    let written: std::io::Result<()> = writeln!(stream, "a line the person reads");
    assert!(written.is_ok(), "a line the stream took is no error");
}

/// What:
///  A line the hand did not take is an error of the write that completed it.
/// Why:
///  A caller that must not lose what its line says keeps its record on this error
///  (`Notice::show`),
///  and the write of the line is the only place the answer is known.
#[test]
fn a_line_the_stream_did_not_take_is_an_error_of_the_write_that_completed_it() {
    let mut stream = StandardErrorOffTheCallingThread::handing_to(|_line: &[u8]| {
        return HandedOver::Unwritten;
    });
    let refused: std::io::Result<()> = writeln!(stream, "a line nobody can read");
    assert_eq!(
        refused.as_ref().err().map(std::io::Error::kind),
        Some(std::io::ErrorKind::BrokenPipe),
        "a line the stream did not take is refused: {refused:?}"
    );
}

/// What:
///  A line whose end never arrived is answered at the flush that hands it over.
/// Why:
///  A line is handed over at its newline or at the end of the stream,
///  so the answer belongs to whichever call hands it over,
///  and a caller that flushes reads the same error there.
#[test]
fn a_line_the_stream_did_not_take_is_an_error_of_the_flush_that_handed_it_over() {
    let mut stream = StandardErrorOffTheCallingThread::handing_to(|_line: &[u8]| {
        return HandedOver::Unwritten;
    });
    let kept: std::io::Result<()> = write!(stream, "a line with no end yet");
    assert!(
        kept.is_ok(),
        "no line left the stream yet, so there is nothing to refuse"
    );
    let refused: std::io::Result<()> = stream.flush();
    assert!(
        refused.is_err(),
        "the flush hands the line over, so the flush reads the refusal"
    );
}

/// What:
///  A line this process's own queue could not put in its stream is an error of the stream,
///  and the line is answered as one that never reached the person.
/// Why:
///  This is the path every note of a command takes,
///  and the answer is what `Notice::show` reads to keep its record.
#[test]
fn a_line_the_thread_of_this_process_refused_is_an_error_of_the_stream() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_the_thread_of_this_process_refused_is_an_error_of_the_stream",
        || {
            crate::diagnostic_lines_off_the_calling_thread::give_the_thread(Box::new(
                crate::test_support::AStreamThatRefusesEveryByte,
            ));
            let mut stream = StandardErrorOffTheCallingThread::new();
            let refused: std::io::Result<()> = writeln!(stream, "a line nobody can read");
            assert!(
                refused.is_err(),
                "the thread that writes standard error says the line did not reach the person"
            );
        },
    );
}

/// What:
///  A line this process's own queue put in its stream is no error of the stream.
/// Why:
///  The check above is only worth its answer if a line that reaches the stream is told apart
///  from one that does not.
#[test]
fn a_line_the_thread_of_this_process_took_is_no_error_of_the_stream() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_the_thread_of_this_process_took_is_no_error_of_the_stream",
        || {
            crate::diagnostic_lines_off_the_calling_thread::give_the_thread(Box::new(
                std::io::sink(),
            ));
            let mut stream = StandardErrorOffTheCallingThread::new();
            let written: std::io::Result<()> = writeln!(stream, "a line the person reads");
            assert!(written.is_ok(), "the stream took the line: {written:?}");
        },
    );
}
