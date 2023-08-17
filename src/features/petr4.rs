use async_process::Command;
use std::path::Path;
use std::str;
use std::env;

pub struct Petr4 {
    petr4_path: String,
    workspace_root: String,
    os: String,
    bool_windows: bool,
    liste_command_str: Vec<Vec<String>>
}


impl Petr4 {
    pub fn new() -> Petr4 {
        let os: &str = env::consts::OS;
        let mut bool_windows: bool = false;
        let mut liste_command_str : Vec<Vec<String>> = vec![vec!["ln".to_string(), "-s".to_string()], vec!["mv".to_string()], vec!["rm".to_string()], vec!["rm".to_string(), "-r".to_string()], vec!["mkdir".to_string(), "-p".to_string()]];
    
        if os == "windows" {
            bool_windows = true;
            liste_command_str = vec![vec!["mklink".to_string()], vec!["move".to_string()], vec!["del".to_string()], vec!["rmdir".to_string(), "/s".to_string(), "/q".to_string()], vec!["mkdir".to_string()]];
        }

        Petr4 {
            petr4_path: "".to_string(),
            workspace_root: "".to_string(),
            os: os.to_string(),
            bool_windows,
            liste_command_str
        }
    }
    pub fn config(&mut self, petr4: String, workspace: String) {
        self.petr4_path = petr4;
        self.workspace_root = workspace;
    }
    pub fn get_petr4_path(&self) -> &Path {
        return Path::new(&self.petr4_path)
    }
    pub fn get_workspace_root(&self) -> &Path {
        return Path::new(&self.workspace_root)
    }
    pub fn get_os(&self) -> String {
        return self.os.clone()
    }
    pub fn get_bool_windows(&self) -> bool {
        return self.bool_windows.clone()
    }
    pub fn get_liste_command_str(&self) -> Vec<Vec<String>> {
        return self.liste_command_str.clone()
    }
    pub fn get(&self) -> Option<(&Path, &Path, bool, Vec<Vec<String>>)> {
        return Some((self.get_petr4_path(), self.get_workspace_root(), self.get_bool_windows(), self.get_liste_command_str()))
    }
    
}

fn create_command(command: &mut Vec<String>) -> Command{
    let mut t: Command = Command::new(command.remove(0));
    for el in command{
        t.arg(el);
    }
    return t;
}

