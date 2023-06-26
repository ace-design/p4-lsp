use std::process::{Command, Stdio};

use super::HostFunction;
use extism::{CurrentPlugin, Error, Function, UserData, Val, ValType};

pub struct HostCommand;

impl HostFunction for HostCommand {
    fn get() -> Function {
        Function::new(
            "host_command",
            [ValType::I64],
            [ValType::I64],
            None,
            host_command,
        )
    }
}

fn host_command(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), Error> {
    let offset = if let Val::I64(value) = inputs[0] {
        value as usize
    } else {
        return Err(Error::msg("Invalid input pointer type."));
    };

    let memory = unsafe { plugin.memory.as_mut().unwrap() };

    let command = if let Some(memory_block) = memory.at_offset(offset) {
        memory.get_str(memory_block).unwrap()
    } else {
        return Err(Error::msg("Invalid input."));
    };

    let output = get_command_output(command);

    let out_offset = memory.alloc_bytes(output).unwrap().offset as i64;

    outputs[0] = Val::I64(out_offset);
    Ok(())
}

fn get_command_output(command: &str) -> Vec<u8> {
    let command_result = Command::new(command).stderr(Stdio::piped()).spawn();

    if let Ok(command) = command_result {
        let result = command.wait_with_output().unwrap();

        result.stderr
    } else {
        vec![]
    }
}
