

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TooMuchData(usize),
    NoPattern(Vec<u8>),
    NoData,
    InvalidJson,
    InvalidGZip
}

