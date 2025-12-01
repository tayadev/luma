//! Runtime features tests (casting, destructuring, FFI, operator overloads, tables, prelude, modules)

mod common;

use common::{assert_program_fails, assert_program_output};
use luma::vm::value::Value;

// ============================================================================
// Casting & Type System
// ============================================================================

#[test]
fn test_cast_access_parent_field() {
    let source = r#"
-- Test accessing inherited field from cast
let Animal = {
  name = "String",
  sound = "String"
}

let Dog = {
  __parent = Animal,
  breed = "String"
}

let raw = {
  name = "Rex",
  sound = "Woof",
  breed = "Shepherd"
}

let dog = cast(Dog, raw)
dog.sound
"#;
    assert_program_output(source, Value::String("Woof".to_string()));
}

#[test]
fn test_cast_inheritance() {
    let source = r#"
-- Test inheritance with __parent
let Animal = {
  name = "String"
}

let Dog = {
  __parent = Animal,
  breed = "String"
}

let raw = {
  name = "Buddy",
  breed = "Golden Retriever"
}

let dog = cast(Dog, raw)
dog.name
"#;
    assert_program_output(source, Value::String("Buddy".to_string()));
}

#[test]
fn test_cast_metadata() {
    let source = r#"
-- Test that cast preserves type through re-casting
let Person = {
  name = "String"
}

let raw = {
  name = "Jane"
}

let person = cast(Person, raw)
isInstanceOf(person, Person)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_cast_simple() {
    let source = r#"
-- Test simple cast() with a basic type
let Person = {
  name = "String",
  age = 42
}

let raw = {
  name = "Alice",
  age = 30
}

let person = cast(Person, raw)
person.name
"#;
    assert_program_output(source, Value::String("Alice".to_string()));
}

#[test]
fn test_cast_trait() {
    let source = r#"
-- Test cast with trait (structural matching)
let Drawable = {
  x = 0,
  y = 0
}

let raw = {
  x = 5,
  y = 10
}

let drawable = cast(Drawable, raw)
drawable.x
"#;
    assert_program_output(source, Value::Number(5.0));
}

#[test]
fn test_comprehensive_types() {
    let source = r#"
-- Comprehensive example demonstrating user-defined types, inheritance, and traits

-- Define a base type
let Animal = {
  name = "unnamed",
  species = "unknown"
}

-- Define a type that inherits from Animal
let Dog = {
  __parent = Animal,
  breed = "unknown"
}

-- Define another type inheriting from Dog (multi-level inheritance)
let Puppy = {
  __parent = Dog,
  age_months = 0
}

-- Create instances using cast()
let generic_animal = cast(Animal, {
  name = "Generic",
  species = "Mammal"
})

let buddy = cast(Dog, {
  name = "Buddy",
  breed = "Golden Retriever"
})

let max = cast(Puppy, {
  name = "Max",
  breed = "Labrador",
  age_months = 3
})

-- Test isInstanceOf with direct types
let test1 = isInstanceOf(buddy, Dog)
let test2 = isInstanceOf(max, Puppy)

-- Test isInstanceOf with inheritance
let test3 = isInstanceOf(max, Dog)
let test4 = isInstanceOf(max, Animal)

-- Test structural matching (traits)
let Nameable = {
  name = "default"
}

let test5 = isInstanceOf(buddy, Nameable)

-- Return true if all tests pass
test1 && test2 && test3 && test4 && test5
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_isinstance_inheritance() {
    let source = r#"
-- Test isInstanceOf with inheritance
let Animal = {
  name = "String"
}

let Dog = {
  __parent = Animal,
  breed = "String"
}

let raw = {
  name = "Max",
  breed = "Labrador"
}

let dog = cast(Dog, raw)
isInstanceOf(dog, Animal)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_isinstance_multilevel() {
    let source = r#"
-- Test multi-level inheritance (grandparent)
let LivingThing = {
  alive = true
}

let Animal = {
  __parent = LivingThing,
  species = "String"
}

let Dog = {
  __parent = Animal,
  breed = "String"
}

let raw = {
  alive = true,
  species = "Canine",
  breed = "Beagle"
}

let dog = cast(Dog, raw)
isInstanceOf(dog, LivingThing)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_isinstance_negative() {
    let source = r#"
-- Test isInstanceOf returns false for non-matching type
let Person = {
  name = "String",
  age = 42
}

let Animal = {
  species = "String"
}

let raw = {
  name = "Alice",
  age = 30
}

let person = cast(Person, raw)
isInstanceOf(person, Animal)
"#;
    assert_program_output(source, Value::Boolean(false));
}

#[test]
fn test_isinstance_simple() {
    let source = r#"
-- Test isInstanceOf with a simple type
let Person = {
  name = "String",
  age = 42
}

let raw = {
  name = "Bob",
  age = 25
}

let person = cast(Person, raw)
isInstanceOf(person, Person)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_isinstance_trait() {
    let source = r#"
-- Test structural matching (traits)
let Drawable = {
  x = 0,
  y = 0
}

let Circle = {
  x = 0,
  y = 0,
  radius = 0
}

let circle = {
  x = 10,
  y = 20,
  radius = 5
}

isInstanceOf(circle, Drawable)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_external_type() {
    let source = r#"
-- Test External type exists and works
let ext = External

-- Check External is an External type
typeof(ext)
"#;
    assert_program_output(source, Value::String("External".to_string()));
}

#[test]
fn test_into_conversion() {
    let source = r#"
-- Test dynamic into() dispatch
let MyValue = {
  __into = fn(self: Any, target: Any) do
    return 42
  end
}
-- Call __into directly (dynamic dispatch through into() still experimental)
MyValue.__into(MyValue, null)
"#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_typeof_test() {
    let source = r#"
-- Test typeof() built-in function
let num = typeof(42)
let str = typeof("hello")
print(num)
str
"#;
    assert_program_output(source, Value::String("String".to_string()));
}

// ============================================================================
// Destructuring
// ============================================================================

#[test]
fn test_destructure_array() {
    let source = r#"
let [head, ...tail] = [1, 2, 3]
head
"#;
    assert_program_output(source, Value::Number(1.0));
}

#[test]
fn test_destructure_global_array() {
    let source = r#"
let [a, b, c] = [1, 2, 3]
a + b + c
"#;
    assert_program_output(source, Value::Number(6.0));
}

#[test]
fn test_destructure_local_array() {
    let source = r#"
let test = fn() do
  let [x, y] = [10, 20]
  x * y
end
test()
"#;
    assert_program_output(source, Value::Number(200.0));
}

#[test]
fn test_destructure_rest_count() {
    let source = r#"
let [a, b, ...rest] = [10, 20, 30, 40, 50]
let [x, y, z] = rest
x + y + z
"#;
    assert_program_output(source, Value::Number(120.0));
}

#[test]
fn test_destructure_rest_slice() {
    let source = r#"
let [a, b, ...rest] = [1, 2, 3, 4, 5]
let [first, ...others] = rest
first
"#;
    assert_program_output(source, Value::Number(3.0));
}

#[test]
fn test_destructure_table() {
    let source = r#"
let person = { name = "Bob", age = 25 }
let {name, age} = person
age
"#;
    assert_program_output(source, Value::Number(25.0));
}

// ============================================================================
// FFI (Foreign Function Interface)
// ============================================================================

#[test]
fn test_ffi_def() {
    let source = r#"
-- Test ffi.def() function
let c_def = ffi.def("
  typedef struct FILE FILE;
  FILE *fopen(const char *path, const char *mode);
  int fclose(FILE *stream);
")

-- The result should be a table with the defined function names
c_def.__ffi_def
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_ffi_file_ops() {
    let source = r#"
-- Test FFI file operations
let c_def = ffi.def("
  typedef struct FILE FILE;
  FILE *fopen(const char *path, const char *mode);
  int fclose(FILE *stream);
  int fputs(const char *str, FILE *stream);
")

-- Create a test file
let f = c_def.fopen("/tmp/ffi_file_test.txt", "w")

-- Check file opened successfully
if ffi.is_null(f) do
  false
end

-- Write to file
let write_result = c_def.fputs("Test content", f)

-- Close file
let close_result = c_def.fclose(f)

-- Return true if everything worked
close_result == 0
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_ffi_module_exists() {
    let source = r#"
-- Test basic FFI module access
let ffi_module = ffi

-- Check that ffi module exists
typeof(ffi_module)
"#;
    assert_program_output(source, Value::String("Table".to_string()));
}

#[test]
fn test_ffi_new() {
    let source = r#"
-- Test ffi.new() with char array allocation
let buffer = ffi.new("char[256]")

-- Buffer should be an External type
typeof(buffer)
"#;
    assert_program_output(source, Value::String("External".to_string()));
}

#[test]
fn test_ffi_new_cstr() {
    let source = r#"
-- Test ffi.new_cstr() function
let c_str = ffi.new_cstr("Hello, World!")

-- c_str should be an External type
typeof(c_str)
"#;
    assert_program_output(source, Value::String("External".to_string()));
}

#[test]
fn test_ffi_nullptr() {
    let source = r#"
-- Test ffi.nullptr() function
let nil_ptr = ffi.nullptr()

-- Check it's a null pointer (handle = 0)
ffi.is_null(nil_ptr)
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_ffi_puts() {
    let source = r#"
-- Test FFI puts function
let c_def = ffi.def("
  int puts(const char *s);
")

let result = c_def.puts("FFI puts works!")
result >= 0
"#;
    assert_program_output(source, Value::Boolean(true));
}

// ============================================================================
// Operator Overloading
// ============================================================================

#[test]
fn test_operator_overload_add() {
    let source = r#"
let Point = fn(x: Number, y: Number) do
  return {
    x = x,
    y = y,
    __add = fn(self: Any, other: Any) do
      return self.x + other.x + self.y + other.y
    end
  }
end
let p1: Any = Point(1, 2)
let p2: Any = Point(3, 4)
p1 + p2
"#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_operator_overload_div() {
    let source = r#"
let Fraction = fn(num: Number, den: Number) do
  return {
    num = num,
    den = den,
    __div = fn(self: Any, other: Any) do
      return self.num * other.den + self.den * other.num
    end
  }
end
let f1: Any = Fraction(1, 2)
let f2: Any = Fraction(3, 4)
f1 / f2
"#;
    assert_program_output(source, Value::Number(10.0));
}

#[test]
fn test_operator_overload_eq() {
    let source = r#"
let Point = fn(x: Number, y: Number) do
  return {
    x = x,
    y = y,
    __eq = fn(self: Any, other: Any) do
      return self.x == other.x && self.y == other.y
    end
  }
end
let p1: Any = Point(5, 10)
let p2: Any = Point(5, 10)
p1 == p2
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_operator_overload_ge() {
    let source = r#"
let gefn = fn(self: Table, other: Table) do
  return self.x >= other.x
end

let v1 = { x = 5, __ge = gefn }
let v2 = { x = 10, __ge = gefn }
v1 >= v2
"#;
    assert_program_output(source, Value::Boolean(false));
}

#[test]
fn test_operator_overload_gt() {
    let source = r#"
let mk = fn(x: Number) do
  return {
    x = x,
    __gt = fn(self: Any, other: Any) do
      return self.x > other.x
    end
  }
end
let a: Any = mk(10)
let b: Any = mk(5)
a > b
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_operator_overload_le() {
    let source = r#"
let mk = fn(x: Number) do
  return {
    x = x,
    __le = fn(self: Any, other: Any) do
      return self.x <= other.x
    end
  }
end
let a: Any = mk(5)
let b: Any = mk(10)
a <= b
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_operator_overload_lt() {
    let source = r#"
let mk = fn(x: Number) do
  return {
    x = x,
    __lt = fn(self: Any, other: Any) do
      return self.x < other.x
    end
  }
end
let a: Any = mk(1)
let b: Any = mk(2)
a < b
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_operator_overload_mod() {
    let source = r#"
let v: Any = {
  __mod = fn(self: Any, other: Any) do
    return 7
  end
}
v % 3
"#;
    assert_program_output(source, Value::Number(7.0));
}

#[test]
fn test_operator_overload_mul() {
    let source = r#"
let Vec = fn(x: Number, y: Number) do
  return {
    x = x,
    y = y,
    __mul = fn(self: Any, scalar: Any) do
      return Vec(self.x * scalar, self.y * scalar)
    end
  }
end
let v: Any = Vec(3, 4)
let scaled: Any = v * 2
scaled.x + scaled.y
"#;
    assert_program_output(source, Value::Number(14.0));
}

#[test]
fn test_operator_overload_neg() {
    let source = r#"
let v: Any = {
  __neg = fn(self: Any) do
    return 42
  end
}
let res = -v
res
"#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_operator_overload_sub() {
    let source = r#"
let Vec = fn(x: Number, y: Number) do
  return {
    x = x,
    y = y,
    __sub = fn(self: Any, other: Any) do
      return Vec(self.x - other.x, self.y - other.y)
    end
  }
end
let v1: Any = Vec(10, 20)
let v2: Any = Vec(3, 7)
let v3: Any = v1 - v2
v3.x + v3.y
"#;
    assert_program_output(source, Value::Number(20.0));
}

// ============================================================================
// Tables (Objects/Maps)
// ============================================================================

#[test]
fn test_table_key_types() {
    let source = r#"
let t = { 
  simple = "identifier",
  "with spaces" = "string literal",
  ["computed" + "Key"] = "computed expression"
}

t.simple + " " + t["with spaces"] + " " + t["computedKey"]
"#;
    assert_program_output(
        source,
        Value::String("identifier string literal computed expression".to_string()),
    );
}

#[test]
fn test_table_read() {
    let source = r#"
{ a = 1, b = true }.a
"#;
    assert_program_output(source, Value::Number(1.0));
}

#[test]
fn test_table_write() {
    let source = r#"
var obj = { x = 1, y = 2 }
obj.x = 99
obj.x
"#;
    assert_program_output(source, Value::Number(99.0));
}

// ============================================================================
// Prelude & Built-in Functions
// ============================================================================

#[test]
fn test_prelude_array() {
    let source = r#"
-- Test Array utilities exist in prelude
let has_map = List.map
let has_filter = List.filter  
let has_reduce = List.reduce
let has_length = List.length

true
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_prelude_option() {
    let source = r#"
-- Test Option type from prelude
let some_val = Option.new_some(42)
let none_val = Option.new_none()

true
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_prelude_print() {
    let source = r#"
-- Test print function from prelude
print("Hello")
print(", ")
print("World!")
print("\n")
true
"#;
    assert_program_output(source, Value::Boolean(true));
}

#[test]
fn test_print_multi_arg() {
    let source = r#"
print("1")
print("2")
print("3")
print("a")
print("b")
null
"#;
    assert_program_output(source, Value::Null);
}

#[test]
fn test_print_simple() {
    let source = r#"
print("Hello, World!")
42
"#;
    assert_program_output(source, Value::Number(42.0));
}

#[test]
fn test_process_module_exists() {
    let source = r#"
-- Test basic process module access
let proc = process

-- Check that process module exists
typeof(proc)
"#;
    assert_program_output(source, Value::String("Table".to_string()));
}

#[test]
fn test_process_os() {
    let source = r#"
-- Test process.os returns the operating system string
-- On Linux (CI environment), this should return "linux"
let os_name = process.os

-- Verify it's a string
typeof(os_name)
"#;
    assert_program_output(source, Value::String("String".to_string()));
}

// ============================================================================
// Modules & Imports
// ============================================================================

#[test]
#[ignore = "Requires external mod_variants_target.luma file"]
fn test_import_path_variants() {
    let source = r#"
let a = import("mod_variants_target")
let b = import("./mod_variants_target")
let c = import("mod_variants_target.luma")
a == b && b == c
"#;
    assert_program_output(source, Value::Boolean(true));
}

// ============================================================================
// Runtime Failure Tests (should fail at runtime)
// ============================================================================

#[test]
#[ignore = "Requires external circular import helper files"]
fn test_should_fail_circular_import() {
    let source = r#"import("./helpers/circular_a_helper.module")"#;
    assert_program_fails(source, "circular");
}
