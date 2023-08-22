#include <cstddef>
#include <cstdint>
#include <iostream>
#include <vector>

#include "./generated.h"

int main() {
  auto editor = ::crate::build_editor();
  if (editor.load_history(::crate::rust_str((uint64_t) "history.txt"))
          .is_err()) {
    std::cout << "No previous history." << std::endl;
  }
  while (true) {
    auto r = editor.readline(::crate::rust_str((uint64_t) ">>> "));
    if (r.is_err()) {
      auto e = r.unwrap_err();
      break;
    } else {
      auto owned_s = r.unwrap();
      auto s = owned_s.as_str();
      std::string cpp_s((char *)::crate::as_ptr(s), (size_t)::crate::len(s));
      std::cout << "Line: " << cpp_s << std::endl;
      editor.add_history_entry(::crate::rust_str((uint64_t)cpp_s.c_str()));
    }
  }
  editor.save_history(::crate::rust_str((uint64_t) "history.txt"));
}
