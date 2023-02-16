const DUST_PER_BLOCK: f64 = 20_000_000_000.0;
const MAX_BLOCK_INDEX: u64 = 2_600_000;

const FIRST_BP: f64 = 20.0;
const REDUCTION_INVERSE: f64 = 0.999994305214327076692;

const MIN_REQUIRED_STAKE_FOR_BOTTOM_BLOCK_DUST: u64 = 20_000_000_000;

const MAX_BOTTOM_BOOST: f64 = 60.0;
const BOTTOM_BOOST_REDUCTION: f64 = 0.9999979999920000;

const MIN_TOP_BOOST: f64 = 0.5;
const TOP_BOOST_REDUCTION: f64 = 1.0000044986146084;

const TOP_FIRST_BOOSTED_BLOCK: f64 = 250.0;
const TOP_BP_WITHOUT_BOOST: u64 = 1;

fn dust_to_staking_sallar(dusts: u64) -> u64 {
    dusts / (1_000_000_000)
}

fn calculate_bp_increase_factor(block_index: u64) -> f64 {
    REDUCTION_INVERSE.powf((block_index - 1) as f64)
}

pub fn calculate_max_bp(block_index: u64) -> f64 {
    let bp_for_single_request = calculate_bp_increase_factor(block_index);

    (FIRST_BP / bp_for_single_request).round()
}

pub fn calculate_dust_per_bp(block_index: u64) -> f64 {
    let max_bp = calculate_max_bp(block_index);
    let result = DUST_PER_BLOCK / max_bp;

    result
}

fn calculate_top_block_max_boost(block_index: u64) -> u64 {
    let exp = block_index as f64 - TOP_FIRST_BOOSTED_BLOCK;
    let pow = TOP_BOOST_REDUCTION.powf(exp);

    let base_boost = MIN_TOP_BOOST * pow;
    let rounded_boost;
    if base_boost < 1e+2 {
        rounded_boost = (base_boost).round() as u64;
    } else if base_boost < 1e+3 {
        rounded_boost = ((base_boost * 0.1) as u64) * 10;
    } else {
        rounded_boost = ((base_boost * 0.01) as u64) * 100;
    }

    rounded_boost
}

fn calculate_top_bp(boost: u64) -> u64 {
    1 + boost
}

pub fn calculate_top_bp_with_boost(block_index: u64) -> u64 {
    let boost = calculate_top_block_max_boost(block_index);

    calculate_top_bp(boost)
}

fn calculate_bottom_block_max_boost(block_index: u64) -> f64 {
    let base_boost = MAX_BOTTOM_BOOST * BOTTOM_BOOST_REDUCTION.powf((MAX_BLOCK_INDEX - block_index) as f64);
    
    (base_boost).round()
}

fn calculate_bottom_bp(user_wallet_balance: u64, boost: f64) -> u64 {
   (1 + boost as u64) * (dust_to_staking_sallar(user_wallet_balance))
}

pub fn calculate_bottom_bp_without_boost(user_wallet_balance: u64) -> u64 {
  calculate_bottom_bp(user_wallet_balance, 0.0)
}

pub fn calculate_bottom_bp_with_boost(block_index: u64, user_wallet_balance: u64) -> u64 {
  let boost = calculate_bottom_block_max_boost(block_index);

  calculate_bottom_bp(user_wallet_balance, boost)
}

fn calculate_user_reward(user_request_without_boost: u8, user_request_with_boost: u8, parts_without_boost: u64, parts_with_boost: u64, dust_per_bp: f64) -> (u64, u64) {
    let amount_without_boost = (user_request_without_boost as u64) * ((parts_without_boost as f64 * dust_per_bp).round()) as u64;
    let amount_with_boost = (user_request_with_boost as u64) * ((parts_with_boost as f64 * dust_per_bp).round()) as u64;

    let total_bp = (user_request_without_boost as u64 * parts_without_boost) + (user_request_with_boost as u64 * parts_with_boost);
    let summary_amount = amount_without_boost + amount_with_boost;

    (total_bp, summary_amount)
}

