import React, { useEffect, useMemo, useState } from 'react';
import { Search } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/interfaces-select';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { formatTlantiCaseStatus } from '@/lib/tlantidb-case-state-machine';
import { Switch } from '@/components/ui/interfaces-switch';
import type { TlantiDbSearchResult } from '@/lib/tlantidb-search';
import type { TlantiCase, TlantiDbState } from '@/stores/tlantidb-case-store';
import type { Language } from '@/types';

export type TlantiDbSidebarPanelId = 'case' | 'assets' | 'settings';

interface TlantiDbSidebarProps {
  language: Language;
  timeZone: string;
  currentTime: Date;
  databaseState: TlantiDbState;
  activeCase: TlantiCase;
  searchResults: TlantiDbSearchResult[];
  numberingSystem: 'FDI' | 'UNIVERSAL';
  assetProfile: 'clinical' | 'lab' | 'demo';
  operatorAlias: string;
  activePanel: TlantiDbSidebarPanelId;
  assetPanelSlot?: React.ReactNode;
  onActivePanelChange: (panel: TlantiDbSidebarPanelId) => void;
  onCaseSelect: (caseId: string) => void;
  onSearch: (query: string) => void;
  onSearchResultSelect: (result: TlantiDbSearchResult) => void;
  onSetClientName: (value: string) => void;
  onSetClientId: (value: string) => void;
  onSetPatientName: (value: string) => void;
  onSetPatientDateOfBirth: (value: string) => void;
  onSetOrderNumber: (value: string) => void;
  onSetLaboratoryName: (value: string) => void;
  onSetProjectName: (value: string) => void;
  onSetTechnicianName: (value: string) => void;
  onSetTechnicianId: (value: string) => void;
  onSetNotes: (value: string) => void;
  onSetNumberingSystem: (value: 'FDI' | 'UNIVERSAL') => void;
  onSetAssetProfile: (value: 'clinical' | 'lab' | 'demo') => void;
  onSetOperatorAlias: (value: string) => void;
  onOpenSettings: () => void;
  onToggleInteractiveOdontogram: (value: boolean) => void;
  onToggleSyncWindows: (value: boolean) => void;
  onToggleOpenModulesInNewWindow: (value: boolean) => void;
}

function InfoField({
  label,
  value,
  onChange,
  icon,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  icon?: React.ReactNode;
}) {
  return (
    <label className="grid gap-1.5">
      <span className="flex items-center gap-2 text-[11px] font-medium uppercase tracking-wide text-text-secondary">
        {icon}
        {label}
      </span>
      <Input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="h-9 border-border bg-card text-text-primary"
      />
    </label>
  );
}

function ToggleRow({
  title,
  subtitle,
  checked,
  onCheckedChange,
}: {
  title: string;
  subtitle: string;
  checked: boolean;
  onCheckedChange: (value: boolean) => void;
}) {
  return (
    <div className="tl-control flex items-center justify-between gap-3 rounded-md px-3 py-2.5">
      <div className="min-w-0">
        <p className="truncate text-sm font-medium text-text-primary">{title}</p>
        <p className="truncate text-xs text-text-secondary">{subtitle}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} aria-label={title} />
    </div>
  );
}

