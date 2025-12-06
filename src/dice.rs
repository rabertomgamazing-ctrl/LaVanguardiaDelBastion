use rand::Rng;
use serde::{Deserialize, Serialize};

/// Resultado de un lanzamiento de dado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollResult {
    pub sides: u8,
    pub value: u8,
    pub modifier: i32,
    pub total: i32,
}

impl RollResult {
    pub fn new(sides: u8, value: u8, modifier: i32) -> Self {
        Self {
            sides,
            value,
            modifier,
            total: value as i32 + modifier,
        }
    }
}

pub fn roll_die(sides: u8) -> RollResult {
    let mut rng = rand::thread_rng();
    let value = rng.gen_range(1..=sides);
    RollResult::new(sides, value, 0)
}

pub fn roll_with_modifier(sides: u8, modifier: i32) -> RollResult {
    let mut rng = rand::thread_rng();
    let value = rng.gen_range(1..=sides);
    RollResult::new(sides, value, modifier)
}

pub fn roll_d20(modifier: i32) -> RollResult {
    roll_with_modifier(20, modifier)
}

pub fn skill_check(modifier: i32, dc: i32) -> (RollResult, bool) {
    let roll = roll_d20(modifier);
    let success = roll.total >= dc;
    (roll, success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_result_respects_bounds() {
        for _ in 0..100 {
            let roll = roll_die(6);
            assert!(roll.value >= 1 && roll.value <= 6);
        }
    }

    #[test]
    fn skill_check_computes_total() {
        let (roll, _success) = skill_check(2, 10);
        assert_eq!(roll.total, roll.value as i32 + 2);
    }
}
