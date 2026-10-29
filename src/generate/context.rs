//! The context wave buffer: a decaying superposition of recent tokens' channels.

use crate::generate::constants::TWO_PI;
use phiton::p64::words::Tokenizer;
use crate::generate::constants::{CONTEXT_LAMBDA, CONTEXT_OMEGA, DEFAULT_CONTEXT_WINDOW, PHASE_CHANNELS};
use phiecs::phasor::facet::Facet;
use phistd::c64;
use std::collections::VecDeque;

pub const CONTEXT_WINDOW_SIZE: usize = DEFAULT_CONTEXT_WINDOW;
#[allow(dead_code)]
pub const CONTEXT_LAYERS_COUNT: usize = 16;
pub const CONTEXT_DECAY_BASE: f64 = 0.5;

/// ContextWaveBuffer — a diagonal complex linear recurrence over the context.
///
/// State evolves as `h_t = λ_k · e^{i ω_k} · h_{t-1} + z_t`, independently per
/// channel, with `λ_k` and `ω_k` spread geometrically so that each channel has
/// its own timescale and rotation frequency.
///
/// The previous buffer was a decayed *sum*, which is commutative: it could not
/// distinguish "dog bites man" from "man bites dog", and every token in a turn
/// carried the same weight regardless of position. A rotation-and-decay
/// recurrence is order-sensitive by construction, gives recent tokens more
/// influence than old ones, and is the same mechanism modern state-space models
/// (S4/S5, LRU, Mamba's linear component) use to carry long-range structure at
/// constant memory.
pub struct ContextWaveBuffer {
    /// Channel-0 state, exposed as (x, y) for compatibility.
    pub sum_x: f64,
    pub sum_y: f64,
    /// Per-channel recurrent state.
    h: Vec<c64>,
    /// Ring buffer of recent tokens
    tokens: VecDeque<String>,
    max_capacity: usize,
}

impl Default for ContextWaveBuffer {
    fn default() -> Self {
        Self::new(CONTEXT_WINDOW_SIZE)
    }
}

impl ContextWaveBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            sum_x: 0.0,
            sum_y: 0.0,
            h: vec![c64::new(0.0, 0.0); PHASE_CHANNELS],
            tokens: VecDeque::with_capacity(capacity.min(4096)),
            max_capacity: capacity,
        }
    }

    /// Decay and rotation for channel `k`.
    ///
    /// Channel 0 keeps the slowest decay and no rotation, so it behaves like a
    /// smoothed running topic; higher channels decay faster and rotate quicker,
    /// giving the state a spectrum of timescales rather than a single one.
    #[inline]
    fn kernel(k: usize) -> c64 {
        let frac = k as f64 / PHASE_CHANNELS as f64;
        let lambda = CONTEXT_LAMBDA.powf(1.0 + 3.0 * frac);
        let omega = CONTEXT_OMEGA * (1.0 + 4.0 * frac);
        c64::from_polar(lambda, omega)
    }

    /// Advances the state by one token.
    pub fn push_token(&mut self, facet: &Facet, token: &str) {
        for k in 0..PHASE_CHANNELS {
            self.h[k] *= Self::kernel(k);
        }
        if let Some(phasor) = facet.lexicon.get(token) {
            for k in 0..PHASE_CHANNELS {
                self.h[k] += c64::from_polar(phasor.amplitude, phasor.theta(k));
            }
        }
        if self.tokens.len() >= self.max_capacity {
            self.tokens.pop_front();
        }
        self.tokens.push_back(token.to_string());
        self.sum_x = self.h[0].re;
        self.sum_y = self.h[0].im;
    }

    /// Appends a whole turn, token by token, so order is preserved.
    pub fn push_turn(&mut self, facet: &Facet, text: &str) {
        for token in Tokenizer::tokenize(text) {
            self.push_token(facet, &token);
        }
    }

    /// Computes the current context phase angle in [0, 2pi).
    pub fn context_phase(&self) -> f64 {
        self.h[0].arg().rem_euclid(TWO_PI)
    }

    /// Returns the context wave magnitude (amplitude).
    pub fn context_amplitude(&self) -> f64 {
        self.h[0].norm()
    }

    /// The full multi-channel context state, for channel-aware retrieval.
    pub fn channels(&self) -> &[c64] {
        &self.h
    }

    /// Clears the recurrent state without dropping the token ring.
    pub fn reset_state(&mut self) {
        for z in self.h.iter_mut() {
            *z = c64::new(0.0, 0.0);
        }
        self.sum_x = 0.0;
        self.sum_y = 0.0;
    }
}
