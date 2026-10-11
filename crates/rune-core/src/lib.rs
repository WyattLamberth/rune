//! Deterministic command policy. No OS or serialization dependencies.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Creating,
    Created,
    Running,
    Stopped,
    Paused,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Start,
    Kill,
    KillAll,
    Delete,
    ForceDelete,
    Pause,
    Resume,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Start,
    Signal,
    Delete,
    Pause,
    Resume,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidTransition;

/// Plans a command against an observed backend state. Completion and persisted
/// state belong to the backend; planning never pretends a syscall succeeded.
pub fn plan(status: Status, action: Action) -> Result<Effect, InvalidTransition> {
    use Action as A;
    use Effect as E;
    use Status as S;
    match (status, action) {
        (S::Created, A::Start) => Ok(E::Start),
        (S::Created | S::Running | S::Paused, A::Kill | A::KillAll) | (S::Stopped, A::KillAll) => {
            Ok(E::Signal)
        }
        (S::Stopped, A::Delete) | (_, A::ForceDelete) => Ok(E::Delete),
        (S::Running, A::Pause) => Ok(E::Pause),
        (S::Paused, A::Resume) => Ok(E::Resume),
        _ => Err(InvalidTransition),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandError<E> {
    InvalidTransition,
    Backend(E),
}

/// Production dispatch boundary: invalid commands invoke no adapter; failures
/// remain errors and never synthesize successful completion.
pub fn execute<E>(
    status: Status,
    action: Action,
    apply: impl FnOnce(Effect) -> Result<(), E>,
) -> Result<(), CommandError<E>> {
    let effect = plan(status, action).map_err(|_| CommandError::InvalidTransition)?;
    apply(effect).map_err(CommandError::Backend)
}

/// OCI IDs are a single bounded path component, never a traversal or separator.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_traversal_and_oversized_ids() {
        for id in [
            "",
            ".",
            "..",
            "../victim",
            "/victim",
            "a/b",
            "a\\b",
            "é",
            "a\0b",
        ] {
            assert!(!valid_id(id), "{id:?}");
        }
        assert!(!valid_id(&"a".repeat(129)));
        assert!(valid_id(&"a".repeat(128)));
        assert!(valid_id("podman-01_test.v1"));
    }
    #[test]
    fn exhaustive_lifecycle_policy() {
        let states = [
            Status::Creating,
            Status::Created,
            Status::Running,
            Status::Stopped,
            Status::Paused,
        ];
        let actions = [
            Action::Start,
            Action::Kill,
            Action::KillAll,
            Action::Delete,
            Action::ForceDelete,
            Action::Pause,
            Action::Resume,
        ];
        for state in states {
            for action in actions {
                let allowed = match action {
                    Action::Start => state == Status::Created,
                    Action::Kill => {
                        [Status::Created, Status::Running, Status::Paused].contains(&state)
                    }
                    Action::KillAll => state != Status::Creating,
                    Action::Delete => state == Status::Stopped,
                    Action::ForceDelete => true,
                    Action::Pause => state == Status::Running,
                    Action::Resume => state == Status::Paused,
                };
                assert_eq!(plan(state, action).is_ok(), allowed, "{state:?} {action:?}");
            }
        }
    }
}
