#include <stdio.h>
#include <string.h>
#include <stdlib.h>

void DumpHex(const char * header, const void *data, size_t len) 
{
  const char * data_char = (const char *)data;
  char *buf = (char *)malloc(len * 2 + 1);
  if (!buf) {
      return;
  }
  if (header) {
    fprintf(stderr, "%s", header);
  }
  for (size_t i = 0; i < len; i++) {
      snprintf(&buf[i * 2], 3, "%02x", data_char[i]);
  }
  buf[len * 2] = '\0';
  printf("%s\n", buf);
  free(buf);
}

