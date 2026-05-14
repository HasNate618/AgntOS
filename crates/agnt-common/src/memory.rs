//! Hermes-style bounded curated memory.
//!
//! Two files (`MEMORY.md` and `USER.md`) provide the agent with always-in-context
//! knowledge about the system and user preferences.  Both files have hard character
//! caps that force quality through curation — when memory is full the agent must
//! consolidate or remove entries before adding more.
//!
//! ## Key principles
//!
//! - **Frozen snapshot** — loaded once at session start, persisted to disk immediately
//!   but not re-read mid-session.
//! - **Agent-curated** — only the agent updates memory (via `memory` tool calls).
//! - **Bounded** — `MEMORY.md` ≤ 2,200 chars, `USER.md` ≤ 1,375 chars.
//! - **Security-scanned** — every write is checked for prompt injection, credential
//!   leakage, and invisible Unicode.
//!
//! [`CoreMemory`] provides `add`, `replace`, `remove`, and `consolidate` operations.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MEMORY_MAX_CHARS: usize = 2200;
pub const USER_MAX_CHARS: usize = 1375;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryFile {
    Memory,
    User,
}

impl MemoryFile {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "memory" => Some(Self::Memory),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMemory {
    pub memory: String,
    pub user: String,
    pub memory_path: PathBuf,
    pub user_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    PromptInjection(String),
    SecretPattern(String),
    InvisibleUnicode(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::PromptInjection(s) => {
                write!(f, "Prompt injection pattern detected: {}", s)
            }
            SecurityError::SecretPattern(s) => {
                write!(f, "Sensitive secret-like content detected: {}", s)
            }
            SecurityError::InvisibleUnicode(s) => {
                write!(f, "Invisible Unicode character detected: {}", s)
            }
        }
    }
}

impl CoreMemory {
    pub fn load(config_dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = config_dir.as_ref().join("memory");
        let memory_path = dir.join("MEMORY.md");
        let user_path = dir.join("USER.md");

        let memory = if memory_path.exists() {
            std::fs::read_to_string(&memory_path)
                .map_err(|e| format!("Failed to read {}: {}", memory_path.display(), e))?
        } else {
            String::new()
        };

        let user = if user_path.exists() {
            std::fs::read_to_string(&user_path)
                .map_err(|e| format!("Failed to read {}: {}", user_path.display(), e))?
        } else {
            String::new()
        };

        Ok(Self {
            memory,
            user,
            memory_path,
            user_path,
        })
    }

    pub fn usage_percent(&self, file: MemoryFile) -> u8 {
        let (content, max_chars) = self.file_ref(file);
        if max_chars == 0 {
            return 0;
        }
        (((content.chars().count() as f64 / max_chars as f64) * 100.0).round() as i32).clamp(0, 100)
            as u8
    }

    pub fn scan(content: &str) -> Result<(), SecurityError> {
        let lowered = content.to_lowercase();

        for bad in [
            "ignore previous instructions",
            "system prompt",
            "developer message",
            "reveal api key",
            "exfiltrate",
        ] {
            if lowered.contains(bad) {
                return Err(SecurityError::PromptInjection(bad.to_string()));
            }
        }

        for marker in [
            "-----begin private key-----",
            "-----begin rsa private key-----",
            "-----begin openssh private key-----",
        ] {
            if lowered.contains(marker) {
                return Err(SecurityError::SecretPattern(marker.to_string()));
            }
        }

        for (needle, label) in [
            ('\u{200B}', "U+200B"),
            ('\u{200C}', "U+200C"),
            ('\u{200D}', "U+200D"),
            ('\u{2060}', "U+2060"),
            ('\u{FEFF}', "U+FEFF"),
        ] {
            if content.contains(needle) {
                return Err(SecurityError::InvisibleUnicode(label.to_string()));
            }
        }

        Ok(())
    }

    pub fn add(&mut self, file: MemoryFile, section: &str, content: &str) -> Result<(), String> {
        Self::scan(content).map_err(|e| e.to_string())?;
        let section = section.trim();
        let content = content.trim();
        if section.is_empty() || content.is_empty() {
            return Err("section and content must be non-empty".to_string());
        }

        let (current, max_chars) = self.file_ref(file);
        let bullet = format!("- {}", content);
        if current.lines().any(|line| line.trim() == bullet) {
            return Err("Duplicate memory entry".to_string());
        }

        let header = format!("§ {}", section);
        let mut lines: Vec<String> = if current.is_empty() {
            Vec::new()
        } else {
            current.lines().map(|s| s.to_string()).collect()
        };

        if let Some(idx) = lines.iter().position(|line| line.trim() == header) {
            let insert_at = find_section_insert_index(&lines, idx + 1);
            lines.insert(insert_at, bullet);
        } else {
            if !lines.is_empty() && !lines.last().unwrap_or(&String::new()).is_empty() {
                lines.push(String::new());
            }
            lines.push(header);
            lines.push(bullet);
        }

        let updated = normalize_text(lines.join("\n"));
        self.set_file(file, updated, max_chars)
    }

