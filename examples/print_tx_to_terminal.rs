use solana_sdk::pubkey;
use std::sync::Arc;
use transactions_crawler::{
    common::Target,
    transactions_crawler::{
        TransactionsCrawler, TransactionsCrawlerConfig, TransactionsCrawlerContext,
    },
    transactions_processors::print_transactions_to_screen,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let tx_crawler = TransactionsCrawler::new_with_config(TransactionsCrawlerConfig::new(vec![
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

    let transaction_processor = |ctx: Arc<TransactionsCrawlerContext>| async move {
        print_transactions_to_screen(ctx).await
    };

    tx_crawler.run_async(&transaction_processor).await.unwrap();
}
