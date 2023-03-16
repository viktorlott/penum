pub trait AsMut<T>
where
    T: ?Sized,
{
    fn as_mut(&mut self) -> &mut T;
}
