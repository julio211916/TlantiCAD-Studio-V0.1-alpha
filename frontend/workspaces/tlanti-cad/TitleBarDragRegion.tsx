const isWindows = navigator.userAgent.includes('Windows');

export function TitleBarDragRegion() {
  if (isWindows) {
    return null;
  }

  return <div data-tauri-drag-region className="fixed left-0 right-0 top-0 z-[9999] h-10" />;
}
