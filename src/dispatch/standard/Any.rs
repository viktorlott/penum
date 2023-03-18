pub trait Any: 'static {
    fn type_id(&self) -> TypeId;
}
