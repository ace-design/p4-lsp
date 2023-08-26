use crate::file_tree::{self as tree, ControlState, Information};
use crate::{file::File, settings::Settings};
use chrono::{Duration, Utc};
use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::lsif::UnknownTag;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tower_lsp::lsp_types::Url;

use std::io::Read;
use std::process::Command;
use tree_sitter::Parser;
use tree_sitter_p4::language;
use walkdir::WalkDir;
use crate::file_tree::Node;
use std::fmt;

const EXP_TIME_FOR_LOCAL: i64 = 500; // Expiration duration in seconds (1 day)

use crate::metadata::SymbolTable;

#[derive(Debug)]
pub struct Exp {
    file: File,
    duration_time: std::time::SystemTime,
    information: Information,
}
impl fmt::Display for Exp{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(format!("File Location {}",self.file.uri.as_str()).as_str())
    }
}
impl Clone for Exp {
    fn clone(&self) -> Self {
        Exp {
            file: self.file.clone(),
            duration_time: self.duration_time.clone(),
            information: self.information.clone(),
        }
    }
}
pub struct Cache {
    pub files: HashMap<Url, Exp>,
    pub file_tree: tree::FileTree,
    parser: Parser,
}

impl Cache {
    pub fn new() -> Cache {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();
        let mut arena = Arena::new();
      
        let data = tree::Node::new(None, None);
        let root_node = arena.new_node(data.clone());

        let files: HashMap<Url, Exp> = HashMap::new();
        //info!("Arena First {:?}",arena);


        let mut result = Cache {
            file_tree: tree::FileTree::initialize(arena, root_node),
            files: files,
            parser: parser,
        };
            
        
        result
    }
    pub fn add_init(&mut self){
        let pre_files = get_workspace_folders().unwrap();
        info!("Staretd Array {:?}",pre_files);
        for (url, state) in pre_files {

                let url_str = url.clone().to_file_path().unwrap().into_os_string().into_string().unwrap().clone();

                
                    if fs::metadata(url_str.clone()).is_ok() {
                        let content = match fs::File::open(url_str.clone()) {
                            Ok(mut file) => {
                                let mut contents = String::new();
                                file.read_to_string(&mut contents)
                                    .expect("Failed to read file");
                                contents
                            }
                            Err(e) => {
                                error!("Error opening file: in cache");
                                String::new()
                            }
                        };
    
                        info!("Staretd Adding File");
                        self.add_file(url.clone(), &content, state);
                        info!("Staretd Adding node");
                        self.add_node(url,false);
                    } else {
                        error!("File does not exist.");
                    }

                
                
        }
        let t  =&self.files.clone();
        for (url,exp) in t{
            self.add_node(url.clone(),true);
        }
    }

    
    pub fn get(&mut self, url: Url) -> Option<(File, Information)> {
        //info!("a");
        let (root_node,arena) = self.file_tree.get_prop().unwrap();

        //info!("a1");
        //info!("a3");
        let mut curr_node = None;
        //info!("a4");
        for element in root_node.children(arena) {
            let node = arena.get(element).unwrap().get();
            let compare_str = node.file_information.clone().unwrap().get_url();

            //info!("a8");
            if(url.to_string().contains(compare_str.as_str())){
                curr_node = Some(node)
            }
            //info!("a10");
        }
        //info!("a11");

        if let Some(x) = curr_node {
            Some((
                x.file.clone().unwrap(),
                x.file_information.clone().unwrap(),
            ))
        } else{
            None
        }
    }
    fn check_exp(&mut self) {
       /*  loop {
            let cloned_files = self.files.clone();
            for (url, exp) in cloned_files {
                let file_path = exp.file.uri.as_str();
                if Path::new(file_path).exists() {
                    let time: std::time::SystemTime = match fs::metadata(file_path) {
                        Ok(metadata) => metadata.modified().unwrap(),
                        Err(e) => {
                            debug!("Error In Cache");

                            exp.duration_time
                        }
                    };
                    if (time != exp.duration_time) {
                        //update file
                        let content = match fs::File::open(url.to_string()) {
                            Ok(mut file) => {
                                let mut contents = String::new();
                                file.read_to_string(&mut contents)
                                    .expect("Failed to read file");
                                contents
                            }
                            Err(e) => {
                                debug!("Error opening file: in cache");
                                String::new()
                            }
                        };

                        let tree = self.parser.parse(content.clone(), None);
                        let metadata = fs::metadata(file_path).unwrap();
                        let modified_time = metadata.modified().unwrap();

                        let new_exp = Exp {
                            file: File::new(url.clone(), &content.as_str().clone(), &tree, None,arena),
                            duration_time: modified_time,
                            information: exp.information,
                        };
                        self.files.insert(url.clone(), new_exp);
                    }
                }
            }
        }*/
    }

