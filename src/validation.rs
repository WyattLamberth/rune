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
fn check_process(process: &Process) -> Result<()> {
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
pub fn bundle(path: &Path) -> Result<Spec> {
    let spec: Spec = read_json(&path.join("config.json"))?;
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
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_unknown_fields_versions_and_nul() -> Result<()> {
        let dir = tempfile::tempdir()?;
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
        let mut spec = Spec::default();
        spec.set_version("1.2.1".to_owned());
        spec.save(dir.path().join("config.json"))?;
        let frozen = bundle(dir.path())?;
        std::fs::write(dir.path().join("config.json"), b"{}")?;
        assert!(bundle(dir.path()).is_err());
        assert_eq!(frozen, spec);
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
}
