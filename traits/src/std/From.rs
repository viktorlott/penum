pub trait From<T>: Sized {
    fn from(value: T) -> Self;
}
