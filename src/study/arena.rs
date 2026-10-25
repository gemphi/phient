//! `arena` — many models, ONE run (docs/176). Separate runs cannot be
//! compared: different streams, different floors, different luck. An arena
//! reads one stream and asks every model the same question at the same
//! position before any of them is told it, so the table at the end says
//! which model is learning better, not which run was easier.

use super::Study;

/// A model in the arena: it guesses the next word, then is told the line.
/// Prequential by construction — [`Arena::read`] always guesses first.
pub trait Reader {
    /// What to call it in the table — the crate or model it stands for.
    fn name(&self) -> &str;
    /// The next word after `said`, or `None` when it has nothing.
    fn guess(&mut self, said: &[String]) -> Option<String>;
    /// One line told, in full, after every guess in it was scored.
    fn tell(&mut self, words: &[String]);
    /// The training half has ended ([`Arena::read_split`]): a model that is
    /// trained by epochs rather than incremented by a count trains here,
    /// once, on everything it has been told. An online reader needs nothing.
    fn ready(&mut self) {}
}

/// Several readers scored on one stream.
pub struct Arena {
    reader: Vec<Box<dyn Reader>>,
    study: Vec<Study>,
}

impl Arena {
    pub fn new(reader: Vec<Box<dyn Reader>>, slices: usize, floor: f64) -> Self {
        let study = reader.iter().map(|r| Study::new(r.name(), slices, floor)).collect();
        Self { reader, study }
    }

    /// Reads the stream: at every position every reader guesses, then the
    /// line is told to all of them. Returns the positions scored.
    pub fn read(&mut self, lines: &[Vec<String>]) -> usize {
        let mut n = 0;
        for (i, words) in lines.iter().enumerate() {
            for j in 1..words.len() {
                let said = &words[..j];
                for (r, s) in self.reader.iter_mut().zip(&mut self.study) {
                    s.tick(i, lines.len(), r.guess(said).as_deref() == Some(words[j].as_str()));
                }
                n += 1;
            }
            for r in &mut self.reader {
                r.tell(words);
            }
        }
        n
    }

    /// Reads a stream in two halves, for models that must be TRAINED before
    /// they can read at all (an operator over a table, not a counter): the
    /// first `train` lines are told to everyone and scored for no one; from
    /// there on every reader guesses the same positions and is told the line
    /// after. The online readers are scored on exactly those positions too,
    /// so the table stays one comparison. Returns the positions scored.
    pub fn read_split(&mut self, lines: &[Vec<String>], train: usize) -> usize {
        let train = train.min(lines.len());
        for words in &lines[..train] {
            for r in &mut self.reader {
                r.tell(words);
            }
        }
        for r in &mut self.reader {
            r.ready();
        }
        self.read(&lines[train..])
    }

    /// The studies, best first.
    pub fn ranked(&self) -> Vec<&Study> {
        let mut out: Vec<&Study> = self.study.iter().collect();
        out.sort_by(|a, b| b.accuracy().total_cmp(&a.accuracy()));
        out
    }

    /// The leaderboard: one row per model, best first, with its curve —
    /// the same positions for all of them.
    pub fn table(&self) -> String {
        let mut s = String::from("| model | top-1 | first → last slice | lift | verdict |\n|---|---|---|---|---|\n");
        for st in self.ranked() {
            let c = st.curve();
            s.push_str(&format!(
                "| {} | {:.1}% | {:.1}% → {:.1}% | {:+.1} | {} |\n",
                st.subject,
                st.accuracy(),
                c.first().copied().unwrap_or(0.0),
                c.last().copied().unwrap_or(0.0),
                st.lift(),
                if st.beats_floor() && st.rises() { "learning" } else if st.beats_floor() { "saturated" } else { "not learning" }
            ));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Parrot(String, Option<String>);
    impl Reader for Parrot {
        fn name(&self) -> &str { &self.0 }
        fn guess(&mut self, _: &[String]) -> Option<String> { self.1.clone() }
        fn tell(&mut self, w: &[String]) { self.1 = w.last().cloned() }
    }

    #[test]
    fn the_split_lane_trains_then_scores_the_rest() {
        let lines: Vec<Vec<String>> = ["a b", "c d", "e f"].iter().map(|l| l.split(' ').map(String::from).collect()).collect();
        let mut a = Arena::new(vec![Box::new(Parrot("p".into(), None))], 2, 50.0);
        assert_eq!(a.read_split(&lines, 2), 1, "only the untold half is scored");
    }

    #[test]
    fn one_stream_scores_every_reader_at_the_same_positions() {
        let lines: Vec<Vec<String>> = ["a b b", "c b b"].iter().map(|l| l.split(' ').map(String::from).collect()).collect();
        let mut a = Arena::new(vec![Box::new(Parrot("p1".into(), None)), Box::new(Parrot("p2".into(), None))], 2, 50.0);
        assert_eq!(a.read(&lines), 4);
        assert_eq!(a.ranked().len(), 2);
        assert!(a.ranked().iter().all(|s| s.positions == 4), "same positions for all");
        assert!(a.table().contains("p1"));
    }
}
