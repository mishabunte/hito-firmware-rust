mod hito_firmware;
mod drivers;

use hito_firmware::HitoFirmware;

fn main() {

    let mut firmware = HitoFirmware::new();

    firmware.init_hardware();
    firmware.main_loop();
}

