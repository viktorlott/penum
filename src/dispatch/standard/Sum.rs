pub trait Sum<A = Self>: Sized {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = A>;
}
