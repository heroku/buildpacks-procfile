#![allow(clippy::module_name_repetitions)]
// This module defines our custom `ProcfileError`
//
// All top level errors are represented by ProcfileError.
//
// ProcfileError is intentionally named something other than `BuildpackError`
// to make it clearer that it maps into libcnb::Error::BuildpackError
// below.
//
// For general buildpack development, it's probably easier to standardize
// on calling the main error enum `BuildpackError` for ease of working
// across multiple buildpacks. This is designed to be a "teaching" buildpack.
//
// ## How to handle an error coming from libncb internals?
//
// When using libcnb, it will return types that contain errors you may wish to
// handle.
//
// For example when developing the buildpack if you get an error like:
//
//```
// error[E0277]: `?` couldn't convert the error to `libcnb::Error<ProcfileError>`
//    --> src/foo.rs:24:33
//     |
// 13  | pub fn build(context: GenericBuildContext) -> Result<(), libcnb::Error<ProcfileError>> {
//     |                                               ---------------------------------------- expected `libcnb::Error<ProcfileError>` because of this
// ...
// 24  |     context.write_launch(launch)?;
//     |                                 ^ the trait `From<TomlFileError>` is not implemented for `libcnb::Error<ProcfileError>`
//```
//
// It would be indicating that you need to tell `ProcfileError` how
// to represent this `TomlFileError`. That can be done by making a new
// enum variant, say calling it `Toml()` and using some syntatic sugar
// from `thiserror::Error` to generate the From logic.
//
// The result might look like this:
//
// ```
// pub enum ProcfileError {
//    //...
//    #[error("TOML Error: {0}")]
//    Toml(#[from] libcnb::TomlFileError),
//    //...
// }
// ```
//
// > Note: While the error said `From<TomlFileError>` the fully qualified error
// > is `libcnb::TomlFileError`.
//
// Also because the code from before `context.write_launch(launch)?` is still
// returning a `TomlFileError` you'll have to explicitly convert it in the code:
//
//```
// context.write_launch(launch).map_err(ProcfileError::from)?;
//```
//
// or you can use the explicit enum variant:
//
// ```
// context.write_launch(launch).map_err(ProcfileError::Toml)?;
// ```
//
// or even more explicitly:
//
//```
// context.write_launch(launch).map_err(|e| ProcfileError::Toml(e))?;
//```
//
// All three are different ways to accomplish the same thing: Mapping an
// error returned from libcnb into a variant that the enum our buildpack
// can understand.
//
// To recap: Error handling requires:
//
// 0. Make a custom error enum and use it as your Result<T, libcnb::Error<E>> type. For example `ProcfileError` becomes `Result<(), libcnb::Error<ProcfileError>`.
// 1. Tell libcnb how to convert the enum to a `libcnb::Error::BuildpackError(E)` (see `impl From` below)
// 2. Tell your error enum how to convert libcnb errors into itself (by making new enum variants, see example above)
// 3. In the code, map libcnb errors into custom enum errors so they can be returned (see example above)
#[derive(thiserror::Error, Debug)]
pub enum ProcfileError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Procfile YAML Parsing Error: {0}")]
    YamlScan(#[from] yaml_rust::scanner::ScanError),
    #[error("Procfile is not in a valid format: {0}")]
    Procfile(&'static str),
    #[error("Invalid ProcessType name: {0}")]
    ProcessType(#[from] libcnb::data::launch::ProcessTypeError),
    #[error("TOML Error: {0}")]
    Toml(#[from] libcnb::TomlFileError),

    #[error("Procfile error: {0}")]
    ProcfileParsingError(#[from] ProcfileParsingError),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ProcfileParsingError {
    #[error("Cannot parse Procfile")]
    CannotParse,
    #[error("Process command cannot be empty")]
    EmptyProcessCommand,
    #[error("Process name cannot be empty")]
    EmptyProcessName,
}

// Tells libcnb how to convert `ProcfileError` into `libcnb::Error<E>` where E
// is `ProcfileError`.
//
// More info on this pattern <https://blog.burntsushi.net/rust-error-handling/#the-from-trait>
// Honestly, read the whole thing...tons of great error handling info there.
//
// The reason libcnb must know how what to do with errors generated from within this library
// is that libcnb executes our functions, so when you write a function with signature:
//
//```
// pub fn build(context: GenericBuildContext) -> Result<(), libcnb::Error<ProcfileError>> {
//```
//
// Libcnb will eventually run this code and handle any errors returned by it. <https://github.com/Malax/libcnb.rs/blob/f46bdda85838c9cc21cd1de3243d4160b85afc12/src/runtime.rs#L68-L70>
//
// So in order to work with libcnb, your buildpack must be able to convert libcnb errors into
// your custom error type (`ProcfileError` in this case). That logic is covered above.
//
// Also, you need to tell libcnb how to convert your custom error type `ProcfileError` into
// it's error enum type for the error handler inside of libcnb to understand.
impl From<ProcfileError> for libcnb::Error<ProcfileError> {
    fn from(error: ProcfileError) -> Self {
        // Libcnb's interface to converting any error is `libcnb::Error::BuildpackError(E)`
        // Here's the `libcnb::Error::BuildpackError` definition: <https://github.com/Malax/libcnb.rs/blob/f46bdda85838c9cc21cd1de3243d4160b85afc12/src/error.rs#L46-L47>
        //
        // Since the return value here is Self which equates to `libcnb::Error<ProcfileError>`
        // it is building `libcnb::Error<ProcfileError>::BuildpackError() with a value of
        // the error passed in.
        //
        Self::BuildpackError(error)
    }
}
