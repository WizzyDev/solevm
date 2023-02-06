use {
    std::{sync::Arc, convert::{TryFrom, TryInto}},
    tokio_postgres::{Client},
    super::{do_connect, DbConfig, block, TxParams, DbError, DbResult},
    solana_sdk::clock::Slot,
    ethnum::U256,
    evm_loader::types::Address,
};


#[derive(Debug, Clone)]
pub struct IndexerDb {
    pub client: Arc<Client>,
}

impl IndexerDb {
    pub fn new(config: &DbConfig) -> Self {
        let client = do_connect(
            &config.indexer_host, &config.indexer_port, &config.indexer_database, &config.indexer_user, &config.indexer_password
        );
        Self {client: Arc::new(client)}
    }

    pub fn get_sol_sig(&self, hash: &[u8; 32]) -> DbResult<[u8; 64]> {

        let hex = format!("0x{}", hex::encode(hash));
        let row = block(|| async {
            self.client.query_one(
                "SELECT S.sol_sig from solana_neon_transactions S, solana_blocks B \
                where S.block_slot = B.block_slot \
                and B.is_active =  true \
                and S.neon_sig = $1",
                &[&hex]
            ).await
        })?;
        let sol_sig_b58: &str = row.try_get(0)?;
        let sol_sig_b58 = sol_sig_b58.to_string();
        let sol_sig = bs58::decode(sol_sig_b58).into_vec()
            .map_err(|e| DbError::Custom(format!("sol_sig_b58 cast error: {}", e)))?;
        sol_sig.as_slice().try_into().map_err(|e| DbError::Custom(format!("sol_sig cast error: {}", e)))
    }

    pub fn get_slot(&self, hash: &[u8; 32]) -> DbResult<Slot>{
        let hex = format!("0x{}", hex::encode(hash));
        let row = block(|| async {
            self.client.query_one(
                "SELECT min(S.block_slot) from solana_neon_transactions S, solana_blocks B \
                where S.block_slot = B.block_slot \
                and B.is_active =  true \
                and S.neon_sig = $1",
                &[&hex]
            ).await
        })?;
        let slot: i64 = row.try_get(0)?;
        u64::try_from(slot).map_err(|e| DbError::Custom(format!("slot cast error: {}", e)))
    }

    pub fn get_transaction_data(&self, hash: &[u8; 32]) -> DbResult<TxParams> {
        let hex = format!("0x{}", hex::encode(hash));

        let row = block(|| async {
            self.client.query_one(
                "select distinct t.from_addr, \
            COALESCE(t.to_addr, t.contract), t.calldata, t.value, t.gas_limit \
             from neon_transactions as t, solana_blocks as b \
                where t.block_slot = b.block_slot \
                and b.is_active =  true \
                and t.neon_sig = $1",
                &[&hex]
            ).await
        })?;

        let from: String = row.try_get(0)?;
        let to: String = row.try_get(1)?;
        let data: String = row.try_get(2)?;
        let value: String = row.try_get(3)?;
        let gas_limit: String = row.try_get(4)?;

        let from = Address::from_hex(&from.as_str()[2..])
            .map_err(|e| DbError::Custom(format!("from_address cast error: {}", e)))?;
        let to = Address::from_hex(&to.as_str()[2..])
            .map_err(|e| DbError::Custom(format!("to_address cast error: {}", e)))?;
        let data =  hex::decode(&data.as_str()[2..])
            .map_err(|e| DbError::Custom(format!("data cast error: {}", e)))?;
        let value: U256 = U256::from_str_hex(&value)
            .map_err(|e| DbError::Custom(format!("value cast error: {}", e)))?;
        let gas_limit: U256 = U256::from_str_hex(&gas_limit)
            .map_err(|e| DbError::Custom(format!("gas_limit cast error: {}", e)))?;

        Ok(TxParams {from, to: Some(to), data: Some(data), value: Some(value), gas_limit: Some(gas_limit)})
    }
}