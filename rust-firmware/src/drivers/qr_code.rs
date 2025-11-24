use qrcodegen_no_heap::QrCode;
use qrcodegen_no_heap::QrCodeEcc;
use qrcodegen_no_heap::Version;
extern crate alloc;
use alloc::vec::Vec;
use crate::log_info;

pub const IMAGE_MAX_WIDTH: usize = 170;

static mut QR_CODE_OUT_BUFFER: [u8; Version::MAX.buffer_len()] = [0u8; Version::MAX.buffer_len()];
static mut QR_CODE_TEMP_BUFFER: [u8; Version::MAX.buffer_len()] = [0u8; Version::MAX.buffer_len()];

fn qr_code_to_u8_vec(qrcode: &QrCode) -> Vec<u8> {
    let mut data = Vec::with_capacity(qrcode.size() as usize * qrcode.size() as usize);
    for y in 0..qrcode.size() {
        for x in 0..qrcode.size() {
            data.push(match qrcode.get_module(x, y) {
                true => 1,
                false => 0,
            });
        }
    }
    data
}

pub struct QrCodeWrapper {
    data: Vec<u8>,
    width: usize,
}

impl QrCodeWrapper {
    pub fn new(data: &str) -> Self {
        unsafe {
            let qr = QrCode::encode_text(data,
            &mut QR_CODE_TEMP_BUFFER, &mut QR_CODE_OUT_BUFFER, QrCodeEcc::Low,
            Version::MIN, Version::MAX, None, true).unwrap();
            log_info!("Generated QR code of size {}", qr.size());
            Self { data: qr_code_to_u8_vec(&qr), width: qr.size() as usize }
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}