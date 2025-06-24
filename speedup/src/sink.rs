use arrow::{array::RecordBatch, csv::WriterBuilder};
use bytes::Bytes;
use tokio::io::AsyncWriteExt;

pub struct CsvSink<W: AsyncWriteExt + Unpin> {
    builder: WriterBuilder,
    writer: W,
}

impl<W: AsyncWriteExt + Unpin> CsvSink<W> {
    pub fn new(inner: W) -> Self {
        Self {
            builder: WriterBuilder::new(),
            writer: inner,
        }
    }

    fn serialize_batch(&self, batch: &RecordBatch) -> Bytes {
        let mut buffer = Vec::with_capacity(4096);
        let builder = self.builder.clone();
        let mut writer = builder.build(&mut buffer);
        writer.write(&batch).unwrap();
        drop(writer);
        Bytes::from(buffer)
    }

    pub async fn write(&mut self, batch: &RecordBatch) {
        self.writer
            .write(&self.serialize_batch(batch))
            .await
            .unwrap();
    }

    pub async fn flush(&mut self) {
        self.writer.flush().await.unwrap();
    }
}
