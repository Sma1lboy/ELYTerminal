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

/// `progress` as the view hands it on, its work outlasting the grace.
fn progress(records: &mut Records, progress: Progress) {
    if let Some(work) = records.progress(progress) {
        records.reveal(work);
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
    progress(&mut records, Progress::Indeterminate);
    progress(&mut records, Progress::Remove);
    assert_eq!(root(&records), Some(State::Done), "work that ends is done");
    assert!(records.seen(), "typing sees it");
    assert_eq!(root(&records), Some(State::Idle), "a seen finish rests");
    progress(&mut records, Progress::Indeterminate);
    records.interrupt();
    progress(&mut records, Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "Esc or Ctrl-C broke it off"
    );
    progress(&mut records, Progress::Set(Some(10)));
    progress(&mut records, Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Done),
        "the next work starts clean"
    );
    records.interrupt();
    progress(&mut records, Progress::Indeterminate);
    progress(&mut records, Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Done),
        "a press while nothing works breaks nothing off"
    );
    records.report("a".into(), record(State::Idle));
    assert_eq!(root(&records), None, "a report replaces what 9;4 mapped");
    progress(&mut records, Progress::Indeterminate);
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
    progress(&mut records, Progress::Indeterminate);
    records.wait();
    assert_eq!(
        root(&records),
        Some(State::Blocked),
        "the program waits on the user"
    );
    progress(&mut records, Progress::Indeterminate);
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
    progress(&mut records, Progress::Remove);
    assert_eq!(root(&records), Some(State::Done), "answered work ends done");
    progress(&mut records, Progress::Indeterminate);
    records.wait();
    records.interrupt();
    records.answer();
    progress(&mut records, Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "Esc on the wait breaks the work off"
    );
    progress(&mut records, Progress::Indeterminate);
    records.wait();
    progress(&mut records, Progress::Remove);
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
    progress(&mut records, Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "a program that starts with a remove rests"
    );
    records.prompt();
    assert_eq!(root(&records), None, "the shell's prompt means it left");
}

#[test]
fn osc_9_4_work_inside_its_grace_shows_nothing() {
    let root = |records: &Records| {
        records
            .list
            .iter()
            .find(|(id, ..)| id.is_empty())
            .map(|(_, record, _)| record.state)
    };
    let mut records = Records::default();
    progress(&mut records, Progress::Remove);
    let work = records
        .progress(Progress::Indeterminate)
        .expect("work begins");
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "the row waits out the grace"
    );
    records.progress(Progress::Remove);
    assert_eq!(
        root(&records),
        Some(State::Idle),
        "a blink of work leaves no finish"
    );
    assert!(!records.reveal(work), "its late reveal finds nothing");
    let first = records
        .progress(Progress::Indeterminate)
        .expect("work begins");
    records.progress(Progress::Remove);
    let second = records
        .progress(Progress::Set(Some(5)))
        .expect("work begins");
    assert!(!records.reveal(first), "an old reveal shows no newer work");
    assert!(records.reveal(second));
    assert_eq!(root(&records), Some(State::Working));
    let work = {
        records.progress(Progress::Remove);
        records
            .progress(Progress::Indeterminate)
            .expect("work begins")
    };
    records.wait();
    assert_eq!(
        root(&records),
        Some(State::Blocked),
        "a wait shows its work at once"
    );
    assert!(!records.reveal(work));
}

#[test]
fn a_still_title_during_osc_9_4_work_waits_once_its_grace_runs_out() {
    let root = |records: &Records| {
        records
            .list
            .iter()
            .find(|(id, ..)| id.is_empty())
            .map(|(_, record, _)| record.state)
    };
    let mut records = Records::default();
    assert_eq!(records.titled(true), None, "no work, nothing to check");
    progress(&mut records, Progress::Indeterminate);
    assert_eq!(records.titled(false), None, "a spinning title works");
    let work = records
        .titled(true)
        .expect("a still title is checked later");
    assert_eq!(root(&records), Some(State::Working), "not at once");
    assert!(
        records.stall(work),
        "still after the grace: it waits on the user"
    );
    assert_eq!(root(&records), Some(State::Blocked));
    records.titled(false);
    assert_eq!(
        root(&records),
        Some(State::Working),
        "a spinning title goes on"
    );
    let work = records.titled(true).expect("checked later");
    progress(&mut records, Progress::Remove);
    assert!(
        !records.stall(work),
        "work that ended under a still title is done"
    );
    assert_eq!(root(&records), Some(State::Done));
}
