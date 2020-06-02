#![feature(proc_macro_hygiene)]

use genco::{quote, Quoted, Rust, Tokens};

#[test]
fn test_quote() {
    let test = "one".quoted();

    let tokens: Tokens<Rust> = quote! {
        fn test() -> u32 {
            println!("{}", #(test));

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", \"one\");\n\n    42\n}",
        tokens.to_string().unwrap()
    );

    let tokens: Tokens<Rust> = quote! {
        fn test() -> u32 {
            println!("{}", #("two".quoted()));

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", \"two\");\n\n    42\n}",
        tokens.to_string().unwrap()
    );

    let tokens: Tokens<Rust> = quote! {
        fn test() -> u32 {
            println!("{}", ##("two".quoted()));

            42
        }
    };

    assert_eq!(
        "fn test() -> u32 {\n    println!(\"{}\", #(\"two\".quoted()));\n\n    42\n}",
        tokens.to_string().unwrap()
    );
}
