pub fn calculate_twap(base_fees: &Vec<f64>) -> f64 {
    let total_base_fee = base_fees.iter().sum::<f64>();
    let twap_result = total_base_fee / base_fees.len() as f64;
    twap_result
}
