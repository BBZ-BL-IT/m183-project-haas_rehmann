use rand::Rng;

pub const SYMBOL_COUNT: i32 = 7;
pub const REELS: usize = 3;

pub struct Outcome {
    pub reels: Vec<i32>,
    pub amount_earned: i64,
}

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

fn payout(reels: &[i32], stake: i64) -> i64 {
    let all_same = reels.windows(2).all(|w| w[0] == w[1]);
    if all_same {
        return stake * triple_multiplier(reels[0]);
    }

    let has_pair = (0..reels.len())
        .any(|i| (i + 1..reels.len()).any(|j| reels[i] == reels[j]));
    if has_pair { stake } else { 0 }
}

fn triple_multiplier(symbol: i32) -> i64 {
    match symbol {
        7 => 50, 
        6 => 25, 
        5 => 15, 
        4 => 10, 
        3 => 8,  
        2 => 6,  
        _ => 5,  
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
