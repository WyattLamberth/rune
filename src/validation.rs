use anyhow::{Result, bail};
use oci_spec::runtime::{Process, Spec};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const MAX_CONFIG_BYTES: u64 = 4 * 1024 * 1024;

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        bail!("configuration exceeds 4 MiB limit");
    }
    let mut ignored = Vec::new();
    let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
    let value =
        serde_ignored::deserialize(&mut deserializer, |field| ignored.push(field.to_string()))?;
    deserializer.end()?;
    if !ignored.is_empty() {
        bail!("unsupported OCI fields: {}", ignored.join(", "));
    }
    Ok(value)
}
pub(crate) fn check_process(process: &Process) -> Result<()> {
    let args = process
        .args()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("missing process.args"))?;
    if args.is_empty() || args[0].is_empty() {
        bail!("process.args must contain an executable");
    }
    if args.iter().any(|arg| arg.contains('\0')) {
        bail!("process.args must not contain NUL bytes");
    }
    if let Some(env) = process.env()
        && env.iter().any(|entry| {
            entry.contains('\0') || entry.split_once('=').is_none_or(|(key, _)| key.is_empty())
        })
    {
        bail!("process.env must contain NUL-free KEY=value entries");
    }
    if !process.cwd().is_absolute() {
        bail!("process.cwd must be absolute");
    }
    Ok(())
}
pub fn process(path: &Path) -> Result<Process> {
    let process = read_json(path)?;
    check_process(&process)?;
    Ok(process)
}
pub fn stored_process(container_dir: &Path) -> Result<Process> {
    let spec: Spec = read_json(&container_dir.join("rune-config.json"))?;
    if let Some(linux) = spec.linux() {
        libcontainer::process::memory_policy::check_memory_policy(linux.memory_policy())?;
    }
    let process = spec
        .process()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("stored configuration is missing process"))?;
    check_process(process)?;
    Ok(process.clone())
}
pub fn bundle(path: &Path) -> Result<Spec> {
    let mut spec: Spec = read_json(&path.join("config.json"))?;
    let version = spec.version();
    if !matches!(
        version.as_str(),
        "1.0.0" | "1.0.1" | "1.0.2" | "1.1.0" | "1.2.0" | "1.2.1" | "1.3.0"
    ) {
        bail!("unsupported OCI version {version}; accepted schema versions are OCI 1.0–1.3");
    }
    let process = spec
        .process()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("missing process"))?;
    check_process(process)?;
    if spec.root().is_none() {
        bail!("missing root filesystem");
    }
    if spec.linux().is_none() {
        bail!("missing linux configuration");
    }
    if spec.windows().is_some() || spec.solaris().is_some() || spec.vm().is_some() {
        bail!("only Linux containers are supported");
    }
    if spec
        .linux()
        .as_ref()
        .is_some_and(|linux| linux.intel_rdt().is_some())
    {
        bail!("Intel RDT is not supported by rune's current cleanup contract");
    }
    libcontainer::process::memory_policy::check_memory_policy(
        spec.linux().as_ref().unwrap().memory_policy(),
    )?;
    spec.canonicalize_rootfs(path)?;
    if !spec
        .root()
        .as_ref()
        .is_some_and(|root| root.path().is_dir())
    {
        bail!("root filesystem must be a directory");
    }
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_unknown_fields_versions_and_nul() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::create_dir(dir.path().join("rootfs"))?;
        let mut spec = serde_json::to_value(Spec::default())?;
        spec["ociVersion"] = serde_json::json!("1.2.1");
        spec["process"]["args"] = serde_json::json!(["/bin/sh"]);
        let path = dir.path().join("config.json");
        std::fs::write(&path, serde_json::to_vec(&spec)?)?;
        assert!(bundle(dir.path()).is_ok());
        spec["linux"]["inventedIsolation"] = serde_json::json!(true);
        std::fs::write(&path, serde_json::to_vec(&spec)?)?;
        assert!(
            bundle(dir.path())
                .unwrap_err()
                .to_string()
                .contains("unsupported OCI fields")
        );
        spec["linux"]
            .as_object_mut()
            .unwrap()
            .remove("inventedIsolation");
        spec["process"]["args"] = serde_json::json!(["/bin/sh", "a\0b"]);
        std::fs::write(&path, serde_json::to_vec(&spec)?)?;
        assert!(bundle(dir.path()).unwrap_err().to_string().contains("NUL"));
        spec["process"]["args"] = serde_json::json!(["/bin/sh"]);
        spec["ociVersion"] = serde_json::json!("2.0.0");
        std::fs::write(&path, serde_json::to_vec(&spec)?)?;
        assert!(
            bundle(dir.path())
                .unwrap_err()
                .to_string()
                .contains("version")
        );
        Ok(())
    }
    #[test]
    fn snapshot_is_unchanged_when_bundle_is_rewritten() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::create_dir(dir.path().join("rootfs"))?;
        let mut spec = Spec::default();
        spec.set_version("1.2.1".to_owned());
        spec.save(dir.path().join("config.json"))?;
        let frozen = bundle(dir.path())?;
        std::fs::write(dir.path().join("config.json"), b"{}")?;
        assert!(bundle(dir.path()).is_err());
        spec.canonicalize_rootfs(dir.path())?;
        assert_eq!(frozen, spec);
        Ok(())
    }
    #[test]
    fn stored_exec_policy_is_checked_before_tenant_adaptation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut spec = serde_json::to_value(Spec::default())?;
        spec["linux"]["memoryPolicy"] = serde_json::json!({
            "mode":"MPOL_BIND", "nodes":"0-4294967295"
        });
        std::fs::write(
            dir.path().join("rune-config.json"),
            serde_json::to_vec(&spec)?,
        )?;
        assert!(
            stored_process(dir.path())
                .unwrap_err()
                .to_string()
                .contains("unsupported")
        );
        spec["linux"]["memoryPolicy"] = serde_json::json!({"mode":"MPOL_LOCAL"});
        std::fs::write(
            dir.path().join("rune-config.json"),
            serde_json::to_vec(&spec)?,
        )?;
        assert!(stored_process(dir.path()).is_ok());
        Ok(())
    }
    #[test]
    fn bounded_and_malformed_configs_fail() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::write(dir.path().join("config.json"), b"{}")?;
        assert!(bundle(dir.path()).is_err());
        std::fs::write(
            dir.path().join("config.json"),
            vec![b' '; MAX_CONFIG_BYTES as usize + 1],
        )?;
        assert!(
            bundle(dir.path())
                .unwrap_err()
                .to_string()
                .contains("4 MiB")
        );
        Ok(())
    }
    #[test]
    fn stored_exec_process_is_bounded_and_independent_of_bundle_changes() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let spec = Spec::default();
        spec.save(dir.path().join("rune-config.json"))?;
        std::fs::write(dir.path().join("config.json"), b"{}")?;
        assert_eq!(
            stored_process(dir.path())?,
            spec.process().as_ref().unwrap().clone()
        );
        let mut spec = serde_json::to_value(spec)?;
        spec["process"]["unknownPrivilege"] = serde_json::json!(true);
        std::fs::write(
            dir.path().join("rune-config.json"),
            serde_json::to_vec(&spec)?,
        )?;
        assert!(stored_process(dir.path()).is_err());
        std::fs::write(
            dir.path().join("rune-config.json"),
            vec![b' '; MAX_CONFIG_BYTES as usize + 1],
        )?;
        assert!(
            stored_process(dir.path())
                .unwrap_err()
                .to_string()
                .contains("4 MiB")
        );
        Ok(())
    }
}
