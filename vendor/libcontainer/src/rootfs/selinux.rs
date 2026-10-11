// rune patch to libcontainer 0.7.0; covered by the upstream Apache-2.0 license.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelMode<'a> {
    None,
    Mount(&'a str),
    Xattr(&'a str),
}

pub fn label_mode<'a>(typ: Option<&str>, label: Option<&'a str>, enabled: bool) -> LabelMode<'a> {
    let Some(label) = label.filter(|label| !label.is_empty()) else {
        return LabelMode::None;
    };
    if !enabled {
        return LabelMode::None;
    }
    match typ {
        Some("proc" | "sysfs" | "cgroup" | "cgroup2") => LabelMode::None,
        Some("mqueue") => LabelMode::Xattr(label),
        _ => LabelMode::Mount(label),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mqueue_uses_inode_label_while_tmpfs_keeps_mount_label() {
        assert_eq!(
            label_mode(Some("mqueue"), Some("label"), true),
            LabelMode::Xattr("label")
        );
        assert_eq!(
            label_mode(Some("tmpfs"), Some("label"), true),
            LabelMode::Mount("label")
        );
        assert_eq!(
            label_mode(Some("cgroup2"), Some("label"), true),
            LabelMode::None
        );
        assert_eq!(
            label_mode(Some("proc"), Some("label"), true),
            LabelMode::None
        );
        assert_eq!(
            label_mode(Some("sysfs"), Some("label"), true),
            LabelMode::None
        );
    }
    #[test]
    fn absent_empty_or_disabled_labels_add_no_mount_option() {
        assert_eq!(label_mode(Some("mqueue"), None, true), LabelMode::None);
        assert_eq!(label_mode(Some("tmpfs"), Some(""), true), LabelMode::None);
        assert_eq!(
            label_mode(Some("tmpfs"), Some("label"), false),
            LabelMode::None
        );
    }
}
