pub trait UpperExp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
