//! The books it writes (owner: "a villager offers a book for sale that has been written by the
//! model"; "create more based of that"): its diary of the target, sold by a villager and, if not
//! bought, brought to their feet later; a letter in their chest; its notes on a lectern; a page
//! in a book of theirs. The facts on their pages are the memory's own, so every one is true;
//! the lines in its voice are the model's (`Kind::DiaryLine`), or the bank's when it has none.

use crate::memory::Memory;

/// The longest page, in characters: a book's page holds about this much in the game's font.
const PAGE_CHARS: usize = 220;

fn plain(id: &str) -> String {
    id.rsplit(':').next().unwrap_or(id).replace('_', " ")
}

/// What it has seen them do, a line each, as true as the memory is: where they sleep, the chest
/// they open most, the way they walk home, what they carry, where else they keep, who comes.
pub fn facts(memory: &Memory) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(bed) = memory.home.as_ref().and_then(|h| h.bed) {
        out.push(format!("you sleep {}.", memory.where_words(bed)));
    }
    if let Some(chest) = memory.used_container() {
        out.push(format!(
            "the chest {} is the one you open first. {} times now.",
            memory.where_words(chest.pos),
            chest.opens
        ));
    }
    if memory.routes.iter().any(|r| r.walked >= 2) {
        out.push("you walk home the same way every time.".to_string());
    }
    if let Some((favourite, _)) = memory.favourites.first() {
        out.push(format!(
            "you carry the {} more than anything.",
            plain(favourite)
        ));
    }
    if let Some(door) = memory.usual_door() {
        out.push(format!(
            "you come and go by the door {}.",
            memory.where_words(door)
        ));
    }
    if let Some(site) = memory.sites.first() {
        out.push(format!("you keep a {} away from the house.", site.kind));
    }
    if let Some(name) = memory.people.visitors.first().map(|v| v.name.clone()) {
        out.push(format!("{name} comes to the house."));
    }
    let things = memory.landmarks.iter().filter(|l| l.ok).count();
    if things > 0 {
        out.push(format!(
            "{things} thing{} i did for you to find.",
            if things == 1 { "" } else { "s" }
        ));
    }

    out
}

/// Lines into pages: as many lines as fit on one, never splitting a line.
pub fn into_pages(lines: &[String]) -> Vec<String> {
    let mut pages: Vec<String> = Vec::new();
    for line in lines {
        match pages.last_mut() {
            Some(page) if page.len() + line.len() + 2 <= PAGE_CHARS => {
                page.push_str("\n\n");
                page.push_str(line);
            }
            _ => pages.push(line.clone()),
        }
    }

    pages
}

/// Its diary of them: its own line first, the facts after, its line again at the end.
pub fn diary(memory: &Memory, line: &str) -> Vec<String> {
    let mut lines = vec![line.to_string()];
    lines.extend(facts(memory));
    lines.push("i am still watching.".to_string());

    into_pages(&lines)
}

/// Its notes on a lectern: a page for each day it did something they would notice, the latest
/// on top, and its line on the first page.
pub fn notes(memory: &Memory, line: &str) -> Vec<String> {
    let mut by_day: Vec<(u32, Vec<String>)> = Vec::new();
    for done in memory.landmarks.iter().filter(|l| l.ok) {
        let day = done.day.floor() as u32 + 1;
        let words = crate::events::find(&done.event)
            .map(|e| e.menu_words().to_string())
            .unwrap_or_else(|| done.event.replace('_', " "));
        match by_day.iter_mut().find(|(d, _)| *d == day) {
            Some((_, lines)) => lines.push(words),
            None => by_day.push((day, vec![words])),
        }
    }
    by_day.sort_by_key(|(day, _)| std::cmp::Reverse(*day));
    let mut pages = vec![line.to_string()];
    for (day, lines) in by_day.into_iter().take(30) {
        pages.extend(into_pages(&[format!("day {day}."), lines.join(". ") + "."]));
    }

    pages
}

/// The author it signs with: their own name.
pub fn signed(memory: &Memory) -> String {
    if memory.target_name.is_empty() {
        "you".to_string()
    } else {
        memory.target_name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_go_onto_pages_whole_and_the_diary_tells_true_things() {
        let lines: Vec<String> = (0..12)
            .map(|i| format!("line {i} of a few words"))
            .collect();
        let pages = into_pages(&lines);
        assert!(pages.len() > 1 && pages.iter().all(|p| p.len() <= PAGE_CHARS));
        assert!(pages.iter().all(|p| !p.starts_with('\n')));

        let mut memory = Memory::default();
        memory.home = Some(crate::memory::Home {
            center: [0, 64, 0],
            bed: Some([1, 64, 1]),
            floor_y: 64,
            ..Default::default()
        });
        memory.favourites = vec![("minecraft:iron_pickaxe".to_string(), 30)];
        let book = diary(&memory, "i watched you today");
        let text = book.join(" ");
        assert!(text.starts_with("i watched you today"));
        assert!(text.contains("you sleep"), "{text}");
        assert!(text.contains("iron pickaxe"), "{text}");
    }

    #[test]
    fn the_notes_keep_a_page_a_day_the_latest_first() {
        let mut memory = Memory::default();
        for (event, day) in [("sign", 1.2), ("cairn", 3.5), ("hole", 3.7)] {
            memory.landmarks.push(crate::memory::Done {
                event: event.to_string(),
                day,
                ok: true,
                ..Default::default()
            });
        }
        let pages = notes(&memory, "what i saw");
        assert_eq!(pages[0], "what i saw");
        assert!(pages[1].starts_with("day 4."), "{pages:?}");
        assert!(pages.iter().any(|p| p.starts_with("day 2.")));
    }
}
