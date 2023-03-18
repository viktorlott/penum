pub trait Product<A = Self>: Sized {
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = A>;
}
