#include <unity.h>

void test_task_storage_adds_and_completes_tasks();

void setup() {
  UNITY_BEGIN();
  RUN_TEST(test_task_storage_adds_and_completes_tasks);
  UNITY_END();
}

void loop() {}
