pub trait Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}
