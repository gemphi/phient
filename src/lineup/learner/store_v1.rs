//! Checkpoint version 1, read and carried over: the layout before the
//! stride (phiano docs/231–232), without `Config::stride` and without the
//! planes' `last`. A learner saved by version 1 goes on under version 2
//! with the whole history kept (`stride` 0) and every row counted as
//! touched now, so nothing decays for the lines it was never clocked over.

use super::plane::Plane;
use super::{Config, Learner};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;

#[derive(Deserialize)]
struct ConfigV1 {
    channels: usize,
    context: usize,
    block: usize,
    pair: bool,
    tie: bool,
    dropout: f32,
    silence: f32,
    negatives: usize,
    rate: f32,
    epochs: usize,
    seed: u64,
}

#[derive(Deserialize)]
struct PlaneV1 {
    d: usize,
    re: Vec<f32>,
    im: Vec<f32>,
    acc: Vec<f32>,
}

#[derive(Deserialize)]
struct StoredV1 {
    #[allow(dead_code)]
    version: u32,
    config: ConfigV1,
    word: Vec<String>,
    seen: Vec<u64>,
    reach: PlaneV1,
    reached: PlaneV1,
    cond: PlaneV1,
    op: PlaneV1,
    pair: PlaneV1,
    bias: Vec<f32>,
    bias_acc: Vec<f32>,
    told: Vec<Vec<u32>>,
    rng: u64,
}

fn config(c: ConfigV1) -> Config {
    Config {
        channels: c.channels,
        context: c.context,
        block: c.block,
        pair: c.pair,
        tie: c.tie,
        dropout: c.dropout,
        silence: c.silence,
        negatives: c.negatives,
        rate: c.rate,
        stride: 0,
        epochs: c.epochs,
        seed: c.seed,
    }
}

fn plane(p: PlaneV1, now: u32) -> Plane {
    let last = vec![now; p.acc.len()];
    Plane { d: p.d, re: p.re, im: p.im, acc: p.acc, last }
}

/// A version-1 file as a learner, every row touched now.
pub(super) fn load(bytes: &[u8]) -> io::Result<Learner> {
    let s: StoredV1 = bincode::deserialize(bytes).map_err(io::Error::other)?;
    let now = s.told.len() as u32;
    let index: HashMap<String, u32> = s.word.iter().enumerate().map(|(i, w)| (w.clone(), i as u32)).collect();
    let bias_last = vec![now; s.bias_acc.len()];
    Ok(Learner {
        config: config(s.config),
        word: s.word,
        index,
        seen: s.seen,
        reach: plane(s.reach, now),
        reached: plane(s.reached, now),
        cond: plane(s.cond, now),
        op: plane(s.op, now),
        pair: plane(s.pair, now),
        bias: s.bias,
        bias_acc: s.bias_acc,
        bias_last,
        told: s.told,
        sampler: Vec::new(),
        logq: Vec::new(),
        fresh_logq: 0.0,
        sampled: 0,
        rng: s.rng,
    })
}
