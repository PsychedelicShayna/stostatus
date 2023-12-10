use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;

mod data;
mod gzip;
mod magic;
mod json;
mod search;

const STO_API_ENDPOINT: &str = "startreklauncher.crypticstudios.com";

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    TooMuchData(usize),
    NoData,
}

#[derive(Debug, Clone, Copy)]
enum HttpMethod {
    POST,
    GET,
}

impl From<&HttpMethod> for String {
    fn from(value: &HttpMethod) -> Self {
        match value {
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::GET => "GET".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct HttpRequest {
    url: String,
    endpoint: String,
    port: u16,
    method: HttpMethod,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    fn construct(&self) -> String {
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

    fn perform(&self) -> Result<Vec<u8>, Error> {
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

        Ok(Vec::<u8>::from(&buffer[0..nbytes_read]))
    }

    fn new(
        url: String,
        endpoint: String,
        method: HttpMethod,
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

fn get_sto_status() -> Result<Vec<u8>, Error> {
    let headers: Vec<(String, String)> = vec![
        ("Host", "startreklauncher.crypticstudios.com"),
        ("Connection", "keep-alive"),
        ("Content-Length", "0"),
        ("Accept", "application/json, text/javascript, */*, q=0.01"),
        ("User-Agent", "Mozilla/4.0 (compatible, CrypticLauncher)"),
        ("X-Accept-Language-Cryptic", "en-US"),
        ("X-Cryptic-Affiliate", "appid=9900"),
        ("X-Cryptic-Version", "3"),
        ("X-Requested-With", "XMLHttpRequest"),
        ("Origin", "http://startreklauncher.crypticstudios.com"),
        (
            "Referer",
            "http://startreklauncher.crypticstudios.com/launcher",
        ),
        ("Accept-Encoding", "gzip, deflate"),
        ("Accept-Language", "en-US,en,q=0.9"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    let request = HttpRequest::new(
        STO_API_ENDPOINT.to_string(),
        "/server_status/".to_string(),
        HttpMethod::GET,
        headers,
        None,
    );

    println!("{:?}", request);
    println!("{}", (0..79).map(|_| "-").collect::<String>());
    println!("{:?}", request.construct());

    let response = request.perform()?;

    Ok(response)
}

fn main() {
    let response: Vec<u8> = get_sto_status().unwrap();
    
    let response_rf = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("./data/response.raw");

    if let Ok(mut response_rf) = response_rf {
        response_rf.write_all(&response);
    } else {
        println!("Could not open response.raw file for writing.");
    }

    let mut gzip_payload = magic::extract_gzip_payload(&response);

    if let Ok(mut gzip_payload) = gzip_payload {
        unsafe {
            let inflated_payload = gzip::gzip_inflate(&mut gzip_payload);

            match inflated_payload {
                Ok(inflated_payload) => {
                    println!("Length {}: {:?}", &inflated_payload.len(), &inflated_payload);

                    let server_status = magic::extract_server_status(&inflated_payload);

                    let file = fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open("./gzip-inflated-payload.json");

                    if let Ok(mut file) = file {
                        file.write_all(&inflated_payload);
                    } else {
                        println!("Could not open file for writing.");
                    }
                }
                Err(e) => {
                    println!("Could not inflate GZip payload: {:?}", e);
                }
            }
        }
    } else {
        println!("Could not find GZip payload.");
    }
}


