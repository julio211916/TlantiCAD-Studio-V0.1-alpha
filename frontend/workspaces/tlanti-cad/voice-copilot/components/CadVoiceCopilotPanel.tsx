import React, { useMemo, useRef, useState } from 'react';
import { Loader2, Mic, MicOff, Sparkles, Volume2, Wand2 } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { askLocalCadCopilot, type CadCopilotContext, type CadCopilotReply } from '@/lib/local-cad-copilot';
import type { FileData, ToolMode } from '@/types';
import { CadTlantiAiPanel } from '@/components/cad/CadTlantiAiPanel';

declare global {
  interface Window {
    webkitSpeechRecognition?: new () => SpeechRecognitionLike;
    SpeechRecognition?: new () => SpeechRecognitionLike;
  }
}

interface SpeechRecognitionLike {
  lang: string;
  interimResults: boolean;
  maxAlternatives: number;
  onresult: ((event: { results: ArrayLike<ArrayLike<{ transcript: string }>> }) => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
}

interface CopilotMessage {
  role: 'user' | 'assistant';
  text: string;
  source?: CadCopilotReply['source'];
  actionSuggestion?: ToolMode | null;
}

interface CadVoiceCopilotPanelProps {
  activeTool: ToolMode;
  activeToolLabel: string;
  selectedFile?: FileData | null;
  fileCount: number;
  isExpertMode: boolean;
  moduleId?: string;
  onSetTool: (tool: ToolMode) => void;
}

export function CadVoiceCopilotPanel({ activeTool, activeToolLabel, selectedFile, fileCount, isExpertMode, moduleId, onSetTool }: CadVoiceCopilotPanelProps) {
  const [prompt, setPrompt] = useState('');
  const [messages, setMessages] = useState<CopilotMessage[]>([
    {
      role: 'assistant',
      text: 'Soy tu copilot local dental. Puedo sugerir acciones CAD, revisar contexto DICOM y ayudarte por voz sin depender de Gemini.',
      source: 'rule-engine',
    },
  ]);
  const [isLoading, setIsLoading] = useState(false);
  const [isDictating, setIsDictating] = useState(false);
  const [dictationError, setDictationError] = useState<string | null>(null);
  const recognitionRef = useRef<SpeechRecognitionLike | null>(null);

  const context = useMemo<CadCopilotContext>(() => ({
    activeTool,
    activeToolLabel,
    selectedFile,
    fileCount,
    isExpertMode,
    moduleId,
  }), [activeTool, activeToolLabel, selectedFile, fileCount, isExpertMode, moduleId]);

  const lastAssistantMessage = [...messages].reverse().find((message) => message.role === 'assistant');
  const TlantiAiDefaultText = lastAssistantMessage?.text ?? 'Resume el siguiente paso clínico en TlantiCAD.';
  const voiceSupported = typeof window !== 'undefined' && Boolean(window.SpeechRecognition || window.webkitSpeechRecognition);
  const speechSupported = typeof window !== 'undefined' && 'speechSynthesis' in window;
  const moduleQuickPrompts = useMemo(() => {
    switch (moduleId) {
      case 'partials':
        return {
          title: 'PartialCAD AI focus',
          prompts: [
            'Sugiere el flujo PartialCAD AI para este caso dental.',
            'Qué revisar antes de sugerir una estructura parcial asistida por IA local?',
          ],
        };
      case 'ceph':
        return {
          title: 'Ceph focus',
          prompts: [
            'Qué landmarks cefalométricos debo confirmar antes del trazado?',
            'Resume riesgos de calidad en este estudio para análisis cefalométrico.',
          ],
        };
      case 'aligners':
        return {
          title: 'Aligners focus',
          prompts: [
            'Dame el siguiente paso para setup de alineadores.',
            'Qué validar antes de crear stages e IPR?',
          ],
        };
      case 'fab':
        return {
          title: 'Fab focus',
          prompts: [
            'Qué validar antes de preparar fabricación CAM?',
            'Resume riesgos de espesor, malla o orientación antes de exportar.',
          ],
        };
      case 'orthocad':
        return {
          title: 'Smile / waxup focus',
          prompts: [
            'Dame el siguiente paso del workflow de smile design.',
            'Qué validaciones debo hacer antes del mockup impreso?',
          ],
        };
      case 'dicom':
        return {
          title: 'DICOM Viewer focus',
          prompts: [
            'Qué revisar en este estudio DICOM antes de pasar a CAD?',
            'Resume riesgos de calidad o metadata clínica faltante.',
          ],
        };
      default:
        return null;
    }
  }, [moduleId]);

  async function runCopilot(input: string) {
    const trimmed = input.trim();
    if (!trimmed) {
      return;
    }

    setMessages((prev) => [...prev, { role: 'user', text: trimmed }]);
    setIsLoading(true);

    try {
      const reply = await askLocalCadCopilot(trimmed, context);
      setMessages((prev) => [...prev, {
        role: 'assistant',
        text: reply.text,
        source: reply.source,
        actionSuggestion: reply.actionSuggestion,
      }]);
      setPrompt('');
    } finally {
      setIsLoading(false);
    }
  }

  function toggleDictation() {
    const Recognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!Recognition) {
      setDictationError('El reconocimiento de voz no está disponible en este runtime.');
      return;
    }

    if (isDictating && recognitionRef.current) {
      recognitionRef.current.stop();
      setIsDictating(false);
      return;
    }

    setDictationError(null);
    const recognition = new Recognition();
    recognition.lang = 'es-MX';
    recognition.interimResults = false;
    recognition.maxAlternatives = 1;
    recognition.onresult = (event) => {
      const transcript = event.results?.[0]?.[0]?.transcript?.trim() ?? '';
      if (transcript) {
        setPrompt(transcript);
        void runCopilot(transcript);
      }
    };
    recognition.onerror = (event) => {
      setDictationError(`Dictado no disponible: ${event.error}`);
      setIsDictating(false);
    };
    recognition.onend = () => {
      setIsDictating(false);
    };
    recognitionRef.current = recognition;
    setIsDictating(true);
    recognition.start();
  }

