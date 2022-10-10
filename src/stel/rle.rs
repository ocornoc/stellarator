#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct RawCompressedBytes(Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedRun {
    pub copies: usize,
    pub run: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CompressedData(Vec<CompressedRun>);

impl CompressedData {
    pub fn decompress(&self) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut buffer;
        for segment in self.0.iter() {
            buffer = segment.run.repeat(segment.copies);
            decompressed.append(&mut buffer);
        }
        decompressed
    }

    /*
    pub fn compress(mut bytes: &[u8]) -> Self {
        todo!()
    }
    */
}
