//! OSC 9;4's mapping: done, waiting, broken off and resting, as Windows' Claude Code reports them.

use super::*;
use crate::term::notify::Progress;

fn record(state: State) -> Record {
    Record {
        state,
        kind: None,
        progress: None,
        app: None,
        title: None,
        msg: None,
    }
}

#[test]
fn osc_9_4_work_that_ends_is_done_unless_broken_off() {
    let root = |records: &Records| {
        records
            .list
            .iter()
            .find(|(id, ..)| id.is_empty())
            .map(|(_, record, _)| record.state)
    };
    let mut records = Records::default();
    records.progress(Progress::Indeterminate);
    records.progress(Progress::Remove);
    assert_eq!(root(&records), Some(State::Done), "work that ends is done");
    assert!(records.seen(), "typing sees it");
    assert_eq!(root(&records), Some(State::Idle), "a seen finish rests");
    records.progress(Progress::Indeterminate);
    records.interrupt();
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "Esc or Ctrl-C broke it off"
    );
    records.progress(Progress::Set(Some(10)));
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Done),
        "the next work starts clean"
    );
    records.interrupt();
    records.progress(Progress::Indeterminate);
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Done),
        "a press while nothing works breaks nothing off"
    );
    records.report("a".into(), record(State::Idle));
    assert_eq!(root(&records), None, "a report replaces what 9;4 mapped");
    records.progress(Progress::Indeterminate);
    assert_eq!(root(&records), None, "OSC 7501 leads, 9;4 passes by");
}

#[test]
fn a_notification_during_osc_9_4_work_waits_until_a_key_answers() {
    let root = |records: &Records| {
        records
            .list
            .iter()
            .find(|(id, ..)| id.is_empty())
            .map(|(_, record, _)| record.state)
    };
    let mut records = Records::default();
    records.wait();
    assert_eq!(root(&records), None, "no work, no wait");
    records.progress(Progress::Indeterminate);
    records.wait();
    assert_eq!(
        root(&records),
        Some(State::Blocked),
        "the program waits on the user"
    );
    records.progress(Progress::Indeterminate);
    assert_eq!(
        root(&records),
        Some(State::Blocked),
        "progress keeps the wait"
    );
    assert!(records.answer(), "a key answers it");
    assert_eq!(root(&records), Some(State::Working));
    assert!(
        !records.answer(),
        "work that waits on nothing has nothing to answer"
    );
    records.progress(Progress::Remove);
    assert_eq!(root(&records), Some(State::Done), "answered work ends done");
    records.progress(Progress::Indeterminate);
    records.wait();
    records.interrupt();
    records.answer();
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "Esc on the wait breaks the work off"
    );
    records.progress(Progress::Indeterminate);
    records.wait();
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Done),
        "a wait that ends on its own ends done"
    );
}

#[test]
fn osc_9_4_s_program_rests_idle_until_the_shell_s_next_prompt() {
    let root = |records: &Records| {
        records
            .list
            .iter()
            .find(|(id, ..)| id.is_empty())
            .map(|(_, record, _)| record.state)
    };
    let mut records = Records::default();
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "a program that starts with a remove rests"
    );
    records.prompt();
    assert_eq!(root(&records), None, "the shell's prompt means it left");
}
