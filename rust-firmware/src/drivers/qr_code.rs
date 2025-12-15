use qrcodegen_no_heap::QrCode;
use qrcodegen_no_heap::QrCodeEcc;
use qrcodegen_no_heap::Version;
extern crate alloc;
use alloc::vec::Vec;
use crate::log_info;

pub const IMAGE_MAX_WIDTH: usize = 170;

static mut QR_CODE_OUT_BUFFER: [u8; Version::MAX.buffer_len()] = [0u8; Version::MAX.buffer_len()];
static mut QR_CODE_TEMP_BUFFER: [u8; Version::MAX.buffer_len()] = [0u8; Version::MAX.buffer_len()];
static mut QR_CODE_WIDTH: usize = Version::MAX.buffer_len();

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

#[derive(Clone)]
pub struct QrCodeWrapper {
    data: Vec<u8>,
    width: usize,
    x: u16,
    y: u16,
}

impl QrCodeWrapper {
    pub fn new() -> Self {
        unsafe {
            Self {
                data: Vec::new(),
                width: 0,
                x: 0,
                y: 0,
            }
        }
    }

    pub fn set_coords(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_data(&mut self, data: &str) {
      unsafe {
        let qr = QrCode::encode_text(data,
        &mut QR_CODE_TEMP_BUFFER, &mut QR_CODE_OUT_BUFFER, QrCodeEcc::Low,
        Version::MIN, Version::MAX, None, true).unwrap();
        self.data = qr_code_to_u8_vec(&qr);
        log_info!("QR code generated with size: {}", qr.size());
        self.width = qr.size() as usize;
      }
    }

    pub fn get_coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}