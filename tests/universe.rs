//! Seeded replay of production command policy and adapter failure propagation.
//! This does not simulate Linux resource setup, syscall rollback or thread races.
use rune_core::{Action, CommandError, Status, execute};
use std::collections::VecDeque;

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
#[test]
fn seeded_policy_fault_replay() {
    let override_seed = std::env::var("RUNE_TEST_SEED")
        .ok()
        .map(|v| v.parse::<u64>().expect("u64 seed"));
    let steps = std::env::var("RUNE_TEST_STEPS")
        .ok()
        .map(|v| v.parse::<usize>().expect("step count"))
        .unwrap_or(10000);
    assert!((1..=1000000).contains(&steps));
    for seed in override_seed
        .map(|seed| vec![seed])
        .unwrap_or_else(|| vec![0, 1, 42, 1337, u64::MAX])
    {
        let mut rng = seed;
        let mut trace = VecDeque::with_capacity(32);
        for step in 0..steps {
            let status = [
                Status::Absent,
                Status::Creating,
                Status::Created,
                Status::Running,
                Status::Stopped,
                Status::Paused,
            ][(next(&mut rng) % 6) as usize];
            let action = [
                Action::Start,
                Action::Kill,
                Action::KillAll,
                Action::Delete,
                Action::ForceDelete,
                Action::Pause,
                Action::Resume,
            ][(next(&mut rng) % 7) as usize];
            let fail = next(&mut rng).is_multiple_of(4);
            let allowed = match action {
                Action::Start => status == Status::Created,
                Action::Kill => {
                    [Status::Created, Status::Running, Status::Paused].contains(&status)
                }
                Action::KillAll => ![Status::Creating, Status::Absent].contains(&status),
                Action::Delete => status == Status::Stopped,
                Action::ForceDelete => true,
                Action::Pause => status == Status::Running,
                Action::Resume => status == Status::Paused,
            };
            if trace.len() == 32 {
                trace.pop_front();
            }
            trace.push_back((status, action, fail));
            let mut calls = 0;
            let result = execute(status, action, |_| {
                calls += 1;
                if fail { Err("injected") } else { Ok(()) }
            });
            let noop = status == Status::Absent && action == Action::ForceDelete;
            let expected = if noop {
                Ok(())
            } else if !allowed {
                Err(CommandError::InvalidTransition)
            } else if fail {
                Err(CommandError::Backend("injected"))
            } else {
                Ok(())
            };
            assert_eq!(result, expected, "seed={seed} step={step} trace={trace:?}");
            assert_eq!(
                calls,
                usize::from(allowed && !noop),
                "seed={seed} step={step} trace={trace:?}"
            );
        }
    }
}
