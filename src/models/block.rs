use ethereum::Header;
use rlp_derive::{RlpDecodable, RlpEncodable};

pub type BlockBody = ethereum::Block<ethereum::TransactionV2>;
pub type BlockHeader = ethereum::Header;

#[derive(RlpDecodable, RlpEncodable, Debug, PartialEq)]
pub struct BodyForStorage {
    pub base_tx_id: u64,
    pub tx_amount: u32,
    pub uncles: Vec<Header>,
}
