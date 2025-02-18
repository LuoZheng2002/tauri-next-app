import { ReactNode, useEffect, useRef, useState } from "react";
import { UpdateProvider } from "./components/UpdateContext";
import { invoke } from "@tauri-apps/api/tauri";
import{TreeNode, get_node} from "./components/TreeNode";


export const TreePage = () =>{
    const [rootNode, setRootNode] = useState<ReactNode | null>(null);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const handleButtonClick = () => {
      fileInputRef.current?.click();
    };
    useEffect(() => {
        invoke("log", {message: "TreePage mounted"});
        const get_root_node = async () => {
          let root_name = await invoke<string>("query_root_name");
          setRootNode(await get_node(root_name, null));
        }
        get_root_node();
        handleButtonClick();
      }, [])
    return (
        <div className="p-4">
          
          <div className="inline-block">
            <button className="mx-3 mb-2 px-4 py-2 bg-blue-600 text-white font-semibold rounded-2xl shadow-md hover:bg-blue-700 transition-all duration-200 ease-in-out active:scale-95">Save</button>
            <button className="mx-3 px-4 py-2 bg-blue-600 text-white font-semibold rounded-2xl shadow-md hover:bg-blue-700 transition-all duration-200 ease-in-out active:scale-95">Back</button>
          </div>
          <h1 className="text-xl font-bold mb-4">文件：</h1>
          {rootNode}
        </div>
      );
}