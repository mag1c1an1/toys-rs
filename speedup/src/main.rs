#![allow(dead_code)]

use std::{sync::Arc, time::Instant};

use arrow::{array::RecordBatch, datatypes::Schema};
use futures::{Stream, StreamExt};
use tokio::{
    sync::mpsc::{Receiver, UnboundedReceiver},
    task::JoinSet,
};
use tpchgen::generators::LineItemGenerator;
use tpchgen_arrow::{LineItemArrow, RecordBatchIterator};
use tracing::debug;

use crate::stream::{
    RecordBatchReceiverStream, RecordBatchStreamAdapter, SendableRecordBatchStream,
};

mod coalesce;
mod demux;
mod stream;
mod task;

fn make_stream(par: usize, part: usize) -> SendableRecordBatchStream {
    let g = LineItemGenerator::new(10.0, (part + 1) as i32, par as i32);
    let a = LineItemArrow::new(g);
    let schema = a.schema().clone();
    let s = futures::stream::iter(a);
    let adapter = RecordBatchStreamAdapter::new(schema, s);
    Box::pin(adapter)
}

async fn flow1() {
    let mut join_set = JoinSet::new();
    for i in 0..8 {
        join_set.spawn(async move {
            let s = make_stream(8, i);
            csv_sink(i, s).await;
        });
    }
    join_set.join_all().await;
}

async fn flow2() {
    let stream = coalesce().await;
    debug!("coalesce ok");
    demux(stream).await
}

async fn demux(s: SendableRecordBatchStream) {
    let (task, file_stream_rx) = demux::start_demux_task(s);

    let f1 = async { task.await.unwrap() };

    let (_r1, _r2) = futures::join!(f1, write_files(file_stream_rx));
}

async fn write_files(mut file_stream_rx: UnboundedReceiver<(String, Receiver<RecordBatch>)>) {
    let mut join_set = JoinSet::new();
    while let Some((location, mut rb_stream)) = file_stream_rx.recv().await {
        join_set.spawn(async move {
            let file = std::fs::File::create_new(location).unwrap();
            let mut w = arrow::csv::Writer::new(file);
            while let Some(rb) = rb_stream.recv().await {
                w.write(&rb).unwrap()
            }
        });
    }

    debug!("after row count");
    join_set.join_all().await;
}

async fn coalesce() -> SendableRecordBatchStream {
    let empty_schema = Arc::new(Schema::empty());
    let mut builder = RecordBatchReceiverStream::builder(empty_schema, 8);
    for part_i in 0..8 {
        builder.run_input(part_i);
    }
    builder.build()
}

fn main() {
    tracing_subscriber::fmt::init();
    debug!("speed up start");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(16)
        .build()
        .unwrap();
    rt.block_on(async {
        let time = Instant::now();
        flow2().await;
        let elap = time.elapsed().as_secs_f64();
        println!("Elapsed {elap} seconds");
    });
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
