use genco::prelude::*;

#[test]
fn test_register() -> genco::fmt::Result {
    let import = rust::imported("std::iter", "FromIterator").alias("_");

    let tokens: Tokens<Rust> = quote! {
        #(import.register())
        // additional lines are ignored!

        fn test() -> u32 {
            42
        }
    };

    assert_eq!(
        vec![
            "use std::iter::FromIterator as _;",
            "",
            "fn test() -> u32 {",
            "    42",
            "}"
        ],
        tokens.to_file_vec()?
    );

    Ok(())
}
