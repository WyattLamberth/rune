//! OCI process composition, with no filesystem or kernel effects.
use anyhow::{Result, bail};
use liboci_cli::Exec;
use oci_spec::runtime::{Capabilities, Capability, LinuxCapabilities, Process};

pub(crate) fn resolve(init: Process, supplied: Option<Process>, args: &Exec) -> Result<Process> {
    let from_file = supplied.is_some();
    let mut process = supplied.unwrap_or_else(|| init.clone());
    if from_file {
        // Process documents are authoritative. Reject combinations whose flags
        // would otherwise be silently ignored, rather than promise overrides.
        if args.cwd.is_some()
            || !args.env.is_empty()
            || args.tty
            || args.user.is_some()
            || !args.additional_gids.is_empty()
            || args.process_label.is_some()
            || args.apparmor.is_some()
            || args.no_new_privs
            || !args.cap.is_empty()
            || !args.command.is_empty()
        {
            bail!("--process cannot be combined with process overrides or a command");
        }
        // runc's newInitConfig inherits container policy when the supplied
        // process leaves these fields unset (or an empty label/rlimit list).
        if process
            .apparmor_profile()
            .as_deref()
            .is_none_or(str::is_empty)
        {
            process.set_apparmor_profile(init.apparmor_profile().clone());
        }
        if process.selinux_label().as_deref().is_none_or(str::is_empty) {
            process.set_selinux_label(init.selinux_label().clone());
        }
        if process.capabilities().is_none() {
            process.set_capabilities(init.capabilities().clone());
        }
        if process.rlimits().as_ref().is_none_or(Vec::is_empty) {
            process.set_rlimits(init.rlimits().clone());
        }
        if process.io_priority().is_none() {
            process.set_io_priority(*init.io_priority());
        }
        if process.scheduler().is_none() {
            process.set_scheduler(init.scheduler().clone());
        }
        if process.exec_cpu_affinity().is_none() {
            process.set_exec_cpu_affinity(init.exec_cpu_affinity().clone());
        }
        if process.no_new_privileges().is_none() {
            process.set_no_new_privileges(init.no_new_privileges());
        }
    } else {
        process.set_args(Some(args.command.clone()));
        process.set_terminal(Some(args.tty));
        if let Some(cwd) = &args.cwd {
            process.set_cwd(cwd.clone());
        }
        if !args.env.is_empty() {
            let mut env = process.env().clone().unwrap_or_default();
            // Preserve CLI assignment order until last-assignment resolution.
            env.extend(args.env.iter().map(|(key, value)| format!("{key}={value}")));
            process.set_env(Some(env));
        }
        if let Some((uid, gid)) = args.user {
            process.user_mut().set_uid(uid);
            if let Some(gid) = gid {
                process.user_mut().set_gid(gid);
            }
        }
        if !args.additional_gids.is_empty() {
            let mut gids = process.user().additional_gids().clone().unwrap_or_default();
            gids.extend(&args.additional_gids);
            process.user_mut().set_additional_gids(Some(gids));
        }
        if let Some(profile) = args.apparmor.as_ref().filter(|s| !s.is_empty()) {
            process.set_apparmor_profile(Some(profile.clone()));
        }
        if let Some(label) = args.process_label.as_ref().filter(|s| !s.is_empty()) {
            process.set_selinux_label(Some(label.clone()));
        }
        if args.no_new_privs {
            process.set_no_new_privileges(Some(true));
        }
        if !args.cap.is_empty() {
            let mut caps = process.capabilities().clone().unwrap_or_else(|| {
                let mut empty = LinuxCapabilities::default();
                empty
                    .set_bounding(None)
                    .set_effective(None)
                    .set_permitted(None)
                    .set_inheritable(None)
                    .set_ambient(None);
                empty
            });
            let additional: Capabilities = args
                .cap
                .iter()
                .map(|cap| {
                    serde_json::from_value::<Capability>(serde_json::Value::String(cap.clone()))
                })
                .collect::<std::result::Result<_, _>>()?;
            fn with_additional(set: &Option<Capabilities>, extra: &Capabilities) -> Capabilities {
                let mut set = set.clone().unwrap_or_default();
                set.extend(extra);
                set
            }
            caps.set_bounding(Some(with_additional(caps.bounding(), &additional)));
            caps.set_effective(Some(with_additional(caps.effective(), &additional)));
            caps.set_permitted(Some(with_additional(caps.permitted(), &additional)));
            if caps.inheritable().is_some() {
                caps.set_ambient(Some(with_additional(caps.ambient(), &additional)));
            }
            process.set_capabilities(Some(caps));
        }
    }
    crate::validation::check_process(&process)?;
    if let Some(env) = process.env() {
        // The backend validates executable lookup using the first PATH entry,
        // but installs environment variables with last assignment winning.
        // Resolve duplicates so those two views cannot disagree.
        let mut seen = std::collections::HashSet::new();
        let mut resolved: Vec<_> = env
            .iter()
            .rev()
            .filter(|entry| seen.insert(entry.split_once('=').expect("validated environment").0))
            .cloned()
            .collect();
        resolved.reverse();
        process.set_env(Some(resolved));
    }
    Ok(process)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use serde_json::json;

    fn args(extra: &[&str]) -> Exec {
        let mut argv = vec!["rune", "exec"];
        argv.extend_from_slice(extra);
        let cli = crate::Cli::try_parse_from(argv).unwrap();
        let crate::Command::Exec(args) = cli.command else {
            panic!("expected exec")
        };
        *args
    }

    fn init() -> Process {
        serde_json::from_value(json!({
            "terminal":true, "consoleSize":{"height":24,"width":80},
            "user":{"uid":1234,"gid":5678,"additionalGids":[42],"umask":63},
            "args":["/init"], "env":["PATH=/bin","KEY=old"], "cwd":"/work",
            "capabilities":{"bounding":["CAP_CHOWN"],"effective":["CAP_CHOWN"],
                "permitted":["CAP_CHOWN"],"inheritable":["CAP_CHOWN"],"ambient":["CAP_CHOWN"]},
            "rlimits":[{"type":"RLIMIT_NOFILE","hard":64,"soft":32}],
            "noNewPrivileges":true, "apparmorProfile":"rune-test-profile",
            "selinuxLabel":"system_u:system_r:container_t:s0", "oomScoreAdj":125,
            "scheduler":{"policy":"SCHED_BATCH","nice":5},
            "ioPriority":{"class":"IOPRIO_CLASS_BE","priority":3},
            "execCPUAffinity":{"initial":"0","final":"0"}
        }))
        .unwrap()
    }

    #[test]
    fn direct_exec_preserves_all_original_policy_except_command_and_terminal() {
        let original = serde_json::to_value(init()).unwrap();
        let resolved = resolve(init(), None, &args(&["test", "/bin/true"])).unwrap();
        let mut actual = serde_json::to_value(resolved).unwrap();
        assert_eq!(actual["args"], json!(["/bin/true"]));
        assert_eq!(actual["terminal"], false);
        actual["args"] = original["args"].clone();
        actual["terminal"] = original["terminal"].clone();
        assert_eq!(
            actual, original,
            "exec must preserve the complete configured policy"
        );
    }

    #[test]
    fn direct_overrides_preserve_unspecified_identity_and_security() {
        let resolved = resolve(
            init(),
            None,
            &args(&[
                "--cwd",
                "/other",
                "--env",
                "KEY=new",
                "--env",
                "KEY=last",
                "--user",
                "2222",
                "--additional-gids",
                "43",
                "--cap",
                "CAP_NET_BIND_SERVICE",
                "test",
                "/bin/true",
            ]),
        )
        .unwrap();
        assert_eq!(resolved.cwd(), &std::path::PathBuf::from("/other"));
        assert_eq!(resolved.user().uid(), 2222);
        assert_eq!(resolved.user().gid(), 5678);
        assert_eq!(resolved.user().umask(), Some(63));
        assert_eq!(
            resolved.user().additional_gids().as_deref(),
            Some(&[42, 43][..])
        );
        assert_eq!(resolved.env().as_ref().unwrap(), &["PATH=/bin", "KEY=last"]);
        assert_eq!(resolved.apparmor_profile(), init().apparmor_profile());
        assert_eq!(resolved.no_new_privileges(), Some(true));
        let caps = resolved.capabilities().as_ref().unwrap();
        assert!(
            caps.bounding()
                .as_ref()
                .unwrap()
                .contains(&Capability::NetBindService)
        );
        assert_eq!(
            caps.inheritable(),
            init().capabilities().as_ref().unwrap().inheritable()
        );
        assert!(
            caps.ambient()
                .as_ref()
                .unwrap()
                .contains(&Capability::NetBindService)
        );
    }

    #[test]
    fn adding_a_capability_to_absent_policy_does_not_grant_default_capabilities() {
        let mut original = init();
        original.set_capabilities(None);
        let resolved = resolve(
            original,
            None,
            &args(&["--cap", "CAP_CHOWN", "test", "/bin/true"]),
        )
        .unwrap();
        let caps = resolved.capabilities().as_ref().unwrap();
        assert_eq!(caps.bounding().as_ref().unwrap().len(), 1);
        assert_eq!(caps.effective().as_ref().unwrap().len(), 1);
        assert_eq!(caps.permitted().as_ref().unwrap().len(), 1);
        assert!(caps.inheritable().is_none());
        assert!(caps.ambient().is_none());
    }

    #[test]
    fn process_document_inherits_container_security_and_optional_resources() {
        let supplied: Process = serde_json::from_value(json!({
            "user":{"uid":2222,"gid":3333}, "cwd":"/file", "args":["/from-file"],
            "apparmorProfile":"", "selinuxLabel":"", "rlimits":[]
        }))
        .unwrap();
        let resolved = resolve(
            init(),
            Some(supplied),
            &args(&["--process", "unused.json", "test"]),
        )
        .unwrap();
        assert_eq!(resolved.user().uid(), 2222);
        assert_eq!(resolved.user().gid(), 3333);
        assert_eq!(resolved.cwd(), &std::path::PathBuf::from("/file"));
        assert_eq!(resolved.args().as_ref().unwrap(), &["/from-file"]);
        assert_eq!(resolved.apparmor_profile(), init().apparmor_profile());
        assert_eq!(resolved.selinux_label(), init().selinux_label());
        assert_eq!(resolved.no_new_privileges(), Some(true));
        assert_eq!(resolved.rlimits(), init().rlimits());
        assert_eq!(resolved.scheduler(), init().scheduler());
        assert_eq!(resolved.io_priority(), init().io_priority());
        assert_eq!(resolved.exec_cpu_affinity(), init().exec_cpu_affinity());
        assert_eq!(resolved.capabilities(), init().capabilities());
    }

    #[test]
    fn supplied_security_policy_is_preserved_and_path_lookup_uses_last_assignment() {
        let mut supplied = init();
        supplied.set_apparmor_profile(Some("explicit-profile".into()));
        supplied.set_env(Some(vec!["PATH=/old".into(), "PATH=/new".into()]));
        let resolved = resolve(
            init(),
            Some(supplied),
            &args(&["--process", "unused.json", "test"]),
        )
        .unwrap();
        assert_eq!(
            resolved.apparmor_profile().as_deref(),
            Some("explicit-profile")
        );
        assert_eq!(resolved.env().as_ref().unwrap(), &["PATH=/new"]);
        let overridden = resolve(
            init(),
            None,
            &args(&[
                "--user",
                "2222:3333",
                "--apparmor",
                "explicit-profile",
                "test",
                "/bin/true",
            ]),
        )
        .unwrap();
        assert_eq!(overridden.user().uid(), 2222);
        assert_eq!(overridden.user().gid(), 3333);
        assert_eq!(overridden.user().umask(), Some(63));
        assert_eq!(
            overridden.apparmor_profile().as_deref(),
            Some("explicit-profile")
        );
    }

    #[test]
    fn ignored_process_document_overrides_are_rejected() {
        for overrides in [
            vec!["--cwd", "/other"],
            vec!["--env", "KEY=value"],
            vec!["--tty"],
            vec!["--user", "2222"],
            vec!["--additional-gids", "43"],
            vec!["--apparmor", "other"],
            vec!["--process-label", "other"],
            vec!["--no-new-privs"],
            vec!["--cap", "CAP_CHOWN"],
        ] {
            let mut argv = vec!["--process", "unused.json"];
            argv.extend(overrides);
            argv.push("test");
            assert!(resolve(init(), Some(init()), &args(&argv)).is_err());
        }
        assert!(
            resolve(
                init(),
                Some(init()),
                &args(&["--process", "unused.json", "test", "/ignored"])
            )
            .is_err()
        );
    }

    #[test]
    fn malformed_overrides_fail_before_backend_execution() {
        for argv in [
            vec!["test"],
            vec!["test", "a\0b"],
            vec!["--cwd", "relative", "test", "/bin/true"],
            vec!["--env", "=value", "test", "/bin/true"],
            vec!["--cap", "CAP_INVENTED", "test", "/bin/true"],
        ] {
            assert!(resolve(init(), None, &args(&argv)).is_err());
        }
    }
}
