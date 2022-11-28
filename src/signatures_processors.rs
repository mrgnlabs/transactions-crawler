use chrono::{Local, TimeZone};
use std::{sync::Arc, time::Duration};

use crate::signatures_crawler::SignaturesCrawlerContext;

pub async fn print_signatures_to_screen(ctx: Arc<SignaturesCrawlerContext>) {
    loop {
        let mut signatures = vec![];
        {
            let signatures_queue = ctx.signature_queue.lock().unwrap();
            while !signatures_queue.is_empty() {
                signatures.push(signatures_queue.pop().unwrap());
            }
        }
        if signatures.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }

        signatures.iter().for_each(|signature_data| {
            println!(
                "{}: {} - {:?}",
                Local
                    .timestamp_opt(signature_data.signature.block_time.unwrap(), 0)
                    .unwrap(),
                signature_data.indexing_address,
                signature_data.signature
            )
        });
    }
}

pub async fn consume_signatures(ctx: Arc<SignaturesCrawlerContext>) {
    loop {
        let mut signatures_data = vec![];
        {
            let signatures_queue = ctx.signature_queue.lock().unwrap();
            while !signatures_queue.is_empty() {
                signatures_data.push(signatures_queue.pop().unwrap());
            }
        }
        if signatures_data.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
    }
}
