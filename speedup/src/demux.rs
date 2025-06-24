// SPDX-FileCopyrightText: LakeSoul Contributors
//
// SPDX-License-Identifier: Apache-2.0

use arrow::array::RecordBatch;
use futures::{Stream, StreamExt};
use rand::distr::SampleString;
use tokio::{
    sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::debug;

use crate::stream::SendableRecordBatchStream;

type RecordBatchReceiver = Receiver<RecordBatch>;
pub type DemuxedStreamReceiver = UnboundedReceiver<(String, RecordBatchReceiver)>;

pub fn start_demux_task(
    s: SendableRecordBatchStream,
) -> (
    JoinHandle<()>,
    UnboundedReceiver<(String, RecordBatchReceiver)>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let task = tokio::spawn(async move { row_count_demuxer("./", tx, s).await });
    (task, rx)
}

pub async fn row_count_demuxer(
    base_output_path: &str,
    mut tx: UnboundedSender<(String, Receiver<RecordBatch>)>,
    mut input: SendableRecordBatchStream,
) {
    let max_rows_per_file = 50000000;
    let max_buffered_batches = 2;
    let minimum_parallel_files = 8;
    let mut part_idx = 0;
    let write_id = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);

    let mut open_file_streams = Vec::with_capacity(minimum_parallel_files);

    let mut next_send_steam = 0;

    let mut row_counts = Vec::with_capacity(minimum_parallel_files);

    let file_extension = String::from("csv");

    while let Some(rb) = input.next().await {
        // ensure we have at least minimum_parallel_files open
        debug!("receive rb");
        if open_file_streams.len() < minimum_parallel_files {
            open_file_streams.push(create_new_file_stream(
                &base_output_path,
                &write_id,
                part_idx,
                &file_extension,
                max_buffered_batches,
                &mut tx,
            ));
            row_counts.push(0);
            part_idx += 1;
        } else if row_counts[next_send_steam] >= max_rows_per_file {
            // 超过了限制就开新流
            // 当当前文件流写入的行数达到 max_rows_per_file 时，关闭并替换为新的文件流。
            row_counts[next_send_steam] = 0;
            open_file_streams[next_send_steam] = create_new_file_stream(
                &base_output_path,
                &write_id,
                part_idx,
                &file_extension,
                max_buffered_batches,
                &mut tx,
            );
            part_idx += 1;
        }
        row_counts[next_send_steam] += rb.num_rows();
        open_file_streams[next_send_steam].send(rb).await.unwrap();

        next_send_steam = (next_send_steam + 1) % minimum_parallel_files;
    }
}

/// Helper for row count demuxer
fn generate_file_path(
    base_output_path: &str,
    write_id: &str,
    part_idx: usize,
    file_extension: &str,
) -> String {
    format!("{base_output_path}/{write_id}_{part_idx}.{file_extension}")
}

/// Helper for row count demuxer
fn create_new_file_stream(
    base_output_path: &str,
    write_id: &str,
    part_idx: usize,
    file_extension: &str,
    max_buffered_batches: usize,
    tx: &mut UnboundedSender<(String, Receiver<RecordBatch>)>,
) -> Sender<RecordBatch> {
    let file_path = generate_file_path(base_output_path, write_id, part_idx, file_extension);
    let (tx_file, rx_file) = mpsc::channel(max_buffered_batches / 2);
    tx.send((file_path, rx_file)).unwrap();
    tx_file
}
