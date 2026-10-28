//! A sentence is a scene: the phiphx island split (phiphx `uv::island`)
//! read on a book.
//!
//! A sentence is a face; its words in order are its vertices, two adjacent
//! words an edge. Two sentences join across an edge they share, unless the
//! edge is a seam — a pair so common it places no scene (`of the`). The
//! islands are the scenes: sentences held together by shared pairs from
//! anywhere in the book. Reading a line, the scene it is joining is the
//! island its edges so far reach (else the last line's: the scene goes
//! on), and what to expect is what followed the last word in that scene's
//! recent sentences. A line whose edges reach no island is the unexpected.

use std::collections::HashMap;

type Edge = (String, String);

pub struct Scene {
    /// How many told sentences hold each edge, and which (while not a seam).
    count: HashMap<Edge, u32>,
    holds: HashMap<Edge, Vec<usize>>,
    seam: u32,
    up: Vec<usize>,
    members: HashMap<usize, Vec<usize>>,
    told: Vec<Vec<String>>,
    /// Lines whose edges reached no scene, of all lines told.
    pub unexpected: (usize, usize),
}

impl Scene {
    /// `seam`: an edge held by this many sentences or more is a seam.
    pub fn new(seam: u32) -> Self {
        Self { count: HashMap::new(), holds: HashMap::new(), seam, up: Vec::new(), members: HashMap::new(), told: Vec::new(), unexpected: (0, 0) }
    }

    fn root(&mut self, mut k: usize) -> usize {
        while self.up[k] != k {
            self.up[k] = self.up[self.up[k]];
            k = self.up[k];
        }
        k
    }

    fn find(&self, mut k: usize) -> usize {
        while self.up[k] != k {
            k = self.up[k];
        }
        k
    }

    fn edges(words: &[String]) -> impl Iterator<Item = Edge> + '_ {
        words.windows(2).map(|p| (p[0].clone(), p[1].clone()))
    }

    /// The scene the line `said` is joining: the island most of its edges
    /// reach, else the last line's.
    fn current(&self, said: &[String]) -> Option<usize> {
        let mut votes: HashMap<usize, u32> = HashMap::new();
        for e in Self::edges(said) {
            if let Some(s) = self.holds.get(&e).and_then(|v| v.first()) {
                *votes.entry(self.find(*s)).or_insert(0) += 1;
            }
        }
        let best = votes.into_iter().max_by_key(|&(r, n)| (n, r)).map(|(r, _)| r);
        best.or_else(|| self.told.len().checked_sub(1).map(|l| self.find(l)))
    }

    /// What followed the word before in the current scene's last 64 lines.
    pub fn dist(&self, said: &[String]) -> HashMap<String, f64> {
        let mut out: HashMap<String, f64> = HashMap::new();
        let (Some(r), Some(last)) = (self.current(said), said.last()) else { return out };
        let lines = self.members.get(&r).map_or(&[][..], |m| &m[m.len().saturating_sub(64)..]);
        for &l in lines {
            for p in self.told[l].windows(2).filter(|p| &p[0] == last) {
                *out.entry(p[1].clone()).or_insert(0.0) += 1.0;
            }
        }
        let n: f64 = out.values().sum();
        out.values_mut().for_each(|x| *x /= n);
        out
    }

    pub fn tell(&mut self, words: &[String]) {
        let line = self.told.len();
        self.up.push(line);
        self.members.insert(line, vec![line]);
        let mut joined = false;
        for e in Self::edges(words).collect::<Vec<_>>() {
            let n = self.count.entry(e.clone()).or_insert(0);
            *n += 1;
            match *n >= self.seam {
                true => {
                    self.holds.remove(&e);
                }
                false => {
                    let held = self.holds.entry(e).or_default();
                    let other = held.first().copied();
                    held.push(line);
                    if let Some(o) = other {
                        joined = true;
                        let (a, b) = (self.root(o), self.root(line));
                        if a != b {
                            let (big, small) = match self.members[&a].len() >= self.members[&b].len() {
                                true => (a, b),
                                false => (b, a),
                            };
                            self.up[small] = big;
                            let moved = self.members.remove(&small).unwrap_or_default();
                            let into = self.members.entry(big).or_default();
                            into.extend(moved);
                            into.sort_unstable();
                        }
                    }
                }
            }
        }
        self.unexpected = (self.unexpected.0 + usize::from(!joined && line > 0), self.unexpected.1 + 1);
        self.told.push(words.to_vec());
    }
}
