/*!
Functionality for asserting statements at compile time, including those using const and type generics.

It works by trying to evaluate a constant, and failing (via panicking at compile time) if the expression evaluates to false.
Since `cargo check` does not evaluate constants, `static_assert!`s with specified generics do not show up as errors,
and full `cargo build` compilations are needed instead.
This is a rather 'hack'y method of doing asserts, so I wouldn't be that surprised if future versions of rust break it.
For now, it still works as of 1.77.2.

Attempts to add const generic functionality in the `static_assert` crate [have been made](https://github.com/nvzqz/static-assertions/issues/40),
but it doesn't seem like it'll be added anytime soon.

These asserts are not present in function signatures or the type system in any way, possibly making it hard to reason about when creating any kind of abstraction.
You should probably use them sparingly, and explicitly document them in functions that rely on static asserts.

# Examples

Asserting constant expressions:
```
static_assert!(() 1 + 2 < 17); // True statement, compiles.

static_assert!(() 45 * 25 < 3); // False statement, does not compile:

// error[E0080]: evaluation of constant value failed
//  |     static_assert!(() 45 * 25 < 3)
//  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'Static assert failed.'
```

\
The error message is optionally specified:
```
static_assert!(() 45 * 25 < 3 => "This is the error message!");
// the evaluated program panicked at 'This is the error message!'
```

\
Putting an assert outsize of a function block will cause an syntax error. 
To get around that you can assign it to a constant of type unit,
but since you can't capture any generics outside of a function block, you might as well just use `assert!`.
```
const FOO: () = static_assert!(() 1 + 1 == 2);
const BAR: () = assert!(1 + 1 == 2);
```

\
It can error conditionally, depending on the value of the generic:
```
fn foo<const N: usize>() {
    // Const generics used in the expression have to be passed in explicitly, along with their type.
    static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
    // Some other functionality.
}
```
`foo::<0>()` will not show an error when using cargo check.
However, attempting to compile still results in an error, as expected.

```
// error[E0080]: evaluation of `foo::Assert::<0>::CHECK` failed
//  |         static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
//  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'N must be a non-zero value!'
//
// note: the above error was encountered while instantiating `fn main::foo::<0>`
//  |     foo::<0>();
```

\
Not declaring the generics present in the expression results in an error.
```
fn bar<const N: usize>() {
    static_assert!(() N != 0 => "N must be a non-zero value!");
    // can't use generic parameters from outer item
}
```

\
Type generics can be used as well.
```
fn baz<T>() {
    static_assert!((T) std::mem::size_of::<T>() == 4 => "T must be 4 bytes long!");
}
```

\
Unsized types need to be passed with this syntax:
```
fn baz<U: ?Sized>() {
    static_assert!((U?) true => "There isn't much you can statically check about unsized types.");
}
```

\
Multiple generics can be used at a time.
```
fn baz<const N: usize, const M: usize, T>() {
    static_assert!((N: usize, M: usize) N > M => "N must be greater than M!");
    static_assert!((N: usize, T) N == std::mem::size_of::<T>() / 2 => "N must be half the size of T!");
}

baz::<4, 7, u64>(); // panics at "N must be greater than M!"
baz::<4, 1, u8>(); // panics at "N must be half the size_of T!"
```
*/

enum Generic {
    Type(syn::Ident),
    UnsizedType(syn::Ident),
    Const(syn::Ident, syn::Type),
}

