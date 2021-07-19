use bytes::Bytes;
use ethereum::{AccessList, TransactionAction};
use ethereum_types::{H256, U256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TxType {
    Legacy = 0,
    EIP2930 = 1,
    EIP1559 = 2,
}

#[derive(Debug)]
pub struct NormalizedTransaction {
    pub tx_type: TxType,
    pub chain_id: Option<u64>,
    pub nonce: u64,
    pub max_priority_fee_per_gas: U256,
    pub max_fee_per_gas: U256,
    pub gas_limit: U256,
    pub action: TransactionAction,
    pub value: U256,
    pub input: Bytes<'static>,
    pub access_list: AccessList,
    pub r: H256,
    pub s: H256,
    pub v: u8,
}

impl From<ethereum::TransactionV2> for NormalizedTransaction {
    fn from(tx: ethereum::TransactionV2) -> Self {
        match tx {
            ethereum::TransactionV2::Legacy(tx) => Self {
                tx_type: TxType::Legacy,
                chain_id: tx.signature.chain_id(),
                nonce: tx.nonce.low_u64(),
                max_priority_fee_per_gas: tx.gas_price,
                max_fee_per_gas: tx.gas_price,
                gas_limit: tx.gas_limit,
                action: tx.action,
                value: tx.value,
                input: tx.input.into(),
                access_list: Default::default(),
                r: *tx.signature.r(),
                s: *tx.signature.s(),
                v: tx.signature.standard_v(),
            },
            ethereum::TransactionV2::EIP2930(_) => todo!(),
            ethereum::TransactionV2::EIP1559(_) => todo!(),
        }
    }
}

// pub trait TxData {
//     fn tx_type(&self) -> TxType;
//     fn chain_id(&self) -> Option<u64>;
//     fn nonce(&self) -> u64;
//     fn max_priority_fee_per_gas(&self) -> U256;
//     fn max_fee_per_gas(&self) -> U256;
//     fn gas_limit(&self) -> U256;
//     fn action(&self) -> TransactionAction;
//     fn value(&self) -> U256;
//     fn input(&self) -> Bytes;
//     fn access_list(&self) -> AccessList;
//     fn v(&self) -> u8;
//     fn r(&self) -> H256;
//     fn s(&self) -> H256;
// }

// impl TxData for ethereum::TransactionV2 {
//     fn tx_type(&self) -> TxType {
//         match self {
//             ethereum::TransactionV2::Legacy(_) => TxType::Legacy,
//             ethereum::TransactionV2::EIP2930(_) => TxType::EIP2930,
//             ethereum::TransactionV2::EIP1559(_) => TxType::EIP1559,
//         }
//     }

//     fn chain_id(&self) -> Option<u64> {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => tx.signature.chain_id(),
//             ethereum::TransactionV2::EIP2930(tx) => Some(tx.chain_id),
//             ethereum::TransactionV2::EIP1559(tx) => Some(tx.chain_id),
//         }
//     }

//     fn nonce(&self) -> u64 {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => tx.nonce.low_u64(),
//             ethereum::TransactionV2::EIP2930(tx) => tx.nonce.low_u64(),
//             ethereum::TransactionV2::EIP1559(tx) => tx.nonce.low_u64(),
//         }
//     }

//     fn max_priority_fee_per_gas(&self) -> U256 {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => tx.gas_price,
//             ethereum::TransactionV2::EIP2930(tx) => tx.gas_price,
//             ethereum::TransactionV2::EIP1559(tx) => tx.max_priority_fee_per_gas,
//         }
//     }

//     fn max_fee_per_gas(&self) -> U256 {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => tx.gas_price,
//             ethereum::TransactionV2::EIP2930(tx) => tx.gas_price,
//             ethereum::TransactionV2::EIP1559(tx) => tx.max_fee_per_gas,
//         }
//     }

//     fn v(&self) -> u8 {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => tx.signature.standard_v(),
//             ethereum::TransactionV2::EIP2930(tx) => tx.odd_y_parity as u8,
//             ethereum::TransactionV2::EIP1559(tx) => tx.odd_y_parity as u8,
//         }
//     }

//     fn r(&self) -> H256 {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => *tx.signature.r(),
//             ethereum::TransactionV2::EIP2930(tx) => tx.r,
//             ethereum::TransactionV2::EIP1559(tx) => tx.r,
//         }
//     }

//     fn s(&self) -> H256 {
//         match self {
//             ethereum::TransactionV2::Legacy(tx) => *tx.signature.s(),
//             ethereum::TransactionV2::EIP2930(tx) => tx.s,
//             ethereum::TransactionV2::EIP1559(tx) => tx.s,
//         }
//     }
// }
