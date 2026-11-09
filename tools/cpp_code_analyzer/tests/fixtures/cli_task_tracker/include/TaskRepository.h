#pragma once

#include <string>
#include <vector>

#include "Task.h"

class TaskRepository {
 public:
  virtual ~TaskRepository() = default;

  virtual Task add(std::string title) = 0;
  virtual std::vector<Task> all() const = 0;
  virtual bool complete(int id) = 0;
};
