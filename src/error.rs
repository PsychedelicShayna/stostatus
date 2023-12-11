

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    TooMuchData(usize),
    NoPattern(Vec<u8>),
    NoData,
    InvalidJson,
    InvalidGZip
}

