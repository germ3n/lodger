//! The situation text the model reads: a fixed frame, the memory's lines, and the way a question
//! and its choices are appended (`docs/brain.md`, The situation text). Everything in it is text
//! the mod wrote, but for the last line of chat about it, quoted as the player said it (its
//! words filtered before anything it writes is shown).

/// The frame around the memory's lines, the same every time.
pub const FRAME: &str = "You are a quiet presence living unseen in a player's house in a game. \
You never chase, never hurt, and never hurry. You learn the player's habits and act on them \
when they are away or asleep, to make them feel known. You want one thing from them.\n\n\
What you know:\n";

/// The frame and the lines, one a line.
pub fn situation(lines: &[String]) -> String {
    let mut text = String::from(FRAME);
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
pub fn writing_prompt(lines: &[String], purpose: &str, examples: &[&str]) -> String {
    let mut text = situation(lines);
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
        let text = situation(&["The player sleeps in a house.".to_string()]);
        assert!(text.starts_with("You are a quiet presence"));
        assert!(text.ends_with("What you know:\nThe player sleeps in a house.\n"));
    }

    #[test]
    fn a_writing_prompt_ends_on_an_open_line_after_the_examples() {
        let text = writing_prompt(&[], "The sign by the door says:", &["here", "wait"]);
        assert!(text.contains("- here\n- wait\n-"), "{text}");
        assert!(text.ends_with("\n-"), "no space after the dash: {text:?}");
    }
}
