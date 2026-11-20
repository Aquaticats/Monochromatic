//! What:
//!  The device-free proof of the read retry rule:
//! a reply ends the read,
//! a
//!  listing's end is no list,
//! a named size grows the buffer and asks again,
//! a size
//!  the buffer already holds is a disagreement,
//! a third attempt stops asking,
//! and a
//!  size past the cap is refused.
//! Why:
//!  The rule decides how many times a read reaches the kernel.
//! A test with
//!  scripted answers pins every branch without a device,
//! and the guest fixture then
//!  proves the same rule against the kernel's real size report.

use super::device_error::DeviceError;
use super::read_reply::{MAXIMUM_READ_ATTEMPTS, ReadAttempt, retry_read};
use crate::nvlist::NvList;
use crate::requests::{INITIAL_REPLY_SIZE, MAXIMUM_REPLY_SIZE, Request, objset_stats};

#[test]
fn a_reply_ends_the_read_without_growing_anything() {
    let mut request: Request = stats_request();
    let read: Option<NvList> = retry_read(&mut request, |_| {
        return Ok(ReadAttempt::Reply {
            list: NvList::new(),
        });
    })
    .expect("a reply is the answer");
    assert!(read.is_some());
    assert_eq!(request.reply().len(), INITIAL_REPLY_SIZE);
}

#[test]
fn a_listings_end_is_no_list() {
    let mut request: Request = stats_request();
    let read: Option<NvList> = retry_read(&mut request, |_| {
        return Ok(ReadAttempt::End);
    })
    .expect("an end of listing is not a failure");
    assert!(read.is_none());
}

#[test]
fn a_named_size_grows_the_buffer_and_asks_again() {
    let mut request: Request = stats_request();
    let mut answers: usize = 0;
    let read: Option<NvList> = retry_read(&mut request, |asking| {
        answers += 1;
        if answers == 1 {
            return Ok(ReadAttempt::NeedsBiggerReply {
                requested: INITIAL_REPLY_SIZE as u64 + 1,
            });
        }
        assert_eq!(asking.reply().len(), INITIAL_REPLY_SIZE * 2);
        return Ok(ReadAttempt::Reply {
            list: NvList::new(),
        });
    })
    .expect("the second answer fits");
    assert!(read.is_some());
    assert_eq!(answers, 2);
}

#[test]
fn a_size_the_buffer_already_holds_is_a_disagreement() {
    let mut request: Request = stats_request();
    let error = retry_read(&mut request, |_| {
        return Ok(ReadAttempt::NeedsBiggerReply {
            requested: INITIAL_REPLY_SIZE as u64,
        });
    })
    .expect_err("a size the buffer holds is not a reason to ask again");
    assert!(matches!(error, DeviceError::UnstableReply { .. }));
}

#[test]
fn asking_past_the_attempt_limit_stops() {
    let mut request: Request = stats_request();
    let mut answers: usize = 0;
    let error = retry_read(&mut request, |asking| {
        answers += 1;
        return Ok(ReadAttempt::NeedsBiggerReply {
            requested: asking.reply().len() as u64 * 2,
        });
    })
    .expect_err("the third attempt is the last");
    assert!(matches!(error, DeviceError::UnstableReply { .. }));
    assert_eq!(answers, MAXIMUM_READ_ATTEMPTS);
}

#[test]
fn a_size_past_the_cap_is_refused() {
    let mut request: Request = stats_request();
    let error = retry_read(&mut request, |_| {
        return Ok(ReadAttempt::NeedsBiggerReply {
            requested: MAXIMUM_REPLY_SIZE as u64 + 1,
        });
    })
    .expect_err("the cap is the cap");
    let Some(failure) = error.request() else {
        panic!("a refused growth is a request failure");
    };
    assert!(failure.to_string().contains("at most"));
}

#[test]
fn a_refused_read_is_reported_unchanged() {
    let mut request: Request = stats_request();
    let error = retry_read(&mut request, |_| {
        return Err(DeviceError::Request {
            source: crate::requests::Error::Call {
                operation: crate::requests::Operation::ObjsetStats,
                target: String::from("pool/dataset"),
                errno: libc::EPERM,
            },
        });
    })
    .expect_err("a refused read is a failure");
    assert_eq!(error.errno(), Some(libc::EPERM));
}

/// A read request whose buffer starts at the initial size.
fn stats_request() -> Request {
    return objset_stats("pool/dataset").expect("the name fits");
}
