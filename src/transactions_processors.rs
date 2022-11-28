use chrono::{Local, TimeZone};
use std::{sync::Arc, time::Duration};

use crate::transactions_crawler::TransactionsCrawlerContext;

pub async fn print_transactions_to_screen(ctx: Arc<TransactionsCrawlerContext>) {
    loop {
        let mut transactions_data = vec![];
        {
            let signatures_queue = ctx.transaction_queue.lock().unwrap();
            while !signatures_queue.is_empty() {
                transactions_data.push(signatures_queue.pop().unwrap());
            }
        }
        if transactions_data.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }

        transactions_data.iter().for_each(|transaction_data| {
            println!(
                "{}: {} - {}",
                Local
                    .timestamp_opt(transaction_data.transaction.block_time.unwrap(), 0)
                    .unwrap(),
                transaction_data.indexing_address,
                transaction_data
                    .transaction
                    .transaction
                    .transaction
                    .decode()
                    .unwrap()
                    .signatures
                    .first()
                    .unwrap()
            )
        });
    }
}

pub async fn consume_transactions(ctx: Arc<TransactionsCrawlerContext>) {
    loop {
        let mut transactions_data = vec![];
        {
            let signatures_queue = ctx.transaction_queue.lock().unwrap();
            while !signatures_queue.is_empty() {
                transactions_data.push(signatures_queue.pop().unwrap());
            }
        }
        if transactions_data.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
    }
}
