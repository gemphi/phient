//! The game of Philosophical Investigations §48, with the composite names
//! of §64: a sentence describes a 3 × 3 arrangement of coloured squares,
//! read row by row, one name a square — R, G, W, B for red, green, white,
//! black — except that four names stand for two squares side by side in a
//! row: U = R G, V = W B, X = G W, Y = B R.
//!
//! What a word does now depends on where the words before it left the
//! reader: the condition of each word is set by the words that precede it.

pub const COLOURS: [&str; 4] = ["R", "G", "W", "B"];
/// Composite names and the two squares each stands for.
pub const PAIRS: [(&str, &str, &str); 4] = [("U", "R", "G"), ("V", "W", "B"), ("X", "G", "W"), ("Y", "B", "R")];
pub const CELLS: usize = 9;

#[derive(Clone, Debug)]
pub struct Episode {
    pub words: Vec<String>,
    pub grid: Vec<String>,
    /// Cell at which each composite name began.
    pub composites_at: Vec<usize>,
}

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
}

/// A random arrangement, described with composite names wherever one fits
/// and `use_pair` says so; at most `max_pairs` of them.
pub fn episode(d: &mut Dice, use_pair: f64, max_pairs: usize) -> Episode {
    let grid: Vec<String> = (0..CELLS).map(|_| COLOURS[d.below(4)].to_string()).collect();
    let (mut words, mut at) = (Vec::new(), Vec::new());
    let mut c = 0;
    while c < CELLS {
        let pair = (c % 3 < 2)
            .then(|| PAIRS.iter().find(|(_, a, b)| grid[c] == *a && grid[c + 1] == *b))
            .flatten()
            .filter(|_| at.len() < max_pairs && d.chance(use_pair));
        match pair {
            Some((w, _, _)) => {
                words.push(w.to_string());
                at.push(c);
                c += 2;
            }
            None => {
                words.push(grid[c].clone());
                c += 1;
            }
        }
    }
    Episode { words, grid, composites_at: at }
}
