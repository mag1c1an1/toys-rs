#![allow(dead_code)]

use arrow::array::RecordBatch;
use futures_lite::{Stream, StreamExt};
use tokio::task::JoinSet;
use tpchgen::generators::LineItemGenerator;
use tpchgen_arrow::LineItemArrow;

async fn make_stream(par: usize, part: usize) -> impl Stream<Item = RecordBatch> + Unpin {
    let g = LineItemGenerator::new(10.0, (part + 1) as i32, par as i32);
    let a = LineItemArrow::new(g);
    futures_lite::stream::iter(a)
}

fn coalesce() {
    todo!()
}

async fn single_sink() {}

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

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(flow1())
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
