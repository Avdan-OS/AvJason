use proc_macro2::Span;
use syn::{punctuated::Punctuated, Token};

///
/// Creates lines of Rustdoc from &self.
///
/// ### Example
/// ```ignore
/// use proc_macro2::Span;
///
/// let boolean_lit = ECMARef {
///     name: syn::LitStr::new("BooleanLiteral", Span::call_site()),
///     href: syn::LitStr::new("https://262.ecma-international.org/5.1/#sec-7.8.2", Span::call_site())
/// };
///
/// boolean_lit.to_rustdoc()
/// ```
///
/// would return:
///
/// ```ignore
/// #[doc = "## BooleanLiteral"]
/// #[doc = "See the official [ECMAScript specification](https://262.ecma-international.org/5.1/#sec-7.8.2)."]
/// #[doc = "***"]
/// ```
///
pub trait ToRustdoc {
    fn to_rustdoc(&self) -> impl IntoIterator<Item = syn::Attribute>;
}

///
/// Produces a line of rust doc.
///
/// ### Example
///
/// ```ignore
/// rustdoc_line("Ridicule!")
/// ```
///
/// will produce:
///
/// ```ignore
/// #[doc = "Ridicule!"]
/// ```
///
fn rustdoc_line(st: impl ToString) -> syn::Attribute {
    syn::Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        meta: syn::Meta::NameValue(syn::MetaNameValue {
            path: syn::Path {
                leading_colon: Default::default(),
                segments: Punctuated::from_iter([syn::PathSegment {
                    ident: syn::Ident::new("doc", Span::call_site()),
                    arguments: syn::PathArguments::None,
                }]),
            },
            eq_token: Default::default(),
            value: syn::Expr::Lit(syn::ExprLit {
                attrs: Default::default(),
                lit: syn::Lit::Str(syn::LitStr::new(&st.to_string(), Span::call_site())),
            }),
        }),
    }
}

///
/// Represents a reference to the ECMAScript specification.
///
pub struct ECMARef {
    name: syn::LitStr,
    href: syn::LitStr,
}

impl syn::parse::Parse for ECMARef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first: syn::LitStr = input.parse()?;
        let _: Token![,] = input.parse()?;
        let second: syn::LitStr = input.parse()?;

        Ok(Self {
            name: first,
            href: second,
        })
    }
}

impl ToRustdoc for ECMARef {
    fn to_rustdoc(&self) -> impl IntoIterator<Item = syn::Attribute> {
        let Self { name, href } = self;
        let (name, href) = (name.value(), href.value());
        [
            format!("## {name}"),
            format!("See more on the [ECMAScript specification]({href})."),
            "***".to_string(),
        ]
        .into_iter()
        .map(rustdoc_line)
    }
}

///
/// Represents a reference to the JSON5 specification.
///
/// ### Example
///
/// ```ignore
/// #[JSON5Ref("Null", "JSON5Null")] // (a)
/// #[JSON5Ref("JSON5Identifier")]   // (b)
/// ```
///
/// would yield:
///
/// ```ignore
/// // (a)
/// JSON5Ref {
///     name: Some(syn::LitStr::new("Null", _)),
///     id: syn::LitStr::new("JSON5Null", _),
/// }
///
/// // (b)
/// JSON5Ref {
///     name: None,
///     id: syn::LitStr::new("JSON5Identifier", _),
/// }
/// ```
///
pub struct JSON5Ref {
    name: Option<syn::LitStr>,
    id: syn::LitStr,
}

impl syn::parse::Parse for JSON5Ref {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first: syn::LitStr = input.parse()?;

        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            let second: syn::LitStr = input.parse()?;

            return Ok(Self {
                name: Some(first),
                id: second,
            });
        }

        Ok(Self {
            name: None,
            id: first,
        })
    }
}

impl ToRustdoc for JSON5Ref {
    fn to_rustdoc(&self) -> impl IntoIterator<Item = syn::Attribute> {
        let Self { name, id } = self;
        let (name, id) = (name.as_ref().map(|s| s.value()), id.value());
        [
            format!("## {}", name.as_ref().unwrap_or(&id)),
            format!("See more on the [JSON5 specification](https://spec.json5.org/#prod-{id})."),
            "***".to_string(),
        ]
        .into_iter()
        .map(rustdoc_line)
    }
}

///
/// Attempt to get the attribute macros for a [syn::Item].
///
pub fn get_item_attrs(item: &mut syn::Item) -> Option<&mut Vec<syn::Attribute>> {
    match item {
        syn::Item::Const(syn::ItemConst { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Enum(syn::ItemEnum { ref mut attrs, .. }) => Some(attrs),
        syn::Item::ExternCrate(syn::ItemExternCrate { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Fn(syn::ItemFn { ref mut attrs, .. }) => Some(attrs),
        syn::Item::ForeignMod(syn::ItemForeignMod { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Impl(syn::ItemImpl { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Macro(syn::ItemMacro { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Mod(syn::ItemMod { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Static(syn::ItemStatic { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Struct(syn::ItemStruct { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Trait(syn::ItemTrait { ref mut attrs, .. }) => Some(attrs),
        syn::Item::TraitAlias(syn::ItemTraitAlias { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Type(syn::ItemType { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Union(syn::ItemUnion { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Use(syn::ItemUse { ref mut attrs, .. }) => Some(attrs),
        syn::Item::Verbatim(_) => None,
        _ => None,
    }
}
