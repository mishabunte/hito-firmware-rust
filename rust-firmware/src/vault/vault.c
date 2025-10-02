#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <hw_unique_key.h>
#include <drivers/flash.h>
#include <sys/crc.h>

void rust_hw_unique_key_is_written() {
  bool res = hw_unique_key_is_written(HUK_KEYSLOT_MKEK);
  if (!res) {
    hw_unique_key_write_random();
	}
}

bool hitoVaultWriteFlash(const void *offset, const void *data, size_t len)
{
  #ifdef __ZEPHYR__
    const struct device *flash_dev;
    flash_dev = device_get_binding(DT_CHOSEN_ZEPHYR_FLASH_CONTROLLER_LABEL);
    // LOG_DBG("Try to write 0x%x to 0x%x", data, offset);
    if (flash_write(flash_dev, (int)offset, data, len) == 0)
    {
      return true;
    } else 
    {
      printk("Flash write error");
      return false;
    }
  #else
    // LOG_DBG("Try to write 0x%x to 0x%x", data, offset);
    memcpy(offset, data, len);
    // TODO check
    return true;
  #endif
}