pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}
