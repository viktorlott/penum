pub trait AsMut<T: ?Sized> {
    fn as_mut(&mut self) -> &mut T;
}
