//! OSC 9;4 as the root record, where a program reports no OSC 7501: Claude Code on Windows, whose probe for
//! 7501 loses to conhost's own DA1 answer.

use std::time::Duration;

use super::{Record, Records, State};
use crate::term::notify::Progress;

/// How long OSC 9;4's work must last before its row shows it: a slash command's blink of work shows nothing.
pub const GRACE: Duration = Duration::from_millis(400);

/// Only Windows reads work from OSC 9;4: elsewhere Claude Code's 7501 works, and 9;4 stands as the spec maps it.
pub const INFERS: bool = cfg!(windows);

impl Records {
    /// OSC 9;4 as the root record, until a program reports through OSC 7501; the number of work that began, which `reveal` shows once `GRACE` runs out.
    pub fn progress(&mut self, progress: Progress) -> Option<u64> {
        if self.reported {
            log::debug!("status: OSC 9;4 passed by, a program reports through OSC 7501");
            return None;
        }
        let root = |state, progress| Record {
            state,
            kind: None,
            progress,
            app: None,
            title: None,
            msg: None,
        };
        if !INFERS {
            match progress {
                Progress::Remove => self.list.retain(|(id, ..)| !id.is_empty()),
                // Paused work is work still, its progress kept.
                Progress::Set(value) | Progress::Pause(value) => {
                    self.put(String::new(), root(State::Working, value))
                }
                Progress::Indeterminate => self.put(String::new(), root(State::Working, None)),
                Progress::Error(_) => self.put(String::new(), root(State::Error, None)),
            }
            return None;
        }
        let working =
            self.pending.is_some() || self.mapped(State::Working) || self.mapped(State::Blocked);
        let mut began = None;
        match progress {
            // Work that ended inside its grace showed nothing, and leaves nothing.
            Progress::Remove if self.pending.take().is_some() => {
                log::debug!("status: OSC 9;4's work ended inside its grace");
            }
            // Work that ends is done, as Claude Code's turns end, unless the user broke it off.
            Progress::Remove if working && !self.interrupted => {
                self.put(String::new(), root(State::Done, None))
            }
            // A program that ends no work, or whose work was broken off, rests until the shell's next prompt.
            Progress::Remove => self.put(String::new(), root(State::Idle, None)),
            // Work that waits on the user keeps waiting until a key answers it.
            Progress::Set(_) | Progress::Pause(_) | Progress::Indeterminate
                if self.mapped(State::Blocked) => {}
            // Paused work is work still, its progress kept.
            Progress::Set(_) | Progress::Pause(_) | Progress::Indeterminate if !working => {
                let value = if let Progress::Set(value) | Progress::Pause(value) = progress {
                    value
                } else {
                    None
                };
                self.works += 1;
                self.pending = Some((self.works, root(State::Working, value)));
                began = Some(self.works);
            }
            Progress::Set(value) | Progress::Pause(value) => {
                self.go_on(root(State::Working, value))
            }
            Progress::Indeterminate => self.go_on(root(State::Working, None)),
            Progress::Error(_) => {
                self.pending = None;
                self.put(String::new(), root(State::Error, None))
            }
        }
        if !working || matches!(progress, Progress::Remove | Progress::Error(_)) {
            self.interrupted = false;
        }
        began
    }

    /// Work still in its grace takes the new progress there; shown work takes it at once.
    fn go_on(&mut self, record: Record) {
        match &mut self.pending {
            Some((_, pending)) => *pending = record,
            None => self.put(String::new(), record),
        }
    }

    /// The grace of OSC 9;4's work number `work` ran out: its row shows it, if it still works.
    pub fn reveal(&mut self, work: u64) -> bool {
        match self.pending.take() {
            Some((number, record)) if number == work => {
                self.put(String::new(), record);
                true
            }
            other => {
                self.pending = other;
                false
            }
        }
    }

    /// A notification during OSC 9;4's work: the program waits on the user, as Claude Code's do.
    pub fn wait(&mut self) {
        if !INFERS {
            return;
        }
        if let Some((work, _)) = self.pending {
            self.reveal(work);
        }
        if self.mapped(State::Working) {
            log::info!("status: a notification during OSC 9;4's work waits on the user");
            self.set_root(State::Blocked);
        }
    }

    /// A key reached the program: work that waited on the user goes on.
    pub fn answer(&mut self) -> bool {
        let waited = INFERS && self.mapped(State::Blocked);
        if waited {
            log::info!("status: a key answers the wait, OSC 9;4's work goes on");
            self.set_root(State::Working);
        }
        waited
    }

    /// Esc or Ctrl-C reached the program: the work OSC 9;4 shows ends broken off, not done.
    pub fn interrupt(&mut self) {
        let working =
            self.pending.is_some() || self.mapped(State::Working) || self.mapped(State::Blocked);
        if INFERS && working && !self.interrupted {
            log::info!("status: Esc or Ctrl-C breaks off OSC 9;4's work");
            self.interrupted = true;
        }
    }

    /// The root record is OSC 9;4's mapping, in `state`.
    pub(super) fn mapped(&self, state: State) -> bool {
        !self.reported
            && self
                .list
                .iter()
                .any(|(id, record, _)| id.is_empty() && record.state == state)
    }

    pub(super) fn set_root(&mut self, state: State) {
        let record = Record {
            state,
            kind: None,
            progress: None,
            app: None,
            title: None,
            msg: None,
        };
        self.put(String::new(), record);
    }
}
