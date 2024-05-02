# static_assert_generic

Functionality for asserting statements at compile time, including those using const and type generics.\
It works by trying to evaluate a constant, and failing (via panicking at compile time) if the expression evaluates to false.
Since `cargo check` does not evaluate constants, `static_assert!`s with specified generics do not show up as errors,
and full `cargo build` compilations are needed instead.
This is a rather 'hack'y method of doing asserts, so I wouldn't be that surprised if future versions of rust break it.
For now, it still works as of 1.77.2.\
Attempts to add const generic functionality in the `static_assert` crate [have been made](https://github.com/nvzqz/static-assertions/issues/40),
but it doesn't seem like it'll be added anytime soon.\
These asserts are not present in function signatures or the type system in any way, possibly making it hard to reason about when creating any kind of abstraction.
You should probably use them sparingly, and explicitly document them in functions that rely on static asserts.

## Examples

```rust
// True statement, compiles.
static_assert!(() 1 + 2 < 17);



// False statement, does not compile:
static_assert!(() 45 * 25 < 3);

// error[E0080]: evaluation of constant value failed
//  |     static_assert!(() 45 * 25 < 3)
//  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'Static assert failed.'



// The panic message is optionally specified.
static_assert!(() 45 * 25 < 3 => "This is the error message!");
// the evaluated program panicked at 'This is the error message!'



// Putting an assert outsize of a function block will cause an syntax error. To get around that you can assign it to a `const`ant of type unit.
const FOO: () = static_assert!(() 1 + 1 == 2);
// But since you can't capture any generics outside of a function block, you might as well just use `assert!`.
const BAR: () = assert!(1 + 1 == 2);



// This will error conditionally, depending on the value of the generic.
fn foo<const N: usize>() {
    // Const generics used in the expression have to be passed in explicitly, along with their type.
    static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
    // Some other functionality.
}

// This will not show an error when using cargo check.
// However, attempting to compile still results in an error, as expected.
foo::<0>();

// error[E0080]: evaluation of `foo::Assert::<0>::CHECK` failed
//  |         static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
//  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'N must be a non-zero value!'
//
// note: the above error was encountered while instantiating `fn main::foo::<0>`
//  |     foo::<0>();


// Not declaring the generics present in the expression results in an error.
fn bar<const N: usize>() {
    static_assert!(() N != 0 => "N must be a non-zero value!");
    // can't use generic parameters from outer item
}


// Type generics can be used as well.
fn baz<T>() {
    static_assert!((T) std::mem::size_of::<T>() == 4 => "T must be 4 bytes long!");
}



// Unsized types need to be passed with this syntax:
fn baz<U: ?Sized>() {
    static_assert!((U?) true => "There isn't much you can statically check about unsized types.");
}



// Multiple generics can be used at a time.
fn baz<const N: usize, const M: usize, T>() {
   static_assert!((N: usize, M: usize) N > M => "N must be greater than M!");
   static_assert!((N: usize, T) N == std::mem::size_of::<T>() / 2 => "N must be half the size of T!");
}

baz::<4, 7, u64>(); // panics at "N must be greater than M!"
baz::<4, 1, u8>(); // panics at "N must be half the size_of T!"
```



I've attempted to find a method to do the same to restrict const generics for types rather than generics, analogous to a where clause:

```rust
fn foo<const N: usize>() where N != 0 {} // do it with static_assert!((N: usize) N != 0); instead
struct Foo<const N: usize> where N != 0 {}
```

But so far no convenient method of doing this seems to exist.
