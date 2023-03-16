pub trait LowerHex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
