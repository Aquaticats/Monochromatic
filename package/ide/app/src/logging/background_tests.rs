//! The background writer against outputs the tests control:
//!  one that blocks until a gate opens,
//! one that refuses a write,
//!  and one that accepts everything.

use super::{Loss, background_with, report};
use std::{
    io::{self, Write},
    sync::{Arc, Condvar, Mutex, atomic::Ordering},
    thread,
    time::{Duration, Instant},
};
use tracing_subscriber::fmt::MakeWriter;

/// An output that blocks every write until its gate is open,
///  then keeps the bytes.
#[derive(Clone)]
struct Gated {
    open: Arc<(Mutex<bool>, Condvar)>,
    written: Arc<Mutex<Vec<u8>>>,
}

impl Gated {
    fn closed() -> Self {
        return Self {
            open: Arc::new((Mutex::new(false), Condvar::new())),
            written: Arc::new(Mutex::new(Vec::new())),
        };
    }

    fn open(&self) {
        let (flag, changed) = &*self.open;
        *flag.lock().expect("gate") = true;
        changed.notify_all();
    }

    /// Open the gate from another thread after `delay`.
    fn open_after(&self, delay: Duration) {
        let gate = self.clone();
        thread::spawn(move || {
            thread::sleep(delay);
            gate.open();
        });
    }

    fn text(&self) -> String {
        return String::from_utf8(self.written.lock().expect("written").clone()).expect("text");
    }
}

impl Write for Gated {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let (flag, changed) = &*self.open;
        let mut open = flag.lock().expect("gate");
        while !*open {
            open = changed.wait(open).expect("gate");
        }
        self.written
            .lock()
            .expect("written")
            .extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
    }
}

/// An output that refuses its first write and keeps the rest.
struct RefusesFirst {
    refused: bool,
    written: Arc<Mutex<Vec<u8>>>,
}

impl Write for RefusesFirst {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.refused {
            self.refused = true;
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "refused once"));
        }
        self.written
            .lock()
            .expect("written")
            .extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
    }
}

/// The loss a gap report names,
///  read back from the output.
fn reported(text: &str) -> Vec<Loss> {
    let mut found = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.split("WARN ide_app::logging: ").nth(1) else {
            continue;
        };
        let words: Vec<&str> = rest.split_whitespace().collect();
        let records = words[0].parse().expect("record count");
        let bytes = words[3]
            .trim_start_matches('(')
            .parse()
            .expect("byte count");
        found.push(Loss { records, bytes });
    }
    return found;
}

fn write_record(writer: &impl for<'writer> MakeWriter<'writer>, text: &str) {
    writer
        .make_writer()
        .write_all(text.as_bytes())
        .expect("a queued write never fails");
}

/// The defect this writer exists for:
///  an output that stalls for seconds must not delay the thread
/// that logs,
///  through the real subscriber.
///  Records that do not fit are counted and reported later.
#[test]
fn a_blocked_output_never_delays_the_logging_thread() {
    let output = Gated::closed();
    let stall = Duration::from_secs(4);
    let (writer, flush) =
        background_with(output.clone(), 64 * 1024, Duration::from_secs(10)).expect("writer thread");
    output.open_after(stall);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(writer)
        .finish();
    let total = 5000;
    let started = Instant::now();
    let mut slowest = Duration::ZERO;
    // The guard keeps the subscriber as this thread's default until it is dropped.
    let default_guard = tracing::subscriber::set_default(subscriber);
    for number in 0..total {
        let before = Instant::now();
        tracing::info!(number, "a record logged while the output is stalled");
        slowest = slowest.max(before.elapsed());
    }
    drop(default_guard);
    let elapsed = started.elapsed();
    assert!(
        slowest < Duration::from_secs(1) && elapsed < Duration::from_secs(3),
        "logging waited for the stalled output: slowest record {slowest:?}, all {total} records {elapsed:?}, output stalled for {stall:?}"
    );
    // The flush waits for the gate, then for every queued record and the report of the rest.
    drop(flush);
    let text = output.text();
    let written = text
        .matches("a record logged while the output is stalled")
        .count();
    let losses = reported(&text);
    let mut lost: u64 = 0;
    for loss in losses.iter() {
        lost += loss.records;
    }
    assert!(
        lost > 0,
        "the stalled output was expected to overflow the queue"
    );
    assert_eq!(
        written as u64 + lost,
        total,
        "every record is either written or counted as lost: {losses:?}"
    );
}

