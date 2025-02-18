"use client";

import { ReactNode, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { UpdateProvider } from "./components/UpdateContext";
import { TreePage } from "./TreePage";



export default function Page() {
  const [page, setPage] = useState<ReactNode | null>(null);
  useEffect(() => {
    setPage(<TreePage />);
  }, []);
  return (
    <UpdateProvider>
      {page}
    </UpdateProvider>
  );
}
