use std::{
    ops::{Div, Mul},
    sync::Arc,
};

use crate::types::Executor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, Bytes, U256},
};
use tracing::{error, info};

/// An executor that sends transactions to the mempool.
pub struct MempoolExecutor<M> {
    client: Arc<M>,
}

/// Information about the gas bid for a transaction.
#[derive(Debug, Clone)]
pub struct GasBidInfo {
    /// Total profit expected from opportunity
    pub total_profit: U256,

    /// Percentage of bid profit to use for gas
    pub bid_percentage: u64,
}

#[derive(Debug, Clone)]
pub struct SubmitTxToMempool {
    pub tx: TypedTransaction,
    pub gas_bid_info: Option<GasBidInfo>,
}

#[derive(Debug, Clone)]
pub struct MempoolBundleRequest {
    pub frontrun_tx: Bytes,
    pub meat_tx: Vec<Bytes>,
    pub backrun_tx: Bytes,
}

pub type MempoolBundle = Vec<MempoolBundleRequest>;

impl<M: Middleware> MempoolExecutor<M> {
    pub fn new(client: Arc<M>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<M> Executor<SubmitTxToMempool> for MempoolExecutor<M>
where
    M: Middleware,
    M::Error: 'static,
{
    /// Send a transaction to the mempool.
    async fn execute(&self, mut action: SubmitTxToMempool) -> Result<()> {
        let gas_usage = self
            .client
            .estimate_gas(&action.tx, None)
            .await
            .context("Error estimating gas usage: {}")?;

        let bid_gas_price;
        if let Some(gas_bid_info) = action.gas_bid_info {
            // gas price at which we'd break even, meaning 100% of profit goes to validator
            let breakeven_gas_price = gas_bid_info.total_profit / gas_usage;
            // gas price corresponding to bid percentage
            bid_gas_price = breakeven_gas_price
                .mul(gas_bid_info.bid_percentage)
                .div(100);
        } else {
            bid_gas_price = self
                .client
                .get_gas_price()
                .await
                .context("Error getting gas price: {}")?;
        }
        action.tx.set_gas_price(bid_gas_price);
        self.client.send_transaction(action.tx, None).await?;
        Ok(())
    }
}

#[async_trait]
impl<M> Executor<MempoolBundle> for MempoolExecutor<M>
where
    M: Middleware,
    M::Error: 'static,
{
    async fn execute(&self, bundle: MempoolBundle) -> Result<()> {
        for mut action in bundle {
            let pending_frontrun = self.client.send_raw_transaction(action.frontrun_tx).await;
            let first_meat = action.meat_tx.remove(0);
            let pending_meat = self.client.send_raw_transaction(first_meat).await;
            let pending_backrun = self.client.send_raw_transaction(action.backrun_tx).await;

            match pending_frontrun {
                Ok(res) => info!("Frontrun Result: {:?}", res.await),
                Err(send_error) => error!("Error sending frontrun: {:?}", send_error),
            }

            match pending_meat {
                Ok(res) => info!("Meat Result: {:?}", res.await),
                Err(send_error) => error!("Error sending Meat: {:?}", send_error),
            }

            match pending_backrun {
                Ok(res) => info!("Backrun Result: {:?}", res.await),
                Err(send_error) => error!("Error sending Backrun: {:?}", send_error),
            }
        }

        Ok(())
    }
}
