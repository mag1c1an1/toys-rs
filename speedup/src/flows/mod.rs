use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{StreamExt, join};
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio::task::JoinSet;
use tracing::debug;
use tracing_subscriber::fmt::format;

use crate::demux::{demux, start_demux_task};
use crate::sink::CsvSink;
use crate::{coalesce, make_stream, sink};

pub async fn flow1() {
    let mut join_set = JoinSet::new();
    let num = Arc::new(AtomicUsize::new(0));
    for i in 0..8 {
        let nc = num.clone();
        join_set.spawn(async move {
            let mut s = make_stream(8, i);

            // let f = tokio::fs::File::create_new(format!("output-{i}.csv"))
            //     .await
            //     .unwrap();
            // let bw = BufWriter::with_capacity(1024 * 1024 * 4, f);
            // let mut sink = CsvSink::new(bw);
            // while let Some(rb) = s.next().await {
            //     sink.write(&rb).await;
            // }
            // sink.flush().await;
            while let Some(rb) = s.next().await {
                nc.fetch_add(rb.num_rows(), Ordering::Relaxed);
            }
        });
    }
    join_set.join_all().await;
    println!("{} Rows Total", num.load(Ordering::Relaxed));
}

pub async fn flow2() {
    let stream = coalesce();
    debug!("coalesce ok");
    demux(stream).await
}

pub async fn flow3() {
    let mut stream = coalesce();

    // let fut = tokio::spawn(async move {
    //     let file = File::create("output.csv").await.unwrap();
    //     let bw = BufWriter::with_capacity(1024 * 1024 * 4, file);
    //     let mut sink = CsvSink::new(bw);
    //     while let Some(rb) = stream.next().await {
    //         sink.write(&rb).await;
    //     }
    // });

    // fut.await.unwrap();

    let mut num = 0;
    while let Some(rb) = stream.next().await {
        num += rb.num_rows();
    }
    println!("{num} Rows Total");
}
