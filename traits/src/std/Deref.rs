pub trait Deref {
    type Target: ?Sized;

    fn deref(&self) -> &Self::Target;
}
