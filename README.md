# static_assert_generic

Functionality for asserting statements at compile time, including those using const and type generics.

It works by trying to evaluate a constant and failing (via panicking at compile time) if the expression evaluates to false.
Since `cargo check` does not evaluate constants, `static_assert!`s with specified generics do not show up as errors,
and full `cargo build` compilations are needed instead.
This is a rather 'hack'y method of doing asserts, so I wouldn't be that surprised if future versions of rust break it.
For now, it still works as of 1.77.2.

Attempts to add const generic functionality in the `static_assert` crate [have been made](https://github.com/nvzqz/static-assertions/issues/40),
but it doesn't seem like it'll be added anytime soon.

These asserts are not present in function signatures or the type system in any way, possibly making it hard to reason about when creating any kind of abstraction.
You should probably use them sparingly and explicitly document them whenever they are used.

## Overview

Static asserts error conditionally, depending on the value of the generic:
```rust
fn foo<const N: usize>() {
    static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
    // Some other functionality.
}

fn main() {
    foo::<12>(); // compiles
    foo::<0>(); // doesn't compile
}
```

```rust
// error[E0080]: evaluation of `foo::Assert::<0>::CHECK` failed
//  |         static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
//  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'N must be a non-zero value!'
//
// note: the above error was encountered while instantiating `fn main::foo::<0>`
//  |     foo::<0>();
```

## Important #1
Static asserts that fail (such as `foo::<0>()` in this case) will not show an error when using `cargo check`.
However, attempting to compile (using `cargo build`) still results in an error, as expected.

## Important #2
Not specifying the type of the const generic will result in a `can't use generic parameters from outer item` error:

```rust
fn foo<const N: u32>() {
    static_assert!((N) N != 0 => "N must be a non-zero value!");
    // can't use generic parameters from outer item
}
```

This is not the macro being broken, this is just a misleading error message.
It can be fixed by simply specifying the type (`static_assert!((N: u32) N != 0)`).

## Important #3
Not declaring the generics present in the expression results in an error.

```rust
fn bar<const N: usize>() {
    static_assert!(() N != 0 => "N must be a non-zero value!");
    // can't use generic parameters from outer item
}
```

## Important #4
If a type generic that is `?Sized` gets passed in, it will result in an error:

```rust
fn foo<T: ?Sized>() {
    static_assert!((T) ...);
    // the associated item `CHECK` exists for struct `Assert<T>`, but its trait bounds were not satisfied
}
```

Optionally sized type generics need to be specified using `?` (`static_assert!((T?) ...);`).

## Examples

Asserting constant expressions:
```rust
static_assert!(() 1 + 2 < 17); // True statement, compiles.

static_assert!(() 45 * 25 < 3); // False statement, does not compile:

// error[E0080]: evaluation of constant value failed
//  |     static_assert!(() 45 * 25 < 3)
//  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'Static assert failed.'
```

\
An error message can be optionally specified:
```rust
static_assert!(() 45 * 25 < 3 => "This is the error message!");
// the evaluated program panicked at 'This is the error message!'
```

\
Pass in const generics using `identifier: type` syntax:
```rust
fn foo<const C: u32>() {
    static_assert!((C: u32) C > 3 => "C must be greater than 3!");
}
```

\
Type generics can be used as well.
```rust
fn baz<T>() {
    static_assert!((T) std::mem::size_of::<T>() == 4 => "T must be 4 bytes long!");
}
```

\
Unsized types need to be passed with this syntax:
```rust
fn baz<U: ?Sized>() {
    static_assert!((U?) true => "There isn't much you can statically check about unsized types.");
}
```

\
Multiple generics can be used at a time.
```rust
fn baz<const N: usize, const M: usize, T>() {
    static_assert!((N: usize, M: usize) N > M => "N must be greater than M!");
    static_assert!((N: usize, T) N == std::mem::size_of::<T>() / 2 => "N must be half the size of T!");
}

baz::<4, 7, u64>(); // panics at "N must be greater than M!"
baz::<4, 1, u8>(); // panics at "N must be half the size_of T!"
```

License: 0BSD