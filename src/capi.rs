use decoder::{Decoder, PixelFormat};
use std::io::BufReader;
use std::os::raw::{c_char, c_void};

fn cmyk_to_rgb(input: &[u8]) -> Vec<u8> {
    let size = input.len() - input.len() / 4;
    let mut output = Vec::with_capacity(size);

    for pixel in input.chunks(4) {
        let c = pixel[0] as f32 / 255.0;
        let m = pixel[1] as f32 / 255.0;
        let y = pixel[2] as f32 / 255.0;
        let k = pixel[3] as f32 / 255.0;

        // CMYK -> CMY
        let c = c * (1.0 - k) + k;
        let m = m * (1.0 - k) + k;
        let y = y * (1.0 - k) + k;

        // CMY -> RGB
        let r = (1.0 - c) * 255.0;
        let g = (1.0 - m) * 255.0;
        let b = (1.0 - y) * 255.0;

        output.push(r as u8);
        output.push(g as u8);
        output.push(b as u8);
    }

    output
}

#[no_mangle]
pub extern "C" fn decoder(data: *const c_char, len: usize, height: *mut u16, width: *mut u16, channels: *mut u16) -> *const c_void {
    let data = unsafe { std::slice::from_raw_parts(data as *const _, len) };

    let mut dec = Decoder::new(BufReader::new(data));
    let mut data = if let Ok(data) = dec.decode() {
        data
    } else {
        unsafe {
            *height = 0;
            *width = 0;
            *channels = 0;
        }
        return std::ptr::null_mut();
    };

    let info = dec.info().unwrap();
    let bytes_per_pixel = match info.pixel_format {
        PixelFormat::L8 => 1,
        PixelFormat::RGB24 => 3,
        PixelFormat::CMYK32 => {
            data = cmyk_to_rgb(&mut data);
            3
        }
    };

    unsafe {
        *height = info.height;
        *width = info.width;
        *channels = bytes_per_pixel;
    }

    Box::into_raw(Box::new(data)) as *mut c_void
}
