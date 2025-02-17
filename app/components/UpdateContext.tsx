import React, { createContext, useContext, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
// Create the context
const UpdateContext = createContext<{ triggerUpdate: (prevModifiedName: string, addedNodeParent: string) => void; context: {prevModifiedName: string, addedNodeParent: string, updateIdx: number} } | null>(null);

// Provider component
export const UpdateProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [context, setContext] = useState({prevModifiedName: "", addedNodeParent: "", updateIdx: 0});
  const [count, SetCount] = useState(1);
  const triggerUpdate = (prevModifiedName: string, addedNodeParent: string) => {
    invoke("log", {message: "triggerUpdate called with prevModifiedName: " + prevModifiedName + " addedNodeParent: " + addedNodeParent + " updateIdx: " + count});
    const newContext = {prevModifiedName: prevModifiedName, addedNodeParent: addedNodeParent, updateIdx: count};
    SetCount(count + 1);
    setContext(newContext);} // Change state to force re-render
  return <UpdateContext.Provider value={{ triggerUpdate,  context: context }}>{children}</UpdateContext.Provider>;
};

// Hook to get the trigger function
export const useTriggerUpdate = () => {
  const context = useContext(UpdateContext);
  if (!context) throw new Error("useTriggerUpdate must be used within an UpdateProvider");
  return context.triggerUpdate;
};

// Hook to listen for updates
export const useUpdateListener = () => {
  const context = useContext(UpdateContext);
  if (!context) throw new Error("useUpdateListener must be used within an UpdateProvider");
  return context.context; // This will cause re-renders when `tick` updates
};
