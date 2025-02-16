"use client";

import { useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { motion } from "framer-motion";
import { ChevronRight, ChevronDown, Folder, FileText, Plus, Trash2 } from "lucide-react";

interface TreeNodeProps {
  id: number;
  name: string;
  isParent: boolean;
  children?: any[];
  // refreshTree: () => void; // Function to re-fetch data from backend
  
}

export const TreeNode = ({ id, name, isParent, children = [] }: TreeNodeProps) => {
  const [expanded, setExpanded] = useState(false);
  const [editing, setEditing] = useState(false);
  const [newName, setNewName] = useState(name);

  // 🔄 Update Node Name
  const updateNodeName = async () => {
    if (newName.trim() !== name) {
      await invoke("update_node", { id: id, name: newName });
      // refreshTree();
    }
    setEditing(false);
  };

  // ➕ Add New Item
  const addNewItem = async () => {
    await invoke("add_node", { parentId: id });
    // refreshTree();
    setExpanded(true);
  };

  // 🗑 Delete Node
  const deleteNode = async () => {
    await invoke("delete_node", { id });
    // refreshTree();
  };

  return (
    <div className="pl-4">
      {/* Node Header */}
      <div className="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
        {/* Expand/Collapse Button for Parent Nodes */}
        {isParent ? (
          <div onClick={() => setExpanded(!expanded)}>
            {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
          </div>
        ) : (
          <FileText size={16} />
        )}

        {/* Folder or File Icon */}
        {isParent ? <Folder size={16} className="text-yellow-500" /> : null}

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

        {/* Delete Button (only for non-intrinsic items) */}
        {!["Category:", "Add"].includes(name) && (
          <button onClick={deleteNode} className="ml-auto text-red-500 hover:text-red-700">
            <Trash2 size={16} />
          </button>
        )}
      </div>

      {/* Children Nodes (if expanded) */}
      {isParent && expanded && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          transition={{ duration: 0.3 }}
          className="pl-4 border-l border-gray-300"
        >
          {/* Special Category Item (Always First) */}
          <TreeNode id={10000} name="Category:" isParent={false}/>

          {/* Actual Children */}
          {children.map((child) => (
            <TreeNode key={child.id} {...child}/>
          ))}

          {/* Special Add Button (Always Last) */}
          <div className="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
            <Plus size={16} />
            <button onClick={addNewItem} className="text-blue-500 hover:text-blue-700">
              Add
            </button>
          </div>
        </motion.div>
      )}
    </div>
  );
};
