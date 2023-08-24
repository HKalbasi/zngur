#include <cstddef>
#include <cstdint>
#include <iostream>
#include <vector>

#include "./generated.h"

int main() {
  auto editor = ::rust::rustyline::DefaultEditor::new_().unwrap();
  if (editor.load_history(::crate::rust_str((signed char *)"history.txt"))
          .is_err()) {
    std::cout << "No previous history." << std::endl;
  }
  while (true) {
    auto r = editor.readline(::crate::rust_str((signed char *)">>> "));
    if (r.is_err()) {
      auto e = r.unwrap_err();
      if (e.matches_Eof()) {
        std::cout << "CTRL-D" << std::endl;
      }
      if (e.matches_Interrupted()) {
        std::cout << "CTRL-C" << std::endl;
      }
      break;
    } else {
      auto owned_s = r.unwrap();
      auto s = owned_s.as_str();
      std::string cpp_s((char *)s.as_ptr(), s.len());
      std::cout << "Line: " << cpp_s << std::endl;
      editor.add_history_entry(::crate::rust_str((signed char *)cpp_s.c_str()));
    }
  }
  editor.save_history(::crate::rust_str((signed char *)"history.txt"));
}
