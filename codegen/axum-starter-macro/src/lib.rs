#![forbid(unsafe_code)]
mod derive_config;
mod derive_provider;
mod from_state_collector;
mod prepare_macro;

use prepare_macro::inputs::attr_name::PrepareName;
use syn::{parse_macro_input, DeriveInput, ItemFn};

#[macro_use]
mod utils;

/// implement [`Provider<T>`](https://docs.rs/axum-starter/latest/axum_starter/trait.Provider.html) for each field of the struct
///
/// ## Example
///
/// ```rust
/// #[derive(Debug, Provider)]
/// #[provider(ref)]
/// struct Configure {
///     // this will impl `Provider<&String>`
///     // because of the `ref` on container and its own `transparent`
///     #[provider(transparent)]
///     foo: String,
///     // this will not impl provide
///     #[provider(skip)]
///     bar: SocketAddr,
///     // this will impl `Provide<FooBar>`
///     // where `FooBar` is `struct FooBar((i32,i32));`
///     // `ignore_global` will ignore the `ref` on the container
///     #[provider(ignore_global)]
///     foo_bar: (i32, i32),
/// }
///
/// fn foo_fetch(foo: &String, FooBar(foo_bar): FooBar){
///     // do somethings
/// }
///
/// ```  
///
/// - using `ref` to impl `Provider` provide reference instant of Owned (with clone) .Can be using on container to apply on all fields
/// - using `transparent` to impl `Provider` the original type instant of generate a wrapper type. Can be using on container to apply on all fields
/// - using `ignore_global` to ignore the `ref` and `transparent` setting on container
/// - using `skip` to not impl `Provider` for this field
/// - using `map_to(ty , by)` to adding extra provide for [Type](syn::Type) by the giving function, if the type need lifetime mark,
/// adding `lifetime = "'a"`, then using`'a` in your type for example `& 'a str`
#[proc_macro_derive(Provider, attributes(provider))]
pub fn derive_config_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    darling_err!(derive_provider::provider_derive(derive_input))
}

/// help macro from impl `ServeAddress`, `LoggerInitialization`, `ConfigureServerEffect`
///
/// ## Example
///
/// ```rust
/// #[derive(Debug, Provider, Configure)]
/// #[conf(
///     address(provide),
///     logger(error = "log::SetLoggerError", func = "Self::init_log"),
///     server
///)]
/// struct Configure {
///     #[provider(transparent)]
///     bar: SocketAddr,
/// }
///
/// impl Configure {
///     fn init_log(&self) -> Result<(), log::SetLoggerError>{
///         // initial the logger
///         Ok(())
///     }
/// }
///
/// ```  
/// ## Usage
/// ### address
/// - using `address(provide)` direct using the config provide to get address,
/// - using `address(provide(ty = "..."))` similar to previous one, but using the provide type
///     **Note**: the provided type need impl [Into<std::net::SocketAddr>](Into<std::net::SocketAddr>)
///
/// - using `address(func(path = "...", ty = "...", associate))` using provide function get the socket address
///     - `path` a path to a function or a closure expr, its signature is `Fn(config: &Self) -> $ty`
///     - `ty` (optional) default is [std::net::SocketAddr]
///     - `associate`(optional) set whether the function to call need argument `Self`,
///        if set `associate` the signature of function to call is `Fn()->$ty`
///
/// ### logger
/// - using `logger(error="...", func="...",associate)` to impl `LoggerInitialization`,
/// the `func` and `associate` is similar to the `path` and `associate` of `address(func(path="...", associate))` but the return type became `Result<(),$error>`
///     - `error` the error that might ocurred during initialization the log system
///
/// ### server
/// - using `server="..."` to impl `ConfigureServerEffect` with internally call the provide func or
/// just using `server` or ignore it to having an empty implement. The function look like `fn (&self, Builder<AddrIncome>) -> Builder<AddrIncome>`
///
#[proc_macro_derive(Configure, attributes(conf))]
pub fn derive_config_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    darling_err!(derive_config::provider_derive(derive_input))
}

/// impl `FromStateCollector` for special type
///
/// this implement is easy but boring, thus need macro to simplify it
#[proc_macro_derive(FromStateCollector)]
pub fn derive_from_state_collector(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    darling_err!(from_state_collector::from_state_collector_macro(
        derive_input
    ))
}

/// make a function can apply as a `Prepare`
///
/// ## Example
///
/// [macro@prepare] support either sync or async function.
/// It can generate type which implement the [`Prepare`](https://docs.rs/axum-starter/latest/axum_starter/trait.Prepare.html) trait
///
/// the function arguments require can be provide by  the `Configure`.
///
/// the return type , usually can be one of :
/// - `()`
/// - `Result<impl @, CustomError>`
/// - `impl @`
/// > the `@` can be `PrepareRouteEffect`,
/// `PrepareStateEffect` or
///  `PrepareMiddlewareEffect`
///
/// **Note** if the return type is `Result<impl @, Error>`, need add `?` following the
/// generate Name
///
/// ```rust
/// #[prepare(Foo?)]
/// fn prepare_foo() -> Result<(), std::io::Error>{
///     // do something that might return Err()
///     todo!()
/// }
/// ```
///
/// the generate type name is present by the macro argument, for example, if you want a Prepare task
/// named `SeaConn`, just using like `#[prepare(SeaConn)]`
///
/// if your function argument contain reference or other types witch need a lifetime, just add the lifetime to the macro arguments list,
/// like this.
///
/// ```rust
/// #[prepare(Foo 'f)]
/// fn prepare_foo(foo_name: &'f String){
///     // do somethings
/// }
/// ```
/// Or,you can not provide any lifetime symbol, the macro will automatic find all needing lifetime places and giving a default symbol
///
/// ```rust
/// #[prepare(Foo)]
/// fn prepare_foo(foo_name: &String){
///     // do somethings
/// }
/// ```
///
/// Or only give lifetime symbol in macro input. The macro will auto replace `'_` into `'arg` if necessary
///
/// ```rust
/// #[prepare(Foo 'arg)]
/// fn prepare_foo(foo_name: &String){
///     // do somethings
/// }
/// ```
///
/// some times store `Future` on stack may cause ***Stack Overflow***, you can using `box` before generate name
/// make the return type became `Pin<Box<dyn Future>>`
///
/// ```rust
/// #[prepare(box Foo)]
/// async fn prepare_foo(){
///     // do something may take place large space
/// }
/// ```
///
/// if you want a `Prepare` return `Ready`, or in other word, a sync `Prepare`,you can use `sync` before the Ident.
/// note that `box` and `sync` cannot use in the same time
///
/// ```rust
/// #[prepare(sync Foo)]
/// fn prepare_foo(){
///     // do something not using `await`
/// }
/// ```
///
/// By default, the macro will not keep the origin function exist, if you want use that original function, using `origin`,
/// the `origin` is after the `box` or `sync`, but before the Ident
///
///```rust
/// #[prepare(sync origin Foo)]
/// fn prepare_foo(){
///     // do something not using `await`
/// }
/// ```
///```
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
