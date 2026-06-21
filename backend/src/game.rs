//! Server-authoritative slot logic.
//!
//! A spin is a single row of three reels with symbols `1..=7`. The server (never
//! the client) rolls the reels and decides the payout, so the outcome can't be
//! tampered with from the browser.
//!
//! Payout tiers (more entertaining than plain all-or-nothing):
//! - **three of a kind** → `stake * triple_multiplier(symbol)` – rarer/higher
//!   symbols pay more (💎 7 → 50×, down to 🍒 1 → 5×).
//! - **exactly two of a kind** → the stake back (a "near miss" push, net 0).
//! - **no match** → nothing (lose the stake).

use rand::Rng;

/// Number of distinct symbols (matches the frontend emoji map 1..=7).
pub const SYMBOL_COUNT: i32 = 7;
/// Reels per spin.
pub const REELS: usize = 3;

/// A computed spin outcome.
pub struct Outcome {
    pub reels: Vec<i32>,
    /// Gross amount won (0 on a loss). Net balance change is `earned - stake`.
    pub amount_earned: i64,
}

/// Roll three reels and compute the payout for the given stake.
pub fn play(stake: i64) -> Outcome {
    let mut rng = rand::rng();
    let reels: Vec<i32> = (0..REELS)
        .map(|_| rng.random_range(1..=SYMBOL_COUNT))
        .collect();

    let amount_earned = payout(&reels, stake);
    Outcome {
        reels,
        amount_earned,
    }
}

/// Payout for a finished set of reels.
fn payout(reels: &[i32], stake: i64) -> i64 {
    let all_same = reels.windows(2).all(|w| w[0] == w[1]);
    if all_same {
        return stake * triple_multiplier(reels[0]);
    }

    // Exactly two matching → return the stake (net zero, keeps the game lively).
    let has_pair = (0..reels.len())
        .any(|i| (i + 1..reels.len()).any(|j| reels[i] == reels[j]));
    if has_pair { stake } else { 0 }
}

/// Three-of-a-kind multiplier per symbol (rarer-feeling symbols pay more).
fn triple_multiplier(symbol: i32) -> i64 {
    match symbol {
        7 => 50, // 💎
        6 => 25, // ⭐
        5 => 15, // 🔔
        4 => 10, // 🍇
        3 => 8,  // 🍊
        2 => 6,  // 🍋
        _ => 5,  // 🍒 (1) and any fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triple_pays_by_symbol() {
        assert_eq!(payout(&[7, 7, 7], 10), 500);
        assert_eq!(payout(&[1, 1, 1], 10), 50);
    }

    #[test]
    fn pair_returns_stake() {
        assert_eq!(payout(&[3, 3, 5], 10), 10);
        assert_eq!(payout(&[5, 3, 3], 10), 10);
    }

    #[test]
    fn no_match_pays_nothing() {
        assert_eq!(payout(&[1, 2, 3], 10), 0);
    }

    #[test]
    fn play_returns_three_reels_in_range() {
        let outcome = play(10);
        assert_eq!(outcome.reels.len(), REELS);
        assert!(outcome.reels.iter().all(|&s| (1..=SYMBOL_COUNT).contains(&s)));
    }
}
