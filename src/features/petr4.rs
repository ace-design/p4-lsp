use async_process::Command;
use std::path::Path;
use std::fs::{File, remove_file};
use std::io::prelude::*;


pub async fn petr4_testing(path_p4: &str, path_petr4: &str) {
    info!("a");
    // verify the stf file exists
    let mut path_stf_str = path_p4.clone().to_string();
    path_stf_str.replace_range((path_p4.len()-2)..(path_p4.len()),"stf");
    let path_stf = Path::new(path_stf_str.as_str());
    if (!path_stf.exists()){
        return;
    }
    info!("b");
    
    // verify the binary of the petr4 exists
    let temp_path = format!("{}/_build/default/bin/test.exe", path_petr4);
    let path_test_binary = Path::new(temp_path.as_str());
    if (path_test_binary.exists()){
        info!("c");
        // copy p4 and stf file

        // find in what name the p4 file will be create for the testing
        let mut name_p4_testing = format!("{}","testing_p4_lsp_file");
        let mut p4_testing_path = format!("{}/_build/default/p4stf/custom-stf-tests/{}", path_petr4, name_p4_testing);
        let mut t = format!("{}.p4",p4_testing_path.clone());
        let mut p4_testing = Path::new(&t);
        while(p4_testing.exists()){
            name_p4_testing = format!("{}_exists",name_p4_testing);
            p4_testing_path = format!("{}_exists",p4_testing_path.clone());
            t = format!("{}.p4",p4_testing_path.clone());
            p4_testing = Path::new(&t);
        }
        info!("d");

        // create the p4 file
        let mut file = File::open(path_p4).unwrap();
        let mut contents = String::new();
        match file.read_to_string(&mut contents){
            Ok(x) => {},
            Err(e) => {
                return;
            }
        }
        info!("e");

        let mut file = File::create(p4_testing.as_os_str()).unwrap();
        match file.write_all(contents.as_bytes()){
            Ok(x) => {},
            Err(e) => {
                return;
            }
        }
        info!("f");

        // verify we can create the stf, if not move the file that block us
        let mut p4_testing_path_stf = p4_testing_path.clone();
        let mut t = format!("{}.stf",p4_testing_path_stf.clone());
        let mut p4_testing_stf = Path::new(&t);
        let mut stf_exist = false;
        while(p4_testing_stf.exists()){
            stf_exist = true;
            p4_testing_path_stf = format!("{}_exists",p4_testing_path_stf.clone());
            t = format!("{}.stf",p4_testing_path_stf.clone());
            p4_testing_stf = Path::new(&t);
        }
        info!("g");
        if (stf_exist){
            let temp = p4_testing_stf.clone();
            let mut t = format!("{}.stf",p4_testing_path.clone());
            p4_testing_stf = Path::new(&t);
            let output = Command::new("mv")
                .arg(p4_testing_stf.as_os_str())
                .arg(temp.as_os_str())
                .output()
                .await;
        }
        info!("h");

        // create the stf file
        let mut file = File::open(path_stf.as_os_str()).unwrap();
        let mut contents = String::new();
        match file.read_to_string(&mut contents){
            Ok(x) => {},
            Err(e) => {
                remove_file(p4_testing.as_os_str());
                if (stf_exist){ /* todo */}
                return;
            }
        }
        info!("i");
    
        match file.write_all(contents.as_bytes()){
            Ok(x) => {},
            Err(e) => {
                remove_file(p4_testing.as_os_str());
                if (stf_exist){ /* todo */}
                return;
            }
        }
        info!("j");

    
        // execute the commande
        let output = Command::new(path_test_binary.as_os_str())
            .output()
            .await;
        info!("k");

        // get the output
        match output {
            Ok(x) => {
                println!("{:?}",x);
                /*let mut error = String::new();
                if (x.stderr.len() > 0) {
                    let text = str::from_utf8(&x.stderr).unwrap();
                }*/
            }
            Err(e) => {},
        };

        // remove the p4 and stf file
    }
}