pub fn calculate_user_reward_bottom_block(user_request_without_boost: u8, user_request_with_boost: u8, parts_without_boost: u64, parts_with_boost: u64, dust_per_bp: f64, user_wallet_balance: u64) -> (u64, u64) {
    if user_wallet_balance < MIN_REQUIRED_STAKE_FOR_BOTTOM_BLOCK_DUST {
      return (0,0);
    }
    
    calculate_user_reward(user_request_without_boost, user_request_with_boost, parts_without_boost, parts_with_boost, dust_per_bp)
}

pub fn calculate_user_reward_top_block(user_request_without_boost: u8, user_request_with_boost: u8, parts_with_boost: u64, dust_per_bp: f64) -> (u64, u64) {    
    calculate_user_reward(user_request_without_boost, user_request_with_boost, TOP_BP_WITHOUT_BOOST, parts_with_boost, dust_per_bp)
}

#[cfg(test)]
mod tests {
    use csv::Writer;
    use std::{error::Error as standardError, fs::File};

    use super::*;

    #[test]
    fn generate_csv_report_top_block() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./dustAndBlockPartReportTop.csv")?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut wtr = Writer::from_path("top_block_reports/top_block_report.csv")?;

        wtr.write_record(&["number", "dust without boost", "dust with boost", "block parts with boost", "dustPerBlockPart"])?;
        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let dust_without_boost_expected = record.get(1).unwrap().parse::<u64>().unwrap();
            let dust_with_boost_expected = record.get(2).unwrap().parse::<u64>().unwrap();
            let bp_with_boost_expected = record.get(3).unwrap().parse::<u64>().unwrap();
            let dust_per_bp_expected = record.get(4).unwrap().parse::<f64>().unwrap();
            let sallar_per_bp = calculate_dust_per_bp(block_index);
            let top_block_bp_with_boost = calculate_top_bp_with_boost(block_index);
            
            let (_, top_block_dust_without_boost) = calculate_user_reward_top_block(1, 0, top_block_bp_with_boost, sallar_per_bp);
            let (_, top_block_dust_with_boost) = calculate_user_reward_top_block(0, 1, top_block_bp_with_boost, sallar_per_bp);
            
            let dust_per_bp = calculate_dust_per_bp(block_index);

            assert_eq!(bp_with_boost_expected.to_string(), top_block_bp_with_boost.to_string());
            assert_eq!(dust_with_boost_expected.to_string(), top_block_dust_with_boost.to_string());
            assert_eq!(dust_without_boost_expected.to_string(), top_block_dust_without_boost.to_string());
            assert_eq!(dust_per_bp_expected.to_string(), dust_per_bp.to_string());

