use std::process::{Command, Stdio};

use super::HostFunction;
use extism::{CurrentPlugin, Error, Function, UserData, Val, ValType};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
struct CommandOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
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

    let split_command = shell_words::split(command).unwrap();

    let (stdout, stderr) = get_command_output(&split_command[0], &split_command[1..]);

    let serialized = serde_json::to_string(&CommandOutput { stdout, stderr })?;

    let stdout_offset = memory.alloc_bytes(serialized).unwrap().offset as i64;

    outputs[0] = Val::I64(stdout_offset);
    Ok(())
}

fn get_command_output(command: &str, args: &[String]) -> (Vec<u8>, Vec<u8>) {
    debug!("Command: {} Args: {:?}", command, args);
    let command_result = Command::new(command)
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    if let Ok(command) = command_result {
        let result = command.wait_with_output().unwrap();

        (result.stdout, result.stderr)
    } else {
        (vec![], vec![])
    }
}
