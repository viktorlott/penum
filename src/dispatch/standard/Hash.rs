pub trait Hash {
    fn hash<H: ~const Hasher>(&self, state: &mut H);
}