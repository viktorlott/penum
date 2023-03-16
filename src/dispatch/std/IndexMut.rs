pub trait IndexMut<Idx>: Index<Idx>
where
    Idx: ?Sized,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output;
}