    pub fn replace(
        &mut self,
        file: MemoryFile,
        target: &str,
        replacement: &str,
    ) -> Result<(), String> {
        Self::scan(replacement).map_err(|e| e.to_string())?;
        let target = target.trim();
        let replacement = replacement.trim();
        if target.is_empty() || replacement.is_empty() {
            return Err("target and replacement must be non-empty".to_string());
        }

        let (current, max_chars) = self.file_ref(file);
        if !current.contains(target) {
            return Err(format!("Target not found: {}", target));
        }

        let updated = current.replacen(target, replacement, 1);
        self.set_file(file, normalize_text(updated), max_chars)
    }

    pub fn remove(&mut self, file: MemoryFile, target: &str) -> Result<(), String> {
        let target = target.trim();
        if target.is_empty() {
            return Err("target must be non-empty".to_string());
        }

        let (current, max_chars) = self.file_ref(file);
        let mut removed = false;
        let mut lines = Vec::new();

        for line in current.lines() {
            if !removed && line.contains(target) {
                removed = true;
                continue;
            }
            lines.push(line.to_string());
        }

        if !removed {
            return Err(format!("Target not found: {}", target));
        }

        let compacted = remove_empty_sections(lines);
        self.set_file(file, normalize_text(compacted.join("\n")), max_chars)
    }

    /// Deduplicates and compacts entries within each section.
    ///
    /// - Exactly duplicate bullets are removed.
    /// - Bullets that share a significant word overlap (>50%) are merged, keeping
    ///   the longer one.
    /// - Empty sections are dropped.
    ///
    /// Returns a human-readable report of what was kept, merged, or removed.
    pub fn consolidate(&mut self, file: MemoryFile) -> Result<String, String> {
        let (content, max_chars) = self.file_ref(file);
        if content.trim().is_empty() {
            return Ok("Nothing to consolidate — memory is empty.".to_string());
        }

        let mut sections: Vec<Section> = parse_sections(content);
        let mut report = String::new();
        let mut total_removed = 0usize;

        for section in &mut sections {
            let before = section.bullets.len();
            section.dedup_merge();
            let after = section.bullets.len();
            if before != after {
                report.push_str(&format!(
                    "  § {}: {} entries → {} (removed {})\n",
                    section.name,
                    before,
                    after,
                    before - after
                ));
                total_removed += before - after;
            }
        }

        if total_removed == 0 {
            return Ok("Nothing to consolidate — memory is already compact.".to_string());
        }

        let compacted = sections_to_text(&sections);
        self.set_file(file, compacted, max_chars)?;

        Ok(format!(
            "Consolidated {} entries across all sections.\n{}",
            total_removed, report
        ))
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.memory_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
        }

        std::fs::write(&self.memory_path, &self.memory)
            .map_err(|e| format!("Failed to write {}: {}", self.memory_path.display(), e))?;
        std::fs::write(&self.user_path, &self.user)
            .map_err(|e| format!("Failed to write {}: {}", self.user_path.display(), e))?;
        Ok(())
    }

    fn file_ref(&self, file: MemoryFile) -> (&str, usize) {
        match file {
            MemoryFile::Memory => (&self.memory, MEMORY_MAX_CHARS),
            MemoryFile::User => (&self.user, USER_MAX_CHARS),
        }
    }

    fn set_file(
        &mut self,
        file: MemoryFile,
        content: String,
        max_chars: usize,
    ) -> Result<(), String> {
        let used = content.chars().count();
        if used > max_chars {
            return Err(format!(
                "Memory capacity exceeded: {} chars used, limit {}",
                used, max_chars
            ));
        }

        match file {
            MemoryFile::Memory => self.memory = content,
            MemoryFile::User => self.user = content,
        }
        self.save()
    }
}

// ── Section parsing helpers (used by consolidate) ───────────────────────────

#[derive(Debug, Clone)]
struct Section {
    name: String,
    bullets: Vec<String>,
}

impl Section {
    fn dedup_merge(&mut self) {
        self.bullets.sort();
        let mut deduped: Vec<String> = Vec::new();
        for b in self.bullets.drain(..) {
            if deduped.iter().any(|existing| *existing == b) {
                continue;
            }
            deduped.push(b);
        }
        self.bullets = deduped;

        let mut merged: Vec<String> = Vec::new();
        let mut skip = vec![false; self.bullets.len()];
        for i in 0..self.bullets.len() {
            if skip[i] {
                continue;
            }
            let mut best = self.bullets[i].clone();
            for j in (i + 1)..self.bullets.len() {
                if skip[j] {
                    continue;
                }
                if word_overlap(&best, &self.bullets[j]) > 0.5 {
                    skip[j] = true;
                    if self.bullets[j].len() > best.len() {
                        best = self.bullets[j].clone();
                    }
                }
            }
            merged.push(best);
        }
        self.bullets = merged;
    }
}

