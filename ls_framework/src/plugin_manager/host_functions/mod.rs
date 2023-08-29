use extism::Function;

mod host_command;

pub trait HostFunction {
    fn get() -> Function;
}

lazy_static! {
    pub static ref FUNCTIONS: Vec<Function> = vec![host_command::HostCommand::get()];
}
