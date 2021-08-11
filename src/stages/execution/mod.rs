use self::precompile::PrecompileSet;
use crate::{
    accessors,
    models::NormalizedTransaction,
    node::db::buffer::Buffer,
    stagedsync::stage::{ExecOutput, Stage, StageInput},
    state::IntraBlockState,
    MutableTransaction, PlainStateWriter, StateBuffer, StateReader, StateWriter,
};
use anyhow::bail;
use async_trait::async_trait;
use ethereum_types::{Address, H160, U256, U512};
use evmodin::{Message, Output};

mod precompile;

#[derive(Debug)]
pub struct Execution;

#[async_trait]
impl<'db, RwTx: MutableTransaction<'db>> Stage<'db, RwTx> for Execution {
    fn id(&self) -> crate::StageId {
        todo!()
    }

    fn description(&self) -> &'static str {
        todo!()
    }

    async fn execute<'tx>(&self, tx: &'tx mut RwTx, input: StageInput) -> anyhow::Result<ExecOutput>
    where
        'db: 'tx,
    {
        let mut stage_progress = input.stage_progress.unwrap_or(0);

        for block_number in stage_progress
            ..input
                .previous_stage
                .map(|(_, b)| b)
                .unwrap_or(stage_progress)
        {
            let receipts = execute_block(tx, block_number).await?;
        }

        Ok(ExecOutput::Progress {
            stage_progress,
            done: true,
            must_commit: true,
        })
    }

    async fn unwind<'tx>(
        &self,
        tx: &'tx mut RwTx,
        input: crate::stagedsync::stage::UnwindInput,
    ) -> anyhow::Result<()>
    where
        'db: 'tx,
    {
        todo!()
    }
}

async fn execute_block<'db: 'tx, 'tx, RwTx: MutableTransaction<'db>>(
    tx: &'tx RwTx,
    block_number: u64,
) -> anyhow::Result<Vec<()>> {
    let block_hash = accessors::chain::canonical_hash::read(tx, block_number)
        .await?
        .ok_or_else(|| anyhow::Error::msg("no canonical block hash"))?;
    let block_header = accessors::chain::header::read(tx, block_hash, block_number)
        .await?
        .ok_or_else(|| anyhow::Error::msg("no block header"))?;

    let block_body_info = accessors::chain::storage_body::read(tx, block_hash, block_number)
        .await?
        .ok_or_else(|| anyhow::Error::msg("no block body"))?;

    let block_body =
        accessors::chain::tx::read(tx, block_body_info.base_tx_id, block_body_info.tx_amount)
            .await?;

    if block_body.len() != block_body_info.tx_amount as usize {
        bail!("block body len mismatch");
    }

    let senders = accessors::chain::tx_sender::read(
        tx,
        block_body_info.base_tx_id,
        block_body_info.tx_amount,
    )
    .await?;

    if senders.len() != block_body_info.tx_amount as usize {
        bail!("senders len mismatch");
    }

    let w = PlainStateWriter::new(tx, block_number);

    let mut state_buffer = Buffer::new(tx);

    let mut ibs = IntraBlockState::new(&mut state_buffer);

    // let mut results = vec![];
    let mut gas_pool = block_header.gas_limit;
    for (ethtx, sender) in block_body.into_iter().zip(senders) {
        let normalized_tx = ethtx.into();

        validate_transaction(&mut ibs, &normalized_tx, sender, gas_pool).await?;

        let execution_result = execute_transaction(&mut ibs, normalized_tx, sender).await?;

        execution_result.gas_left;
    }

    let mut receipts = vec![];

    Ok(receipts)
}

async fn validate_transaction<'storage, 'r, B: StateBuffer<'storage>>(
    state: &mut IntraBlockState<'storage, 'r, B>,
    tx: &NormalizedTransaction,
    sender: Address,
    gas_pool: U256,
) -> anyhow::Result<()> {
    if state.get_nonce(sender).await? != tx.nonce {
        bail!("invalid nonce");
    }

    // https://github.com/ethereum/EIPs/pull/3594
    let max_gas_cost = U512::from(tx.gas_limit) * U512::from(tx.max_fee_per_gas);
    // See YP, Eq (57) in Section 6.2 "Execution"
    let v0 = max_gas_cost + tx.value;
    if U512::from(state.get_balance(sender).await?) < v0 {
        bail!("insufficient funds");
    }

    if gas_pool < tx.gas_limit {
        // Corresponds to the final condition of Eq (58) in Yellow Paper Section 6.2 "Execution".
        // The sum of the transaction’s gas limit and the gas utilized in this block prior
        // must be no greater than the block’s gas limit.
        bail!("block gas limit exceeded");
    }

    Ok(())
}

async fn execute_transaction<'storage, 'r, R: StateBuffer<'storage>>(
    ibs: &mut IntraBlockState<'storage, 'r, R>,
    tx: NormalizedTransaction,
    sender: H160,
) -> anyhow::Result<Output> {
    todo!()
}

async fn execute_call<'storage>(
    precompile_set: &PrecompileSet,
    w: &impl StateWriter,
    r: &impl StateReader<'storage>,
    call: Message,
) -> anyhow::Result<Output> {
    todo!()
}
