pub trait SplitArgs {
    fn split_args(&self) -> Vec<String>;
}
impl<T> SplitArgs for T 
where 
    T: AsRef<str> {
    fn split_args(&self) -> Vec<String> {
        self.as_ref()
            .split_whitespace()
            .map(|i| i.to_string())
            .collect()

    }
}
