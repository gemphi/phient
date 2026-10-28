#[allow(unused_imports)]
use super::*;

#[test]
fn test_encode_is_unit_energy_and_flat_without_evidence() {
    let p = SpectralPhasor::seeded("word", 1.0, 1);
    let f = encode(&p);
    let e: f64 = f.iter().map(|z| z.norm_sqr()).sum();
    assert!((e - 1.0).abs() < 1e-12);
    let mags: Vec<f64> = f.iter().map(|z| z.norm()).collect();
    assert!(mags.iter().all(|m| (m - mags[0]).abs() < 1e-12), "flat when no evidence");
    assert!((cosine(&f, &f) - 1.0).abs() < 1e-12);
}

#[test]
fn test_encode_is_loud_where_evidence_is() {
    let mut p = SpectralPhasor::seeded("word", 1.0, 1);
    p.deposit(9, 100);
    p.deposit(2, 1);
    let f = encode(&p);
    assert!(f[9].norm() > f[2].norm());
    assert!(f[3].norm() < 1e-12, "no evidence, no amplitude");
}