  function speakLastReply() {
    if (!speechSupported || !lastAssistantMessage?.text) {
      return;
    }

    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(lastAssistantMessage.text);
    utterance.lang = 'es-MX';
    utterance.rate = 1;
    utterance.pitch = 1;
    window.speechSynthesis.speak(utterance);
  }

  return (
    <div className="pointer-events-auto flex h-full w-full flex-col rounded-[1.25rem] border border-white/8 bg-[#101215]/95 p-4 shadow-[0_18px_40px_rgba(0,0,0,0.28)] backdrop-blur-xl">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <Sparkles className="size-4 text-text-display" />
            <h4 className="text-sm font-semibold text-text-primary">CAD voice copilot</h4>
          </div>
          <p className="mt-1 text-xs text-text-secondary text-pretty">
            TlantiCAD Tools local copilot: dictado + TTS local del sistema y chat dental con modelo pequeño descargable.
          </p>
        </div>
        <div className="flex flex-wrap justify-end gap-2">
          <Badge variant="outline">Local</Badge>
          <Badge variant="outline">Tiny model</Badge>
        </div>
      </div>

      <div className="mt-3 flex flex-wrap gap-2 text-[11px] text-text-secondary">
        <Badge variant="outline">tool · {activeToolLabel}</Badge>
        <Badge variant="outline">{selectedFile ? `${selectedFile.type} · ${selectedFile.name}` : 'no active asset'}</Badge>
        <Badge variant="outline">{fileCount} files</Badge>
      </div>

      {moduleQuickPrompts ? (
        <div className="mt-3 rounded-xl border border-white/8 bg-[#14181d] p-3">
          <div className="flex items-center justify-between gap-2">
            <p className="text-[11px] uppercase text-text-secondary">{moduleQuickPrompts.title}</p>
            <Badge variant="outline">{moduleId}</Badge>
          </div>
          <div className="mt-2 grid gap-2 sm:grid-cols-2">
            {moduleQuickPrompts.prompts.map((quickPrompt) => (
              <button
                key={quickPrompt}
                type="button"
                onClick={() => void runCopilot(quickPrompt)}
                className="rounded-xl border border-white/8 bg-[#101215] px-3 py-2 text-left text-[11px] text-text-secondary transition-colors hover:bg-surface-raised hover:text-text-primary"
              >
                {quickPrompt}
              </button>
            ))}
          </div>
        </div>
      ) : null}

      <div className="mt-3 min-h-0 flex-1 space-y-2 overflow-y-auto rounded-xl border border-white/8 bg-[#14181d] p-3">
        {messages.map((message, index) => (
          <div key={`${message.role}-${index}`} className="rounded-xl border border-white/8 bg-[#101215] px-3 py-2">
            <div className="flex items-center justify-between gap-2">
              <p className="text-[11px] uppercase text-text-secondary">{message.role === 'assistant' ? 'Copilot' : 'You'}</p>
              {message.source ? <span className="text-[10px] uppercase text-text-secondary">{message.source === 'local-model' ? 'tiny model' : 'rule fallback'}</span> : null}
            </div>
            <p className="mt-1 text-sm text-text-primary text-pretty">{message.text}</p>
            {message.actionSuggestion ? (
              <div className="mt-2">
                <Button size="sm" variant="outline" onClick={() => onSetTool(message.actionSuggestion!)}>
                  <Wand2 className="mr-2 size-4" />
                  Activar {message.actionSuggestion}
                </Button>
              </div>
            ) : null}
          </div>
        ))}
      </div>

      <div className="mt-3 grid gap-2">
        <textarea
          value={prompt}
          onChange={(event) => setPrompt(event.target.value)}
          placeholder="Ejemplo: revisa si esta malla necesita reparación antes de diseñar la corona"
          className="min-h-24 rounded-xl border border-white/8 bg-[#14181d] px-3 py-2 text-sm text-text-primary outline-none transition-colors focus:border-text-display"
        />
        {dictationError ? <p className="text-xs text-amber-300">{dictationError}</p> : null}
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Button type="button" variant="secondary" disabled={isLoading} onClick={() => void runCopilot(prompt)}>
            {isLoading ? <Loader2 className="mr-2 size-4 animate-spin" /> : <Sparkles className="mr-2 size-4" />}
            Ask local copilot
          </Button>
          <Button type="button" variant="outline" onClick={toggleDictation}>
            {isDictating ? <MicOff className="mr-2 size-4" /> : <Mic className="mr-2 size-4" />}
            {isDictating ? 'Stop dictation' : 'Dictate'}
          </Button>
          <Button type="button" variant="outline" onClick={speakLastReply} disabled={!speechSupported || !lastAssistantMessage?.text}>
            <Volume2 className="mr-2 size-4" />
            Speak reply
          </Button>
        </div>
        <p className="text-[11px] text-text-secondary text-pretty">
          Voz: {voiceSupported ? 'dictado disponible' : 'dictado no disponible'} · TTS: {speechSupported ? 'speechSynthesis disponible' : 'speechSynthesis no disponible'}
        </p>
      </div>

      <div className="mt-3">
        <CadTlantiAiPanel
          defaultText={TlantiAiDefaultText}
          clinicalContext={`TlantiCAD CAD context: active tool ${activeToolLabel}, files ${fileCount}, module ${moduleId ?? 'general'}.`}
        />
      </div>
    </div>
  );
}
