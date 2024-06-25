#![cfg(FIXME)]
#![allow(non_snake_case)]

#[test]
fn wf_outlives_82() {
    crate::assert_ok!(
        //@check-pass
        [
            crate Foo {
                struct Ref<ty T, lt a> where T lt: a {}

                trait Foo {}
                struct Bar {}

                impl <ty A> Foo for Bar where for<lt b> Ref<A, b> lt: b {}
            }
        ]

        expect_test::expect!["TODO"]
    );
    crate::assert_err!(
        [
            crate Foo {
                struct NoRef<ty T, lt a> {}

                trait Foo {}
                struct Bar {}

                impl <ty A> Foo for Bar where for<lt b> NoRef<A, b> lt: b {}
            }
        ]

        [/* TODO */]

        expect_test::expect!["TODO"]
    );
    crate::assert_err!(
        [
            crate Foo {
                struct NestedRef<lt a, lt b> where a lt: b {}

                trait Foo {}
                struct Bar {}

                impl <lt a> Foo for Bar where for <lt b> NestedRef<a, a> lt: b {}
            }
        ]

        [/* TODO */]

        expect_test::expect!["TODO"]
    );
}

// from github issue https://github.com/rust-lang/a-mir-formality/issues/82
// this test would cause the old implementation to run out of memory
//
// #lang racket
// (require redex/reduction-semantics
//          "../../util.rkt"
//          "../grammar.rkt"
//          "../prove.rkt"
//          "../libcore.rkt"
//          )
//
// (module+ test
//   (redex-let*
//    formality-rust
//
//    [(Rust/Program (term ([(crate C { (struct Ref[(type T) (lifetime a)]
//                                        where [(T : a)]
//                                        { })
//                                      (struct NoRef[(type T) (lifetime a)]
//                                        where []
//                                        { })
//                                      (struct NestedRef[(lifetime a) (lifetime b)]
//                                        where [(a: b)]
//                                        { })
//                                      })] C)))
//
//     ]
//
//    (traced '()
//            (test-term-true
//             (rust:can-prove-where-clause-in-program
//              Rust/Program
//              (∀ [(type A)]
//                 where []
//                 ; key point here:
//                 ;
//                 ;     requires proving `A : 'b`, but that's implied by
//                 ;     Ref<A, 'b> being WF
//                 (for[(lifetime b)] ((Ref < A b >) : b))
//                 )
//              )
//             ))
//
//    (traced '()
//            (test-term-false
//             (rust:can-prove-where-clause-in-program
//              Rust/Program
//              (∀ [(type A)]
//                 where []
//                 ; in contrast to previous test, the `NoRef` struct does not
//                 ; imply a connection between `A` and `b`
//                 (for[(lifetime b)] ((NoRef < A b >) : b))
//                 )
//              )
//             ))
//
//    (traced '()
//            (test-term-false
//             (rust:can-prove-where-clause-in-program
//              Rust/Program
//              (∀ [(lifetime a)]
//                 where []
//                 (for[(lifetime b)] ((NestedRef < a a >) : b))
//                 )
//              )
//             ))
//    )
//   )
