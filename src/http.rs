use crate::{error::Error, pattern::find_pattern};

use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug, Clone, Copy)]
pub enum Method {
    POST,
    GET,
}

impl From<&Method> for String {
    fn from(value: &Method) -> Self {
        match value {
            Method::POST => "POST".to_string(),
            Method::GET => "GET".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    data: Vec<u8>
    // status_code: u16,
}

impl Response {

    pub fn gz_extract(&self) -> Result<Vec<u8>, Error> {
        // let header_magic: Vec<u8> = vec![ 0x0d, 0x0a, 0x0d, 0x0a];
        let gzip_magic_number: Vec<u8> = vec![0x1f, 0x8b, 0x08];

        match find_pattern(&gzip_magic_number, &self.data) {
            Some((beg, _)) => Ok(self.data[beg..].to_vec()),
            None => Err(Error::NoData),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub endpoint: String,
    pub port: u16,
    pub method: Method,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn construct(&self) -> String {
        let mut data = String::new();

        data.push_str(format!("{} {}", String::from(&self.method), self.endpoint).as_str());
        data.push_str(&self.url);
        data.push_str(" HTTP/1.1\r\n");

        for (key, value) in &self.headers {
            data.push_str(&key);
            data.push_str(": ");
            data.push_str(&value);
            data.push_str("\r\n");
        }

        data.push_str("\r\n");

        data
    }

    pub fn send(&self) -> Result<Response, Error> {
        let mut stream = TcpStream::connect(format!("{}:{}", &self.url, self.port)).unwrap();
        let http_payload = self.construct();

        if let Err(e) = stream.write(http_payload.as_bytes()) {
            return Err(Error::IoError(e));
        }

        stream
            .write(http_payload.as_bytes())
            .map_err(|e| Error::IoError(e))?;

        // This is already a generous buffer size, but limited to prevent
        // the other side from just streaming as much data as they want.
        let mut buffer: [u8; 16384] = [0; 16384];

        let nbytes_read: usize = stream.read(&mut buffer).map_err(|e| Error::IoError(e))?;

        if nbytes_read == 0 {
            return Err(Error::NoData);
        } else if nbytes_read >= buffer.len() {
            return Err(Error::TooMuchData(nbytes_read));
        }


        Ok(Response {
            data: Vec::<u8>::from(&buffer[0..nbytes_read]),
        })
    }

    pub fn new(
        url: String,
        endpoint: String,
        method: Method,
        headers: Vec<(String, String)>,
        port: Option<u16>,
    ) -> Self {
        let headers: HashMap<String, String> = headers.into_iter().collect();

        Self {
            url,
            endpoint,
            port: port.unwrap_or(80),
            method,
            headers,
        }
    }
}
