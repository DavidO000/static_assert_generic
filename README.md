This crate has been made redundant as a result of the release of Rust 1.79, that added inline `const` expressions:
    
```
fn foo<const N: usize>() {
    const { assert!(N > 30); }
}
```

Other versions have not been yanked since they still do work as intended.

License: 0BSD
