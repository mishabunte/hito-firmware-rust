extern crate alloc;
use alloc::vec::Vec;

const HITO_BLE_MAX_PACKET_LEN: usize = 512;

pub trait HitoBLE {
    fn init(&mut self);
    fn start(&mut self);
    fn stop(&mut self);

    fn has_data(&self) -> bool;

    fn has_error(&self) -> bool;
    fn clear_error(&self);

    fn has_data_package(&self) -> bool;
    fn is_connected(&self) -> bool;

    fn get_data(&mut self) -> Option<Vec<u8>>;
    fn get_data_length(&self) -> u16;
    fn clear_data(&mut self);
    fn clear_data_package(&mut self);

    fn get_data_package(&mut self) -> Option<Vec<u8>>;
    fn get_data_package_length(&self) -> u16;

    fn send(&mut self, data: &[u8]) -> bool;
}