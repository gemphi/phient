//! The builders' world of Philosophical Investigations §2, extended as §8
//! extends it (numerals, `there`, `this`, colour samples), with §21's two
//! games (the same words as an order and as a report) and §27's words that
//! name nothing (`help`, `away`, `no`).
//!
//! An episode is what a learner sees: the tokens said and pointed at, the
//! game being played, and what followed — the action, in five aspects.
//! Nothing tells a learner what any word means.

pub const TYPES: [&str; 4] = ["block", "pillar", "slab", "beam"];
pub const COLOURS: [&str; 3] = ["red", "blue", "yellow"];
pub const NUMERALS: [&str; 10] = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
pub const PLACES: usize = 5;
/// The aspects of what follows an utterance.
pub const ASPECTS: [&str; 5] = ["act", "what", "colour", "count", "place"];

#[derive(Clone, Debug, PartialEq)]
pub struct Episode {
    /// `order`, `report` or `ostension`.
    pub game: String,
    /// Words, and gestures as `⟨point:…⟩` tokens, in the order given.
    pub tokens: Vec<String>,
    /// One value per aspect, in [`ASPECTS`] order; `-` where none applies.
    pub action: [String; 5],
}

impl Episode {
    pub fn key(&self) -> String {
        format!("{}|{}", self.game, self.tokens.join(" "))
    }
}

/// A small deterministic generator (splitmix64).
pub struct Dice(pub u64);

impl Dice {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    pub fn chance(&mut self, p: f64) -> bool {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64 <= p
    }
    pub fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.below(xs.len())]
    }
}

fn s(x: &str) -> String {
    x.to_string()
}

/// An order (A calls, B acts) or a report (B tells, A records): the same
/// sentence forms, a different act.
pub fn sentence(d: &mut Dice, game: &str, colours: &[&str]) -> Episode {
    if game == "order" && d.chance(0.1) {
        let (w, act) = [("help", "come"), ("away", "clear"), ("no", "undo")][d.below(3)];
        return Episode { game: s(game), tokens: vec![s(w)], action: [s(act), s("-"), s("-"), s("-"), s("-")] };
    }
    let mut tokens = Vec::new();
    let mut count = 1;
    if d.chance(0.6) {
        count = 1 + d.below(NUMERALS.len());
        tokens.push(s(NUMERALS[count - 1]));
    }
    let mut colour = "any";
    if d.chance(0.5) {
        colour = d.pick(colours);
        tokens.push(s(colour));
    }
    let what = d.pick(&TYPES);
    if d.chance(0.15) {
        tokens.push(s("this"));
        tokens.push(format!("⟨point:{what}⟩"));
    } else {
        tokens.push(s(what));
    }
    let mut place = s("here");
    if d.chance(0.4) {
        let p = 1 + d.below(PLACES);
        place = format!("p{p}");
        tokens.push(s("there"));
        tokens.push(format!("⟨point:{place}⟩"));
    }
    let act = if game == "order" { "bring" } else { "record" };
    Episode { game: s(game), tokens, action: [s(act), s(what), s(colour), count.to_string(), place] }
}

/// §28–30: a new word said while pointing at a group. The scene has every
/// aspect at once; which one the word names, the pointing cannot say.
/// With `grammar`, the word is introduced as a colour-word ("that colour
/// is called sepia"), and the learner is told so.
pub fn ostension(d: &mut Dice, word: &str, value: &str, grammar: bool) -> Episode {
    let mut tokens = vec![];
    if grammar {
        tokens.push(s("colour"));
    }
    tokens.push(s(word));
    tokens.push(s("⟨point:group⟩"));
    let count = 1 + d.below(NUMERALS.len());
    let place = format!("p{}", 1 + d.below(PLACES));
    Episode { game: s("ostension"), tokens, action: [s("attend"), s(d.pick(&TYPES)), s(value), count.to_string(), place] }
}
