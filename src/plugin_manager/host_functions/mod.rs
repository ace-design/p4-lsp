use extism::Function;

mod helloworld;

pub trait HostFunction {
    fn get() -> Function;
}

lazy_static! {
    pub static ref FUNCTIONS: Vec<Function> = vec![helloworld::HelloWorld::get()];
}
