pub struct StrategyGenerator;

impl StrategyGenerator {
    pub fn generate(
        start: f64,
        end: f64,
        profitier: f64,
        multiplier: f64,
        item_no: usize,
        choice: usize,
    ) -> (Vec<(f64, f64)>, f64) {
        match choice {
            0 => Self::basic_strategy(start, end, profitier, multiplier, item_no),
            1 => Self::advanced_strategy(start, end, profitier, multiplier, item_no),
            2 => Self::multi_advanced_strategy(start, end, profitier, multiplier, item_no),
            _ => Self::basic_strategy(start, end, profitier, multiplier, item_no),
        }
    }

    fn basic_strategy(
        start: f64,
        end: f64,
        profitier: f64,
        multiplier: f64,
        item_no: usize,
    ) -> (Vec<(f64, f64)>, f64) {
        let mut tuples = Vec::new();
        let mut current = start;

        for _ in 0..item_no.min(13) {
            if current > end {
                break;
            }
            tuples.push((current, multiplier));
            current = Self::multiply_rounded(current, profitier, 2);
        }

        let expected = Self::calculate_expected(&tuples, multiplier);
        (tuples, expected)
    }

    fn advanced_strategy(
        start: f64,
        end: f64,
        profitier: f64,
        multiplier: f64,
        item_no: usize,
    ) -> (Vec<(f64, f64)>, f64) {
        let mut tuples = Vec::new();
        let mut current = start;
        let mut mult = multiplier;

        for _ in 0..item_no.min(13) {
            if current > end {
                break;
            }
            tuples.push((current, mult));
            current = Self::multiply_rounded(current, profitier, 2);
            mult = Self::multiply_rounded(mult, 1.05, 2);
        }

        let expected = Self::calculate_expected(&tuples, multiplier);
        (tuples, expected)
    }

    fn multi_advanced_strategy(
        start: f64,
        end: f64,
        profitier: f64,
        multiplier: f64,
        item_no: usize,
    ) -> (Vec<(f64, f64)>, f64) {
        let mut tuples = Vec::new();
        let mut current = start;

        for i in 0..item_no.min(13) {
            if current > end {
                break;
            }
            let mult = if i % 2 == 0 { multiplier } else { multiplier * 1.2 };
            tuples.push((current, mult));
            current = Self::multiply_rounded(current, profitier, 2);
        }

        let expected = Self::calculate_expected(&tuples, multiplier);
        (tuples, expected)
    }

    fn multiply_rounded(a: f64, b: f64, decimals: u32) -> f64 {
        let factor = 10f64.powi(decimals as i32);
        (a * b * factor).round() / factor
    }

    fn calculate_expected(tuples: &[(f64, f64)], base_multiplier: f64) -> f64 {
        let total_bet: f64 = tuples.iter().map(|(odd, _)| odd).sum();
        let avg_multiplier = tuples.iter().map(|(_, m)| m).sum::<f64>() / tuples.len().max(1) as f64;
        total_bet * avg_multiplier * base_multiplier
    }
}
