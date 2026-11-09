#include "TaskStorage.h"

#include <utility>

// Deliberate evaluation violation: the analyzer should report this global.
int storage_revision = 1;

Task TaskStorage::add(std::string title) {
  Task task(static_cast<int>(tasks_.size()) + 1, std::move(title));
  tasks_.push_back(task);
  return task;
}

std::vector<Task> TaskStorage::all() const {
  return tasks_;
}

bool TaskStorage::complete(int id) {
  for (Task& task : tasks_) {
    if (task.id() == id) {
      task.complete();
      return true;
    }
  }
  return false;
}

#ifdef UNIT_TEST
#include <unity.h>

void test_task_storage_adds_and_completes_tasks() {
  TaskStorage storage;
  const Task task = storage.add("write tests");

  TEST_ASSERT_EQUAL(1, task.id());
  TEST_ASSERT_TRUE(storage.complete(task.id()));
  TEST_ASSERT_TRUE(storage.all().front().completed());
}
#endif
