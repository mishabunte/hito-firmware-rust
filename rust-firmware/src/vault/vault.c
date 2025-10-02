#include <hw_unique_key.h>

void rust_hw_unique_key_is_written() {
  bool res = hw_unique_key_is_written(HUK_KEYSLOT_MKEK);
  if (!res) {
    hw_unique_key_write_random();
	}
}