    pub fn add_file(&mut self, url: Url, content: &str, control_state: ControlState) {
        if(!self.files.contains_key(&url)){
        //Creating node from data
        let arena: &mut Arena<Node> =self.file_tree.get_arena();
        let tree = self.parser.parse(content, None);
        let url_str = url.clone().to_file_path().unwrap().into_os_string().into_string().unwrap().clone();
        let url_custom = Url::parse(&url.as_str()).unwrap();
        //Getting modified date
        let metadata = fs::metadata(url_str.as_str()).unwrap();
        let modified_time = metadata.modified().unwrap();
    
        let file = File::new(url_custom.clone(), content, &tree, None,arena);
        
        let information: Information = Information::new(url_custom.clone(), control_state);
 

        let new_exp = Exp {
            file: file,
            duration_time: modified_time,
            information: information,
        };
        self.files.insert(url_custom.clone(), new_exp);
        error!("Added File {}",url_str);
    }
    }

    pub fn update_node(&mut self, url: Url) {
       // let arena = self.file_tree.get_arena();
       /* let root_visit_node = self.file_tree.visit_root();
        let mut child_str = root_visit_node.get_children();
        let mut visit_node = None;
        for element in child_str.iter_mut() {
            let compare_str = self.file_tree.get_arena()
                .get(element.get_id().unwrap())
                .unwrap()
                .get()
                .file_information
                .clone()
                .unwrap()
                .get_url()
                .to_string();
            if (compare_str.contains(url.as_str())) {
                visit_node = Some(self.file_tree.get_arena().get(element.get_id().unwrap()).unwrap().get());
            }
        }
        /*let visit_node = child_str
        .iter()
        .find(|element| {
            let compare_str = arena.get(element.get_id().unwrap()).unwrap().get().file_information.clone().unwrap().get_url().to_string();

            /*let sub_str = element
                .get()
                .unwrap()
                .file_information
                .clone()
                .unwrap()
                .get_url()
                .to_string();*/
            compare_str.contains(url.as_str())
        })
        .unwrap();*/
        let file = visit_node.unwrap().file.clone().unwrap();
        self.add_node_include(file);*/
    }

    pub fn add_node(&mut self, url: Url,state:bool) {

        
        let mut map: HashMap<Url,NodeId> = HashMap::new();

        let (root_node,arena) = self.file_tree.get_prop().unwrap();
        let cloned_arena = arena.clone();
        let test:Option<Url> = None;
       //info!("Root Node {:?}",root_node);
       let mut files = self.files.clone();
       info!("Fileeeeeeeeeeeeeee: {}",url);

        if let Some(exp) = files.get(&url) {
            let mut temp = None;
            for element in root_node.children(arena) {
                
                let node = arena.get(element).unwrap().get();
               
                let compare_str = node.file_information.clone().unwrap().get_url().as_ref().to_string();
                
                if (compare_str.contains(url.clone().as_str())) {
                    info!("Exist");
                    temp = Some(element);
                }
            }

            let node_id = match temp{
                Some(x) => x,
                None  => fun_name1(exp.clone(), url.clone(), map, arena)
            };

            //info!("Add Node 1");

            // Get a shared reference to the file.
            let file = &exp.file;
            //info!("File retrieved {}", file.uri);
            let array_temp= get_all_include_files(file.clone(),root_node,arena,&node_id);
                //let array_temp = add_node_include(file.clone(),root_node,arena,&files);


                let mut nodes_array: Vec<NodeId> = Vec::new();

               // let mut children =None;
               
                if(state == true){
                    fun_name(array_temp, node_id.to_owned(), arena, url.clone(), file.to_owned(), &mut nodes_array);

                }
                root_node.append(node_id, arena);
                //info!("c");
                //info!("Adt Arena {:?}",arena);
            
            

               /* for i in array_temp {
                    //info!("asd32");
                    node_id.append(i, arena);
                    let temp_node = arena.get(i).unwrap().get();
                    let table = temp_node
                        .file
                        .clone()
                        .unwrap()
                        .symbol_table_manager
                        .lock()
                        .unwrap()
                        .clone()
                        .symbol_table;
                    map.insert(temp_node.file.clone().unwrap().uri, table);
                }
                let node_temp = arena.get_mut(node_id.clone()).unwrap().get_mut();

                node_temp.file = Some(File::new(
                    url.clone(),
                    file.source_code.as_str(),
                    &file.tree,
                    Some(map.clone()),
                ));*/
                
                ////info!("RooteL {}",root_node);


               // //info!("Node: {}",node_id);
               // //info!("arena {}",self.file_tree.get_arena());
              
                //info!("c");
                ////info!("Adt Arena {:?}",arena);

               // test.unwrap().has_host();
            
        }

    }

   

}

