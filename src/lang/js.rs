//! Specialization for JavaScript code generation.
//!
//! # Examples
//!
//! Basic example:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: js::Tokens = quote! {
//!     function foo(v) {
//!         return v + ", World";
//!     }
//!
//!     foo("Hello");
//! };
//!
//! assert_eq!(
//!     vec![
//!         "function foo(v) {",
//!         "    return v + \", World\";",
//!         "}",
//!         "",
//!         "foo(\"Hello\");",
//!     ],
//!     toks.to_file_vec()?
//! );
//! # Ok(())
//! # }
//! ```
//!
//! String quoting in JavaScript:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: js::Tokens = quote!("hello \n world");
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//!
//! let toks: js::Tokens = quote!(#(quoted("hello \n world")));
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

use crate::fmt;
use crate::tokens::ItemStr;
use relative_path::{RelativePath, RelativePathBuf};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<JavaScript>;

impl_lang! {
    /// JavaScript language specialization.
    pub JavaScript {
        type Config = Config;
        type Format = Format;
        type Import = Import;

        /// Start a string quote.
        fn open_quote(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
            has_eval: bool,
        ) -> fmt::Result {
            use std::fmt::Write as _;

            if has_eval {
                out.write_char('`')?;
            } else {
                out.write_char('"')?;
            }

            Ok(())
        }

        /// End a string quote.
        fn close_quote(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
            has_eval: bool,
        ) -> fmt::Result {
            use std::fmt::Write as _;

            if has_eval {
                out.write_char('`')?;
            } else {
                out.write_char('"')?;
            }

            Ok(())
        }

        fn start_string_eval(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
        ) -> fmt::Result {
            out.write_str("${")?;
            Ok(())
        }

        fn end_string_eval(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
        ) -> fmt::Result {
            out.write_char('}')?;
            Ok(())
        }

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            for c in input.chars() {
                match c {
                    '\t' => out.write_str("\\t")?,
                    '\u{0007}' => out.write_str("\\b")?,
                    '\n' => out.write_str("\\n")?,
                    '\r' => out.write_str("\\r")?,
                    '\u{0014}' => out.write_str("\\f")?,
                    '\'' => out.write_str("\\'")?,
                    '"' => out.write_str("\\\"")?,
                    '\\' => out.write_str("\\\\")?,
                    c => out.write_char(c)?,
                };
            }

            Ok(())
        }

        fn format_file(
            tokens: &Tokens,
            out: &mut fmt::Formatter<'_>,
            config: &Self::Config,
        ) -> fmt::Result {
            let mut imports = Tokens::new();
            Self::imports(&mut imports, tokens, config);
            let format = Format::default();
            imports.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            let name = match self.kind {
                ImportKind::Named => self.alias.as_ref().unwrap_or(&self.name),
                _ => &self.name,
            };

            out.write_str(name)
        }

        fn as_import(&self) -> Option<&Self> {
            Some(self)
        }
    }
}

/// Format state for JavaScript.
#[derive(Debug, Default)]
pub struct Format {}

/// Configuration for JavaScript.
#[derive(Debug, Default)]
pub struct Config {
    module_path: Option<RelativePathBuf>,
}

impl Config {
    /// Configure the path to the current module being renderer.
    ///
    /// This setting will determine what path imports are renderer relative
    /// towards. So importing a module from `"foo/bar.js"`, and setting this to
    /// `"foo/baz.js"` will cause the import to be rendered relatively as
    /// `"../bar.js"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let foo1 = js::import(js::Module::Path("foo/bar.js".into()), "Foo1");
    /// let foo2 = js::import(js::Module::Path("foo/bar.js".into()), "Foo2");
    /// let react = js::import("react", "React").into_default();
    ///
    /// let toks: js::Tokens = quote! {
    ///     #foo1
    ///     #foo2
    ///     #react
    /// };
    ///
    /// let mut w = fmt::VecWriter::new();
    ///
    /// let config = js::Config::default().with_module_path("foo/baz.js");
    /// let fmt = fmt::Config::from_lang::<JavaScript>();
    ///
    /// toks.format_file(&mut w.as_formatter(fmt), &config)?;
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {Foo1, Foo2} from \"../bar.js\";",
    ///         "import React from \"react\";",
    ///         "",
    ///         "Foo1",
    ///         "Foo2",
    ///         "React"
    ///     ],
    ///     w.into_vec()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_module_path<M>(self, module_path: M) -> Self
    where
        M: Into<RelativePathBuf>,
    {
        Self {
            module_path: Some(module_path.into()),
            ..self
        }
    }
}

/// Internal type to determine the kind of import used.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum ImportKind {
    Named,
    Default,
    Wildcard,
}

/// An imported JavaScript item `import {foo} from "module.js"`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// The kind of the import.
    kind: ImportKind,
    /// Module of the imported name.
    module: Module,
    /// Name imported.
    name: ItemStr,
    /// Alias of an imported item.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    alias: Option<ItemStr>,
}

