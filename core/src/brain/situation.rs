//! The situation text the model reads: a fixed frame, the memory's lines, and the way a question
//! and its choices are appended (`docs/brain.md`, The situation text). Everything in it is text
//! the mod wrote, but for the last line of chat about it, quoted as the player said it (its
//! words filtered before anything it writes is shown).

/// The frame's start, the same every stage: told of it, as the lines and the questions are ("it
/// chose to...", "Should it do something soon?"); it said "you" before, beside lines that say "it".
pub const FRAME: &str = "This is a story about a lodger in a game: a presence that has moved into \
a player's world uninvited. It learns the player's habits and uses them, so the player feels \
known.";

/// Where the stay is, a sentence for each stage. The frame said it never chased, never hurt
/// and acted only while they were away or asleep, which held the model back from the very
/// events the later stages offer (owner: "should this mention the never chase, never hurt
/// thing?"); when it acts is its own question.
pub const STAGES: [&str; 5] = [
    "It has only just arrived, and nothing it does yet could not be explained away.",
    "It stays unseen; what it does could still be chance.",
    "It no longer minds being noticed.",
    "It shows itself now, and wants the player afraid.",
    "It has stopped hiding. It comes for the player.",
];

/// The frame for a stage (1 to 5; 0, before it moves in, reads as the first), up to the lines.
pub fn frame(stage: u8) -> String {
    let stage = STAGES[(stage.clamp(1, 5) - 1) as usize];
    format!("{FRAME} {stage} It wants one thing from them.\n\nWhat it knows:\n")
}

/// The frame and the lines, one a line.
pub fn situation(lines: &[String], stage: u8) -> String {
    let mut text = frame(stage);
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }

    text
}

/// The prompt for a written line of `kind`: the situation, what the line is for, the kind's
/// examples one a line, and an open line for the model to finish. The open line ends on `-`
/// with no space after it: Qwen's tokens carry a word's space at their front (` here`), so a
/// prompt ending in a space leaves the model only the tokens with none, digits among the
/// likeliest (the dev client's first sign said `123456`).
pub fn writing_prompt(lines: &[String], stage: u8, purpose: &str, examples: &[&str]) -> String {
    let mut text = situation(lines, stage);
    text.push('\n');
    text.push_str(purpose);
    text.push_str(" Each line is a few plain words, lower case, no punctuation.\n");
    for example in examples {
        text.push_str("- ");
        text.push_str(example);
        text.push('\n');
    }
    text.push('-');

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_situation_is_the_frame_and_the_lines() {
        let text = situation(&["The player sleeps in a house.".to_string()], 2);
        assert!(text.starts_with("This is a story about a lodger"));
        assert!(text.contains(STAGES[1]), "the stage's sentence: {text}");
        assert!(text.ends_with("What it knows:\nThe player sleeps in a house.\n"));
    }

    #[test]
    fn each_stage_has_its_sentence_and_none_says_it_never_hurts() {
        for stage in 1..=5 {
            let text = frame(stage);
            assert!(text.contains(STAGES[stage as usize - 1]));
            assert!(
                !text.contains("never hurt") && !text.contains("You "),
                "{text}"
            );
        }
        assert_eq!(frame(0), frame(1), "before it moves in, as the first");
        assert_eq!(frame(9), frame(5));
    }

    #[test]
    fn a_writing_prompt_ends_on_an_open_line_after_the_examples() {
        let text = writing_prompt(&[], 3, "The sign by the door says:", &["here", "wait"]);
        assert!(text.contains("- here\n- wait\n-"), "{text}");
        assert!(text.ends_with("\n-"), "no space after the dash: {text:?}");
    }
}