impl Generic {
    pub fn definition(&self) -> proc_macro2::TokenStream {
        match self {
            Generic::Type(i) => quote::quote! { #i, },
            Generic::UnsizedType(i) => quote::quote! { #i: ?Sized, },
            Generic::Const(i, t) => quote::quote! { const #i: #t, },
        }
    }

    pub fn placement(&self) -> proc_macro2::TokenStream {
        match self {
            Generic::Type(i) => quote::quote! { #i, },
            Generic::UnsizedType(i) => quote::quote! { #i, },
            Generic::Const(i, _t) => quote::quote! { #i, },
        }
    }

    pub fn placement_type(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            Generic::Type(_i) => Some(self.placement()),
            Generic::UnsizedType(_i) => Some(self.placement()),
            Generic::Const(_i, _t) => None,
        }
    }
}

impl syn::parse::Parse for Generic {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        Ok(if input.parse::<syn::Token![:]>().is_ok() {
            if let Ok(qm) = input.parse::<syn::Token![?]>() {
                return Err(syn::Error::new(qm.span, format!("Syntax error, if you want to make the type unsized do {ident}? instead of {ident}: ?Sized.")))
            }
            Generic::Const(ident, input.parse()?)
        } else if input.parse::<syn::Token![?]>().is_ok() {
            Generic::UnsizedType(ident)
        } else {
            Generic::Type(ident)
        })
    }
}

struct StaticAssertInput {
    generics: Vec<Generic>,
    expression: syn::Expr,
    message: Option<proc_macro2::TokenStream>,
}

impl syn::parse::Parse for StaticAssertInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(StaticAssertInput {
            generics: {
                let generics_buf;
                syn::parenthesized!(generics_buf in input);
                generics_buf.parse_terminated(Generic::parse, syn::Token![,])?.into_iter().collect()
            },
            expression: input.parse()?,
            message: if input.parse::<syn::Token![=>]>().is_ok() { Some(input.parse()?) } else { None },
        })
    }
}

/// The main use-case for this crate.\
/// Macro for asserting statements at compile-time, with the possibility of passing in generics as well.
/// Refer to to the crate-level documentation for more information.
#[proc_macro]
pub fn static_assert(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let StaticAssertInput { generics, expression, message } = syn::parse_macro_input!(input as StaticAssertInput);

    let generic_definitions: proc_macro2::TokenStream = generics.iter().map(Generic::definition).collect();
    let generic_placement: proc_macro2::TokenStream = generics.iter().map(Generic::placement).collect();
    let generic_placement_types: Vec<proc_macro2::TokenStream> = generics.iter().filter_map(Generic::placement_type).collect();

    quote::quote! {
        _ = {
            struct Assert<#generic_definitions>(#(core::marker::PhantomData<#generic_placement_types>),*);
            impl<#generic_definitions> Assert<#generic_placement> {
                #[allow(unused)]
                const CHECK: () = if !(#expression) { panic!(#message) };
            }
            Assert::<#generic_placement>::CHECK
        }
    }.into()
}



/// Experimental macro that forces variables of a certin type to not have their destructor run, using the same methods as `static_assert!`.\
/// Its primary use case is being able to drop objects that require updating other structures. 
/// It also has some serious drawbacks, meaning that you should likely refrain from using it in any serious project.\
/// 
/// # Example:
/// 
/// Consider a situation like this, where multiple allocators may be present at a time:\
/// 
/// ```
/// struct Allocator {
///     ...
/// }
///
/// impl Allocator {
///     pub fn alloc<T>(&mut self) -> Allocation<T> {
///         todo!()
///     }
///
///     pub fn free<T>(&mut self, allocation: Allocation<T>) {
///         // ...
///     }
/// }
///
/// struct Allocation<T> {
///     ptr_to_allocation: *const T
/// }
/// ```
/// 
/// Idealy, when the `Allocation` runs out of scope, it would be freed by the same `Allocator` that allocated it.
/// However, implementing such `Drop` functionality would require the allocation to also hold some kind of reference to said `Allocator`.
/// This would double its size and may become an issue if `Allocation` appears often.
/// Still, if the programmer forgets to free the `Allocation` a memory leak would take place.
///
/// Using `explicitly_drop!` would give a compile-time error if `Allocation`'s `drop` method appears anywhere in the code:
/// 
/// ```
/// impl<T> Drop for Allocation<T> {
///     explicitly_drop!(T => "Allocation must be freed explicitly!");
/// }
/// ```
/// 
/// The `free` method would have to make sure that `Allocation`'s `drop` method doesn't appear either.
/// 
/// ```
/// pub fn free<T>(&mut self, allocation: Allocation<T>) {
///     let allocation = std::mem::ManuallyDrop::new(allocation);
///     // ...
/// }
/// ```
/// 
/// Now if someone forgets to `free` an `Allocation`, the compiler will give an error 
/// (just like `static_assert!`, this isn't caught by `cargo check`, and a full build is needed instead):
/// 
/// ```
/// fn foo(allocator: Allocator) {
///     let allocation: Allocation::<Whatever> = allocator.alloc();
///     // ... allocation is not freed
///     // allocation's drop method appears
/// }
/// 
/// foo(my_allocator);
/// 
/// // error[E0080]: evaluation of `<Allocation<T> as std::ops::Drop>::drop::Assert::<Whatever>::MANUAL_DROP` failed
/// //    |
/// //    |         explicitly_drop!(T => "Allocation must be freed explicitly!");
/// //    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the evaluated program panicked at 'Allocation must be freed explicitly!'
/// //    |
/// //
/// // note: the above error was encountered while instantiating `fn <Allocation<Whatever> as std::ops::Drop>::drop`
/// //    |
/// //    | pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
/// //    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
/// 
/// `free` (or prevent its `drop` method from appearing in any way) and the error dissapears:
/// 
/// ```
/// fn foo(allocator: Allocator) {
///     let allocation: Allocation::<Whatever> = allocator.alloc();
///     // ... do something
///     allocator.free(allocation);
/// }
/// 
/// foo(my_allocator);
/// 
/// // compiles just fine
/// ```
/// 
/// 
/// 
/// # Drawbacks
/// 
/// To prevent constant evaluation from always happening, the functionality is dependant on at least one generic (be it type or const)
/// from the type it's implemented in, so if the type in question doesn't have that the macro won't work:
/// 
/// ```
/// impl<T> Drop for Foo<T> {
///     explicitly_drop!(T => "Dependant on type generic");
/// }
/// 
/// impl<T: ?Sized> Drop for Foo<T> {
///     explicitly_drop!(T? => "If the type is unsized it needs special syntax");
/// }
/// 
/// impl<const C: u8> Drop for Foo<{C}> {
///     explicitly_drop!(const C: u8 => "Dependant on const generic (specifying the type is needed)");
/// }
/// 
/// impl<const C: u8, const D: u16, T, U, V> Drop for Foo<{C}, {D}, T, U, V> {
///     explicitly_drop!(const C: u8 => "Just one is needed, even if the type has more.");
/// }
/// ```
/// 
/// Using a lifetime as a generic doesn't work.
/// 
/// Secondly, the drop method might appear even when it doesn't seem it should at first glance.
/// For example, if a panic ever occours, all variables in scope, including those that need to be `explicitly_drop`ped, have their `drop` 
/// method run, so even if the panic never occours at runtime, the simple appearance of `drop` will still cause a compile-time error:
/// 
/// ```
/// fn bar(allocator: Allocator) {
///     let allocation: Allocation::<Whatever> = allocator.alloc();
///     if rand::random::<usize>() == 27 {
///         panic!();
///     }
///     allocator.free(allocation);
/// }
/// 
/// foo(my_allocator);
/// 
/// // ... 'Allocation must be freed explicitly!' ...
/// ```
/// 
/// A huge amount of functionality in rust can result in panics. 
/// Even if explicit `panic!`, `todo!`, or `unwrap`s are avoided, these operations, and many more, can also panic:
/// - Indexing into a container without bounds checking.
/// - Basically every unchecked heap allocation.
/// - Any mathematical operation that can over/underflow (on a debug build).
/// - Possible division or modulo operation by 0.
/// 
/// This method also assumes that rust optimises out, and as such doesn't attempt to evaluate 
/// constants if the method they are in isn't use, which might not even always be the case.
#[proc_macro]
pub fn explicitly_drop(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct ExplicitlyDropInput {
        generic: Generic,
        message: Option<proc_macro2::TokenStream>,
    }
    
    impl syn::parse::Parse for ExplicitlyDropInput {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(ExplicitlyDropInput {
                generic: input.parse()?,
                message: if input.parse::<syn::Token![=>]>().is_ok() { Some(input.parse()?) } else { None },
            })
        }
    }

    let ExplicitlyDropInput { generic, message } = syn::parse_macro_input!(input as ExplicitlyDropInput);

    let generic_definition = generic.definition();
    let generic_placement = generic.placement();
    let phantomdatas = generic.placement_type()
        .map(|x| quote::quote! { (core::marker::PhantomData<#x>) });

    quote::quote! {
        fn drop(&mut self) {
            _ = {
                struct Assert<#generic_definition>#phantomdatas;
                impl<#generic_definition> Assert<#generic_placement> {
                    const MANUAL_DROP: () = panic!(#message);
                }
                Assert::<#generic_placement>::MANUAL_DROP
            };
        }
    }.into()
}