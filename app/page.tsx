"use client";

import { ReactNode, useEffect, useState } from "react";
import {TreeNode, get_node} from "./components/TreeNode";
import { invoke } from "@tauri-apps/api/tauri";
import { UpdateProvider } from "./components/UpdateContext";



export default function Page() {
  const [rootNode, setRootNode] = useState<ReactNode | null>(null);
  useEffect(() => {
    const get_root_node = async () => {
      let root_name = await invoke<string>("query_root_name");
      setRootNode(await get_node(root_name));
    }
    get_root_node();
  }, [])
  return (
    <div className="p-4">
      <h1 className="text-xl font-bold mb-4">Tree View</h1>
      <UpdateProvider>
        {rootNode}
      </UpdateProvider>
    </div>
  );
}
