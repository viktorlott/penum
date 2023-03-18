pub trait Pointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