            wtr.write_record(&[block_index.to_string(), top_block_dust_without_boost.to_string(), top_block_dust_with_boost.to_string(), top_block_bp_with_boost.to_string(), dust_per_bp.to_string()])?;
        }

        wtr.flush()?;

        Ok(())
    }

    #[test]
    fn generate_csv_report_bottom_block() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./sallarAndBlockPartReportBottom.csv")?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut wtr = Writer::from_path("bottom_block_reports/bottom_block_report.csv")?;
        wtr.write_record(&["number", "walletBalance", "dust without boost", "block parts without boost", "dust with boost",	"block parts with boost", "dustPerBlockPart"])?;
        
        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let balance = record.get(1).unwrap().parse::<u64>().unwrap();
            
            let sallar_without_boost_expected = record.get(2).unwrap().parse::<u64>().unwrap();
            let bp_without_boost_expected = record.get(3).unwrap().parse::<u64>().unwrap();
            let sallar_with_boost_expected = record.get(4).unwrap().parse::<u64>().unwrap();
            let bp_with_boost_expected = record.get(5).unwrap().parse::<u64>().unwrap();
            let sallar_per_bp_expected = record.get(6).unwrap().parse::<f64>().unwrap();

            let bottom_block_bp_with_boost = calculate_bottom_bp_with_boost(block_index, balance);
            let bottom_block_bp_without_boost = calculate_bottom_bp_without_boost(balance);
            let sallar_per_bp = calculate_dust_per_bp(block_index);
          
            let (_, bottom_block_staking_dust_without_boost) = calculate_user_reward_bottom_block(1, 0, bottom_block_bp_without_boost, bottom_block_bp_with_boost, sallar_per_bp, balance);
            let (_, bottom_block_staking_dust_with_boost) = calculate_user_reward_bottom_block(0, 1, bottom_block_bp_without_boost, bottom_block_bp_with_boost, sallar_per_bp, balance);
            
            let bp_without_boost = calculate_bottom_bp_without_boost(balance);

            assert_eq!( bp_without_boost_expected.to_string(), bp_without_boost.to_string());
            assert_eq!( bp_with_boost_expected.to_string(), bottom_block_bp_with_boost.to_string());
            
            assert_eq!( sallar_per_bp_expected.to_string(), sallar_per_bp.to_string());
            
            assert_eq!( sallar_without_boost_expected.to_string(), bottom_block_staking_dust_without_boost.to_string());
            assert_eq!( sallar_with_boost_expected.to_string(), bottom_block_staking_dust_with_boost.to_string());

           wtr.write_record(&[block_index.to_string(), balance.to_string(), bottom_block_staking_dust_without_boost.to_string(), bp_without_boost.to_string(),  bottom_block_staking_dust_with_boost.to_string(), bottom_block_bp_with_boost.to_string(), sallar_per_bp.to_string()])?;
        }

        wtr.flush()?;

        Ok(())
    }

    #[test]
    pub fn calculate_user_reward_top_block_test() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./topBlockTransferTestData.csv")?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut wtr = Writer::from_path("top_block_reports/top_block_calculate_user_reward_report.csv")?;
        wtr.write_record(&["block_index", "req_without_boost", "req_with_boost", "reward_dust"])?;
        
        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let user_request_without_boost = record.get(1).unwrap().parse::<u64>().unwrap();
            let user_request_with_boost = record.get(2).unwrap().parse::<u64>().unwrap();

            let reward_dust_expected = record.get(3).unwrap().parse::<u64>().unwrap();
            
            let top_block_bp_with_boost = calculate_top_bp_with_boost(block_index);
            let dust_per_bp = calculate_dust_per_bp(block_index);
            let (_, reward_dust) = calculate_user_reward_top_block(user_request_without_boost as u8, user_request_with_boost as u8, top_block_bp_with_boost, dust_per_bp);
            
            assert_eq!(reward_dust_expected.to_string(), reward_dust.to_string());

            wtr.write_record(&[block_index.to_string(), user_request_without_boost.to_string(), user_request_with_boost.to_string(), reward_dust.to_string()])?;
        }

        Ok(())
    }

    #[test]
    pub fn calculate_user_reward_bottom_block_test() -> Result<(), Box<dyn standardError>> {
        let file = File::open("./bottomBlockTransferTestData_01.csv")?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut wtr = Writer::from_path("bottom_block_reports/botom_block_calculate_user_reward_report.csv")?;
        wtr.write_record(&["block_index", "req_without_boost", "req_with_boost", "wallet_balance", "reward_dust"])?;
        
        for result in rdr.records() {
            let record = result?;

            let block_index = record.get(0).unwrap().parse::<u64>().unwrap();
            let user_request_without_boost = record.get(1).unwrap().parse::<u64>().unwrap();
            let user_request_with_boost = record.get(2).unwrap().parse::<u64>().unwrap();

            let user_wallet_balance = record.get(3).unwrap().parse::<u64>().unwrap();
            let reward_dust_expected = record.get(4).unwrap().parse::<u64>().unwrap();
            
            let bottom_block_bp_without_boost = calculate_bottom_bp_without_boost(user_wallet_balance);
            let bottom_block_bp_with_boost = calculate_bottom_bp_with_boost(block_index, user_wallet_balance);
            let dust_per_bp = calculate_dust_per_bp(block_index);

            let (_, reward_dust) = calculate_user_reward_bottom_block(user_request_without_boost as u8, user_request_with_boost as u8,  bottom_block_bp_without_boost,bottom_block_bp_with_boost, dust_per_bp, user_wallet_balance);

            assert_eq!(reward_dust_expected.to_string(), reward_dust.to_string());

            wtr.write_record(&[block_index.to_string(), user_request_without_boost.to_string(), user_request_with_boost.to_string(), user_wallet_balance.to_string(), reward_dust.to_string()])?;
        }

        Ok(())
    }
}