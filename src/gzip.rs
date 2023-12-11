use std::{
    ffi::{c_ulong, c_void, CStr},
    io::{self},
    os::raw::{c_char, c_int},
};

#[repr(C)]
struct ZStream {
    next_in: *mut u8,
    avail_in: c_ulong,
    total_in: c_ulong,

    next_out: *mut u8,
    avail_out: c_ulong,
    total_out: c_ulong,

    msg: *const c_char,
    state: *mut c_void, // Pointer to internal state
    zalloc: Option<extern "C" fn(*mut c_void, c_ulong, c_ulong) -> *mut c_void>,
    zfree: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    opaque: *mut c_void,

    data_type: c_int,
    adler: c_ulong,
    reserved: c_ulong,
}

#[link(name = "z")]
extern "C" {
    // fn zlibVersion() -> *const c_char;

    fn inflateInit2_(
        strm: *mut ZStream,
        windowBits: c_int,
        version: *const c_char,
        stream_size: c_int,
    ) -> c_int;
    fn inflate(strm: *mut ZStream, flush: c_int) -> c_int;
    fn inflateEnd(strm: *mut ZStream) -> c_int;
}

pub unsafe fn gzip_inflate(compressed: &mut Vec<u8>) -> Result<Vec<u8>, io::Error> {
    // let version = unsafe {
    //     let c_str = zlibVersion();
    //     CStr::from_ptr(c_str).to_str().unwrap()
    // };

    let mut z_stream = ZStream {
        next_in: compressed.as_mut_ptr(),
        avail_in: compressed.len() as c_ulong,
        total_in: 0,

        next_out: std::ptr::null_mut(),
        avail_out: 0,
        total_out: 0,

        msg: std::ptr::null(),
        state: std::ptr::null_mut(),
        zalloc: None,
        zfree: None,
        opaque: std::ptr::null_mut(),

        data_type: 0,
        adler: 0,
        reserved: 0,
    };

    // Initialize zlib
    // 15 is the default, 32 is the "enable gzip decoding" flag
    let window_bits = 15 + 32;

    let init_result = inflateInit2_(
        &mut z_stream,
        window_bits,
        CStr::from_bytes_with_nul(b"1.3.00\0").unwrap().as_ptr(),
        std::mem::size_of::<ZStream>() as c_int,
    );

    if init_result != 0 {
        let error_message = match init_result {
            1 => "Stream error: invalid parameter passed to inflateInit2_".to_string(),
            2 => "Memory error: not enough memory to initialize zlib".to_string(),
            6 => "Version error: zlib library version (ZLIB_VERSION) is incompatible with the version assumed by the caller (ZLIB_VERNUM)".to_string(),
            n => format!("Unknown error {}", n)
        };

        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to initialize zlib, error: {}", error_message),
        ));
    }

    let mut decompressed: Vec<u8> = vec![];
    let mut buffer: Vec<u8> = vec![0; compressed.len() * 5];

    // let mut buffer: [u8; 1024] = [0; 1024];

    loop {
        z_stream.next_out = buffer.as_mut_ptr();
        z_stream.avail_out = buffer.len() as c_ulong;

        let inflate_result = inflate(&mut z_stream, 0);

        // Z_BUF_ERROR
        if inflate_result == -5 {
            let delta = buffer.len() - z_stream.avail_out as usize;
            decompressed.extend_from_slice(&buffer[0..delta]);
        } else if inflate_result != 0 && inflate_result != 1 {
            // Error or incomplete stream.
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to inflate data: {}", inflate_result),
            ));
        }
        // Decompression finished
        else {
            let bytes_written = buffer.len() - z_stream.avail_out as usize;
            decompressed.extend_from_slice(&buffer[0..bytes_written]);
            break;
        }
    }

    let result = inflateEnd(&mut z_stream);

    if result != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to end zlib stream",
        ));
    }

    Ok(decompressed)
}