fn get_all_include_files(file: File,root_node_id: &mut NodeId,arena:&mut Arena<Node>,root_node:&NodeId) ->  Vec<NodeId>{

    let ast_query = file.ast_manager.lock().unwrap();

   
    let includes = ast_query.ast.get_includes();
    info!("Getting Includes {:?}",includes);
    let mut nodes_array: Vec<NodeId> = Vec::new();
    info!("rotNoe {:?}",arena.clone().get(root_node_id.clone()).unwrap().get());

    info!("ssss {:?}",arena.clone().get(root_node.clone()).unwrap().get().file_information.clone().unwrap().get_url().as_str());
    info!("arena {:?}",arena.count());
    let children = root_node_id.descendants(arena)
    .clone()
    .map(|f| {
        let file_info = arena.clone().get(f).unwrap().get().file_information.clone();
        return match file_info{
            Some(x) => x.get_url().to_string(),
            None => String::from("")
        };
        
    })
    .collect::<Vec<_>>();

    info!("Children {:?}",children);
    for include in includes {
        let mut visit_node = None;
        info!("asd9 ");
            for element in root_node_id.children(arena) {
                info!("Node ID{}",element);
                
                let node = arena.get(element).unwrap().get();
               
                let compare_str = node.file_information.clone().unwrap().get_url().as_ref().to_string();
                info!("Child: {} : {} : {}",compare_str,include.as_str(),compare_str.contains(include.as_str()));
                if (compare_str.contains(include.as_str())) {
                    info!("Inside");
                    visit_node = Some(element);
                    match visit_node {
                        Some(visit_node) => {
                            nodes_array.push(visit_node);
                        },
                        None => {}
                    }
                }
            }
    }
    info!("Added To array {}{:?}",file.uri.as_str(),nodes_array);
    nodes_array
}

