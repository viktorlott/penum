pub trait AbcTrait {
    fn a(&self) -> Option<i32>;
    fn b(&self) -> &Option<i32>;
    fn c(&self) -> (Option<i32>, &Option<&String>);
    fn d(&self) -> &String;
}