//! Bounded NUMA node-list expansion; no kernel or filesystem access.
#![forbid(unsafe_code)]

// Match pinned runc's fixed unix.CPUSet bound for memoryPolicy nodes.
const MAX_NODE_ID: u32 = (std::mem::size_of::<nix::libc::cpu_set_t>() * 8 - 1) as u32;
pub(super) const MAX_NODE_TEXT_BYTES: usize = 4 * 1024 * 1024;

pub(super) fn parse(nodes: &str) -> Result<Vec<u32>, String> {
    if nodes.len() > MAX_NODE_TEXT_BYTES {
        return Err("NUMA node list exceeds 4 MiB supported limit".into());
    }
    let mut ranges = Vec::new();
    for part in nodes
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let number = |text: &str| {
            text.trim()
                .parse::<u32>()
                .map_err(|_| "invalid NUMA node number".to_owned())
        };
        let (start, end) = if let Some((left, right)) = part.split_once('-') {
            (number(left)?, number(right)?)
        } else {
            let node = number(part)?;
            (node, node)
        };
        if start > end {
            return Err("reversed NUMA node range".into());
        }
        if end > MAX_NODE_ID {
            return Err(format!("NUMA node IDs above {MAX_NODE_ID} are unsupported"));
        }
        ranges.push((start, end));
    }
    // Merge before expansion: duplicate ranges cannot multiply allocation/work.
    ranges.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (start, end) in ranges {
        if let Some(last) = merged.last_mut()
            && start <= last.1 + 1
        {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    let count = merged
        .iter()
        .map(|(start, end)| (end - start + 1) as usize)
        .sum();
    let mut result = Vec::with_capacity(count);
    for (start, end) in merged {
        result.extend(start..=end);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_excessive_ranges_ids_text_and_malformed_numbers() {
        for text in [
            "0-4294967295",
            "4294967295",
            "65536",
            "4294967296",
            "2-1",
            "0-1-2",
            "-1",
            "one",
        ] {
            assert!(parse(text).is_err());
        }
        assert!(parse(&(MAX_NODE_ID + 1).to_string()).is_err());
        assert!(parse(&" ".repeat(MAX_NODE_TEXT_BYTES + 1)).is_err());
    }

    #[test]
    fn repeated_overlapping_ranges_expand_each_node_once() {
        let nodes = parse(&format!("0-{MAX_NODE_ID},").repeat(1000)).unwrap();
        assert_eq!(nodes.len(), MAX_NODE_ID as usize + 1);
        assert_eq!(nodes.first(), Some(&0));
        assert_eq!(nodes.last(), Some(&MAX_NODE_ID));
        assert_eq!(parse(" 5, 0-2, 1-4, ,5 ").unwrap(), vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn bounded_ranges_match_set_semantics() {
        for start in 0..=32 {
            for end in start..=32 {
                assert_eq!(
                    parse(&format!("{start}-{end},0,32")).unwrap(),
                    (0..=32)
                        .filter(|node| *node == 0 || *node == 32 || (start..=end).contains(node))
                        .collect::<Vec<_>>()
                );
            }
        }
        assert!(parse(" , , ").unwrap().is_empty());
    }
}
