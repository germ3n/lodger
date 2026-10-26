//! The words the lodger writes and says, by what they are for. Each kind has a bank of short
//! handwritten lines; today an event picks one from its bank by seed. In phase 4 the same banks
//! are the examples the model is shown before it writes a line of that kind itself
//! (`brain.text = "generate"`, the default; see `docs/brain.md`, Text), so a sign after a kill
//! is shown sign-after-a-kill examples and a whisper is shown whispers, and `bank` falls back
//! to picking from here when a generated line fails its checks.

/// What a line is for. Each kind reads differently: a sign is one or two words, a chat line in
/// the target's own name is lower case with no punctuation, a whisper is a single word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// One word on a sign at the lodger's place or by the door.
    SignWord,
    /// A line in the target's own name, as if they typed it.
    ChatLine,
    /// A line about what they killed; `{kind}` and `{n}` are filled in.
    KillLine,
    /// A sign counting what they killed; `{kind}` and `{n}` are filled in.
    KillSign,
    /// A whisper to the target from the target: a word or two.
    Whisper,
    /// A name for one of their animals.
    AnimalName,
    /// A sign on a grave for what they kill most; `{kind}` is filled in.
    GraveSign,
}

impl Kind {
    /// The bank for this kind, which doubles as the model's examples.
    pub fn examples(self) -> &'static [&'static str] {
        match self {
            Kind::SignWord => &[
                "here", "wait", "mine", "home", "soon", "yours", "under", "still", "back", "listen",
            ],
            Kind::ChatLine => &[
                "i know",
                "it was me",
                "look behind the door",
                "im still here",
                "under the house",
                "you left it open",
                "come back",
                "i moved it",
                "dont look up",
                "the chest again",
            ],
            Kind::KillLine => &[
                "{n} {kind}s",
                "i saw what you did to the {kind}s",
                "the {kind}s remember",
                "{n} so far",
                "you like the {kind}s",
                "one more {kind}",
            ],
            Kind::KillSign => &["{n} {kind}s", "{n} so far", "{kind}s: {n}", "{n}"],
            Kind::Whisper => &["here", "behind you", "wait", "yours", "soon", "look"],
            Kind::AnimalName => &[
                "yours",
                "mine",
                "wait",
                "home",
                "still",
                "the other one",
                "again",
            ],
            Kind::GraveSign => &["{kind}", "rest", "the {kind}s", "{kind}. rest."],
        }
    }

    /// How many words a line of this kind may run to, for the model's leash in phase 4.
    pub fn most_words(self) -> usize {
        match self {
            Kind::SignWord | Kind::Whisper => 2,
            Kind::AnimalName => 3,
            Kind::KillSign | Kind::GraveSign => 3,
            Kind::ChatLine | Kind::KillLine => 8,
        }
    }
}

/// Words a written line may never carry, since the line appears in the player's own name:
/// slurs, sexual words and self-harm, nothing else. Horror words (kill, blood, dead) are the
/// mod's own subject and stay. A generated line that trips this is dropped for a bank line,
/// and the log keeps what the model wrote.
const BLOCKED: &[&str] = &[
    "rape", "suicide", "nazi", "hitler", "fuck", "shit", "cunt", "bitch", "nigg", "fag", "retard",
    "whore", "slut", "porn", "sex", "dick", "cock", "pussy", "kys",
];

/// What the line is for, in the prompt's words.
impl Kind {
    pub fn purpose(self) -> &'static str {
        match self {
            Kind::SignWord => "The sign it puts up near the house says one or two words:",
            Kind::ChatLine => "What it types in the player's own name, as if they said it:",
            Kind::KillLine => "What it types in the player's own name about what they killed:",
            Kind::KillSign => "The sign it puts up counting what the player killed says:",
            Kind::Whisper => "The whisper the player gets from themselves says:",
            Kind::AnimalName => "The name it gives one of the player's animals:",
            Kind::GraveSign => "The sign on the small grave it raised says:",
        }
    }
}

/// A written line cleaned and checked for its kind: lower case, letters, digits and single
/// spaces alone, at most the kind's words, not empty, nothing from the blocklist. `None` is a
/// line to drop for a bank line.
pub fn check(kind: Kind, written: &str) -> Option<String> {
    let cleaned: String = written
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.is_empty() || words.len() > kind.most_words() {
        return None;
    }
    if words
        .iter()
        .any(|w| BLOCKED.iter().any(|b| w.contains(b)) || w.len() > 14)
    {
        return None;
    }

    Some(words.join(" "))
}

/// One line of the kind, picked by seed, with `{kind}` and `{n}` filled in.
pub fn pick(kind: Kind, seed: usize, mob: &str, count: u32) -> String {
    let bank = kind.examples();
    let line = bank[seed % bank.len()];

    fill(line, mob, count)
}

/// `{kind}` and `{n}` filled in; a plural `{kind}s` of a kind ending in `s` keeps one `s`.
pub fn fill(line: &str, mob: &str, count: u32) -> String {
    let mob = mob.replace('_', " ");
    let same_in_plural =
        mob.ends_with('s') || ["sheep", "cod", "salmon", "squid", "fox"].contains(&mob.as_str());
    let plural = if same_in_plural {
        mob.clone()
    } else {
        format!("{mob}s")
    };
    let singular = if count == 1 { &mob } else { &plural };

    line.replace("{kind}s", singular)
        .replace("{kind}", &mob)
        .replace("{n}", &count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_examples_within_its_word_count() {
        for kind in [
            Kind::SignWord,
            Kind::ChatLine,
            Kind::KillLine,
            Kind::KillSign,
            Kind::Whisper,
            Kind::AnimalName,
            Kind::GraveSign,
        ] {
            assert!(kind.examples().len() >= 4, "{kind:?} has too few examples");
            for line in kind.examples() {
                let filled = fill(line, "cow", 14);
                assert!(
                    filled.split_whitespace().count() <= kind.most_words(),
                    "{kind:?}: {filled:?} is too long"
                );
                assert!(!filled.contains('{'), "{filled}");
            }
        }
    }

    #[test]
    fn a_written_line_is_cleaned_and_a_bad_one_dropped() {
        assert_eq!(check(Kind::SignWord, "Under!").as_deref(), Some("under"));
        assert_eq!(
            check(Kind::ChatLine, "  i   KNOW  ").as_deref(),
            Some("i know")
        );
        assert_eq!(
            check(Kind::SignWord, "one two three"),
            None,
            "too many words"
        );
        assert_eq!(check(Kind::Whisper, ""), None);
        assert_eq!(check(Kind::ChatLine, "fuck you"), None, "blocked");
        assert_eq!(
            check(Kind::ChatLine, "i saw you kill them").as_deref(),
            Some("i saw you kill them"),
            "horror words stay"
        );
        assert_eq!(
            check(Kind::ChatLine, "supercalifragilisticexpialidocious"),
            None
        );
        assert!(!Kind::Whisper.purpose().is_empty());
    }

    #[test]
    fn filling_handles_plurals_and_counts() {
        assert_eq!(fill("{n} {kind}s", "cow", 14), "14 cows");
        assert_eq!(fill("{n} {kind}s", "cow", 1), "1 cow");
        assert_eq!(
            fill("the {kind}s remember", "iron_golem", 3),
            "the iron golems remember"
        );
        assert_eq!(pick(Kind::KillSign, 0, "sheep", 5), "5 sheep");
    }
}