fn fun_name1(exp: Exp, url: Url, map: HashMap<Url, NodeId>, arena: &mut Arena<Node>) -> NodeId {
    let content = exp.file.source_code.as_str().clone();
    let information = Information::new(url.clone(), tree::ControlState::InControl);

    let tree = exp.file.tree.clone();
    let node = tree::Node::new(
        Some(File::new(url.clone(), content, &tree, Some(map),arena)),
        Some(information.clone()),
    );
    //info!("Root Arena {:?}",arena);

    //info!("Sub  Node {:?}",node);
    let node_id = arena.new_node(node);
    node_id
}

fn fun_name(array_temp: Vec<NodeId>, visit_node: NodeId, arena: &mut Arena<Node>, url: Url, file: File, nodes_array: &mut Vec<NodeId>) {
    let mut map: HashMap<Url,&mut SymbolTable> = HashMap::new();
  
    info!("Array For info  {} {:?} ",visit_node,array_temp);
    for i in array_temp {
        //info!("asd22");
        if(i != visit_node){

            visit_node.append(i.clone(), arena);

        
            let temp_node = arena.get_mut(i.clone()).unwrap().get();
            let t = temp_node.file.as_ref().unwrap();
            let mut tt = t.symbol_table_manager.lock().unwrap();
            map.insert(temp_node.file.clone().unwrap().uri, &mut tt.symbol_table);
            // map.insert(temp_node.file.clone().unwrap().uri, i);
        }
    }
    info!("Data Added To Node {:?}",map);
    let node_temp = arena.get_mut(visit_node.clone()).unwrap().get_mut();
    let file = node_temp.file.as_mut().unwrap();
    let t = file.ast_manager.lock().unwrap();
    let nodeId = t.ast.visit_root();
    file.symbol_table_manager.lock().unwrap().symbol_table.parse_usages(nodeId, url, Some(map));
    nodes_array.push(visit_node);
}



fn get_includes_folder() -> Option<HashMap<Url, ControlState>> {
    let output = if cfg!(target_os = "windows") {
        Command::new("where")
            .arg("p4c")
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("which")
            .arg("p4c")
            .output()
            .expect("failed to execute process")
    };

    let text = "./include"; //str::from_utf8(&output.stderr).expect("failed to convert output to string");
    //info!("Parse :{}", text);
    //let folder_path = Url::parse("./include").unwrap();
    let file_extension = "p4";
    let mut file_map: HashMap<Url, ControlState> = HashMap::new();
    //let t= folder_path.as_str();

    let walker = WalkDir::new(text).into_iter();
    for entry in walker {
        if let Ok(entry) = entry {
            //info!("Running");
            let path = entry.path();

            if path.extension().unwrap_or_default() == file_extension {
                //info!("Stuff {:?}", path.canonicalize().unwrap().display());
                let temp = Url::from_file_path(path.canonicalize().unwrap().as_path());
                file_map.insert(temp.unwrap(), ControlState::NotInControl);
            }
        } else {
            //info!("Error");
        }
    }
    Some(file_map)
}


fn get_workspace_folders() -> Option<HashMap<Url, ControlState>> {
    let output = if cfg!(target_os = "windows") {
        Command::new("where")
            .arg("p4c")
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("which")
            .arg("p4c")
            .output()
            .expect("failed to execute process")
    };

    let text = "./examples"; //str::from_utf8(&output.stderr).expect("failed to convert output to string");
    //info!("Parse :{}", text);
    //let folder_path = Url::parse("./include").unwrap();
    let file_extension = "p4";
    let mut file_map: HashMap<Url, ControlState> = HashMap::new();
    //let t= folder_path.as_str();

    let walker = WalkDir::new(text).into_iter();
    for entry in walker {
        if let Ok(entry) = entry {
            //info!("Running");
            let path = entry.path();

            if path.extension().unwrap_or_default() == file_extension {
                //info!("Stuff {:?}", path.canonicalize().unwrap().display());
                let temp = Url::from_file_path(path.canonicalize().unwrap().as_path());
                file_map.insert(temp.unwrap(), ControlState::InControl);
            }
        } else {
            //info!("Error");
        }
    }
    Some(file_map)
}

