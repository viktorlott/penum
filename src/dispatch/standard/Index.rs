pub trait Index<Idx>
where
    Idx: ?Sized,
{
    type Output: ?Sized;

    fn index(&self, index: Idx) -> &Self::Output;
}