export function TlantiDbSidebar(props: TlantiDbSidebarProps) {
  const {
    language,
    timeZone,
    currentTime,
    databaseState,
    activeCase,
    searchResults,
    numberingSystem,
    assetProfile,
    operatorAlias,
    activePanel,
    assetPanelSlot,
    onActivePanelChange,
    onCaseSelect,
    onSearch,
    onSearchResultSelect,
    onSetNumberingSystem,
    onSetAssetProfile,
    onSetOperatorAlias,
    onOpenSettings,
    onToggleInteractiveOdontogram,
    onToggleSyncWindows,
    onToggleOpenModulesInNewWindow,
  } = props;

  const [assetsOpen, setAssetsOpen] = useState(false);
  const [searchValue, setSearchValue] = useState('');

  const formattedDate = useMemo(
    () => currentTime.toLocaleDateString(language, { month: 'short', day: 'numeric' }),
    [currentTime, language],
  );

  const panelMeta = {
    case: {
      title: 'Workspace',
      subtitle: `${formattedDate} · ${timeZone}`,
    },
    assets: {
      title: 'Assets',
      subtitle: 'Files and documents',
    },
    settings: {
      title: 'Settings',
      subtitle: 'Workspace controls',
    },
  } satisfies Record<TlantiDbSidebarPanelId, { title: string; subtitle: string }>;

  const openAssetsPanel = () => {
    onActivePanelChange('assets');
    setAssetsOpen(true);
  };

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      onSearch(searchValue);
    }, 220);

    return () => window.clearTimeout(timeoutId);
  }, [onSearch, searchValue]);

  return (
    <div className="tl-panel flex h-full min-h-0 flex-col overflow-hidden md:rounded-md">
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="border-b border-glass-border bg-panel-bg px-3 py-2.5">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold text-text-display">{panelMeta[activePanel].title}</p>
              <p className="truncate text-xs text-text-secondary">{panelMeta[activePanel].subtitle}</p>
            </div>
            <Badge className="border border-glass-border bg-control-bg text-text-primary">{activeCase.caseNumber}</Badge>
          </div>
          {activePanel === 'case' ? (
          <div className="tl-control mt-3 flex items-center gap-2 rounded-md px-2">
            <Search className="size-4 shrink-0 text-text-secondary" />
            <input
              type="search"
              value={searchValue}
              onChange={(event) => setSearchValue(event.target.value)}
              placeholder="Search case, XML, lab"
              className="h-9 min-w-0 flex-1 bg-transparent text-sm text-text-primary outline-none placeholder:text-text-secondary"
            />
          </div>
          ) : null}
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto p-3">
          {activePanel === 'case' ? (
            <div className="grid gap-2.5">
              <div className="tl-panel rounded-md px-3 py-3">
                <div className="mb-2 flex items-center justify-between gap-3">
                  <div className="min-w-0">
                    <p className="truncate text-sm font-semibold text-text-display">Active case</p>
                    <p className="truncate text-xs text-text-secondary">{activeCase.name}</p>
                  </div>
                  <Badge variant="outline">{formatTlantiCaseStatus(activeCase.status)}</Badge>
                </div>
                <Select value={activeCase.id} onValueChange={onCaseSelect}>
                  <SelectTrigger className="h-9 w-full text-sm">
                    <SelectValue placeholder="Select case" />
                  </SelectTrigger>
                  <SelectContent>
                    {databaseState.cases.map((item) => (
                      <SelectItem key={item.id} value={item.id}>{item.caseNumber} · {item.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {searchResults.length > 0 ? (
                <div className="grid gap-2">
                  {searchResults.map((result) => (
                    <button
                      key={result.id}
                      type="button"
                      onClick={() => onSearchResultSelect(result)}
                      className="tl-control flex w-full items-start justify-between gap-3 rounded-md px-3 py-2.5 text-left transition-colors"
                    >
                      <div className="min-w-0">
                        <p className="truncate text-sm text-text-primary">{result.label}</p>
                        <p className="truncate text-xs text-text-secondary">{result.description}</p>
                      </div>
                      <Badge variant="outline">{result.kind}</Badge>
                    </button>
                  ))}
                </div>
              ) : null}
            </div>
          ) : null}

          {activePanel === 'assets' ? (
            <Card className="tl-panel gap-3 rounded-md py-3">
              <CardHeader className="px-3">
                <CardTitle className="text-sm text-text-display">Assets</CardTitle>
                <CardDescription className="text-xs">Use the sheet for full document and preview tools.</CardDescription>
              </CardHeader>
              <CardContent className="px-3">
                <Button type="button" className="w-full" onClick={() => setAssetsOpen(true)}>
                  Open assets sheet
                </Button>
              </CardContent>
            </Card>
          ) : null}

          {activePanel === 'settings' ? (
            <div className="grid gap-3">
              <Card className="tl-panel gap-3 rounded-md py-3">
                <CardHeader className="px-3">
                  <CardTitle className="text-sm text-text-display">Preferences</CardTitle>
                </CardHeader>
                <CardContent className="grid gap-3 px-3">
                  <label className="grid gap-1.5">
                    <span className="text-[11px] font-medium uppercase tracking-wide text-text-secondary">Numbering</span>
                    <Select value={numberingSystem} onValueChange={(value) => onSetNumberingSystem(value as 'FDI' | 'UNIVERSAL')}>
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="Select numbering" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="FDI">FDI</SelectItem>
                        <SelectItem value="UNIVERSAL">Universal</SelectItem>
                      </SelectContent>
                    </Select>
                  </label>

                  <label className="grid gap-1.5">
                    <span className="text-[11px] font-medium uppercase tracking-wide text-text-secondary">Asset profile</span>
                    <Select value={assetProfile} onValueChange={(value) => onSetAssetProfile(value as 'clinical' | 'lab' | 'demo')}>
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="Select asset profile" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="clinical">Clinical</SelectItem>
                        <SelectItem value="lab">Lab</SelectItem>
                        <SelectItem value="demo">Demo</SelectItem>
                      </SelectContent>
                    </Select>
                  </label>

                  <InfoField label="Operator" value={operatorAlias} onChange={onSetOperatorAlias} />
                </CardContent>
              </Card>

              <Card className="tl-panel gap-3 rounded-md py-3">
                <CardContent className="grid gap-2 px-3">
                  <ToggleRow
                    title="Native windows"
                    subtitle="Open CAD modules as desktop windows"
                    checked={databaseState.preferences.openModulesInNewWindow}
                    onCheckedChange={onToggleOpenModulesInNewWindow}
                  />
                  <ToggleRow
                    title="Sync windows"
                    subtitle="Keep open workspaces in sync"
                    checked={databaseState.preferences.syncWindows}
                    onCheckedChange={onToggleSyncWindows}
                  />
                  <ToggleRow
                    title="Interactive odontogram"
                    subtitle="Use the large SVG dental surface"
                    checked={databaseState.preferences.useInteractiveOdontogram}
                    onCheckedChange={onToggleInteractiveOdontogram}
                  />
                  <Button type="button" variant="outline" onClick={onOpenSettings}>
                    Detailed settings
                  </Button>
                </CardContent>
              </Card>
            </div>
          ) : null}
        </div>
      </div>

      <Sheet open={assetsOpen} onOpenChange={setAssetsOpen}>
        <SheetContent side="left" className="w-[min(92vw,34rem)] overflow-y-auto border-glass-border bg-window-bg p-0 text-text-primary sm:max-w-none">
          <SheetHeader className="border-b border-glass-border">
            <SheetTitle className="text-text-display">Case assets</SheetTitle>
            <SheetDescription>DICOM, previews, documents and XML for {activeCase.caseNumber}.</SheetDescription>
          </SheetHeader>
          <div className="p-4">
            {assetPanelSlot ?? (
              <div className="rounded-md border border-dashed border-border p-4 text-sm text-text-secondary">
                No asset panel is mounted for this workspace.
              </div>
            )}
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}
