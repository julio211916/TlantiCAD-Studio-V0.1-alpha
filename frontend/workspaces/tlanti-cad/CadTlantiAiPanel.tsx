import React from 'react';

export function CadTlantiAiPanel({
  defaultPrompt,
  defaultText,
  clinicalContext,
}: {
  defaultPrompt?: string;
  defaultText?: string;
  clinicalContext?: string;
}) {
  return (
    <div className="rounded-md border border-white/10 bg-black/20 p-2 text-[11px] text-text-secondary">
      {defaultText ?? defaultPrompt ?? 'TlantiAI local panel'}
      {clinicalContext ? <div className="mt-1 opacity-70">{clinicalContext}</div> : null}
    </div>
  );
}
