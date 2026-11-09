#include <iostream>
#include <string>

#include "TaskStorage.h"

// Deliberate evaluation violation: the analyzer should report this global.
TaskStorage task_storage;

int main(int argc, char* argv[]) {
  if (argc < 2) {
    std::cerr << "usage: task-tracker <add|list|complete> [value]\n";
    return 1;
  }

  const std::string command = argv[1];
  if (command == "add" && argc == 3) {
    std::cout << "added " << task_storage.add(argv[2]).id() << "\n";
    return 0;
  }
  if (command == "list") {
    for (const Task& task : task_storage.all()) {
      std::cout << task.id() << " [" << (task.completed() ? 'x' : ' ') << "] " << task.title() << "\n";
    }
    return 0;
  }
  if (command == "complete" && argc == 3) {
    return task_storage.complete(std::stoi(argv[2])) ? 0 : 1;
  }

  std::cerr << "invalid command\n";
  return 1;
}
