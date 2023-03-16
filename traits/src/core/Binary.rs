pub trait Binary {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}
