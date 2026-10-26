//! The decision log: every ask, its situation, its menu, its pick and what became of it, in
//! `config/lodger/decisions.log`, so a strange act can be traced to what the lodger believed.
//! See `docs/brain.md`, The decision log. Kept in memory and flushed to the file on every
//! write; a stay without a log directory (the tests) keeps the lines alone.

use crate::events::Event;
use craftkit::world::BlockPos;
use std::io::Write;
use std::path::PathBuf;

pub struct Log {
    path: Option<PathBuf>,
    pub lines: Vec<String>,
    /// The bytes written so far, for the rotation at `log.keep_mb`.
    written: u64,
    keep_bytes: u64,
    enabled: bool,
}

impl Log {
    pub fn new(path: Option<PathBuf>, keep_mb: u32, enabled: bool) -> Self {
        Self {
            path,
            lines: Vec::new(),
            written: 0,
            keep_bytes: keep_mb as u64 * 1024 * 1024,
            enabled,
        }
    }

    pub fn decided(
        &mut self,
        stage: u8,
        day: f32,
        situation: &[String],
        menu: &[&Event],
        chosen: &str,
        pos: Option<BlockPos>,
        brain: &str,
    ) {
        let mut entry = format!("--- stage={stage} day={day:.2} brain={brain}\n");
        for line in situation {
            entry.push_str("  ");
            entry.push_str(line);
            entry.push('\n');
        }
        entry.push_str("  menu:");
        for event in menu {
            entry.push(' ');
            entry.push_str(event.id);
        }
        entry.push_str(&format!("\n  chose {chosen}"));
        if let Some(pos) = pos {
            entry.push_str(&format!(" at ({},{},{})", pos[0], pos[1], pos[2]));
        }
        self.write(entry);
    }

    pub fn acted(&mut self, event: &str, pos: Option<BlockPos>, commands: usize) {
        self.write(format!(
            "  acted {event}{} with {commands} command(s)",
            pos.map(|p| format!(" at ({},{},{})", p[0], p[1], p[2]))
                .unwrap_or_default()
        ));
    }

    pub fn reported(&mut self, event: &str, done: bool, reason: &str) {
        self.write(format!(
            "  {event}: {}{}",
            if done { "done" } else { "not done" },
            if reason.is_empty() {
                String::new()
            } else {
                format!(", {reason}")
            }
        ));
    }

    /// A line the model wrote, with its kind and whether it was kept.
    pub fn wrote(&mut self, kind: &str, written: &str, kept: Option<&str>) {
        self.write(format!(
            "  wrote ({kind}): {written:?} -> {}",
            kept.map(|k| format!("{k:?}"))
                .unwrap_or_else(|| "dropped".to_string())
        ));
    }

    pub fn note(&mut self, text: &str) {
        self.write(format!("  {text}"));
    }

    fn write(&mut self, entry: String) {
        if !self.enabled {
            return;
        }

        if self.lines.len() >= 256 {
            self.lines.remove(0);
        }
        self.lines.push(entry.clone());

        let Some(path) = &self.path else { return };

        if self.written > self.keep_bytes {
            let _ = std::fs::rename(path, path.with_extension("log.1"));
            self.written = 0;
        }

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{entry}");
            self.written += entry.len() as u64 + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::find;

    #[test]
    fn a_decision_is_one_entry_with_the_situation_the_menu_and_the_pick() {
        let dir = std::env::temp_dir().join(format!("lodger-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("decisions.log");
        let mut log = Log::new(Some(path.clone()), 16, true);

        log.decided(
            2,
            4.5,
            &["The player is indoors.".to_string()],
            &[find("wait").unwrap(), find("knock").unwrap()],
            "knock",
            Some([1, 2, 3]),
            "rules",
        );
        log.acted("knock", Some([1, 2, 3]), 1);
        log.reported("knock", true, "");

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("--- stage=2 day=4.50 brain=rules"), "{text}");
        assert!(text.contains("  The player is indoors."), "{text}");
        assert!(text.contains("  menu: wait knock"), "{text}");
        assert!(text.contains("  chose knock at (1,2,3)"), "{text}");
        assert!(text.contains("  knock: done"), "{text}");
        assert_eq!(log.lines.len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_disabled_log_writes_nothing_and_a_pathless_one_keeps_lines_alone() {
        let mut off = Log::new(None, 16, false);
        off.note("nothing");
        assert!(off.lines.is_empty());

        let mut memory_only = Log::new(None, 16, true);
        memory_only.note("kept");
        assert_eq!(memory_only.lines, vec!["  kept".to_string()]);
    }
}
