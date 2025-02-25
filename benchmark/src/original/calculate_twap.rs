use eyre::Result;

pub fn calculate_twap(data: &Vec<(i64, i64)>) -> f64 {
    let total_base_fee = data
        .iter()
        .try_fold(0.0, |acc, (_, base_fee)| -> Result<f64> {
            let fee = *base_fee as f64;
            Ok(acc + fee)
        })
        .unwrap();

    let twap_result = total_base_fee / data.len() as f64;

    twap_result
}
