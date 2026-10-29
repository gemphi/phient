#[allow(unused_imports)]
use super::*;

impl RiverFlow {

    /// Builds the river's path through sectors.
    pub(super) fn build_path(source: u16, depth: usize) -> Vec<u16> {
        let n = Wave::sector_count();
        let half = (depth / 2).max(2).min((n / 2 - 1) as usize);
        let mut path = Vec::new();

        for i in 0..half {
            path.push((source + i as u16) % n);
        }

        path.push(Wave::opposite_sector(source));

        for i in 1..half {
            let sector = (source + n - i as u16) % n;
            path.push(sector);
        }

        path
    }
}
