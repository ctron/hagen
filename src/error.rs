use failure::Fail;

#[derive(Debug, Fail)]
enum GeneratorError {
    #[fail(display = "{}", _0)]
    GenericError(String),
}
