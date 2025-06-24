#![allow(dead_code)]

use std::pin::Pin;

use arrow::array::RecordBatch;
use futures::{Stream, StreamExt};
use tokio::{
    sync::mpsc::{self, Receiver, unbounded_channel},
    task::JoinSet,
};
use tpchgen::generators::LineItemGenerator;
use tpchgen_arrow::LineItemArrow;

use crate::demux::row_count_demuxer;

mod coalesce;
mod demux;
mod stream;

async fn make_stream(par: usize, part: usize) -> impl Stream<Item = RecordBatch> + Unpin {
    let g = LineItemGenerator::new(10.0, (part + 1) as i32, par as i32);
    let a = LineItemArrow::new(g);
    futures::stream::iter(a)
}

async fn flow1() {
    let mut join_set = JoinSet::new();
    for i in 0..8 {
        join_set.spawn(async move {
            let s = make_stream(8, i).await;
            csv_sink(i, s).await;
        });
    }
    join_set.join_all().await;
}

async fn flow2() {
    let s = coalesce().await;
    demux(s).await
}

async fn demux(s: Pin<Box<impl Stream<Item = RecordBatch> + Send>>) {
    let (tx, mut rx) = unbounded_channel::<(String, Receiver<RecordBatch>)>();

    let _x = row_count_demuxer("./", tx, s).await;

    let mut join_set = JoinSet::new();
    while let Some((p, mut rx)) = rx.recv().await {
        join_set.spawn(async move {
            let file = std::fs::File::create_new(p).unwrap();
            let mut w = arrow::csv::Writer::new(file);

            while let Some(rb) = rx.recv().await {
                w.write(&rb).unwrap()
            }
        });
    }
    join_set.join_all().await;
}

async fn coalesce() -> Pin<Box<impl Stream<Item = RecordBatch> + Send>> {
    let (tx, mut rx) = mpsc::channel(2);
    let mut join_set = JoinSet::new();
    for i in 0..8 {
        let txc = tx.clone();
        join_set.spawn(async move {
            let mut s = make_stream(8, i).await;
            while let Some(rb) = s.next().await {
                txc.send(rb).await.unwrap();
            }
        });
    }
    // Box::pin(stream)
    let stream = async_stream::stream! {
        while let Some(item) = rx.recv().await {
            yield item;
        }
    };
    join_set.join_all().await;
    Box::pin(stream)
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(flow2())
}

async fn csv_sink(part: usize, mut stream: impl Stream<Item = RecordBatch> + Unpin) {
    let file = std::fs::File::create_new(format!("output-{}.csv", part)).unwrap();
    let mut w = arrow::csv::Writer::new(file);

    while let Some(rb) = stream.next().await {
        w.write(&rb).unwrap();
    }
}

// fn single_sink(par: usize) {
//     _sinke
// }
