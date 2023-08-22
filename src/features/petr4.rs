use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::env;

pub struct Petr4 {
    petr4_path: String,
    bool_windows: bool,
}

fn get_command_output(
    command: &str,
    current_dir: &str,
    args: &mut Vec<String>,
) -> (Vec<u8>, Vec<u8>) {
    debug!("Command: {} Args: {:?}", command, args);
    let mut partial_command = Command::new(command);

    if !args.is_empty() {
        partial_command.args(args);
    }

    if current_dir != "" {
        partial_command.current_dir(current_dir);
    }

    let command_result = partial_command
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    if let Ok(command) = command_result {
        let result = command.wait_with_output().unwrap();

        (result.stdout, result.stderr)
    } else {
        (vec![], "error".as_bytes().to_vec())
    }
}

impl Petr4 {
    pub fn new() -> Petr4 {
        let os: &str = env::consts::OS;
        let mut bool_windows: bool = false;

        if os == "windows" {
            bool_windows = true;
        }

        Petr4 {
            petr4_path: "".to_string(),
            bool_windows,
        }
    }
    pub fn config(&mut self, petr4: String) {
        self.petr4_path = petr4;
    }
    pub fn get_petr4_path(&self) -> &Path {
        return Path::new(&self.petr4_path);
    }
    pub fn get_bool_windows(&self) -> bool {
        return self.bool_windows.clone();
    }

    pub fn write_output(&self, path_output: &Path, content: &[u8]) {
        let mut file = File::create(path_output).unwrap();
        file.write_all(content).unwrap();
    }

    pub fn fail(
        &self,
        path_output: &Path,
        number_command: i32,
        type_command: &str,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    ) {
        let content = format!("Petr4 plugin fail.\nNumber commande : {}\nThe type of commande : {}\nstdout : {}\n stderr : {}", number_command, type_command, String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap());
        self.write_output(path_output, content.as_bytes());
    }

    pub fn pass(&self, path_output: &Path) {
        self.write_output(path_output, "success".as_bytes());
    }

    pub fn windows(&self, path_output: &Path) {
        self.write_output(
            path_output,
            "the plugin don't work for windows yet.".as_bytes(),
        );
    }

    pub fn testing(&self, p4: &str) {
        let path_p4 = Path::new(p4);
        let mut number_commande = 0;
        let mut command_using = 0;

        let path_output = path_p4.clone().with_extension("output");
        get_command_output(
            "rm",
            "",
            &mut vec![format!("{}", path_output.as_os_str().to_str().unwrap())],
        );

        if self.get_bool_windows() {
            self.windows(&path_output);
            return;
        }

        // verify the stf file exists
        let path_stf: std::path::PathBuf = path_p4.clone().with_extension("stf");
        if !path_stf.exists() {
            return;
        }

        // verify the binary of the petr4 exists
        let path_test_binary = self.get_petr4_path().join("_build/default/bin/test.exe");
        if path_test_binary.exists() {
            // execute the commande
            let (stdout, stderr) = get_command_output(
                self.get_petr4_path()
                    .clone()
                    .join("_build/default/bin/test.exe")
                    .as_os_str()
                    .to_str()
                    .unwrap(),
                "",
                &mut vec![
                    "-t".to_string(),
                    path_p4.as_os_str().to_str().unwrap().to_string(),
                ],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(
                    &path_output,
                    number_commande,
                    "./bin/test.exe",
                    stdout,
                    stderr,
                );
                return;
            }

            // get the output
            let parts = str::from_utf8(&stdout).unwrap().split("\n");
            for part in parts {
                if part.contains("file testing") && part.contains("p4_lsp stf tests") {
                    if part.contains("[FAIL]") {
                        let mut index = "0";

                        let mut p4_testing_name = "p4_lsp stf tests.".to_string();
                        let ext = format!("{}.output", index.clone());
                        let mut t = format!("{}{}", p4_testing_name.clone(), ext.clone());
                        let mut p4_testing = self
                            .get_petr4_path()
                            .join(format!("_build/default/_build/_tests/Stf-tests/{}", t));
                        while !p4_testing.exists() {
                            p4_testing_name = format!("{}0", p4_testing_name.clone());
                            t = format!("{}{}", p4_testing_name.clone(), ext.clone());
                            p4_testing = p4_testing.with_file_name(t);
                        }

                        command_using = 1;
                        let (stdout, stderr) = get_command_output(
                            "mv",
                            "",
                            &mut vec![
                                format!("{}", p4_testing.as_os_str().to_str().unwrap()),
                                format!("{}", path_output.as_os_str().to_str().unwrap()),
                            ],
                        );
                        number_commande += 1;
                        if !stderr.is_empty() {
                            self.fail(&path_output, number_commande, "mv", stdout, stderr);
                            return;
                        }
                    } else {
                        self.pass(&path_output);
                    }
                    break;
                }
            }
        }
    }
}
