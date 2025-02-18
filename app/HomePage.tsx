import { useEffect, useRef, useState } from "react"


export const HomePage = () => {
    const [selectedFile, setSelectedFile] = useState<File | null>(null);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        if (event.target.files && event.target.files.length > 0) {
          setSelectedFile(event.target.files[0]);
        }
      };
    useEffect(() => {
    }, []);
    return(
        <div>
            <input type="file" onChange={handleFileChange} ref={fileInputRef} className="hidden" />
            <button onClick={() => fileInputRef.current?.click()}></button>
        </div>
    );
}