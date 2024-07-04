
pub fn calculate_interest(b: i128, interest_rate: u32, days: u64) -> i128 {
    let year_interest = b * interest_rate as i128;
    let current_interest = (year_interest * days as i128) / 360_i128;
    current_interest
}