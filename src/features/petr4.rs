use async_process::Command;
use std::path::Path;
use std::str;


pub async fn petr4_testing(path_p4: &str, path_petr4: &str) {
    info!("a");
    let mut path_output = path_p4.clone().to_string();
    path_output.replace_range((path_p4.len()-2)..(path_p4.len()),"output");
    Command::new("rm")
        .arg(path_output.clone())
        .output()
        .await;

    // verify the stf file exists
    let mut path_stf_str = path_p4.clone().to_string();
    path_stf_str.replace_range((path_p4.len()-2)..(path_p4.len()),"stf");
    let path_stf = Path::new(path_stf_str.as_str());
    if (!path_stf.exists()){
        info!("a4");
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
        let mut name_p4_testing = format!("{}","zzztesting_p4_lsp_file");
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
        match Command::new("cp")
            .arg(path_p4)
            .arg(p4_testing.as_os_str())
            .output()
            .await {
                Ok(x) => {}
                Err(e) => {
                    return;
                }
            }
        info!("f");

        // create the stf file
        let stf_testing = format!("{}.stf",p4_testing_path.clone());
        match Command::new("cp")
            .arg(path_stf.as_os_str())
            .arg(stf_testing.clone())
            .output()
            .await {
                Ok(x) => {}
                Err(e) => {
                    Command::new("rm")
                        .arg(p4_testing.as_os_str())
                        .output().await;
                    return;
                }
            }
        info!("j");

    
        // execute the commande
        let output = Command::new("./bin/test.exe")
            .current_dir(format!("{}/_build/default", path_petr4))
            .output()
            .await;
        info!("k");

        // get the output
        match output {
            Ok(x) => {
                info!("work,{:?}",x);
                let parts = str::from_utf8(&x.stdout).unwrap().split("\n");
                for part in parts{
                    if (part.contains(format!(" {}.p4",name_p4_testing).as_str()) && part.contains("petr4 stf tests")){
                        info!("{}",part);

                        if (part.contains("[FAIL]")){
                            let mut index = "-1".to_string();
                            for el in part.split(" "){
                                if let Ok(x) = el.parse::<i32>(){
                                    index = x.to_string();
                                    break;
                                }
                            }
                            
                            let mut p4_testing_path = format!("{}/_build/default/_build/_tests/Stf-tests/petr4 stf tests.", path_petr4);
                            let ext = format!("{}.output", index.clone());
                            let mut t = format!("{}{}",p4_testing_path.clone(), ext.clone());
                            let mut p4_testing = Path::new(&t);
                            while(!p4_testing.exists()){
                                p4_testing_path = format!("{}0",p4_testing_path.clone());
                                t = format!("{}{}",p4_testing_path.clone(), ext.clone());
                                p4_testing = Path::new(&t);
                            }
                            info!("{:?}",p4_testing);

                            Command::new("mv")
                                .arg(p4_testing)
                                .arg(path_output.clone())
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
        Command::new("rm")
            .arg(p4_testing.as_os_str())
            .output().await;

        Command::new("rm")
            .arg(stf_testing.clone())
            .output().await;

    }
}