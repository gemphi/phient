#[allow(unused_imports)]
use super::*;

impl Default for SectorReadout {
    fn default() -> Self {
        Self::with_shape(KEY_CHANNELS, KEY_SECTORS)
    }
}

impl SectorReadout {
    pub fn new() -> Self {
        Self::default()
    }

    /// A table at an explicit key resolution.
    pub fn with_shape(key_channels: usize, key_sectors: usize) -> Self {
        Self {
            bias: HashMap::new(),
            hits: HashMap::new(),
            key_channels: key_channels.max(1),
            key_sectors: key_sectors.clamp(2, TARGET_SECTORS),
            lookups: std::sync::atomic::AtomicU64::new(0),
            found: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Number of context cells the table has learned.
    pub fn cells(&self) -> usize {
        self.bias.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bias.is_empty()
    }

    /// Fraction of `bias_for` calls that landed in a fitted cell.
    ///
    /// This is the number that decides whether a discretised readout can
    /// generalise at all. At coverage 0 the table is a no-op on held-out text
    /// however well it fits the training split.
    pub fn coverage(&self) -> f64 {
        use std::sync::atomic::Ordering::Relaxed;
        match self.lookups.load(Relaxed) {
            0 => 0.0,
            n => self.found.load(Relaxed) as f64 / n as f64,
        }
    }

    pub fn reset_coverage(&self) {
        use std::sync::atomic::Ordering::Relaxed;
        self.lookups.store(0, Relaxed);
        self.found.store(0, Relaxed);
    }

    /// Hashes a multi-channel context into a discrete cell.
    ///
    /// Nearby phases inside one bucket map to the same cell; crossing a bucket
    /// boundary changes the cell entirely. That discontinuity is the point.
    pub fn key(&self, channels: &[f64]) -> u64 {
        let mut h: u64 = 14695981039346656037;
        let width = std::f64::consts::TAU / self.key_sectors as f64;
        for a in channels.iter().take(self.key_channels) {
            let b = (a.rem_euclid(std::f64::consts::TAU) / width).floor() as u64;
            h ^= b % self.key_sectors as u64;
            h = h.wrapping_mul(1099511628211);
        }
        h
    }

    /// Sector index of a target phase.
    #[inline]
    pub fn target_sector(phase: f64) -> usize {
        (Wave::sector_of(phase) as usize) % TARGET_SECTORS
    }

    /// Learned bias for a target sector in a context cell.
    #[inline]
    pub fn bias_for(&self, key: u64, target_sector: usize) -> f64 {
        use std::sync::atomic::Ordering::Relaxed;
        self.lookups.fetch_add(1, Relaxed);
        match self.bias.get(&key) {
            Some(v) => {
                self.found.fetch_add(1, Relaxed);
                v.get(target_sector % TARGET_SECTORS).copied().unwrap_or(0.0) as f64
            }
            None => 0.0,
        }
    }

    /// Raises the bias of the sector a true continuation fell in, and lowers a
    /// sampled wrong one — the same contrastive shape as the phase objective.
    ///
    /// The per-cell rate decays with visits so a cell seen once cannot overwrite
    /// what a cell seen a thousand times learned.
    pub fn learn(&mut self, key: u64, positive: usize, negative: usize, lr: f64) {
        let n = self.hits.entry(key).or_insert(0);
        *n = n.saturating_add(1);
        let rate = (lr / (1.0 + (*n as f64).sqrt())) as f32;

        let v = self.bias.entry(key).or_insert_with(|| vec![0.0; TARGET_SECTORS]);
        let p = positive % TARGET_SECTORS;
        let q = negative % TARGET_SECTORS;
        v[p] = (v[p] + rate).clamp(-2.0, 2.0);
        if q != p {
            v[q] = (v[q] - rate * 0.5).clamp(-2.0, 2.0);
        }
    }

    /// Magnitude gate: suppress channels below `tau`. A ReLU in the magnitude
    /// domain, allowing sparse, competitive channel activation.
    pub fn gate(channels: &mut [c64], tau: f64) -> usize {
        let mut kept = 0;
        for z in channels.iter_mut() {
            match z.norm() >= tau {
                true => kept += 1,
                false => *z = c64::new(0.0, 0.0),
            }
        }
        kept
    }
}
