use super::HostFunction;
use extism::{CurrentPlugin, Error, Function, UserData, Val, ValType};

pub struct HelloWorld;

impl HostFunction for HelloWorld {
    fn get() -> Function {
        Function::new(
            "hello_world",
            [ValType::I64],
            [ValType::I64],
            None,
            hello_world,
        )
    }
}

fn hello_world(
    _plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), Error> {
    debug!("Hello from Rust!");
    outputs[0] = inputs[0].clone();
    Ok(())
}
