// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs::{self, ReadDir};
use std::process::exit;
use std::sync::Mutex;
// from files
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct FileModel {
    name: String,
    algorithm: Option<String>,
    children: Option<Vec<String>>,
}

// the actual representation in the backend
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Model{
    name: String,
    algorithm: Option<String>,
    children: Option<Vec<String>>,
    ref_count: u64,
}

// API
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Node {
    name: String,
    ref_count: u64,
    has_children: bool
}

#[derive(Clone)]
struct TauriState {
    models: HashMap<String, Model>,
    root_name: String,
}

// rust side keep track of instances ...
// What's the purpose of ...
// cache?
// invalidate ...
// each item has a hidden id

// have a context with id -> function pair
// all the children are queried
fn main() {
    println!("Current Directory: {:?}", std::env::current_dir().unwrap());
    let models_dir = "../models_test".to_string();
    let models = match load_models(models_dir) {
        Ok(models) => models,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };
    let root_name = "A".to_string();
    models.iter().for_each(|(name, model)| {
        println!("模型{}：算法: {:?}，子节点: {:?}，引用计数: {}", name, model.algorithm, model.children, model.ref_count);
    });
    let tauri_state = Mutex::new(TauriState { models, root_name });
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            update_node_name,
            add_node,
            delete_node,
            query_root_name,
            query_node,
            query_children,
            query_algorithm,
            query_ref_count,
            toggle_has_children,
            update_algorithm,
            log
        ])
        .manage(tauri_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_models(dir: String) -> Result<HashMap<String, Model>> {
    let mut file_models = fs::read_dir(dir.clone())
        .context(format!("未找到模型路径：{}", dir))?
        .map(|entry| entry.context("路径入口错误")) // Get Result<Result<DirEntry>>
        .collect::<Result<Vec<_>, _>>()? // short circuit error handling
        .iter()
        .map(|entry| entry.path()) // Get file paths
        .filter(|path| path.is_file()) // Keep only files
        .map(|path| {
            let content = fs::read_to_string(&path)
                .context(format!("读取模型文件{:?}错误", path.file_name()))?;
            let model = serde_json::from_str::<FileModel>(&content).context("解析模型文件错误")?;
            // 目前模型文件中一定有children和algorithm字段，无children的模型不另起文件单独存放
            if model.children.is_none(){
                Err(anyhow::anyhow!("模型文件{:?}中未找到children字段", path.file_name()))?;
            }
            if model.algorithm.is_none(){
                Err(anyhow::anyhow!("模型文件{:?}中未找到algorithm字段", path.file_name()))?;
            }
            Ok((model.name.clone(), model))
        }) // Read file content
        .collect::<Result<HashMap<_, _>>>()?; // short circuit error handling
    // 遍历所有模型及其children，将所有名字放入集合中
    let mut names = HashSet::new();
    file_models.iter().for_each(|(_name, model)| {
        names.insert(model.name.clone());
        if let Some(children) = &model.children {
            children.iter().for_each(|child| {
                names.insert(child.clone());
            });
        }
    });
    // 在原有模型集合的基础上加入叶节点模型
    names.iter().for_each(|name| {
        if !file_models.contains_key(name) {
            file_models.insert(name.clone(), FileModel{name: name.clone(), algorithm: None, children: None});
        }
    });
    let mut ref_counts = file_models.iter().map::<(String, u64), _>(|(name, _model)| {
        (name.to_string(), 0)
    }).collect::<HashMap<String, u64>>();
    file_models.iter().for_each(|(_name, model)| {
        if let Some(children) = &model.children {
            children.iter().for_each(|child| {
                let count = ref_counts.get(child).expect(format!("child {} does not exist in ref count", child).as_str());
                ref_counts.insert(child.clone(), count + 1);
            });
        }
    });
    let result = file_models.into_iter().map::<(String, Model),_>(|(name, model)| {
        (name.clone(), Model{name: model.name, algorithm: model.algorithm, children: model.children, ref_count: ref_counts.get(&name).expect("model does not exist in ref count").clone()})
    }).collect();
    Ok(result)
}

// program logic:
// 1. load all model files from a specified folder into a hashmap, with root as a special element
// 2. start tauri app
// 3. handle request_root from render side
// 4. render side will request children when an element is expanded
// 5. for each child, need to specify its name and whether it has a child

#[derive(serde::Serialize, serde::Deserialize)]
struct UpdateNameResponse{
    new_name: String,
    requires_update: bool
}

fn update_dup_name_no_children_backend(old_name: &str, new_name: &str, state: &mut TauriState){
    // this function is only called when the new name is duplicated, and the model does not have children
    // the model should snap to the one that originally has this new name
    // iterate through all the models and replace the children with the new name
    replace_old_name_no_children(old_name, new_name, &mut state.models);
    // update the reference count
    update_reference_count(&mut state.models);
}

fn suggest_new_name_dupe(new_name: &str, models: &HashMap<String, Model>) -> String{
    let mut new_name = new_name.to_string();
    while models.get(&new_name).is_some(){
        new_name = format!("{}（错误：重名）", new_name);
    }
    new_name
}
fn suggest_new_name_add(models: &HashMap<String, Model>) -> String{
    let mut new_name = "新节点".to_string();
    let mut i = 0;
    while models.get(&new_name).is_some(){
        i += 1;
        new_name = format!("新节点{}", i);
    }
    new_name
}

fn update_dup_name_has_children_backend(old_name: &str, new_processed_name: &str, state: &mut TauriState){
    // this function is called when the new name is duplicated, and the model has children
    // the model will not snap to any existing node because the new name is supposed to be different from any existing ...
    replace_old_name_has_children(old_name, new_processed_name, &mut state.models);
    // reference count should not change in this case
}

fn update_non_dup_name_backend(old_name: &str, new_name: &str, state: &mut TauriState){
    // the logic should be the same as dup_name_has_children
    update_dup_name_has_children_backend(old_name, new_name, state);
}

fn replace_old_name_has_children(old_name: &str, new_processed_name: &str, models: &mut HashMap<String, Model>){
    // the new processed name should be different from any existing names
    assert!(models.get(new_processed_name).is_none(), "model {} already exists", new_processed_name);
    // get the model corresponding to the old name
    println!("模型{}被移除", old_name);
    let mut model = models.remove(old_name).expect(format!("model {} does not exist", old_name).as_str());
    model.name = new_processed_name.to_string();
    println!("模型{}被加入", new_processed_name);
    models.insert(new_processed_name.to_string(), model);
    // modify children to have the new name
    replace_old_name_no_children(old_name, new_processed_name, models);
}

fn replace_old_name_no_children(old_name: &str, new_name: &str, models: &mut HashMap<String, Model>){
    models.iter_mut().for_each(|(_name, model)|{
        model.children.iter_mut().for_each(|children|{
            children.iter_mut().for_each(|child|{
                if child == old_name{
                    *child = new_name.to_string();
                }
            });
        });
    });
}
fn update_reference_count(models: &mut HashMap<String, Model>){
    let mut ref_counts = models.iter().map::<(String, u64), _>(|(name, _model)| {
        (name.to_string(), 0)
    }).collect::<HashMap<String, u64>>();
    models.iter().for_each(|(_name, model)| {
        if let Some(children) = &model.children {
            children.iter().for_each(|child| {
                let count = ref_counts.get(child).expect(format!("child {} does not exist in ref count", child).as_str());
                ref_counts.insert(child.clone(), count + 1);
            });
        }
    });
    models.iter_mut().for_each(|(name, model)|{
        model.ref_count = ref_counts.get(name).expect(format!("model {} does not exist in ref count", name).as_str()).clone();
    });
}


#[tauri::command]
fn update_node_name(name: &str, new_name: &str, state: tauri::State<Mutex<TauriState>>) -> UpdateNameResponse {
    println!("update_node called with current name: {} and new name: {}", name, new_name);
    // 1. if the new node name is not duplicated, then simply apply (no trigger update, the same modified name)
      // 2. if the node name is duplicated, then check:
      // if the node itself does not have children, then accept the change, update reference count, and: (reference count: needs to be updated)
      //    if the nodes with the same name have children, then add all the children to the renamed node (updated)
      //    if the nodes with the same name do not have children, do nothing (reference count updated)
      // if the node has children, then rename the node to something else (different modified name, no update)
    let mut state = state.lock().unwrap();
    if name == new_name{
        return UpdateNameResponse{new_name: new_name.to_string(), requires_update: false};
    }
    // check for duplicate names
    match state.models.get(new_name){
        Some(_) =>{
            // new name is duplicated with old names
            match state.models.get(name){
                Some(model)=>{
                    // old name exists
                    match &model.children{
                        Some(_) => {
                            let new_processed_name = suggest_new_name_dupe(new_name, &state.models);
                            println!("新名称重名，模型{}有子节点，重命名为\"{}\"，更新所有节点", name, new_processed_name);
                            // 虽然局部看起来不需要更新，但是可能有其他父节点有这个节点，所以需要更新
                            update_dup_name_has_children_backend(name, &new_processed_name, &mut state);
                            UpdateNameResponse{new_name: new_processed_name, requires_update: true}
                        }
                        None => {
                            println!("新名称重名，模型{}无子节点，重命名为\"{}\"，更新所有节点", name, new_name);
                            // 后端搜索所有节点，将原名为name的节点重命名为new_name，更新reference count
                            update_dup_name_no_children_backend(name, new_name, &mut state);
                            UpdateNameResponse{new_name: new_name.to_string(), requires_update: true}
                        }
                    }
                },
                None =>{
                    eprintln!("前后端失去同步：未找到原名为\"{}\"的模型", name);
                    exit(1);
                }
            }
        },
        None =>{
            println!("模型{}重命名为\"{}\"，更新所有节点", name, new_name);
            // 虽然局部看起来不需要更新，但是可能有其他父节点有这个节点，所以需要更新
            update_non_dup_name_backend(name, new_name, &mut state);
            UpdateNameResponse{new_name: new_name.to_string(), requires_update: true}
        }
    }
}

fn add_node_to_parent(parent_name: &str, new_name: &str, models: &mut HashMap<String, Model>){
    // asser new name does not exist in models
    assert!(models.get(new_name).is_none());
    // add new name to models with no children or algorithm
    models.insert(new_name.to_string(), Model{name: new_name.to_string(), algorithm: None, children: None, ref_count: 0});
    let parent = models.get_mut(parent_name).expect(format!("parent {} does not exist", parent_name).as_str());
    let children = parent.children.as_mut().expect(format!("parent {} does not have children", parent_name).as_str());
    children.push(new_name.to_string());
    // update reference counts
    update_reference_count(models);
}

#[tauri::command]
fn add_node(parent_name: &str, state: tauri::State<Mutex<TauriState>>) -> String {
    let mut state = state.lock().unwrap();
    println!("Rust: add_node called with parent_name: {}", parent_name);
    let new_name = suggest_new_name_add(&mut state.models);
    add_node_to_parent(parent_name, &new_name, &mut state.models);
    new_name
}

fn remove_node_from_parent(parent_name: &str, name: &str, models: &mut HashMap<String, Model>){
    // remove the node from the models
    let model = models.get(name).expect(format!("model {} does not exist", name).as_str());
    if model.ref_count == 1{
        models.remove(name).expect(format!("model {} does not exist", name).as_str());
    }
    // remove the node from the parent
    let parent = models.get_mut(parent_name).expect(format!("parent {} does not exist", parent_name).as_str());
    let children = parent.children.as_mut().expect(format!("parent {} does not have children", parent_name).as_str());
    children.retain(|child| child != name);
    // update reference counts
    update_reference_count(models);
}

#[tauri::command]
fn delete_node(parent_name: &str, name: &str, state: tauri::State<Mutex<TauriState>>) {
    let mut state = state.lock().unwrap();
    println!("delete_node called with name: {}", name);
    // this is tricky because we should only delete the node inside its parent. If it is referenced by other nodes, we should not remove it entirely from the models
    // if its reference count is 1, then we can remove it entirely
    // if its reference count is more than 1, then we should only remove it from its parent
    remove_node_from_parent(parent_name, name, &mut state.models);
}

#[tauri::command]
fn query_root_name(state: tauri::State<Mutex<TauriState>>) -> String {
    let state = state.lock().unwrap();
    state.root_name.clone()
}

#[tauri::command]
fn query_node(name: &str, state: tauri::State<Mutex<TauriState>>) -> Node {
    println!("Rust: query_node called with name: {}", name);
    let state = state.lock().unwrap();
    let model = match state.models.get(name) {
        Some(model) => model,
        None => {
            eprintln!("query node 错误：未找到模型{}", name);
            exit(1);
        }
    };
    let has_children = model.children.is_some();
    let has_algorithm = model.algorithm.is_some();
    if has_children != has_algorithm {
        eprintln!("错误：模型{}状态冲突：{}子节点但{}算法声明", name, if has_children { "有" } else { "无" }, if has_algorithm { "有" } else { "无" });
        exit(1);
    }
    Node {
        name: model.name.clone(),
        ref_count: 0,
        has_children,
    }    
}
#[tauri::command]
fn query_children(parent_name: &str, state: tauri::State<Mutex<TauriState>>) -> Vec<String> {
    println!("Rust: query_children called with parent_name: {}", parent_name);
    let state = state.lock().unwrap();
    let model = match state.models.get(parent_name) {
        Some(model) => model,
        None => {
            eprintln!("query children 错误：未找到模型{}", parent_name);
            exit(1);
        }
    };
    match &model.children {
        Some(children) => children.clone(),
        None => {
            eprintln!("错误：模型{}无子节点", parent_name);
            exit(1);
        }
    }
}
#[tauri::command]
fn query_algorithm(parent_name: &str, state: tauri::State<Mutex<TauriState>>) -> String {
    println!("Rust: query_algorithm called with parent_name: {}", parent_name);
    let state = state.lock().unwrap();
    let model = match state.models.get(parent_name) {
        Some(model) => model,
        None => {
            eprintln!("query algorithm 错误：未找到模型{}", parent_name);
            exit(1);
        }
    };
    match &model.algorithm {
        Some(algorithm) => algorithm.clone(),
        None => {
            eprintln!("错误：模型{}无算法", parent_name);
            exit(1);
        }
    }
}

#[tauri::command]
fn query_ref_count(name: &str, state: tauri::State<Mutex<TauriState>>) -> u64 {
    println!("Rust: query_ref_count called with name: {}", name);
    let state = state.lock().unwrap();
    match state.models.get(name) {
        Some(model) => model.ref_count,
        None => {
            eprintln!("ref count 警告：可能被丢弃的模型{}", name);
            0
        }
    }
}
#[tauri::command]
fn toggle_has_children(name: &str, state: tauri::State<Mutex<TauriState>>) {
    let mut state = state.lock().unwrap();
    let model = state.models.get_mut(name).expect(format!("model {} does not exist", name).as_str());
    match model.children{
        Some(_)=>{
            assert!(model.algorithm.is_some());
            model.children = None;
            model.algorithm = None;
        }
        None=>{
            model.children = Some(vec![]);
            model.algorithm = Some("未定义算法".to_string());
        }
    }
}
#[tauri::command]
fn update_algorithm(name: &str, algorithm: &str, state: tauri::State<Mutex<TauriState>>) {
    let mut state = state.lock().unwrap();
    let model = state.models.get_mut(name).expect(format!("model {} does not exist", name).as_str());
    model.algorithm = Some(algorithm.to_string());
}

#[tauri::command]
fn log(message: String){
    println!("{}", message);
}