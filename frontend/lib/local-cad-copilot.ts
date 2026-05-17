import type { FileData, ToolMode } from '@/types';

export interface CadCopilotContext {
  activeTool: ToolMode;
  activeToolLabel: string;
  selectedFile?: FileData | null;
  fileCount: number;
  isExpertMode: boolean;
  moduleId?: string;
}

export interface CadCopilotReply {
  text: string;
  actionSuggestion: ToolMode | null;
  source: 'local-model' | 'rule-engine';
  modelLabel: string;
}

const LOCAL_MODEL_ID = 'Xenova/flan-t5-small';

type GeneratorResult = Array<{ generated_text?: string; summary_text?: string; text?: string }>;
type TextGenerator = (input: string, options?: Record<string, unknown>) => Promise<GeneratorResult>;

let generatorPromise: Promise<TextGenerator> | null = null;

function buildContextSummary(context: CadCopilotContext) {
  const selectedSummary = context.selectedFile
    ? `${context.selectedFile.type} / ${context.selectedFile.name}`
    : 'No selected file';

  return [
    `Active tool: ${context.activeToolLabel}.`,
    `Selected asset: ${selectedSummary}.`,
    `Open asset count: ${context.fileCount}.`,
    `Workspace mode: ${context.isExpertMode ? 'expert' : 'assistant'}.`,
    `Module: ${context.moduleId ?? 'cad'}.`,
  ].join(' ');
}

function inferToolSuggestion(prompt: string): ToolMode | null {
  const value = prompt.toLowerCase();
  if (/(medir|measure|distancia|clearance|espesor)/.test(value)) return 'MEASURE';
  if (/(mover|move|translate|traslad)/.test(value)) return 'MOVE';
  if (/(rotar|rotate|girar)/.test(value)) return 'ROTATE';
  if (/(escala|scale|escalar)/.test(value)) return 'SCALE';
  if (/(segment|segmentar|máscara|mask)/.test(value)) return 'SEGMENT';
  if (/(cortar|crop|recorta|recorte)/.test(value)) return 'CROP';
  if (/(clip|plano de corte)/.test(value)) return 'CLIP';
  if (/(esculp|sculpt)/.test(value)) return 'SCULPT';
  if (/(selecciona|select|cursor)/.test(value)) return 'SELECT';
  return null;
}

function buildRuleBasedReply(prompt: string, context: CadCopilotContext): string {
  const value = prompt.toLowerCase();
  const selected = context.selectedFile;

  if (!selected && /(margen|margin|mesh|malla|dicom|cbct|segment)/.test(value)) {
    return 'Primero importa o selecciona un asset dental. Sin una malla o estudio DICOM activo no puedo darte una acción CAD específica.';
  }

  if (selected?.type === 'DICOM' || /(dicom|cbct|panor[aá]m|mpr)/.test(value)) {
    return 'Para un caso DICOM dental, empieza revisando orientación, window/level y slice spacing. Si el estudio es CBCT, la siguiente acción útil es preparar MPR y validar la metadata antes de segmentar mandíbula, maxila y dientes.';
  }

  if (/(margen|margin|prep|preparaci[oó]n)/.test(value)) {
    return 'Sobre una preparación dental, prioriza tres cosas: aislar la pieza correcta, revisar huecos/holes de la malla y después correr detección heurística de margen o medición de espesores antes de modelar la restauración.';
  }

  if (/(agujero|hole|watertight|repar|repair|normal)/.test(value)) {
    return 'La malla debería pasar por un gate de calidad: watertight, normales consistentes y triángulos degenerados mínimos. Si falla, el siguiente paso es reparación y luego volver a medir antes de diseñar.';
  }

  if (/(implante|implant|gu[ií]a|guide|oclusi[oó]n|upper jaw)/.test(value)) {
    return 'Para flujos dentales avanzados, usa el copilot como checklist: validar datos clínicos, confirmar referencia oclusal y recién después mover a guía quirúrgica, implante o upper jaw motion.';
  }

  return `Contexto actual: ${buildContextSummary(context)} Respuesta corta: enfócate en la herramienta activa, valida el asset seleccionado y ejecuta una sola operación CAD por vez para evitar errores acumulados.`;
}

async function getGenerator(): Promise<TextGenerator> {
  if (!generatorPromise) {
    generatorPromise = (async () => {
      const transformers = await import('@huggingface/transformers');
      if ('env' in transformers) {
        transformers.env.allowLocalModels = false;
        transformers.env.useBrowserCache = true;
      }

      const pipeline = (transformers as { pipeline: (...args: unknown[]) => Promise<unknown> }).pipeline;
      return pipeline('text2text-generation', LOCAL_MODEL_ID) as Promise<TextGenerator>;
    })();
  }

  return generatorPromise;
}

function normalizeModelText(raw: string, fallback: string): string {
  const text = raw.trim();
  if (!text) {
    return fallback;
  }

  const cleaned = text
    .replace(/^answer\s*[:：-]?/i, '')
    .replace(/^response\s*[:：-]?/i, '')
    .trim();

  return cleaned || fallback;
}

export async function askLocalCadCopilot(prompt: string, context: CadCopilotContext): Promise<CadCopilotReply> {
  const normalizedPrompt = prompt.trim();
  const actionSuggestion = inferToolSuggestion(normalizedPrompt);
  const fallback = buildRuleBasedReply(normalizedPrompt, context);

  if (!normalizedPrompt) {
    return {
      text: 'Escribe o dicta una instrucción dental CAD para empezar.',
      actionSuggestion: null,
      source: 'rule-engine',
      modelLabel: LOCAL_MODEL_ID,
    };
  }

  try {
    const generator = await getGenerator();
    const result = await generator(
      [
        'You are TlantiCAD Copilot, a concise dental CAD/DICOM assistant.',
        'Answer in Spanish using at most 3 short bullet points or 2 sentences.',
        'Prefer practical next actions for dental CAD, DICOM, segmentation and mesh repair.',
        buildContextSummary(context),
        `User request: ${normalizedPrompt}`,
      ].join('\n'),
      {
        max_new_tokens: 96,
        temperature: 0.2,
        repetition_penalty: 1.15,
      },
    );

    const first = result?.[0]?.generated_text ?? result?.[0]?.summary_text ?? result?.[0]?.text ?? '';
    return {
      text: normalizeModelText(first, fallback),
      actionSuggestion,
      source: 'local-model',
      modelLabel: LOCAL_MODEL_ID,
    };
  } catch {
    return {
      text: fallback,
      actionSuggestion,
      source: 'rule-engine',
      modelLabel: LOCAL_MODEL_ID,
    };
  }
}
