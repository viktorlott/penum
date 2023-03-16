pub trait Termination {
    fn report(self) -> ExitCode;
}
