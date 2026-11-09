#pragma once

#include <string>
#include <vector>

#include "TaskRepository.h"

class TaskStorage : public TaskRepository {
 public:
  Task add(std::string title) override;
  std::vector<Task> all() const override;
  bool complete(int id) override;

 private:
  std::vector<Task> tasks_;
};
