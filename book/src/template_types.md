# Template types

Template type definitions provide a shorthand syntax to avoid repeating the definition of methods on common generic types. All of the items within a template type definition are copied into each matching concrete type definition. At the moment, template types are considered unstable and ust be activated with the `#unstable(template_types)` directive.

For example:

```
#unstable(template_types)

type<T> ::std::option::Option<T> {
    constructor Some(T);
    constructor None;
    fn unwrap(self) -> T;
    fn is_some(&self) -> bool;
}

type ::std::option::Option<i32> {
    #layout(size = 8, align = 8);
    wellknown_traits(Copy, Debug);
    // the constructors and methods are automatically defined
}
```

You can also define well-known traits and layouts in template types:

```
type<T> [T] {
    wellknown_traits(?Sized);
}

type<T> ::std::vec::Vec<T> {
    #layout(size = 24, align = 8);
}

type<T> option::Option<&T> {
    #layout(size = 8, align = 8);
    wellknown_traits(Copy);
}
```

Note that multiple template definitions can apply to the same concrete type, so `Option<&i32>` would inherit definitions from the `Option<T>` template and the `Option<&T>` template.

You can also override the layout given in a template:

```
type<T> Box<T> {
    #layout(size = 8, align = 8);
    // Most Boxed types are 8 bytes
}

type<T> Box<i32> {} // Inherits the 8 byte layouts

type Box<dyn crate::MyTrait> {
    #layout(size = 16, align = 8);
    // Boxed trait objects are 16 bytes
}
```

Zngur will still only emit C++ definitions for the concrete types that are defined. Template definitions on their own do not result in any C++ code. This also means that at the moment, any type mentioned in a template definition must be individually defined as well. For example, the following code will fail to compile unless `Option<String>` is defined to make `Result::<String, i64>::ok` valid. If the `Option<T>` template from above is defined, then we must add a definition for `String` as well to support `Option::<String>::unwrap`.

```
type<T, E> ::std::result::Result<T, E> {
    fn ok(self) -> ::std::option::Option<T>;
}

type ::std::result::Result<::std::string::String, i64> {
    #layout(size = 24, align = 8);
}
```
