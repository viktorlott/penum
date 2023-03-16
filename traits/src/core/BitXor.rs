pub trait BitXor<Rhs = Self> {
    type Output;

    fn bitxor(self, rhs: Rhs) -> Self::Output;
}