pub async fn testing(p4: &str, petr4_args: Option<(&Path, &Path, bool, Vec<Vec<String>>)>) {
    let path_p4 = Path::new(p4);


    if let Some((petr4_path, workspace_root, bool_windows, mut liste_command_str)) = petr4_args{
        info!("a");
        let path_output = path_p4.clone().with_extension("output");
        create_command(&mut liste_command_str[2])
            .arg(format!("\"{}\"",path_output.as_os_str().to_str().unwrap()))
            .output()
            .await;
    
        // verify the stf file exists
        let path_stf = path_p4.clone().with_extension("stf");
        if !path_stf.exists(){
            return;
        }
        info!("b");

        // verify the binary of the petr4 exists
        let path_test_binary = petr4_path.clone().join("_build/default/bin/test.exe");
        if path_test_binary.exists(){
            info!("c");
            // find in the new name for the p4 folder of testing file
            let new_p4_testing = petr4_path.clone().join("_build/default/p4stf/custom-stf-tests");
            while new_p4_testing.exists(){
                new_p4_testing.join("_exists");
            }
            info!("d");
            let new_p4_testing_path = new_p4_testing.as_os_str().to_str().unwrap();
    
            let p4_testing_path = petr4_path.clone().join("_build/default/p4stf/custom-stf-tests");
            match create_command(&mut liste_command_str[1])
                .arg(format!("\"{}\"",p4_testing_path.as_os_str().to_str().unwrap()))
                .arg(format!("\"{}\"",new_p4_testing_path))
                .output()
                .await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }
            match create_command(&mut liste_command_str[4])
                .arg(format!("\"{}\"",p4_testing_path.as_os_str().to_str().unwrap()))
                .output()
                .await {
                    Ok(_) => {}
                    Err(_) => {
                        delete(petr4_path,new_p4_testing_path, "", &mut liste_command_str).await;
                        return;
                    }
                }
            
            // find in the new name for the p4 include folder of testing file
            let new_p4_include_testing = petr4_path.clone().join("_build/default/examples");
            while new_p4_include_testing.exists(){
                new_p4_include_testing.join("_exists");
            }
            info!("d");
            let new_p4_include_testing_path_str = new_p4_include_testing.as_os_str().to_str().unwrap();
    
            let p4_include_testing_path = petr4_path.clone().join("_build/default/examples");
            match create_command(&mut liste_command_str[1])
                .arg(format!("\"{}\"",p4_include_testing_path.as_os_str().to_str().unwrap()))
                .arg(format!("\"{}\"",new_p4_include_testing_path_str))
                .output()
                .await {
                    Ok(_) => {}
                    Err(_) => {
                        delete(petr4_path,new_p4_testing_path, "", &mut liste_command_str).await;
                        return;
                    }
                }
                match create_command(&mut liste_command_str[4])
                    .arg("-p")
                    .arg(format!("\"{}\"",p4_include_testing_path.clone().join("checker_tests/good").as_os_str().to_str().unwrap()))
                    .output()
                    .await {
                        Ok(_) => {
                        }
                        Err(_) => {
                            delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                            return;
                        }
                    }
                match create_command(&mut liste_command_str[4])
                    .arg("-p")
                    .arg(format!("\"{}\"",p4_include_testing_path.clone().join("checker_tests/excluded/good").as_os_str().to_str().unwrap()))
                    .output()
                    .await {
                        Ok(_) => {
                        }
                        Err(_) => {
                            delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                            return;
                        }
                    }


            // get the include file and copy it the include folder :
            if bool_windows{
                /* TODO : chat gpt for the find :
                Command::new("cmd")
            .arg("/C")
            .arg(format!(
                r#"for /r {} %%F in (*.p4) do xcopy /s /i "%%F" "{}""#,
                source_dir, dest_dir
            )) */
            }else{
                match Command::new("sh")
                .arg("-c")
                .arg(&format!(r#"find "{}" -type f -iname '*.p4' -exec cp --parents "{{}}" "{}" \;"#, ".", p4_include_testing_path.as_os_str().to_str().unwrap()))
                .current_dir(workspace_root.as_os_str().to_str().unwrap())
                .output()
                .await {
                    Ok(x) => {
                        info!("a{:?}",x);
                    }
                    Err(e) => {
                        info!("b{:?}",e);
                        delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                        return;
                    }
                }

                match Command::new("which")
                .arg("p4c")
                .output()
                .await {
                    Ok(x) => {
                        let stdout = str::from_utf8(&x.stdout).unwrap();
                        info!("{}",stdout);
                        let parts = ((stdout.split("\n").collect::<Vec<&str>>())[0]).split("/");
                        let mut path_include = "".to_string();
                        let mut index = 1;
                        let length = parts.clone().count() -1;
                        for part in parts{
                            info!("{}",part);
                            if index == length{
                                path_include = format!("{}/{}", path_include, "share");
                            } else{
                                path_include = format!("{}/{}", path_include, part);
                            }
                            index += 1;
                        }
                        path_include = format!("{}/{}", path_include, "p4include");
                        info!("{}",path_include);
                        match Command::new("sh")
                        .arg("-c")
                        .arg(&format!(r#"find "{}" -type f -iname '*.p4' -exec cp --parents "{{}}" "{}" \;"#, ".", p4_include_testing_path.as_os_str().to_str().unwrap()))
                        .current_dir(path_include.clone())
                        .output()
                        .await {
                            Ok(x) => {
                                info!("c{:?}",x);
                            }
                            Err(e) => {
                                info!("d{:?}",e);
                                delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                                return;
                            }
                        }}
                    Err(e) => {
                        info!("e{:?}",e);
                        delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                        return;
                    }
                }
            }


            // copy p4 and stf file
            // find in what name the p4 file will be create for the testing
            let mut name_p4_testing = "testing_p4_lsp_file".to_string();
            let mut p4_testing = petr4_path.clone().join(format!("_build/default/p4stf/custom-stf-tests/{}.p4",name_p4_testing));
            while p4_testing.exists(){
                name_p4_testing = format!("{}_exists",name_p4_testing);
                p4_testing = p4_testing.with_file_name(format!("{}.p4",name_p4_testing))
            }
            info!("d");
    
            // create the p4 file
            match create_command(&mut liste_command_str[0])
                .arg(format!("\"{}\"",path_p4.as_os_str().to_str().unwrap()))
                .arg(format!("\"{}\"",p4_testing.as_os_str().to_str().unwrap()))
                .output()
                .await {
                    Ok(_) => {}
                    Err(_) => {
                        delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                        return;
                    }
                }
            info!("f");
    
            // create the stf file
            match create_command(&mut liste_command_str[1])
                .arg(format!("\"{}\"",path_stf.as_os_str().to_str().unwrap()))
                .arg(format!("\"{}\"",p4_testing_path.clone().with_extension("stf").as_os_str().to_str().unwrap()))
                .output()
                .await {
                    Ok(_) => {}
                    Err(_) => {
                        delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
                        return;
                    }
                }
            info!("j");
    
        
            // execute the commande
            let output = Command::new("./bin/test.exe")
                .current_dir(format!("\"{}\"",petr4_path.clone().join("/_build/default").as_os_str().to_str().unwrap()))
                .output()
                .await;
            info!("k");
    
            // get the output
            match output {
                Ok(x) => {
                    info!("work,{:?}",x);
                    let parts = str::from_utf8(&x.stdout).unwrap().split("\n");
                    for part in parts{
                        if part.contains(format!(" {}.p4",name_p4_testing).as_str()) && part.contains("petr4 stf tests"){
                            info!("{}",part);
    
                            if part.contains("[FAIL]"){
                                let mut index = "-1".to_string();
                                for el in part.split(" "){
                                    if let Ok(x) = el.parse::<i32>(){
                                        index = x.to_string();
                                        break;
                                    }
                                }
                                
                                let mut p4_testing_name = "petr4 stf tests.".to_string();
                                let ext = format!("{}.output", index.clone());
                                let mut t = format!("{}{}",p4_testing_name.clone(), ext.clone());
                                let mut p4_testing = petr4_path.clone().join(format!("_build/default/_build/_tests/Stf-tests/{}",t));
                                while !p4_testing.exists(){
                                    p4_testing_name = format!("{}0",p4_testing_name.clone());
                                    t = format!("{}{}",p4_testing_name.clone(), ext.clone());
                                    p4_testing = p4_testing.with_file_name(t);
                                }
                                info!("{:?}",p4_testing);
    
                                create_command(&mut liste_command_str[1])
                                    .arg(format!("\"{}\"",p4_testing.as_os_str().to_str().unwrap()))
                                    .arg(format!("\"{}\"",path_output.as_os_str().to_str().unwrap()))
                                    .output()
                                    .await;
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    info!("fail,{}",e);
                },
            };
    
            // remove the p4 and stf file
        delete(petr4_path,new_p4_testing_path, new_p4_include_testing_path_str, &mut liste_command_str).await;
        }
    }
}

async fn delete(petr4_path: &Path, new_petr4_path: &str, new_petr4_include_path: &str, liste_command_str: &mut Vec<Vec<String>>){
    info!("a");
    let p4_testing_path = petr4_path.clone().join("_build/default/p4stf/custom-stf-tests");
    let p4_include_testing_path = petr4_path.clone().join("_build/default/examples");

    create_command(&mut liste_command_str[3])
        .arg(format!("\"{}\"",p4_testing_path.as_os_str().to_str().unwrap()))
        .output()
        .await;

    create_command(&mut liste_command_str[1])
        .arg(format!("\"{}\"",new_petr4_path))
        .arg(format!("\"{}\"",p4_testing_path.as_os_str().to_str().unwrap()))
        .output()
        .await;

    create_command(&mut liste_command_str[3])
        .arg(format!("\"{}\"",p4_include_testing_path.as_os_str().to_str().unwrap()))
        .output()
        .await;

    create_command(&mut liste_command_str[1])
        .arg(format!("\"{}\"",new_petr4_include_path))
        .arg(format!("\"{}\"",p4_include_testing_path.as_os_str().to_str().unwrap()))
        .output()
        .await;
    
}