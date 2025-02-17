"use client";

import { ReactNode, useEffect, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { motion } from "framer-motion";
import { ChevronRight, ChevronDown, Folder, FileText, Plus, Trash2, CircuitBoard, Dot } from "lucide-react";
import { useTriggerUpdate, useUpdateListener } from "./UpdateContext";

let index = 0;

function generateIndex() {
  return index++;
}


interface TreeNodeProps {
  id: number;
  name: string;
  hasChildren: boolean;
  parent: string | null;
  // refreshTree: () => void; // Function to re-fetch data from backend
}

export const get_node = async (name: string, parent: string): Promise<ReactNode> => {
  await invoke("log", {message: "getting node: " + name});
  return invoke<any>("query_node", { name: name }).then((node) => {
    const id = generateIndex();
    // invoke("log", {message: "generated id: " + id});
    return <TreeNode key={id} id={id} name={node.name} hasChildren={node.has_children} parent={parent}/>;
  });
}

export const TreeNode = ({ id, name, hasChildren, parent }: TreeNodeProps) => {
  invoke("log", {message: "Next: 生成新的节点：" + name});
  const [children, setChildren] = useState<ReactNode[]>([]);
  const [childrenNames, setChildrenNames] = useState<string[]>([]);
  const [algorithm, setAlgorithm] = useState("加载中");
  const [expanded, setExpanded] = useState(false);
  const [prevExpanded, setPrevExpanded] = useState(expanded);
  const [editing, setEditing] = useState(false);
  const [algoEditing, setAlgoEditing] = useState(false);
  const [prevName, setPrevName] = useState(name);
  const [newName, setNewName] = useState(name);
  const [refCount, setRefCount] = useState(0);

  const triggerUpdate = useTriggerUpdate();
  const context = useUpdateListener();

  useEffect(() => {
    invoke("log", {message: "useEffect 被调用，名字：" + newName});
    const fetchChildren = async () => {
      // await invoke("log", {message: "querying children for " + newName});
      const response = await invoke<string[]>("query_children", { parentName: newName });
      const get_children = async () =>{
        let children: ReactNode[] = [];
        for (let i = 0; i < response.length; i++){
          children.push(await get_node(response[i], newName));
        }
        return children;
      }
      invoke("log", {message: "children names of " + newName + ": " + response});
      setChildren(await get_children());
      setChildrenNames(response);
      return ()=>{
        invoke("log", {message: "Next: 删除节点：" + newName});
      }
    }
    const fetchAlgorithm = async () =>{
      const response = await invoke<string>("query_algorithm", { parentName: newName });
      setAlgorithm(response);
    }
    const fetchRefCount = async () =>{
      const response = await invoke<number>("query_ref_count", { name: newName });
      setRefCount(response);
    }
    // invoke("log", {message: "children names of " + newName + ": " + childrenNames});
    invoke("log", {message: "prevModifiedName: " + context.prevModifiedName});
    fetchRefCount();


    // fetchRefCount();
    if (expanded && (!prevExpanded || newName == context.addedNodeParent || childrenNames.includes(context.prevModifiedName))){
      // assert(hasChildren);
      invoke("log", {message: "fetching children for " + newName});
      fetchChildren();
      fetchAlgorithm();
    }
    else if (!expanded){

    }
    setPrevExpanded(expanded);
  }, [expanded, context]);

  // 🔄 Update Node Name
  const updateNodeName = async () => {
    invoke("log", {message: "prevName: " + prevName + " newName: " + newName});
    if (newName.trim() !== prevName.trim()) {
      let response = await invoke<any>("update_node_name", {name: prevName, newName: newName });
      invoke("log", {message: "Next: " + prevName + " renamed to " + response.new_name});
      setNewName(response.new_name);
      if (response.requires_update){
        invoke("log", {message: "----------更新被触发了：" + prevName});
        triggerUpdate(prevName, "");
      }
      // the response is composed of two parts: the actual modified name and the actions to be taken ...?

      // 

      // 1. if the node name is not duplicated, then simply apply (no trigger update, the same modified name)
      // 2. if the node name is duplicated, then check:
      // if the node itself does not have children, then accept the change, update reference count, and: (reference count: needs to be updated)
      //    if the nodes with the same name have children, then add all the children to the renamed node (updated)
      //    if the nodes with the same name do not have children, do nothing (reference count updated)
      // if the node has children, then rename the node to something else (different modified name, no update)
      // refreshTree();
    }
    setEditing(false);
    setPrevName(newName);
    invoke("log", {message: "prevName now set to: " + newName});
  };

  // ➕ Add New Item
  const addNewItem = async () => {
    const newChildName = await invoke("add_node", { parentName: newName });
    invoke("log", {message: "Next: 尝试在"+newName +"中添加新的节点：" + newChildName});
    invoke("log", {message: "----------更新因为Add被触发了：" + prevName});
    triggerUpdate("", newName);
    // refreshTree();
    // setExpanded(true);
  };
  const toggleHasChildren = async () => {
    invoke("toggle_has_children", { name: newName });
    triggerUpdate(newName, "");
  }
  const updateAlgorithm = async() => {
    invoke("update_algorithm", { name: newName, algorithm: algorithm });
    triggerUpdate("", newName);
    setAlgoEditing(false);
  }

  // 🗑 Delete Node
  const deleteNode = async () => {
    await invoke("delete_node", { parentName: parent, name: newName });
    triggerUpdate(newName, "");
    // refreshTree();
  };

  return (
    <div>
      {/* Node Header */}
      <div className="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
        {/* Expand/Collapse Button for Parent Nodes */}
        {hasChildren ? (
          <div onClick={() => setExpanded(!expanded)}>
            {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
          </div>
        ) : (
          <div className="w-4 h-4 inline-block" />
        )}

        {/* Folder or File Icon */}
        {hasChildren ? <Folder size={16} className="text-yellow-500" /> : <FileText size={16} className="text-yellow-500" />}

        {/* Editable Name */}
        {editing ? (
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onBlur={updateNodeName}
            onKeyDown={(e) => e.key === "Enter" && updateNodeName()}
            autoFocus
            className="border px-1 rounded"
          />
        ) : (
          <span onDoubleClick={() => setEditing(true)}>{newName}</span>
        )}

        {/* Delete Button */}
        {
          <div className="ml-auto">
            <button className="text-blue-500 hover:text-blue-700 mr-3" onClick={toggleHasChildren} >
              {hasChildren ? "删除子项" : "启用子项"}
            </button>
            <div className="inline-block mr-3">引用计数：{refCount}</div>
            <button className="text-red-500 hover:text-red-700" onClick={deleteNode} >
              <Trash2 size={16} />
            </button>
          </div>
          
        }
      </div>

      {/* Children Nodes (if expanded) */}
      {hasChildren && expanded && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          transition={{ duration: 0.3 }}
          className="pl-4 border-l border-gray-300"
        >
          {/* Special Category Item (Always First) */}
          <div className="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md" key="algorithm">
            <div className="w-4 h-4 inline-block" />
            <CircuitBoard size={16}/>
            <div>算法：</div>
            {/* Editable Name */}
            {algoEditing ? (
              <input
                type="text"
                value={algorithm}
                onChange={(e) => setAlgorithm(e.target.value)}
                onBlur={updateAlgorithm}
                onKeyDown={(e) => e.key === "Enter" && updateAlgorithm()}
                autoFocus
                className="border px-1 rounded"
              />
            ) : (
              <span onDoubleClick={() => setAlgoEditing(true)}>{algorithm}</span>
            )}
          </div>
          {/* Actual Children */}
          {children}
          {/* Special Add Button (Always Last) */}
          <div  key="add" className="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
            <div className="w-4 h-4 inline-block" />
            <button onClick={addNewItem} className="text-blue-500 hover:text-blue-700">
              添加
            </button>
          </div>
        </motion.div>
      )}
    </div>
  );
};

