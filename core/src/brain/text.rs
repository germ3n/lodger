//! The words the lodger writes and says, by what they are for. Each kind has a bank of short
//! handwritten lines. The same banks are the examples the model is shown before it writes a line
//! of that kind itself
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
    /// A name on one of their things, as an anvil would write it.
    ItemName,
    /// A sign on a grave for what they kill most; `{kind}` is filled in.
    GraveSign,
    /// An answer to what the target just typed, in their own name, to them alone.
    Reply,
    /// A line in a book it writes of them, in its own voice: the diary, a letter, its notes,
    /// a page in their book.
    DiaryLine,
}

impl Kind {
    /// Every kind, for the tests that go through them all.
    pub const ALL: [Kind; 10] = [
        Kind::SignWord,
        Kind::ChatLine,
        Kind::KillLine,
        Kind::KillSign,
        Kind::Whisper,
        Kind::AnimalName,
        Kind::GraveSign,
        Kind::Reply,
        Kind::ItemName,
        Kind::DiaryLine,
    ];

    /// Whether a line of this kind may carry digits: only the ones that count what the player
    /// killed. A sign, a whisper, a chat line or a name is words.
    pub fn counts(self) -> bool {
        matches!(self, Kind::KillLine | Kind::KillSign)
    }

    /// How far the model may write a line of this kind: its words, two tokens a word and two
    /// more, digits only for the lines that count.
    pub fn leash(self) -> crate::model::Leash {
        crate::model::Leash {
            max_tokens: self.most_words() * 2 + 2,
            words: self.most_words(),
            digits: self.counts(),
            temperature: 0.0,
        }
    }

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
            Kind::ItemName => &[
                "mine",
                "yours",
                "borrowed",
                "not yours",
                "keep it",
                "for later",
                "again",
                "the other one",
            ],
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
            Kind::DiaryLine => &[
                "i watched you sleep again",
                "you always look back at the door",
                "i know which chest you open first",
                "you were gone a long time today",
                "i walked the way you walk home",
                "you did not see me today",
                "i kept the light on for you",
                "you put it back where i left it",
            ],
            Kind::Reply => &[
                "i heard you",
                "inside",
                "you know who",
                "always",
                "not yet",
                "behind the door",
                "it was me",
                "keep asking",
                "here",
                "dont",
            ],
        }
    }

    /// How many words a line of this kind may run to, the model's leash.
    pub fn most_words(self) -> usize {
        match self {
            Kind::SignWord | Kind::Whisper => 2,
            Kind::AnimalName | Kind::ItemName => 3,
            Kind::KillSign | Kind::GraveSign => 3,
            Kind::ChatLine | Kind::KillLine | Kind::Reply => 8,
            Kind::DiaryLine => 12,
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
            Kind::AnimalName => {
                "The name it gives one of the player's animals, or a villager near them:"
            }
            Kind::ItemName => "The name it writes on one of the player's things:",
            Kind::GraveSign => "The sign on the small grave it raised says:",
            Kind::Reply => "What it types back in the player's own name, answering them:",
            Kind::DiaryLine => {
                "A line in the book it keeps about the player, in its own quiet voice:"
            }
        }
    }
}

/// A written line cleaned and checked for its kind: lower case, letters and single spaces alone
/// (digits too for the kinds that count, [`Kind::counts`]), at most the kind's words, at least
/// one letter, nothing from the blocklist. `None` is a line to drop for a bank line.
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
    let digits = cleaned.chars().any(|c| c.is_ascii_digit());
    let letters = cleaned.chars().any(|c| c.is_ascii_alphabetic());
    if (digits && !kind.counts()) || (!letters && kind != Kind::KillSign) {
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

/// What the target typed, as the writing prompt may quote it: lower case, letters and single
/// spaces, the first eight words. None when nothing is left or a word is blocked, and then the
/// line is not answered at all. Their own words reach the model only this way (owner: a reply "may echo a word").
pub fn heard(typed: &str) -> Option<String> {
    let cleaned: String = typed
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphabetic() { c } else { ' ' })
        .collect();
    let words: Vec<&str> = cleaned.split_whitespace().take(8).collect();
    if words.is_empty()
        || words
            .iter()
            .any(|w| BLOCKED.iter().any(|b| w.contains(b)) || w.len() > 14)
    {
        return None;
    }

    Some(words.join(" "))
}

