#![allow(dead_code)]

use static_assert_generic::*;

const FOO: () = static_assert!(() 1 + 1 == 2);
#[allow(clippy::assertions_on_constants, clippy::eq_op)] const BAR: () = assert!(1 + 1 == 2);

#[test]
fn test() {
    // compiles
    static_assert!(() 1 + 2 < 17);


    // fails
    // static_assert!(() 45 * 25 < 3);

    // fails
    // static_assert!(() 45 * 25 < 3 => "This is the error message!");


    fn foo<const N: usize>() {
        static_assert!((N: usize) N != 0 => "N must be a non-zero value!");
    }
    // fails
    // foo::<0>();


    // fails
    // fn bar<const N: usize>() {
    //     static_assert!(() N != 0 => "N must be a non-zero value!"); 
    // }



    fn baz<const N: usize, const M: usize>() {
       static_assert!((N: usize, M: usize) N > M => "N must be greater than M!");
    }
    // fails
    // baz::<4, 7>();



    // compiles
    fn spam<T>() {
        static_assert!((T) std::mem::size_of::<T>() == 4 => "T must be 4 bytes long!");
    }



    // compiles
    fn eggs<U: ?Sized>() {
        static_assert!((U?) true => "There isn't much you can statically check about unsized types.");
    }



    fn fie<const N: usize, const M: usize, T>() {
        static_assert!((N: usize, M: usize) N > M => "N must be greater than M!");
        static_assert!((N: usize, T) N == std::mem::size_of::<T>() / 2 => "N must be half the size of T!");
    }

    // fie::<4, 7, u64>(); // fails at "N must be greater than M!"
    // fie::<4, 1, u8>(); // fails at "N must be half the size_of T!"
}
