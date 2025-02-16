// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs::{self, ReadDir};
use std::process::exit;
// from files
#[derive(serde::Serialize, serde::Deserialize)]
struct FileModel {
    name: String,
    algorithm: Option<String>,
    children: Option<Vec<String>>,
}

// the actual representation in the backend
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Model{
    name: String,
    algorithm: Option<String>,
    children: Option<Vec<String>>,
    ref_count: u64,
}

// API
#[derive(serde::Serialize, serde::Deserialize)]
struct Node {
    name: String,
    ref_count: u64,
    has_children: bool
}

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
    let models_dir = "../models".to_string();
    let models = match load_models(models_dir) {
        Ok(models) => models,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };
    let root_name = "健康指数".to_string();
    models.iter().for_each(|(name, model)| {
        println!("模型{}：算法: {:?}，子节点: {:?}，引用计数: {}", name, model.algorithm, model.children, model.ref_count);
    });
    let tauri_state = TauriState { models, root_name };
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            update_node,
            add_node,
            delete_node,
            query_root_name,
            query_node,
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

#[tauri::command]
fn update_node(id: i64, name: &str) {
    println!("update_node called with ID: {} and Name: {}", id, name);
}

#[tauri::command]
fn add_node(parent_id: i64) {
    println!("add_node called with Parent ID: {}", parent_id);
}

#[tauri::command]
fn delete_node(id: i64) {
    println!("delete_node called with ID: {}", id);
}

#[tauri::command]
fn query_root_name(state: tauri::State<TauriState>) -> String {
    state.root_name.clone()
}

#[tauri::command]
fn query_node(name: String, state: tauri::State<TauriState>) -> Node {
    let model = match state.models.get(&name) {
        Some(model) => model,
        None => {
            eprintln!("错误：未找到模型{}", name);
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