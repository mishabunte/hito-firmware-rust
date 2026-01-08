use crate::drivers::qr_code::QrCodeWrapper;

pub static mut QR_CODE: Option<QrCodeWrapper> = None;

pub fn set_qr_data(data: &str) {
    unsafe {
        if QR_CODE.is_none() {
            QR_CODE = Some(QrCodeWrapper::new());
        }
        QR_CODE.as_mut().unwrap().set_data(data);
        QR_CODE.as_mut().unwrap().set_centered_coords();
    }
}

pub fn set_qr_coords(x: u16, y: u16) {
    unsafe {
        if let Some(qr_code) = QR_CODE.as_mut() {
            qr_code.set_coords(x, y);
        }
    }
}
