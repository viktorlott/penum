pub trait IntoFuture {
    type Output;
    type IntoFuture: Future<Output = Self::Output>;

    fn into_future(self) -> Self::IntoFuture;
}
