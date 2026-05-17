import React, { useState } from 'react';
import { FileData } from '../types';
import { Eye, EyeOff, Folder, File, GripVertical, ChevronRight, ChevronDown, Plus, Trash2, Box, Archive, Image as ImageIcon, Activity } from 'lucide-react';
import clsx from 'clsx';
import { useViewportProfile } from '../hooks/useViewportProfile';
import { CadResponsivePanel } from './cad/CadResponsivePanel';

interface FileManagerProps {
  files: FileData[];
  setFiles: React.Dispatch<React.SetStateAction<FileData[]>>;
  selectedId: string | null;
  setSelectedId: (id: string | null) => void;
  themeMode: 'dark' | 'light';
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function FileManager({ files, setFiles, selectedId, setSelectedId, themeMode, open = true, onOpenChange }: FileManagerProps) {
  const viewport = useViewportProfile();
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  const handleDragStart = (e: React.DragEvent, id: string) => {
    e.dataTransfer.setData('text/plain', id);
    setDraggedId(id);
  };

  const handleDragOver = (e: React.DragEvent, id: string) => {
    e.preventDefault();
    setDragOverId(id);
  };

  const handleDrop = (e: React.DragEvent, targetId: string) => {
    e.preventDefault();
    const sourceId = e.dataTransfer.getData('text/plain');
    setDraggedId(null);
    setDragOverId(null);

    if (sourceId === targetId) return;

    const sourceFile = files.find(f => f.id === sourceId);
    const targetFile = files.find(f => f.id === targetId);

    if (!sourceFile || !targetFile) return;

    // Prevent cycles: check if target is a descendant of source
    let currentTarget: FileData | undefined = targetFile;
    while (currentTarget) {
      if (currentTarget.id === sourceId) {
        // Target is inside source, cannot drop
        return;
      }
      currentTarget = currentTarget.parentId ? files.find(f => f.id === currentTarget!.parentId) : undefined;
    }

    let newFiles = [...files];
    const sourceIndex = newFiles.findIndex(f => f.id === sourceId);
    const targetIndex = newFiles.findIndex(f => f.id === targetId);

    // Remove source
    newFiles.splice(sourceIndex, 1);

    if (targetFile.type === 'GROUP') {
      // Add to group
      sourceFile.parentId = targetId;
      newFiles.push(sourceFile);
    } else {
      // Reorder
      sourceFile.parentId = targetFile.parentId;
      const newTargetIndex = newFiles.findIndex(f => f.id === targetId);
      newFiles.splice(newTargetIndex, 0, sourceFile);
    }

    setFiles(newFiles);
  };

  const createGroup = () => {
    const newGroup: FileData = {
      id: `group-${Date.now()}`,
      name: 'New Group',
      type: 'GROUP',
      visible: true,
      opacity: 1,
      wireframe: false,
      position: [0, 0, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],
      windowCenter: 0,
      windowWidth: 0,
      sliceIndex: 0,
      isExpanded: true,
    };
    setFiles([...files, newGroup]);
  };

  const toggleVisibility = (id: string) => {
    setFiles(files.map(f => {
      if (f.id === id) {
        return { ...f, visible: !f.visible };
      }
      return f;
    }));
  };

  const toggleExpand = (id: string) => {
    setFiles(files.map(f => f.id === id ? { ...f, isExpanded: !f.isExpanded } : f));
  };

  const deleteFile = (id: string) => {
    const getDescendantIds = (parentId: string): string[] => {
      const children = files.filter(f => f.parentId === parentId).map(f => f.id);
      return children.reduce((acc, childId) => [...acc, ...getDescendantIds(childId)], children);
    };

    const idsToDelete = [id, ...getDescendantIds(id)];
    setFiles(files.filter(f => !idsToDelete.includes(f.id)));
    if (selectedId && idsToDelete.includes(selectedId)) setSelectedId(null);
  };

  const getFileIcon = (file: FileData) => {
    if (file.type === 'GROUP') return <Folder size={14} className="text-text-secondary shrink-0" />;
    
    const name = file.name.toLowerCase();
    if (name.endsWith('.stl') || name.endsWith('.obj') || name.endsWith('.ply')) {
      return <Box size={14} className="text-text-display shrink-0" />;
    }
    if (name.endsWith('.dcm') || name.endsWith('.dicom')) {
      return <Activity size={14} className="text-accent shrink-0" />;
    }
    if (name.endsWith('.zip')) {
      return <Archive size={14} className="text-text-secondary shrink-0" />;
    }
    if (name.endsWith('.png') || name.endsWith('.jpg') || name.endsWith('.jpeg')) {
      return <ImageIcon size={14} className="text-text-secondary shrink-0" />;
    }
    return <File size={14} className="text-text-disabled shrink-0" />;
  };

  const renderItem = (file: FileData, level: number = 0) => {
    const isGroup = file.type === 'GROUP';
    const children = files.filter(f => f.parentId === file.id);
    const isSelected = selectedId === file.id;
    const isDraggedOver = dragOverId === file.id;

    return (
      <div key={file.id}>
        <div
          draggable
          onDragStart={(e) => handleDragStart(e, file.id)}
          onDragOver={(e) => handleDragOver(e, file.id)}
          onDrop={(e) => handleDrop(e, file.id)}
          onClick={() => setSelectedId(file.id)}
          className={clsx(
            "flex items-center gap-2 p-2 rounded-lg cursor-pointer transition-all duration-200 text-sm font-mono",
            isSelected 
              ? "bg-text-display text-black" 
              : "hover:bg-surface-raised text-text-primary",
            isDraggedOver && "border border-text-display"
          )}
          style={{ paddingLeft: `${level * 16 + 8}px` }}
        >
          <GripVertical size={14} className={clsx("cursor-grab active:cursor-grabbing shrink-0", isSelected ? "text-black opacity-70" : "text-text-disabled")} />
          
          {isGroup && (
            <button onClick={(e) => { e.stopPropagation(); toggleExpand(file.id); }} className={clsx("p-0.5 rounded shrink-0", isSelected ? "hover:bg-black/10" : "hover:bg-surface-raised")}>
              {file.isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            </button>
          )}

          {getFileIcon(file)}
          
          <span className="flex-1 truncate">{file.name}</span>
          
          <button onClick={(e) => { e.stopPropagation(); toggleVisibility(file.id); }} className={clsx("p-1 rounded shrink-0", isSelected ? "hover:bg-black/10" : "text-text-secondary hover:text-text-primary hover:bg-surface-raised")}>
            {file.visible ? <Eye size={14} /> : <EyeOff size={14} />}
          </button>
          <button onClick={(e) => { e.stopPropagation(); deleteFile(file.id); }} className={clsx("p-1 rounded shrink-0", isSelected ? "hover:bg-red-500/20 hover:text-red-900" : "text-text-secondary hover:text-error hover:bg-error/10")}>
            <Trash2 size={14} />
          </button>
        </div>
        
        {isGroup && file.isExpanded && (
          <div className="flex flex-col gap-1 mt-1">
            {children.map(child => renderItem(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  const rootFiles = files.filter(f => !f.parentId);

  return (
    <CadResponsivePanel
      title="Layers"
      compact={viewport.isCompact}
      open={open}
      onOpenChange={onOpenChange}
      desktopClassName={clsx(
        viewport.isTablet
          ? 'left-4 top-24 bottom-24 w-60'
          : selectedId
            ? 'left-6 top-24 bottom-40 w-72'
            : 'left-6 top-24 bottom-24 w-72'
      )}
      contentClassName="flex h-full flex-col gap-2 p-2"
      headerAction={
        <button onClick={createGroup} className="rounded-lg p-1.5 text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary" title="New Group" aria-label="Crear grupo">
          <Plus size={16} />
        </button>
      }
    >
      {rootFiles.length === 0 ? (
        <div className="rounded-lg border border-dashed border-border p-4 text-center text-[11px] font-mono uppercase tracking-widest text-text-disabled">
          No files loaded
        </div>
      ) : (
        rootFiles.map(f => renderItem(f))
      )}

      <div 
        className="mt-2 h-8 rounded-lg border border-dashed border-transparent transition-colors hover:border-border-visible"
        onDragOver={(e) => { e.preventDefault(); setDragOverId('root'); }}
        onDrop={(e) => {
          e.preventDefault();
          const sourceId = e.dataTransfer.getData('text/plain');
          setDraggedId(null);
          setDragOverId(null);
          setFiles(files.map(f => f.id === sourceId ? { ...f, parentId: undefined } : f));
        }}
      />
    </CadResponsivePanel>
  );
}
