#![allow(dead_code)]

use std::{io::BufWriter, sync::Arc, time::Instant};

use arrow::{array::RecordBatch, datatypes::Schema};
use futures::{Stream, StreamExt};

use tpchgen::generators::LineItemGenerator;
use tpchgen_arrow::{LineItemArrow, RecordBatchIterator};
use tracing::debug;

use crate::{
    flows::{flow1, flow2, flow3},
    stream::{RecordBatchReceiverStream, RecordBatchStreamAdapter, SendableRecordBatchStream},
};

mod demux;
mod flows;
mod sink;
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

fn coalesce() -> SendableRecordBatchStream {
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
        .build()
        .unwrap();
    rt.block_on(async {
        let time = Instant::now();
        // flow1().await;
        flow2().await;
        // flow3().await;
        let elap = time.elapsed().as_secs_f64();
        println!("Elapsed {elap} seconds");
    });
}
