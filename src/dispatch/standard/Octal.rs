pub trait Octal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
