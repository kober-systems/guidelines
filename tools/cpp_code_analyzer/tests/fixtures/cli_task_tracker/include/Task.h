#pragma once

#include <string>
#include <utility>

class Task {
 public:
  Task(int id, std::string title) : id_(id), title_(std::move(title)) {}

  int id() const { return id_; }
  const std::string& title() const { return title_; }
  bool completed() const { return completed_; }
  void complete() { completed_ = true; }

 private:
  int id_;
  std::string title_;
  bool completed_ = false;
};
