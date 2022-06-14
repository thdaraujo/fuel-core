use crate::Config;
use anyhow::Result;
use fuel_core_interfaces::fuel_tx::Transaction;
use fuel_core_interfaces::fuel_types::Word;
use std::cmp::Ordering;
use std::sync::Arc;

pub fn select_transactions(
    mut includable_txs: Vec<Arc<Transaction>>,
    config: &Config,
) -> Result<Vec<Transaction>> {
    // basic selection mode for now. Select all includable txs up until gas limit, sorted by fees.
    let mut used_block_space = Word::MIN;

    // sort txs by fee
    includable_txs.sort_by(|a, b| compare_fee(a, b));

    Ok(includable_txs
        .into_iter()
        .take_while(|tx| {
            let tx_block_space = (tx.metered_bytes_size() as Word) + tx.gas_limit();
            let new_used_space = used_block_space.saturating_add(tx_block_space);
            let hit_end = new_used_space > config.max_gas_per_block;
            used_block_space = new_used_space;
            !hit_end
        })
        .map(|tx| tx.as_ref().clone())
        .collect())
}

fn compare_fee(tx1: &Transaction, tx2: &Transaction) -> Ordering {
    todo!()
}
