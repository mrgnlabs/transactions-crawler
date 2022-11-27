use solana_sdk::pubkey;
use std::sync::Arc;
use transactions_crawler::{
    transaction_processor::print_transactions_to_screen, Context, CrawlerConfig, Target,
    TransactionCrawler,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let tx_crawler = TransactionCrawler::new_with_config(CrawlerConfig::new(vec![
        Target {
            address: pubkey!("8kNQpnBYGznxxjZVX5Ss9ika6KkFCVMgu7BRJJhCV4bi"),
            before: None,
            until: None,
        },
        Target {
            address: pubkey!("SoL7vsQCCJoCnm32DhVW5VatZ5mX1cXHKSx62a2i2qU"),
            before: None,
            until: None,
        },
    ]));

    let transaction_processor =
        |ctx: Arc<Context>| async move { print_transactions_to_screen(ctx).await };

    tx_crawler.run_async(&transaction_processor).await.unwrap();
}
