//! Read-only pinned Unicode 16 catalogue adapted from Katran 14e7373.
//! Rust code MIT; bundled data Unicode License V3 (data/LICENSE.txt).
use std::sync::OnceLock;
pub const UNICODE_VERSION: &str = "16.0";
#[derive(Debug)]
pub struct EmojiEntry {
    pub text: String,
    pub name: String,
    pub group: String,
    pub subgroup: String,
}
pub fn emoji_entries() -> &'static Vec<EmojiEntry> {
    static ENTRIES: OnceLock<Vec<EmojiEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut entries = Vec::new();
        let mut group = "";
        let mut subgroup = "";
        for line in include_str!("data/emoji-test.txt").lines() {
            if let Some(value) = line.strip_prefix("# group: ") {
                group = value;
                continue;
            }
            if let Some(value) = line.strip_prefix("# subgroup: ") {
                subgroup = value;
                continue;
            }
            let Some((fields, comment)) = line.split_once('#') else {
                continue;
            };
            let Some((codepoints, qualification)) = fields.split_once(';') else {
                continue;
            };
            if qualification.trim() != "fully-qualified" {
                continue;
            }
            let text: String = codepoints
                .split_whitespace()
                .map(|cp| {
                    char::from_u32(u32::from_str_radix(cp, 16).expect("Unicode codepoint"))
                        .expect("Unicode scalar")
                })
                .collect();
            let mut words = comment.split_whitespace();
            let _display = words.next();
            let _version = words.next();
            let name = words.collect::<Vec<_>>().join(" ");
            assert!(!group.is_empty() && !subgroup.is_empty() && !name.is_empty());
            entries.push(EmojiEntry {
                text,
                name,
                group: group.into(),
                subgroup: subgroup.into(),
            });
        }
        entries
    })
}
pub fn emoji_groups() -> &'static Vec<String> {
    static GROUPS: OnceLock<Vec<String>> = OnceLock::new();
    GROUPS.get_or_init(|| {
        let mut groups = Vec::new();
        for entry in emoji_entries() {
            if groups.last() != Some(&entry.group) {
                groups.push(entry.group.clone());
            }
        }
        groups
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor {
    offset: usize,
    query: String,
    group: Option<String>,
    version: &'static str,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLimit,
    CursorMismatch,
}
#[derive(Debug)]
pub struct Divider {
    pub before: usize,
    pub level: &'static str,
    pub label: &'static str,
}
#[derive(Debug)]
pub struct Page {
    pub entries: Vec<&'static EmojiEntry>,
    pub dividers: Vec<Divider>,
    pub next: Option<Cursor>,
    pub total: usize,
}

/// The opaque cursor is session-local, not a persisted wire format.
pub fn page(
    query: &str,
    group: Option<&str>,
    cursor: Option<&Cursor>,
    limit: usize,
) -> Result<Page, Error> {
    if !(1..=1000).contains(&limit) {
        return Err(Error::InvalidLimit);
    }
    let query = query.to_lowercase();
    let filtered: Vec<_> = emoji_entries()
        .iter()
        .filter(|entry| {
            group.is_none_or(|g| entry.group == g)
                && (entry.text.contains(&query)
                    || entry.name.to_lowercase().contains(&query)
                    || entry.group.to_lowercase().contains(&query)
                    || entry.subgroup.to_lowercase().contains(&query))
        })
        .collect();
    let start = if let Some(c) = cursor {
        if c.query != query
            || c.group.as_deref() != group
            || c.version != UNICODE_VERSION
            || c.offset > filtered.len()
        {
            return Err(Error::CursorMismatch);
        }
        c.offset
    } else {
        0
    };
    let end = start.saturating_add(limit).min(filtered.len());
    let mut dividers = Vec::new();
    for (before, entry) in filtered[start..end].iter().enumerate() {
        let previous = (start + before)
            .checked_sub(1)
            .and_then(|i| filtered.get(i));
        let changed = previous.is_none_or(|p| p.group != entry.group);
        if changed {
            dividers.push(Divider {
                before,
                level: "group",
                label: entry.group.as_str(),
            });
        }
        if changed || previous.is_none_or(|p| p.subgroup != entry.subgroup) {
            dividers.push(Divider {
                before,
                level: "subgroup",
                label: entry.subgroup.as_str(),
            });
        }
    }
    let next = (end < filtered.len()).then(|| Cursor {
        offset: end,
        query,
        group: group.map(str::to_owned),
        version: UNICODE_VERSION,
    });
    Ok(Page {
        entries: filtered[start..end].to_vec(),
        dividers,
        next,
        total: filtered.len(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_sequences_and_qualification() {
        assert!(emoji_entries().len() > 3000);
        for sequence in ["🐄", "👨‍👩‍👧‍👦", "🇰🇿"] {
            assert!(emoji_entries().iter().any(|e| e.text == sequence));
        }
        assert!(
            emoji_entries()
                .iter()
                .all(|e| !e.name.is_empty() && !e.group.is_empty() && !e.subgroup.is_empty())
        );
    }
    #[test]
    fn scoped_pages_and_category_boundaries() {
        let all = page("", None, None, 1000).unwrap();
        let first = page("", None, None, 1).unwrap();
        let second = page("", None, first.next.as_ref(), 1).unwrap();
        assert_eq!(first.entries[0].text, all.entries[0].text);
        assert_eq!(second.entries[0].text, all.entries[1].text);
        assert_eq!(first.dividers.len(), 2);
        let previous = all.entries[0];
        let current = all.entries[1];
        assert_eq!(
            second.dividers.is_empty(),
            previous.group == current.group && previous.subgroup == current.subgroup
        );
        assert_eq!(
            page("cow", None, first.next.as_ref(), 1).unwrap_err(),
            Error::CursorMismatch
        );
        assert_eq!(
            page("", Some(&emoji_groups()[0]), first.next.as_ref(), 1).unwrap_err(),
            Error::CursorMismatch
        );
        let group = &emoji_groups()[0];
        assert!(
            page("", Some(group), None, 1000)
                .unwrap()
                .entries
                .iter()
                .all(|e| &e.group == group)
        );
        assert_eq!(page("", None, None, 0).unwrap_err(), Error::InvalidLimit);
    }
}