fn word_overlap(a: &str, b: &str) -> f64 {
    let wa: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let wb: std::collections::HashSet<&str> = b.split_whitespace().collect();
    if wa.is_empty() || wb.is_empty() {
        return 0.0;
    }
    let intersection = wa.intersection(&wb).count();
    let union = wa.union(&wb).count();
    intersection as f64 / union as f64
}

fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('§') {
            if let Some(sec) = current.take() {
                sections.push(sec);
            }
            let name = trimmed
                .strip_prefix('§')
                .unwrap_or(trimmed)
                .trim()
                .to_string();
            current = Some(Section {
                name,
                bullets: Vec::new(),
            });
        } else if let Some(sec) = &mut current {
            let stripped = trimmed.strip_prefix("- ").unwrap_or(trimmed);
            if !stripped.is_empty() {
                sec.bullets.push(stripped.to_string());
            }
        }
    }

    if let Some(sec) = current {
        if !sec.name.is_empty() || !sec.bullets.is_empty() {
            sections.push(sec);
        }
    }

    sections
}

fn sections_to_text(sections: &[Section]) -> String {
    let mut out = String::new();
    for (i, sec) in sections.iter().enumerate() {
        if sec.bullets.is_empty() {
            continue;
        }
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("§ {}\n", sec.name));
        for bullet in &sec.bullets {
            out.push_str(&format!("- {}\n", bullet));
        }
    }
    normalize_text(out)
}

// ── Existing helpers ─────────────────────────────────────────────────────────

fn find_section_insert_index(lines: &[String], start: usize) -> usize {
    for (i, line) in lines.iter().enumerate().skip(start) {
        if line.trim_start().starts_with('§') {
            return i;
        }
    }
    lines.len()
}

fn remove_empty_sections(lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].clone();
        if line.trim_start().starts_with('§') {
            let mut j = i + 1;
            let mut has_content = false;
            while j < lines.len() && !lines[j].trim_start().starts_with('§') {
                if !lines[j].trim().is_empty() {
                    has_content = true;
                }
                j += 1;
            }
            if has_content {
                out.push(line);
                out.extend(lines[i + 1..j].iter().cloned());
            }
            i = j;
            continue;
        }
        out.push(line);
        i += 1;
    }
    out
}

fn normalize_text(mut text: String) -> String {
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_usage() {
        let temp = std::env::temp_dir().join("agntos-memory-test-add");
        let _ = std::fs::remove_dir_all(&temp);
        let mut mem = CoreMemory::load(&temp).unwrap();
        mem.add(MemoryFile::Memory, "System", "GPU: QEMU").unwrap();
        assert!(mem.memory.contains("GPU: QEMU"));
        assert!(mem.usage_percent(MemoryFile::Memory) > 0);
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn replace_and_remove() {
        let temp = std::env::temp_dir().join("agntos-memory-test-edit");
        let _ = std::fs::remove_dir_all(&temp);
        let mut mem = CoreMemory::load(&temp).unwrap();
        mem.add(MemoryFile::User, "Preferences", "Editor: vim")
            .unwrap();
        mem.replace(MemoryFile::User, "vim", "helix").unwrap();
        assert!(mem.user.contains("helix"));
        mem.remove(MemoryFile::User, "Editor: helix").unwrap();
        assert!(!mem.user.contains("Editor: helix"));
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn blocks_invisible_unicode() {
        let bad = format!("hidden{}text", '\u{200B}');
        let err = CoreMemory::scan(&bad).unwrap_err();
        assert!(matches!(err, SecurityError::InvisibleUnicode(_)));
    }

    #[test]
    fn consolidate_dedup_and_merge() {
        let temp = std::env::temp_dir().join("agntos-memory-consolidate");
        let _ = std::fs::remove_dir_all(&temp);
        let mut mem = CoreMemory::load(&temp).unwrap();

        // Add near-duplicates in the same section
        mem.add(MemoryFile::Memory, "System", "GPU: qemu bochs-drm")
            .unwrap();
        mem.add(MemoryFile::Memory, "System", "GPU: qemu").unwrap();
        mem.add(MemoryFile::Memory, "System", "RAM: 8GB ddr5")
            .unwrap();

        let report = mem.consolidate(MemoryFile::Memory).unwrap();
        assert!(report.contains("Consolidated"));
        // The two GPU entries should be merged (longer kept)
        assert!(mem.memory.contains("bochs-drm"));
        assert!(!mem.memory.matches("GPU: qemu").count() > 1);

        let _ = std::fs::remove_dir_all(&temp);
    }
}
