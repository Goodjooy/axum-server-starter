#![forbid(unsafe_code)]

mod derive_provider;
mod prepare_macro;
use prepare_macro::inputs::attr_name::PrepareName;
use syn::{parse_macro_input, DeriveInput, ItemFn};

#[macro_use]
mod utils;

/// implement `Provider<T>` for each field of the struct
///
/// ## Example
///
/// ```rust
/// #[derive(Debug, Provider)]
/// struct Configure {
///     // this will impl `Provider<&String>`  
///     #[provider(ref, transparent)]
///     foo: String,
///     // this will not impl provide
///     #[provider(skip)]
///     bar: SocketAddr,
///     // this will impl `Provide<FooBar>`
///     // where `FooBar` is `struct FooBar((i32,i32));`
///     foo_bar: (i32, i32),
/// }
///
/// fn foo_fetch(foo: &String, FooBar(foo_bar): FooBar){
///     // do somethings
/// }
///
/// ```  
///
/// - using `ref` to impl `Provider` provide reference instant of Owned (with clone)  
/// - using `transparent` to impl `Provider` the original type instant of generate a wrapper type
/// - using `skip` to not impl `Provider` for this field
/// - using `map_to(ty , by)` to adding extra provide for [Type](syn::Type) by the giving function, if the type need lifetime mark, 
/// adding `lifetime = "'a"`, then using`'a` in your type for example `& 'a str`
#[proc_macro_derive(Provider, attributes(provider))]
pub fn derive_config_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    darling_err!(derive_provider::provider_derive(derive_input))
}

/// make a function can apply as a `Prepare`
///
/// ## Example
///
/// [macro@prepare] support either sync or async function.
///
/// the function arguments require can be provide by  the `Configure`.
///
/// the return type require impl the trait `IntoFallibleEffect`, usually can be one of :
/// - `()`
/// - `Result<impl PreparedEffect, CustomError>`
/// - `impl PreparedEffect`
/// - `impl IntoFallibleEffect`
///
/// the generate type name is present throw the macro argument,for example, if you want a Prepare task
/// named `SeaConn`, just using like `#[prepare(SeaConn)]`
///
/// if your function argument contain reference or other types witch need a lifetime, just add the lifetime to the macro arguments list,
/// like this
/// ```rust
/// #[prepare(Foo 'f)]
/// fn prepare_foo(foo_name: &'f String){
/// // do somethings
/// }
/// ```
#[proc_macro_attribute]
pub fn prepare(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let prepare_name = parse_macro_input!(attrs as PrepareName);
    let fn_item = parse_macro_input!(input as ItemFn);

    match prepare_macro::prepare_macro(&prepare_name, fn_item) {
        Ok(token) => token,
        Err(error) => error.to_compile_error().into(),
    }
}
