use anyhow::Result;
use concurrent_queue::ConcurrentQueue;
use futures::{future::join_all, Future};
use log::{info, warn};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_measure::measure::Measure;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::Signature,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::{str::FromStr, time::Duration};
use tokio::runtime::Builder;

use crate::common::{
    Target, DEFAULT_MAX_PENDING_SIGNATURES, DEFAULT_MONITOR_INTERVAL, DEFAULT_RPC_ENDPOINT,
    DEFAULT_SIGNATURE_FETCH_LIMIT,
};

#[derive(Debug, Clone)]
pub struct SignaturesCrawlerConfig {
    pub rpc_endpoint: String,
    pub signature_fetch_limit: usize,
    pub max_pending_signatures: usize,
    pub monitor_interval: u64,
    pub targets: Vec<Target>,
}

impl SignaturesCrawlerConfig {
    pub fn new(targets: Vec<Target>) -> Self {
        Self {
            rpc_endpoint: DEFAULT_RPC_ENDPOINT.to_string(),
            signature_fetch_limit: DEFAULT_SIGNATURE_FETCH_LIMIT,
            max_pending_signatures: DEFAULT_MAX_PENDING_SIGNATURES,
            monitor_interval: DEFAULT_MONITOR_INTERVAL,
            targets,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignatureData {
    pub indexing_address: Pubkey,
    pub signature: RpcConfirmedTransactionStatusWithSignature,
}

#[derive(Clone)]
pub struct SignaturesCrawlerContext {
    pub config: Arc<SignaturesCrawlerConfig>,
    rpc_client: Arc<RpcClient>,
    pub signature_queue: Arc<Mutex<ConcurrentQueue<SignatureData>>>,
}

impl SignaturesCrawlerContext {
    pub fn new(config: &SignaturesCrawlerConfig) -> Self {
        Self {
            config: Arc::new(config.clone()),
            rpc_client: Arc::new(RpcClient::new_with_commitment(
                config.rpc_endpoint.clone(),
                CommitmentConfig {
                    commitment: CommitmentLevel::Finalized,
                },
            )),
            signature_queue: Arc::new(Mutex::new(ConcurrentQueue::unbounded())),
        }
    }
}

pub struct SignaturesCrawler {
    context: SignaturesCrawlerContext,
}

impl SignaturesCrawler {
    pub fn new(targets: Vec<Target>) -> Self {
        let config = SignaturesCrawlerConfig::new(targets);
        let context = SignaturesCrawlerContext::new(&config);
        Self { context }
    }

    pub fn new_with_config(config: SignaturesCrawlerConfig) -> Self {
        let context = SignaturesCrawlerContext::new(&config);
        Self { context }
    }

    pub fn run<F, Fut>(&self, signature_processor: &F) -> Result<()>
    where
        F: (FnOnce(Arc<SignaturesCrawlerContext>) -> Fut) + Clone + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

        runtime.block_on(self.run_async(signature_processor))
    }

    pub async fn run_async<'a, F, Fut>(&self, signature_processor: &F) -> Result<()>
    where
        F: (FnOnce(Arc<SignaturesCrawlerContext>) -> Fut) + Clone + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let context = Arc::new(self.context.clone());

        let fetch_signatures_handle = tokio::spawn({
            let context = context.clone();
            async move { Self::crawl_signatures_for_address(context).await }
        });
        let signature_transactions_handle = tokio::spawn({
            let context = context.clone();
            let signature_processor = signature_processor.clone();
            async move { signature_processor(context).await }
        });
        let monitor_handle = tokio::spawn({
            let context = context.clone();
            async move { Self::monitor(context).await }
        });

        join_all([
            fetch_signatures_handle,
            signature_transactions_handle,
            monitor_handle,
        ])
        .await;

        Ok(())
    }

    async fn crawl_signatures_for_address(ctx: Arc<SignaturesCrawlerContext>) {
        let mut last_fetched_signature_per_address = ctx
            .config
            .targets
            .iter()
            .map(|target| (target.address, target.before))
            .collect::<HashMap<_, Option<Signature>>>();

        loop {
            // Concurrent data fetching from RPC
            let signatures_futures = ctx
                .config
                .targets
                .iter()
                .map(|target| {
                    ctx.rpc_client.get_signatures_for_address_with_config(
                        &target.address,
                        GetConfirmedSignaturesForAddress2Config {
                            before: *last_fetched_signature_per_address
                                .get(&target.address)
                                .unwrap(),
                            until: target.until,
                            limit: Some(ctx.config.signature_fetch_limit),
                            ..Default::default()
                        },
                    )
                })
                .collect::<Vec<_>>();
            let signatures_result = join_all(signatures_futures).await;

            // Discard failed requests (will naturally be refetched from same sig next iteration for dropped addresses)
            let new_signatures_per_address = signatures_result
                .iter()
                .zip(ctx.config.targets.clone())
                .filter_map(|(sig_result, target)| match sig_result {
                    Ok(signatures) => Some((target.address, signatures)),
                    Err(_) => None,
                })
                .collect::<HashMap<Pubkey, _>>();

            // Flatten and sort signatures (relative ordering of same-block signatures cross-addresses is not guaranteed)
            let mut signatures_to_push = new_signatures_per_address
                .iter()
                .flat_map(|(indexing_address, sig_data_list)| {
                    (*sig_data_list)
                        .clone()
                        .iter()
                        .map(|sig_data| (indexing_address.clone(), sig_data.clone()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            signatures_to_push.sort_by(|s1, s2| s2.1.slot.cmp(&s1.1.slot));

            // Early return if no successful tx in batch
            if signatures_to_push.is_empty() {
                // Update last fetched signature per address
                new_signatures_per_address
                    .iter()
                    .for_each(|(address, signatures)| {
                        let last_sig_for_address =
                            last_fetched_signature_per_address.get_mut(address).unwrap();
                        if let Some(last_sig) = signatures.last() {
                            *last_sig_for_address =
                                Some(Signature::from_str(&last_sig.signature).unwrap());
                        }
                    });
                continue;
            }

            let mut timing = Measure::start("sig_Q_push_lock_wait");
            let signature_queue = ctx.signature_queue.lock().unwrap();
            timing.stop();
            if timing.as_ms() > 0 {
                warn!("{}", timing);
            }

            // Bail if not enough room for additional signatures (simpler than having to refetch from last fitting sig...)
            let current_pending_amount = signature_queue.len();
            if (current_pending_amount + signatures_to_push.len())
                > ctx.config.max_pending_signatures
            {
                // Last fetched signature per address not updated so as to resume from same point next iteration
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Push sigs to queue
            signatures_to_push
                .into_iter()
                .for_each(|(indexing_address, signature)| {
                    signature_queue
                        .push(SignatureData {
                            indexing_address,
                            signature,
                        })
                        .unwrap();
                });

            // Update last fetched signature per address
            new_signatures_per_address
                .iter()
                .for_each(|(address, signatures)| {
                    let last_sig_for_address =
                        last_fetched_signature_per_address.get_mut(address).unwrap();
                    if let Some(last_sig) = signatures.last() {
                        *last_sig_for_address =
                            Some(Signature::from_str(&last_sig.signature).unwrap());
                    }
                });
        }
    }

    async fn monitor(ctx: Arc<SignaturesCrawlerContext>) {
        let mut main_timing = Measure::start("main");

        loop {
            tokio::time::sleep(Duration::from_secs(ctx.config.monitor_interval)).await;
            main_timing.stop();
            let current_fetch_time = main_timing.as_s();

            let sig_queue_size = ctx.signature_queue.lock().unwrap().len();

            info!(
                "Time: {:.1}s | sig Q size: {}",
                current_fetch_time, sig_queue_size,
            );
        }
    }
}
