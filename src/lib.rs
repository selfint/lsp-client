pub struct LspClient<T> {
    server: T,
}

impl<T> LspClient<T> {
    pub fn new(server: T) -> Self {
        Self { server }
    }
}