/// Whether a line repeats one already used this stay: the same, or opening with the same three
/// words (`the salmon remember` and `the salmon remember you`).
pub fn is_repeat(line: &str, used: &[String]) -> bool {
    let head = |l: &str| l.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
    let mine = head(line);
    used.iter()
        .any(|u| u == line || (line.split_whitespace().count() >= 3 && head(u) == mine))
}

/// A line of the kind not yet used this stay, from the seed's place in the bank on; the seed's
/// own when every one was used.
pub fn pick_fresh(kind: Kind, seed: usize, mob: &str, count: u32, used: &[String]) -> String {
    let bank = kind.examples();
    (0..bank.len())
        .map(|i| fill(bank[(seed + i) % bank.len()], mob, count))
        .find(|line| !is_repeat(line, used))
        .unwrap_or_else(|| pick(kind, seed, mob, count))
}

/// One line of the kind, picked by seed, with `{kind}` and `{n}` filled in.
pub fn pick(kind: Kind, seed: usize, mob: &str, count: u32) -> String {
    let bank = kind.examples();
    let line = bank[seed % bank.len()];

    fill(line, mob, count)
}

/// A creature's name for `count` of them: `cow`, `cows`; `sheep` and the others the same in
/// the plural stay so.
pub fn plural(mob: &str, count: u32) -> String {
    let mob = mob.replace('_', " ");
    let same_in_plural = mob.ends_with('s')
        || [
            "sheep",
            "cod",
            "salmon",
            "squid",
            "glow squid",
            "fish",
            "axolotl",
        ]
        .contains(&mob.as_str());
    if count == 1 || same_in_plural {
        mob
    } else if mob == "fox" {
        "foxes".to_string()
    } else if mob == "wolf" {
        "wolves".to_string()
    } else {
        format!("{mob}s")
    }
}

/// `{kind}` and `{n}` filled in; `{kind}s` is the plural for the count ([`plural`]).
pub fn fill(line: &str, mob: &str, count: u32) -> String {
    line.replace("{kind}s", &plural(mob, count))
        .replace("{kind}", &mob.replace('_', " "))
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
    fn what_they_typed_is_cleaned_to_words_and_a_blocked_line_is_not_heard() {
        assert_eq!(
            heard("Who's in my HOUSE?!").as_deref(),
            Some("who s in my house")
        );
        assert_eq!(
            heard("one two three four five six seven eight nine ten").as_deref(),
            Some("one two three four five six seven eight")
        );
        assert_eq!(heard("123 ?!"), None);
        assert_eq!(heard("fuck off"), None);
    }

    #[test]
    fn a_line_used_once_is_not_used_again_and_the_bank_moves_on() {
        // The dev client: `17 salmon`, `the salmon remember`, round and round.
        let used = vec!["17 salmon".to_string(), "the salmon remember".to_string()];
        assert!(is_repeat("17 salmon", &used));
        assert!(
            is_repeat("the salmon remember you", &used),
            "the same opening"
        );
        assert!(!is_repeat("salmon", &used));
        let first = pick_fresh(Kind::KillLine, 0, "salmon", 17, &[]);
        let next = pick_fresh(
            Kind::KillLine,
            0,
            "salmon",
            17,
            std::slice::from_ref(&first),
        );
        assert_ne!(first, next);
    }

    #[test]
    fn digits_are_for_counting_lines_alone() {
        assert_eq!(
            check(Kind::SignWord, "123456"),
            None,
            "seen in the dev client"
        );
        assert_eq!(check(Kind::Whisper, "behind 2"), None);
        assert_eq!(check(Kind::ChatLine, "4 4 4"), None);
        assert_eq!(check(Kind::AnimalName, "7"), None);
        assert_eq!(check(Kind::GraveSign, "12"), None);
        assert_eq!(check(Kind::KillLine, "4 sheep").as_deref(), Some("4 sheep"));
        assert_eq!(check(Kind::KillLine, "44"), None, "a kill line has words");
        assert_eq!(check(Kind::KillSign, "14").as_deref(), Some("14"));
        for kind in Kind::ALL {
            for line in kind.examples() {
                let filled = fill(line, "sheep", 4);
                assert!(
                    check(kind, &filled).is_some(),
                    "{kind:?}'s own example {filled:?} fails its check"
                );
            }
        }
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
        assert_eq!(plural("sheep", 6), "sheep", "the dev client said 6 sheeps");
        assert_eq!(plural("fox", 2), "foxes");
        assert_eq!(plural("wolf", 3), "wolves");
        assert_eq!(plural("cow", 1), "cow");
    }
}
