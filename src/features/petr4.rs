use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

pub struct Petr4 {
    petr4_path: String,
    workspace_root: String,
    bool_windows: bool,
    liste_command_str: Vec<Vec<String>>,
}

fn get_command_output(
    current_dir: String,
    args: &mut Vec<String>,
    plus_args: &mut Vec<String>,
) -> (Vec<u8>, Vec<u8>) {
    let command = args.remove(0);
    args.append(plus_args);
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
        let mut liste_command_str: Vec<Vec<String>> = vec![
            vec!["ln".to_string(), "-s".to_string()],
            vec!["mv".to_string()],
            vec!["rm".to_string()],
            vec!["rm".to_string(), "-r".to_string()],
            vec!["mkdir".to_string(), "-p".to_string()],
            vec!["which".to_string()]
        ];

        if os == "windows" {
            bool_windows = true;
            liste_command_str = vec![
                vec!["mklink".to_string()],
                vec!["move".to_string()],
                vec!["del".to_string()],
                vec!["rmdir".to_string(), "/s".to_string(), "/q".to_string()],
                vec!["mkdir".to_string()],
                vec!["where".to_string()]
            ];
        }

        Petr4 {
            petr4_path: "".to_string(),
            workspace_root: "".to_string(),
            bool_windows,
            liste_command_str,
        }
    }
    pub fn config(&mut self, petr4: String, workspace: String) {
        self.petr4_path = petr4;
        self.workspace_root = workspace;
    }
    pub fn get_petr4_path(&self) -> &Path {
        return Path::new(&self.petr4_path);
    }
    pub fn get_workspace_root(&self) -> &Path {
        return Path::new(&self.workspace_root);
    }
    pub fn get_bool_windows(&self) -> bool {
        return self.bool_windows.clone();
    }
    pub fn get_liste_command_str(&self) -> Vec<Vec<String>> {
        return self.liste_command_str.clone();
    }

    pub fn write_output(&self, path_output: &Path, content: &[u8]) {
        let mut file = File::create(path_output).unwrap();
        file.write_all(content).unwrap();
    }

    pub fn fail(
        &self,
        path_output: &Path,
        number_command: i32,
        type_command: usize,
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
        self.write_output(path_output, "the plugin don't work for windows yet.".as_bytes());
    }

    pub fn testing(&self, p4: &str) {

        let path_p4 = Path::new(p4);
        let mut number_commande = 0;
        let mut command_using = 0;

        let path_output = path_p4.clone().with_extension("output");
        if self.get_bool_windows(){
            self.windows(&path_output);
            return;
        }
        get_command_output(
            "".to_string(),
            &mut self.liste_command_str[2].clone(),
            &mut vec![format!("{}", path_output.as_os_str().to_str().unwrap())],
        );

        // verify the stf file exists
        let path_stf: std::path::PathBuf = path_p4.clone().with_extension("stf");
        if !path_stf.exists() {
            return;
        }

        // verify the binary of the petr4 exists
        let path_test_binary = self.get_petr4_path().join("_build/default/bin/test.exe");
        if path_test_binary.exists() {
            // find in the new name for the p4 folder of testing file
            let mut temp_name = "custom-stf-tests".to_string();
            let mut new_p4_testing = self
                .get_petr4_path()
                .join(format!("_build/default/p4stf/{}", temp_name.clone()));
            while new_p4_testing.exists() {
                temp_name = format!("{}_exists", temp_name.clone());
                new_p4_testing = new_p4_testing.with_file_name(temp_name.clone());
            }

            let new_p4_testing_path = new_p4_testing.as_os_str().to_str().unwrap();

            let p4_testing_path = self
                .get_petr4_path()
                .join("_build/default/p4stf/custom-stf-tests");
            command_using = 1;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![
                    format!("{}", p4_testing_path.as_os_str().to_str().unwrap()),
                    format!("{}", new_p4_testing_path),
                ],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                return;
            }

            command_using = 4;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![format!("{}", p4_testing_path.as_os_str().to_str().unwrap())],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(new_p4_testing_path, "", "");
                return;
            }

            // find in the new name for the p4 include folder of testing file
            let mut temp_name = "examples".to_string();
            let mut new_p4_include_testing = self
                .get_petr4_path()
                .join(format!("_build/default/{}", temp_name.clone()));
            while new_p4_include_testing.exists() {
                temp_name = format!("{}_exists", temp_name.clone());
                new_p4_include_testing = new_p4_include_testing.with_file_name(temp_name.clone());
            }

            let new_p4_include_testing_path_str =
                new_p4_include_testing.as_os_str().to_str().unwrap();

            let p4_testing_file_folder_path = path_p4.parent().unwrap();
            let p4_include_testing_path = self.get_petr4_path().join("_build/default/examples");

            // create temp folder
            let mut temp_name = "p4_lsp_testing_petr4".to_string();
            let mut p4_testing_file_folder_path_link =
                env::temp_dir().clone().join(temp_name.clone());
            while new_p4_testing.exists() {
                temp_name = format!("{}_exists", temp_name.clone());
                p4_testing_file_folder_path_link =
                    p4_testing_file_folder_path_link.with_file_name(temp_name.clone());
            }

            let p4_testing_file_folder_path_link_root = p4_testing_file_folder_path_link.join(
                p4_testing_file_folder_path
                    .strip_prefix(self.get_workspace_root())
                    .unwrap(),
            );

            command_using = 1;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![
                    format!("{}", p4_include_testing_path.as_os_str().to_str().unwrap()),
                    format!("{}", new_p4_include_testing_path_str),
                ],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(
                    new_p4_testing_path,
                    new_p4_include_testing_path_str,
                    p4_testing_file_folder_path_link
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                );
                return;
            }

            // add folder for petr4 testing
            command_using = 4;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![format!(
                    "{}",
                    p4_testing_file_folder_path_link_root
                        .clone()
                        .join("checker_tests/good")
                        .as_os_str()
                        .to_str()
                        .unwrap()
                )],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(
                    new_p4_testing_path,
                    new_p4_include_testing_path_str,
                    p4_testing_file_folder_path_link
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                );
                return;
            }

            command_using = 4;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![format!(
                    "{}",
                    p4_testing_file_folder_path_link_root
                        .clone()
                        .join("checker_tests/excluded/good")
                        .as_os_str()
                        .to_str()
                        .unwrap()
                )],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(
                    new_p4_testing_path,
                    new_p4_include_testing_path_str,
                    p4_testing_file_folder_path_link
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                );
                return;
            }

            // get the include file and copy it the include folder :
            let look_path = self.get_workspace_root().as_os_str().to_str().unwrap();
            if self.bool_windows {
                /* from chatgpt */
                command_using = self.liste_command_str.len() + 10;
                let (stdout, stderr) = get_command_output(
                    "".to_string(),
                    &mut vec!["cmd".to_string(), "/C".to_string()],
                    &mut vec![format!(
                        r#"setlocal enabledelayedexpansion

                        set "d={look}"
                        set "c={copy}"
                        
                        for /r "%d%" %%F in (*.p4) do (
                            set "file=%%F"
                            set "relative=!file:%d%\=!"
                        
                            if exist "%%F\" (
                                mkdir "%c%\!relative!"
                            ) else if exist "%%F" (
                                set "dir=%%~dpF"
                                set "relative_dir=!relative:*\=!"
                                set "relative_dir=!relative_dir:~0,-1!"
                                if not "!relative_dir!"=="!relative!" (
                                    if exist "!dir!" (
                                        if not exist "%c%\!relative_dir!" (
                                            mkdir "%c%\!relative_dir!"
                                        )
                                    )
                                )
                                mklink "%c%\!relative!" "%%F"
                            )
                        )
                        
                        endlocal"#,
                        copy = p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                        look = look_path.clone()
                    )],
                );
                number_commande += 1;
                if !stderr.is_empty() {
                    self.fail(&path_output, number_commande, command_using, stdout, stderr);
                    self.delete(
                        new_p4_testing_path,
                        new_p4_include_testing_path_str,
                        p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    );
                    return;
                }

                // add p4 include testing
                command_using = 5;
                let (stdout, stderr) = get_command_output(
                    "".to_string(),
                    &mut self.liste_command_str[command_using].clone(),
                    &mut vec!["p4c".to_string()],
                );
                number_commande += 1;
                if !stderr.is_empty() {
                    self.fail(&path_output, number_commande, command_using, stdout, stderr);
                    self.delete(
                        new_p4_testing_path,
                        new_p4_include_testing_path_str,
                        p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    );
                    return;
                }

                // TODO : get the path to the p4 include folder from the where, and create a link
            } else {
                // do this command : https://unix.stackexchange.com/questions/406561/gnu-find-get-absolute-and-relative-path-in-exec
                command_using = self.liste_command_str.len() + 10;
                let (stdout, stderr) = get_command_output(
                    "".to_string(),
                    &mut vec!["sh".to_string(), "-c".to_string()],
                    &mut vec![format!(
                        r#"find {look} -iname "*.p4" -exec sh -c '
                        file="{{}}"
                        d="{look}"
                        relative=${{file#"$d/"}}
                        if [ -d $file ]; then
                            mkdir -p "{copy}/$relative"
                        elif [ -f $file ]; then
                            dir=${{file%/*}}
                            relative_dir=${{relative%/*}}
                            relative_dir=${{relative_dir:-.}}
                            if [ "$relative_dir" != "$relative" ] && [ -d "$dir" ] && [ ! -d "{copy}/$relative_dir" ]; then
                                mkdir -p "{copy}/$relative_dir"
                            fi
                            ln -s "$file" "{copy}/$relative"
                        fi
                      ' \;"#,
                        copy = p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                        look = look_path.clone()
                    )],
                );
                number_commande += 1;
                if !stderr.is_empty() {
                    self.fail(&path_output, number_commande, command_using, stdout, stderr);
                    self.delete(
                        new_p4_testing_path,
                        new_p4_include_testing_path_str,
                        p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    );
                    return;
                }

                // add p4 include testing
                command_using = 5;
                let (stdout, stderr) = get_command_output(
                    "".to_string(),
                    &mut self.liste_command_str[command_using].clone(),
                    &mut vec!["p4c".to_string()],
                );
                number_commande += 1;
                if !stderr.is_empty() {
                    self.fail(&path_output, number_commande, command_using, stdout, stderr);
                    self.delete(
                        new_p4_testing_path,
                        new_p4_include_testing_path_str,
                        p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    );
                    return;
                }
                let stdout = str::from_utf8(&stdout).unwrap();
                let parts = ((stdout.split("\n").collect::<Vec<&str>>())[0]).split("/");
                let mut path_include = "".to_string();
                let mut index = 1;
                let length = parts.clone().count() - 1;
                for part in parts {
                    if index == length {
                        path_include = format!("{}/{}", path_include, "share");
                    } else {
                        path_include = format!("{}/{}", path_include, part);
                    }
                    index += 1;
                }
                path_include = format!("{}/{}", path_include, "p4include");
                command_using = self.liste_command_str.len() + 10;
                let (stdout, stderr) = get_command_output(
                    "".to_string(),
                    &mut vec!["sh".to_string(), "-c".to_string()],
                    &mut vec![format!(
                        r#"find {look} -iname "*.p4" -exec sh -c '
                        file="{{}}"
                        d="{look}"
                        relative=${{file#"$d/"}}
                        if [ -d $file ]; then
                            mkdir -p "{copy}/$relative"
                        elif [ -f $file ]; then
                            dir=${{file%/*}}
                            relative_dir=${{relative%/*}}
                            relative_dir=${{relative_dir:-.}}
                            if [ "$relative_dir" != "$relative" ] && [ -d "$dir" ] && [ ! -d "{copy}/$relative_dir" ]; then
                                mkdir -p "{copy}/$relative_dir"
                            fi
                            ln -s "$file" "{copy}/$relative"
                        fi
                      ' \;"#,
                        copy = p4_testing_file_folder_path_link_root
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                        look = path_include.clone()
                    )],
                );
                number_commande += 1;
                if !stderr.is_empty() {
                    self.fail(&path_output, number_commande, command_using, stdout, stderr);
                    self.delete(
                        new_p4_testing_path,
                        new_p4_include_testing_path_str,
                        p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    );
                    return;
                }

                // create link
                command_using = 0;
                let (stdout, stderr) = get_command_output(
                    "".to_string(),
                    &mut self.liste_command_str[command_using].clone(),
                    &mut vec![
                        format!(
                            "{}",
                            p4_testing_file_folder_path_link_root
                                .as_os_str()
                                .to_str()
                                .unwrap()
                        ),
                        format!("{}", p4_include_testing_path.as_os_str().to_str().unwrap()),
                    ],
                );
                number_commande += 1;
                if !stderr.is_empty() {
                    self.fail(&path_output, number_commande, command_using, stdout, stderr);
                    self.delete(
                        new_p4_testing_path,
                        new_p4_include_testing_path_str,
                        p4_testing_file_folder_path_link
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    );
                    return;
                }
            }

            // copy p4 and stf file
            // find in what name the p4 file will be create for the testing
            let name_p4_testing = "testing_p4_lsp_file";
            let mut p4_testing = self.get_petr4_path().join(format!(
                "_build/default/p4stf/custom-stf-tests/{}.p4",
                name_p4_testing
            ));

            // create the p4 file
            command_using = 0;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![
                    format!("{}", path_p4.as_os_str().to_str().unwrap()),
                    format!("{}", p4_testing.as_os_str().to_str().unwrap()),
                ],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(
                    new_p4_testing_path,
                    new_p4_include_testing_path_str,
                    p4_testing_file_folder_path_link
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                );
                return;
            }

            // create the stf file
            command_using = 0;
            let (stdout, stderr) = get_command_output(
                "".to_string(),
                &mut self.liste_command_str[command_using].clone(),
                &mut vec![
                    format!("{}", path_stf.as_os_str().to_str().unwrap()),
                    format!(
                        "{}",
                        p4_testing
                            .clone()
                            .with_extension("stf")
                            .as_os_str()
                            .to_str()
                            .unwrap()
                    ),
                ],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(
                    new_p4_testing_path,
                    new_p4_include_testing_path_str,
                    p4_testing_file_folder_path_link
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                );
                return;
            }

            // execute the commande
            command_using = self.liste_command_str.len() + 20;
            let (stdout, stderr) = get_command_output(
                format!(
                    "{}",
                    self.get_petr4_path()
                        .clone()
                        .join("_build/default")
                        .as_os_str()
                        .to_str()
                        .unwrap()
                ),
                &mut vec!["./bin/test.exe".to_string()],
                &mut vec![],
            );
            number_commande += 1;
            if !stderr.is_empty() {
                self.fail(&path_output, number_commande, command_using, stdout, stderr);
                self.delete(
                    new_p4_testing_path,
                    new_p4_include_testing_path_str,
                    p4_testing_file_folder_path_link
                        .as_os_str()
                        .to_str()
                        .unwrap(),
                );
                return;
            }

            // get the output
            let parts = str::from_utf8(&stdout).unwrap().split("\n");
            for part in parts {
                if part.contains(format!(" {}.p4", name_p4_testing).as_str())
                    && part.contains("petr4 stf tests")
                {
                    if part.contains("[FAIL]") {
                        let mut index = "-1".to_string();
                        for el in part.split(" ") {
                            if let Ok(x) = el.parse::<i32>() {
                                index = x.to_string();
                                break;
                            }
                        }

                        let mut p4_testing_name = "petr4 stf tests.".to_string();
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
                            "".to_string(),
                            &mut self.liste_command_str[command_using].clone(),
                            &mut vec![
                                format!("{}", p4_testing.as_os_str().to_str().unwrap()),
                                format!("{}", path_output.as_os_str().to_str().unwrap()),
                            ],
                        );
                        number_commande += 1;
                        if !stderr.is_empty() {
                            self.fail(&path_output, number_commande, command_using, stdout, stderr);
                            self.delete(
                                new_p4_testing_path,
                                new_p4_include_testing_path_str,
                                p4_testing_file_folder_path_link
                                    .as_os_str()
                                    .to_str()
                                    .unwrap(),
                            );
                            return;
                        }
                    } else {
                        self.pass(&path_output);
                    }
                    break;
                }
            }

            // remove the p4 and stf file
            self.delete(
                new_p4_testing_path,
                new_p4_include_testing_path_str,
                p4_testing_file_folder_path_link
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            );
        }
    }

    fn delete(
        &self,
        new_petr4_path: &str,
        new_petr4_include_path: &str,
        p4_testing_file_folder_path_link: &str,
    ) {
        let p4_testing_path = self
            .get_petr4_path()
            .join("_build/default/p4stf/custom-stf-tests");

        get_command_output(
            "".to_string(),
            &mut self.liste_command_str[3].clone(),
            &mut vec![format!("{}", p4_testing_path.as_os_str().to_str().unwrap())],
        );
        get_command_output(
            "".to_string(),
            &mut self.liste_command_str[1].clone(),
            &mut vec![
                format!("{}", new_petr4_path),
                format!("{}", p4_testing_path.as_os_str().to_str().unwrap()),
            ],
        );

        if new_petr4_include_path != "" {
            let p4_include_testing_path = self.get_petr4_path().join("_build/default/examples");

            get_command_output(
                "".to_string(),
                &mut self.liste_command_str[3].clone(),
                &mut vec![format!(
                    "{}",
                    p4_include_testing_path.as_os_str().to_str().unwrap()
                )],
            );
            get_command_output(
                "".to_string(),
                &mut self.liste_command_str[1].clone(),
                &mut vec![
                    format!("{}", new_petr4_include_path),
                    format!("{}", p4_include_testing_path.as_os_str().to_str().unwrap()),
                ],
            );
        }

        if p4_testing_file_folder_path_link != "" {
            get_command_output(
                "".to_string(),
                &mut self.liste_command_str[3].clone(),
                &mut vec![format!("{}", p4_testing_file_folder_path_link)],
            );
        }
    }
}