/// At a clean exit every queued record reaches the output,
///  in order,
///  without a gap report.
#[test]
fn queued_records_reach_the_output_in_order_at_the_exit_flush() {
    let output = Gated::closed();
    output.open();
    let (writer, flush) = background_with(output.clone(), 1024 * 1024, Duration::from_secs(10))
        .expect("writer thread");
    let mut expected = String::new();
    for number in 0..200 {
        let line = format!("record {number}\n");
        write_record(&writer, &line);
        expected.push_str(&line);
    }
    drop(flush);
    assert_eq!(output.text(), expected);
}

/// A gap is reported right before the next record that fits,
///  with the dropped count and size.
#[test]
fn a_loss_is_reported_before_the_next_record_that_fits() {
    let output = Gated::closed();
    let (writer, flush) =
        background_with(output.clone(), 100, Duration::from_secs(10)).expect("writer thread");
    let first = format!("{}\n", "a".repeat(59));
    let dropped = format!("{}\n", "b".repeat(59));
    let third = format!("{}\n", "c".repeat(59));
    write_record(&writer, &first);
    // The first record waits for the gate and still counts against the budget, so this one is dropped.
    write_record(&writer, &dropped);
    output.open();
    // The writer thread releases a record's bytes from the budget after the output accepted it.
    let deadline = Instant::now() + Duration::from_secs(10);
    while !output.text().contains(&first) || writer.shared.queued.load(Ordering::SeqCst) != 0 {
        assert!(
            Instant::now() < deadline,
            "the first record was not written"
        );
        thread::sleep(Duration::from_millis(5));
    }
    write_record(&writer, &third);
    drop(flush);
    let text = output.text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert_eq!(lines[0], first.trim_end());
    assert_eq!(
        reported(lines[1]),
        vec![Loss {
            records: 1,
            bytes: 60
        }],
        "{text}"
    );
    assert_eq!(lines[2], third.trim_end());
    assert!(!text.contains(&dropped));
}

/// A record larger than the whole budget is never queued;
///  the exit flush reports it.
#[test]
fn a_record_larger_than_the_budget_is_reported_at_the_exit_flush() {
    let output = Gated::closed();
    output.open();
    let (writer, flush) =
        background_with(output.clone(), 10, Duration::from_secs(10)).expect("writer thread");
    write_record(&writer, "twenty bytes, more!\n");
    drop(flush);
    assert_eq!(
        reported(&output.text()),
        vec![Loss {
            records: 1,
            bytes: 20
        }]
    );
}

/// An output that refuses a write loses that record,
///  and the loss is reported once it accepts again.
#[test]
fn a_refused_write_is_counted_and_reported() {
    let written = Arc::new(Mutex::new(Vec::new()));
    let output = RefusesFirst {
        refused: false,
        written: Arc::clone(&written),
    };
    let (writer, flush) =
        background_with(output, 1024, Duration::from_secs(10)).expect("writer thread");
    write_record(&writer, "refused\n");
    drop(flush);
    let text = String::from_utf8(written.lock().expect("written").clone()).expect("text");
    assert!(!text.contains("refused\n"), "{text}");
    assert_eq!(
        reported(&text),
        vec![Loss {
            records: 1,
            bytes: 8
        }],
        "{text}"
    );
}

/// An output that accepts nothing does not hold the exit longer than the grace.
#[test]
fn the_exit_flush_gives_up_after_its_grace() {
    let output = Gated::closed();
    let grace = Duration::from_millis(300);
    let (writer, flush) = background_with(output.clone(), 1024, grace).expect("writer thread");
    write_record(&writer, "never written before the exit\n");
    let started = Instant::now();
    drop(flush);
    let waited = started.elapsed();
    output.open();
    assert!(
        waited >= grace && waited < Duration::from_secs(3),
        "the exit flush waited {waited:?} with a grace of {grace:?}"
    );
}

/// The report line names the loss in words,
///  after a timestamp in the subscriber's format.
#[test]
fn the_gap_report_reads_like_a_warning_record() {
    let mut line = Vec::new();
    assert!(report(
        &mut line,
        Loss {
            records: 3,
            bytes: 42
        }
    ));
    let text = String::from_utf8(line).expect("text");
    assert!(text.ends_with(
        "  WARN ide_app::logging: 3 log records (42 bytes) were not written because the log output did not accept them in time; the log has a gap here\n"
    ), "{text}");
    assert!(
        text.starts_with("20"),
        "the line starts with its timestamp: {text}"
    );
    let mut nothing = Vec::new();
    assert!(report(&mut nothing, Loss::default()));
    assert!(nothing.is_empty());
}