impl Import {
    /// Change alias of imported item.
    ///
    /// This implies that the import is a named import.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let a = js::import("collections", "vec");
    /// let b = js::import("collections", "vec").with_alias("list");
    ///
    /// let toks = quote! {
    ///     #a
    ///     #b
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {vec, vec as list} from \"collections\";",
    ///         "",
    ///         "vec",
    ///         "list",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_alias<N>(self, alias: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            kind: ImportKind::Named,
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Convert into a default import.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let default_vec = js::import("collections", "defaultVec").into_default();
    ///
    /// let toks = quote!(#default_vec);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import defaultVec from \"collections\";",
    ///         "",
    ///         "defaultVec",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_default(self) -> Self {
        Self {
            kind: ImportKind::Default,
            alias: None,
            ..self
        }
    }

    /// Convert into a wildcard import.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let all = js::import("collections", "all").into_wildcard();
    ///
    /// let toks = quote!(#all);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import * as all from \"collections\";",
    ///         "",
    ///         "all",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_wildcard(self) -> Self {
        Self {
            kind: ImportKind::Wildcard,
            alias: None,
            ..self
        }
    }
}

/// A module being imported.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Module {
    /// A module imported from a specific path.
    ///
    /// The path will be relativized according to the module specified in the
    /// [Config::with_module_path].
    Path(RelativePathBuf),
    /// A globally imported module.
    Global(ItemStr),
}

impl<'a> From<&'a str> for Module {
    fn from(value: &'a str) -> Self {
        Self::Global(value.into())
    }
}

impl From<String> for Module {
    fn from(value: String) -> Self {
        Self::Global(value.into())
    }
}

impl JavaScript {
    /// Translate imports into the necessary tokens.
    fn imports(out: &mut Tokens, tokens: &Tokens, config: &Config) {
        use crate as genco;
        use crate::prelude::*;

        let mut modules = BTreeMap::<&Module, ResolvedModule<'_>>::new();
        let mut wildcards = BTreeSet::new();

        for import in tokens.walk_imports() {
            match import.kind {
                ImportKind::Named => {
                    let module = modules.entry(&import.module).or_default();

                    module.set.insert(match &import.alias {
                        None => ImportedElement::Plain(&import.name),
                        Some(alias) => ImportedElement::Aliased(&import.name, alias),
                    });
                }
                ImportKind::Default => {
                    let module = modules.entry(&import.module).or_default();
                    module.default_import = Some(&import.name);
                }
                ImportKind::Wildcard => {
                    wildcards.insert((&import.module, &import.name));
                }
            }
        }

        if modules.is_empty() && wildcards.is_empty() {
            return;
        }

        for (module, name) in wildcards {
            out.push();
            quote_in! { *out =>
                import * as #name from #(ref t => render_from(t, config.module_path.as_deref(), module));
            }
        }

        for (name, module) in modules {
            out.push();
            quote_in! { *out =>
                import #(ref tokens => {
                    if let Some(default) = module.default_import {
                        tokens.append(ItemStr::from(default));

                        if !module.set.is_empty() {
                            tokens.append(",");
                            tokens.space();
                        }
                    }

                    if !module.set.is_empty() {
                        tokens.append("{");

                        let mut it = module.set.iter().peekable();

                        while let Some(el) = it.next() {
                            match *el {
                                ImportedElement::Plain(name) => {
                                    tokens.append(name);
                                },
                                ImportedElement::Aliased(name, alias) => {
                                    quote_in!(*tokens => #name as #alias);
                                }
                            }

                            if it.peek().is_some() {
                                tokens.append(",");
                                tokens.space();
                            }
                        }

                        tokens.append("}");
                    }
                }) from #(ref t => render_from(t, config.module_path.as_deref(), name));
            };
        }

        out.line();

        #[derive(Default)]
        struct ResolvedModule<'a> {
            default_import: Option<&'a ItemStr>,
            set: BTreeSet<ImportedElement<'a>>,
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
        enum ImportedElement<'a> {
            Plain(&'a ItemStr),
            Aliased(&'a ItemStr, &'a ItemStr),
        }

        fn render_from(t: &mut js::Tokens, module_path: Option<&RelativePath>, module: &Module) {
            quote_in! { *t =>
                #(match (module_path, module) {
                    (_, Module::Global(from)) => #(quoted(from)),
                    (None, Module::Path(path)) => #(quoted(path.as_str())),
                    (Some(module_path), Module::Path(path)) => #(quoted(module_path.relative(path).as_str())),
                })
            }
        }
    }
}

/// Import an element from a module
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let default_vec = js::import("collections", "defaultVec").into_default();
/// let all = js::import("collections", "all").into_wildcard();
/// let vec = js::import("collections", "vec");
/// let vec_as_list = js::import("collections", "vec").with_alias("list");
///
/// let toks = quote! {
///     #default_vec
///     #all
///     #vec
///     #vec_as_list
/// };
///
/// assert_eq!(
///     vec![
///         "import * as all from \"collections\";",
///         "import defaultVec, {vec, vec as list} from \"collections\";",
///         "",
///         "defaultVec",
///         "all",
///         "vec",
///         "list",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn import<M, N>(module: M, name: N) -> Import
where
    M: Into<Module>,
    N: Into<ItemStr>,
{
    Import {
        kind: ImportKind::Named,
        module: module.into(),
        name: name.into(),
        alias: None,
    }
}
