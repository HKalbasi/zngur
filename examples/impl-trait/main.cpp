#include "./generated.h"

int main() {
  std::cout << "=== Testing impl Trait support in zngur ===" << std::endl;

  // Test 1: impl Trait in return position - create_person
  std::cout << "\n--- Test 1: create_person returns impl Greeter ---" << std::endl;
  auto person_greeter = rust::crate::create_person("Alice"_rs.to_owned());
  std::cout << person_greeter.greet() << std::endl;

  // Test 2: impl Trait in return position - create_robot
  std::cout << "\n--- Test 2: create_robot returns impl Greeter ---" << std::endl;
  auto robot_greeter = rust::crate::create_robot(42);
  std::cout << robot_greeter.greet() << std::endl;

  // Test 3: Traditional Box<dyn Greeter> return
  std::cout << "\n--- Test 3: create_greeter_by_type returns Box<dyn Greeter> ---" << std::endl;
  auto greeter1 = rust::crate::create_greeter_by_type(true, "Bob"_rs.to_owned(), 0);
  std::cout << greeter1.greet() << std::endl;

  auto greeter2 = rust::crate::create_greeter_by_type(false, ""_rs.to_owned(), 999);
  std::cout << greeter2.greet() << std::endl;

  // Test 4: Concrete type functions (impl Trait in argument position works internally)
  std::cout << "\n--- Test 4: Concrete type functions ---" << std::endl;
  rust::crate::Person person = rust::crate::Person{"Charlie"_rs.to_owned()};
  rust::crate::print_greeting_person(std::move(person));

  rust::crate::Robot robot = rust::crate::Robot{100};
  rust::crate::print_greeting_robot(std::move(robot));

  std::cout << "\n=== All impl Trait tests passed! ===" << std::endl;
  return 0;
}
