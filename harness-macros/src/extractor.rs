use syn::ItemFn;

pub(crate) struct Extractor {}

impl Extractor {
    pub fn new() -> Self {
        Extractor {}
    }

    /// Creation of a candid service signature from the original function signature
    ///
    /// # Example
    /// ```rust,ignore
    /// #[harness]
    /// fn hello(name: String) -> String {
    ///    format!("Hello, {name}!")
    /// }
    ///
    /// #[harness(name = "otherHello")]
    /// fn hi()-> String {
    ///     "Hi".to_string()
    /// }
    /// ```
    ///
    /// The above function will be converted to the below candid service signature
    /// ```ignore
    /// service: {
    ///     "hello": (name: text) -> (text);
    ///     "otherHello": () -> (text);
    /// }
    /// ```
    pub fn extract_signature(self, fn_: &ItemFn) -> Self {
        todo!()
    }
}
