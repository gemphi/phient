//! Every trigram of every line: the one iteration the measures sum over.
//!
//! Three of them wrote it out - the counted baseline's held-out
//! perplexity, the harness's two-word perplexity and its gamma sweep - and
//! each also guarded `toks.len() < 3`, which never fired: `windows(3)`
//! yields nothing from a shorter line. The lines come in already cut, so
//! the caller says what a line is (tokenised, or padded with `<s>`), and
//! this says what a trigram is.

/// Calls `f` with each three words in turn, over every line.
pub fn each(lines: impl Iterator<Item = Vec<String>>, mut f: impl FnMut(&str, &str, &str)) {
    for line in lines {
        for w in line.windows(3) {
            f(&w[0], &w[1], &w[2]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::each;

    fn line(words: &str) -> Vec<String> {
        words.split(' ').map(str::to_string).collect()
    }

    #[test]
    fn every_three_words_in_turn_and_none_from_a_short_line() {
        let mut said = Vec::new();
        let lines = [line("a b c d"), line("e f"), line("g h i")];
        each(lines.into_iter(), |a, b, c| said.push(format!("{a}{b}{c}")));
        assert_eq!(said, ["abc", "bcd", "ghi"], "two words yield no trigram");

        let mut none = 0;
        each(std::iter::empty(), |_, _, _| none += 1);
        assert_eq!(none, 0);
    }
}
