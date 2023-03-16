pub trait BitOr<Rhs = Self> {
    type Output;

    fn bitor(self, rhs: Rhs) -> Self::Output;
}
