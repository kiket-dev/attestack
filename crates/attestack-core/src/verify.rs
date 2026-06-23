use crate::error::{AttestackError, Result};
use crate::hash::hash_event_for_chain;
use crate::types::{Event, EVENT_SCHEMA_V1};

pub fn verify_event_chain(events: &[Event]) -> Result<()> {
    let mut previous_hash: Option<String> = None;

    for (index, event) in events.iter().enumerate() {
        let expected_sequence = index as u64 + 1;

        if event.schema_version != EVENT_SCHEMA_V1 {
            return Err(AttestackError::InvalidSchemaVersion(event.schema_version.clone()));
        }

        if event.sequence != expected_sequence {
            return Err(AttestackError::HashChainBroken {
                sequence: event.sequence,
                reason: format!("expected sequence {expected_sequence}, found {}", event.sequence),
            });
        }

        if event.prev_event_hash != previous_hash {
            return Err(AttestackError::HashChainBroken {
                sequence: event.sequence,
                reason: "previous hash link mismatch".into(),
            });
        }

        let computed = hash_event_for_chain(event)?;
        if computed != event.event_hash {
            return Err(AttestackError::HashChainBroken {
                sequence: event.sequence,
                reason: "event hash mismatch".into(),
            });
        }

        previous_hash = Some(event.event_hash.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Event, EventKind, EventPayload, SessionNoteAddedPayload, SessionStartedPayload,
        SessionStoppedPayload,
    };

    fn started(sequence: u64, prev: Option<String>) -> Event {
        let mut event = Event::new(
            "ses_test".into(),
            sequence,
            EventKind::SessionStarted,
            EventPayload::SessionStarted(SessionStartedPayload { title: "demo".into() }),
            prev,
        );
        event.event_hash = hash_event_for_chain(&event).unwrap();
        event
    }

    #[test]
    fn valid_chain_passes() {
        let first = started(1, None);
        let mut second = Event::new(
            "ses_test".into(),
            2,
            EventKind::SessionStopped,
            EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            Some(first.event_hash.clone()),
        );
        second.event_hash = hash_event_for_chain(&second).unwrap();
        verify_event_chain(&[first, second]).unwrap();
    }

    #[test]
    fn tampered_payload_fails() {
        let mut first = started(1, None);
        if let EventPayload::SessionStarted(ref mut payload) = first.payload {
            payload.title = "tampered".into();
        }
        assert!(verify_event_chain(&[first]).is_err());
    }

    #[test]
    fn duplicate_sequence_fails() {
        let first = started(1, None);
        let mut second = Event::new(
            "ses_test".into(),
            2,
            EventKind::SessionStopped,
            EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            Some(first.event_hash.clone()),
        );
        second.event_hash = hash_event_for_chain(&second).unwrap();
        let mut duplicate = Event::new(
            "ses_test".into(),
            2,
            EventKind::SessionNoteAdded,
            EventPayload::SessionNoteAdded(SessionNoteAddedPayload { text: "dup".into() }),
            Some(second.event_hash.clone()),
        );
        duplicate.event_hash = hash_event_for_chain(&duplicate).unwrap();
        assert!(verify_event_chain(&[first, second, duplicate]).is_err());
    }

    #[test]
    fn missing_previous_hash_fails() {
        let first = started(1, None);
        let mut second = Event::new(
            "ses_test".into(),
            2,
            EventKind::SessionStopped,
            EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            None,
        );
        second.event_hash = hash_event_for_chain(&second).unwrap();
        assert!(verify_event_chain(&[first, second]).is_err());
    }

    #[test]
    fn reordered_events_fail() {
        let first = started(1, None);
        let mut second = Event::new(
            "ses_test".into(),
            2,
            EventKind::SessionStopped,
            EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            Some(first.event_hash.clone()),
        );
        second.event_hash = hash_event_for_chain(&second).unwrap();
        assert!(verify_event_chain(&[second, first]).is_err());
    }
}
