use solana_sdk::pubkey;
use std::sync::Arc;
use transactions_crawler::{
    common::Target,
    signatures_crawler::{SignaturesCrawler, SignaturesCrawlerConfig, SignaturesCrawlerContext},
    signatures_processors::print_signatures_to_screen,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let sig_crawler = SignaturesCrawler::new_with_config(SignaturesCrawlerConfig::new(vec![
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

    let signature_processor =
        |ctx: Arc<SignaturesCrawlerContext>| async move { print_signatures_to_screen(ctx).await };

    sig_crawler.run_async(&signature_processor).await.unwrap();
}
