#include <cstddef>
#include <cstdint>
#include <iostream>
#include <vector>

#include "./generated.h"

int main() {
  auto editor = rust::rustyline::DefaultEditor::new_().unwrap();
  if (editor.load_history("history.txt"_rs).is_err()) {
    std::cout << "No previous history." << std::endl;
  }
  while (true) {
    auto r = editor.readline(">>> "_rs);
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
      auto s = r.as_ref().unwrap().as_str();
      std::string cpp_s((char *)s.as_ptr(), s.len());
      std::cout << "Line: " << cpp_s << std::endl;
      editor.add_history_entry(s);
    }
  }
  editor.save_history("history.txt"_rs);
}
