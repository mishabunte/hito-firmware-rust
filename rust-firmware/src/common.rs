use crate::{drivers::{Display, qr_code::QrCodeWrapper}};

pub static mut QR_CODE: Option<QrCodeWrapper> = None;
pub static mut PROGRESS_BAR_PROPERTIES: Option<ProgressBarProperties> = None;

#[derive(Clone)]
pub struct ProgressBarProperties {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub border_width: u16,
}

pub fn ui_get_progress_bar_properties() -> Option<ProgressBarProperties> {
    unsafe {
        PROGRESS_BAR_PROPERTIES.clone()
    }
}

pub fn ui_set_progress_bar_properties(x: u16, y: u16, w: u16, h: u16, border_width: u16) {
    unsafe {
        PROGRESS_BAR_PROPERTIES = Some(ProgressBarProperties {
            x,
            y,
            w,
            h,
            border_width,
        });
    }
}


pub fn ui_draw_qr(data: &str) {
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
