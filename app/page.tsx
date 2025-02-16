"use client";

import { useState } from "react";
import {TreeNode} from "./components/TreeNode";

const get_shared_state = ()=>{
  
}

export default function Page() {
  // Hardcoded initial data
  const [treeData, setTreeData] = useState({
    id: 1,
    name: "Root Node",
    isParent: true,
    children: [
      { id: 2, name: "Child 1", isParent: false },
      { id: 3, name: "Child 2", isParent: true, children: [] },
    ],
  });

  return (
    <div className="p-4">
      <h1 className="text-xl font-bold mb-4">Tree View (Hardcoded)</h1>
      {treeData && <TreeNode {...treeData} />}
    </div>
  );
}
