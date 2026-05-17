import React, { useEffect, useState, useRef } from 'react';
import { DicomWorkspaceView, FileData, ThemeMode } from '../types';
import { ChevronUp, ChevronDown, Play, Pause, SlidersHorizontal, RotateCcw, RotateCw, Crop, CheckCircle2, Activity, Box, FileText, Layers3 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useViewportProfile } from '../hooks/useViewportProfile';
import { Button } from '@/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';

interface DicomControlsProps {
  file: FileData;
  onUpdate: (id: string, updates: Partial<FileData>) => void;
  themeMode: ThemeMode;
  onStartPlanning?: () => void;
}

const DICOM_VIEWS: Array<{ value: DicomWorkspaceView; label: string; icon: React.ComponentType<{ size?: number; className?: string }> }> = [
  { value: 'review', label: 'Review', icon: Layers3 },
  { value: 'volume', label: 'Volume', icon: Box },
  { value: 'ai', label: 'AI', icon: Activity },
  { value: 'report', label: 'Report', icon: FileText },
];

const defaultAdjustments = {
  rotation: 0,
  crop: { top: 0, right: 0, bottom: 0, left: 0 },
};

export const DicomControls: React.FC<DicomControlsProps> = ({ file, onUpdate, themeMode, onStartPlanning }) => {
  const viewport = useViewportProfile();
  const [playing, setPlaying] = useState(false);
  const [adjustOpen, setAdjustOpen] = useState(false);
  const playInterval = useRef<NodeJS.Timeout | null>(null);

  const totalSlices = file.buffers?.length || 1;
  const currentIndex = file.sliceIndex || 0;
  const adjustments = file.dicomAdjustments ?? defaultAdjustments;
  const activeView = file.dicomWorkspaceView ?? 'review';

  const fileRef = useRef(file);
  fileRef.current = file;

  useEffect(() => {
    if (playing) {
      playInterval.current = setInterval(() => {
        const currentFile = fileRef.current;
        const total = currentFile.buffers?.length || 1;
        const nextIndex = ((currentFile.sliceIndex || 0) + 1) % total;

        onUpdate(currentFile.id, {
          sliceIndex: nextIndex
        });
      }, 100); // 10 FPS
    } else {
      if (playInterval.current) clearInterval(playInterval.current);
    }
    return () => {
      if (playInterval.current) clearInterval(playInterval.current);
    };
  }, [playing, onUpdate]); // Removed file.sliceIndex and totalSlices dependencies

  const handleSliceChange = (delta: number) => {
    const newIndex = Math.min(Math.max(currentIndex + delta, 0), totalSlices - 1);
    onUpdate(file.id, { sliceIndex: newIndex });
  };

  const handleSliderChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onUpdate(file.id, { sliceIndex: parseInt(e.target.value) });
  };

  const handleWheel = (e: React.WheelEvent) => {
    e.stopPropagation();
    // Scroll Down (positive delta) -> Next Slice (+1)
    // Scroll Up (negative delta) -> Previous Slice (-1)
    const delta = e.deltaY > 0 ? 1 : -1;
    handleSliceChange(delta);
  };

  return (
    <div
      onWheel={handleWheel}
      className={cn(
        'pointer-events-none absolute inset-x-4 bottom-4 z-30 md:inset-x-auto md:left-6',
        viewport.isCompact ? 'right-4' : 'sm:right-[17rem] md:right-[21rem]'
      )}
    >
      <div className={cn(
        'pointer-events-auto rounded-[1.35rem] border px-3 py-3 shadow-xl backdrop-blur-xl',
        viewport.isCompact ? 'w-full' : 'w-[min(48rem,calc(100vw-28rem))]',
        themeMode === 'dark' ? 'border-white/10 bg-black/72 text-white' : 'border-black/10 bg-white/75 text-black',
      )}>
        <div className="flex flex-wrap items-center gap-3">
          <Tabs value={activeView} onValueChange={(value) => onUpdate(file.id, { dicomWorkspaceView: value as DicomWorkspaceView })}>
            <TabsList className="w-auto bg-transparent border-none p-0">
              {DICOM_VIEWS.map(({ value, label, icon: Icon }) => (
                <TabsTrigger key={value} value={value} className="min-w-0 gap-2 rounded-xl border border-border-visible bg-surface px-3 py-2 text-[11px] uppercase text-text-secondary data-[state=active]:border-[#FA93FA] data-[state=active]:bg-[#FA93FA] data-[state=active]:text-black">
                  <Icon size={13} />
                  {label}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>

          <Button
            type="button"
            variant={adjustOpen ? 'secondary' : 'outline'}
            size="sm"
            onClick={() => setAdjustOpen((prev) => !prev)}
            className={cn('rounded-xl', adjustOpen && 'border-[#FA93FA] bg-[#FA93FA] text-black hover:bg-[#FA93FA]')}
          >
            <SlidersHorizontal size={14} />
            Adjust DICOM
          </Button>

          <div className="flex flex-col items-start gap-0.5">
            <span className="text-[11px] opacity-60">Slice</span>
            <span className="text-lg font-semibold text-[#FA93FA] tabular-nums">
              {currentIndex + 1}<span className="text-xs opacity-50 text-gray-400">/{totalSlices}</span>
            </span>
          </div>

          {activeView === 'review' && totalSlices > 1 ? (
            <div className="flex min-w-[12rem] flex-1 items-center gap-2">
              <Button type="button" variant="outline" size="icon" onClick={() => handleSliceChange(-1)} aria-label="Previous slice">
                <ChevronDown size={18} />
              </Button>
              <div className="relative flex h-8 flex-1 items-center justify-center py-2">
                <div className={cn('absolute h-1 w-full rounded-full', themeMode === 'dark' ? 'bg-white/10' : 'bg-black/10')} />
                <div
                  className="absolute left-0 h-1 rounded-full bg-[#FA93FA] transition-all duration-75"
                  style={{ width: `${(currentIndex / Math.max(totalSlices - 1, 1)) * 100}%`, maxWidth: '100%' }}
                />

                <input
                  type="range"
                  min="0"
                  max={totalSlices - 1}
                  value={currentIndex}
                  onChange={handleSliderChange}
                  aria-label="DICOM slice selector"
                  title="DICOM slice selector"
                  className="absolute h-8 w-full cursor-pointer appearance-none bg-transparent opacity-0"
                />

                <div
                  className="absolute h-4 w-4 rounded-full border-2 border-white bg-[#FA93FA] shadow-lg pointer-events-none transition-all duration-75"
                  style={{ left: `calc(${(currentIndex / Math.max(totalSlices - 1, 1)) * 100}% - 8px)` }}
                />
              </div>
              <Button type="button" variant="outline" size="icon" onClick={() => handleSliceChange(1)} aria-label="Next slice">
                <ChevronUp size={18} />
              </Button>
            </div>
          ) : null}

          <Button
            type="button"
            size="icon"
            variant={playing ? 'secondary' : 'outline'}
            onClick={() => setPlaying(!playing)}
            aria-label={playing ? 'Pause playback' : 'Play slices'}
            className={cn(playing && 'border-[#FA93FA] bg-[#FA93FA] text-black hover:bg-[#FA93FA]')}
            disabled={activeView !== 'review' || totalSlices <= 1}
          >
            {playing ? <Pause size={18} fill="currentColor" /> : <Play size={18} fill="currentColor" />}
          </Button>
        </div>

        {adjustOpen ? (
          <div className={cn('mt-3 grid gap-3 rounded-2xl border p-3', themeMode === 'dark' ? 'border-white/10 bg-black/30' : 'border-black/10 bg-white/40', 'w-full')}>
            <div className="flex items-center justify-between gap-2">
              <span className="text-[11px] text-text-secondary">Rotate alignment</span>
              <div className="flex gap-2">
                <Button type="button" variant="outline" size="icon" onClick={() => onUpdate(file.id, { dicomAdjustments: { ...adjustments, rotation: adjustments.rotation - 90 } })} aria-label="Rotate left">
                  <RotateCcw size={14} />
                </Button>
                <Button type="button" variant="outline" size="icon" onClick={() => onUpdate(file.id, { dicomAdjustments: { ...adjustments, rotation: adjustments.rotation + 90 } })} aria-label="Rotate right">
                  <RotateCw size={14} />
                </Button>
              </div>
            </div>

            {(['top', 'right', 'bottom', 'left'] as const).map((edge) => (
              <label key={edge} className="grid gap-1">
                <div className="flex items-center justify-between text-[11px] text-text-secondary">
                  <span><Crop className="mr-1 inline size-3" />{edge}</span>
                  <span>{adjustments.crop[edge]}%</span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="35"
                  step="1"
                  value={adjustments.crop[edge]}
                  onChange={(event) => onUpdate(file.id, {
                    dicomAdjustments: {
                      ...adjustments,
                      crop: {
                        ...adjustments.crop,
                        [edge]: Number(event.target.value),
                      },
                    },
                  })}
                  className="w-full accent-[#FA93FA]"
                />
              </label>
            ))}

            <div className="flex flex-wrap gap-2 pt-1">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onUpdate(file.id, { dicomAdjustments: defaultAdjustments })}
              >
                Reset adjustments
              </Button>
              <Button
                type="button"
                size="sm"
                onClick={() => {
                  setAdjustOpen(false);
                  onStartPlanning?.();
                }}
                className="bg-[#FA93FA] text-black hover:bg-[#FA93FA]"
              >
                <CheckCircle2 className="mr-1 inline size-3" />
                Start planning
              </Button>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
};
