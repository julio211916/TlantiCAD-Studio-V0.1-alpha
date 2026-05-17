import type { ToolMode } from '@/types';

export type SmileDesignPlaybookId = 'ai-mockup' | 'manual-mockup' | 'final-veneers' | 'mockup-model';

export interface SmileDesignStage {
  id: string;
  title: string;
  objective: string;
  tasks: string[];
  recommendedTool: ToolMode | null;
  deliverable: string;
}

export interface SmileDesignPlaybook {
  id: SmileDesignPlaybookId;
  title: string;
  subtitle: string;
  source: string;
  targetOutput: string;
  stages: SmileDesignStage[];
}

export const SMILE_DESIGN_PLAYBOOKS: SmileDesignPlaybook[] = [
  {
    id: 'ai-mockup',
    title: 'Direct print mockup with AI',
    subtitle: 'Baseline para smile design asistido por IA con segmentación, alineación fotográfica y mockup imprimible.',
    source: 'Exocad Smile Design & Veneers · Direct Print Mockup with AI',
    targetOutput: 'Mockup imprimible splinted de espesor mínimo controlado.',
    stages: [
      {
        id: 'ai-case-setup',
        title: 'Case setup + arch scope',
        objective: 'Crear el caso, definir el rango distal a distal y dejar el mockup listo para 3D Print.',
        tasks: [
          'Abrir el Workspace TlantiCAD y crear tratamiento nuevo con nombre del paciente.',
          'Seleccionar distal-most tooth y el contralateral para el tramo del smile design.',
          'Elegir Mockup + 3D Print con espesor mínimo 0.2–0.3 mm según impresora.',
          'Eliminar conectores iniciales no deseados y guardar el caso base.',
        ],
        recommendedTool: 'SELECT',
        deliverable: 'Caso inicial de mockup dental guardado con alcance completo del arco estético.',
      },
      {
        id: 'ai-scan-orientation',
        title: 'Load scans + orientation',
        objective: 'Cargar upper/lower scans y dejar la orientación oclusal lista para segmentación.',
        tasks: [
          'Cargar archivos intraorales o de modelo en el orden upper → lower.',
          'Rotar a top-down occlusal view con ligera visibilidad facial anterior.',
          'Confirmar que la geometría no esté invertida y avanzar a Start Segmentation.',
        ],
        recommendedTool: 'ROTATE',
        deliverable: 'Escaneos alineados para segmentación AI del arco superior e inferior.',
      },
      {
        id: 'ai-segmentation',
        title: 'AI segmentation + tooth axes',
        objective: 'Completar la segmentación superior e inferior y corregir ejes largos/landmarks dentales.',
        tasks: [
          'Ejecutar Start Segmentation y dejar que IA auto-segmente el arco si el scan es limpio.',
          'Completar dientes faltantes desde el pictograma clicando groove central o borde incisal.',
          'Usar correction para pintar/agregar o quitar regiones amarillas cuando un diente falle.',
          'Alinear flechas/ejes largos desde central groove o incisal edge para cada diente.',
        ],
        recommendedTool: 'SEGMENT',
        deliverable: 'Arcos segmentados con ejes dentales consistentes para el Smile Creator.',
      },
      {
        id: 'ai-photo-alignment',
        title: 'Photo alignment + facial references',
        objective: 'Alinear retracted photo y smile photo, y fijar labios, pupilas y gauges de proporción.',
        tasks: [
          'Cargar retracted image y verificar alineación automática de scan a fotografía.',
          'Cargar smile image y validar que el maxilar no quede borroso tras la autoalineación.',
          'Marcar lip line, centro pupilar y referencias faciales básicas.',
          'Ajustar proportion gauge, facial midline, gingival line y smile arc al objetivo clínico.',
        ],
        recommendedTool: 'MEASURE',
        deliverable: 'Entorno fotográfico alineado con referencias faciales y gauge listo para diseño.',
      },
      {
        id: 'ai-smile-composition',
        title: 'Smile composition + tooth positioning',
        objective: 'Colocar la propuesta estética inicial y mover el arco/anteriores hasta lograr una solución aditiva.',
        tasks: [
          'Configurar ratio de proporción 100-65-50 y ubicar el gauge sobre línea media e incisal edge deseado.',
          'Mover cuspid positions y verificar central width ~8 mm y largo 10 mm aprox.',
          'Encender preoperative teeth y contrastar waxup vs diente natural.',
          'Rotar palatal/anterior-posterior el arco hasta ver una capa delgada de wax sobre facial.',
        ],
        recommendedTool: 'MOVE',
        deliverable: 'Setup de sonrisa aditiva con midline, gingival zenith e incisal edges validados.',
      },
      {
        id: 'ai-individual-tuning',
        title: 'Individual tooth tuning + mirror',
        objective: 'Ajustar dientes individuales, movimientos simétricos y morfología fina por diente.',
        tasks: [
          'Mover cada diente de forma individual hasta dejar wax uniforme sobre facial.',
          'Activar mirror movements cuando convenga mantener simetría contralateral.',
          'Usar Free-Form / Anatomic para microajustes de proximal, incisal y contornos cervicales.',
          'Comprobar medidas visuales y proporciones antes de pasar a espesor.',
        ],
        recommendedTool: 'MOVE',
        deliverable: 'Arreglo dentario fino con simetría clínica y contornos listos para espesor.',
      },
      {
        id: 'ai-thickness-printability',
        title: 'Thickness control + printability',
        objective: 'Eliminar zonas demasiado finas y garantizar que el mockup sea imprimible.',
        tasks: [
          'Activar Visualize Thickness y fijar 0.2–0.3 mm según la resolución del printer.',
          'Agregar material en zonas pink hasta dejar solo borde marginal comprometido.',
          'Usar brush grande con movimientos pequeños para no distorsionar anatomía.',
          'Hacer preview fotorealista y revisar continuidad estética de 8–9 y del sector anterior.',
        ],
        recommendedTool: 'SCULPT',
        deliverable: 'Mockup imprimible sin zonas críticas de espesor y con estética estable.',
      },
      {
        id: 'ai-insertion-export',
        title: 'Insertion path + export',
        objective: 'Fijar dirección de inserción, adaptar la base y exportar el STL final.',
        tasks: [
          'Ajustar path of insertion casi facial con leve visibilidad incisal.',
          'Eliminar blackout wax/lingual flash donde sea necesario para asiento íntimo.',
          'Equalize dynamic contacts y dejar veneers/mockup sin conflictos estáticos indeseados.',
          'Adapt jaw scan hard cut al final y exportar visible objects only como STL.',
        ],
        recommendedTool: 'CROP',
        deliverable: 'STL final del mockup directo listo para impresión clínica.',
      },
    ],
  },
  {
    id: 'manual-mockup',
    title: 'Direct print mockup without AI',
    subtitle: 'Flujo manual de smile creator sin auto-segmentación, orientado a mockups aditivos y control manual.',
    source: 'Exocad Smile Design & Veneers · Direct Print Mockup without AI',
    targetOutput: 'Mockup impreso sin IA, validado para asiento, dinámica y export STL.',
    stages: [
      {
        id: 'manual-case',
        title: 'Case setup + mockup scope',
        objective: 'Configurar el caso y el rango de diseño del mockup.',
        tasks: [
          'Elegir cliente default, nombre del paciente y distal-most teeth.',
          'Seleccionar Mockup + 3D Print con 0.3 mm seguro o 0.2 mm si la impresora lo soporta.',
          'Mantener SHIFT para incluir el contralateral y remover los conectores visuales.',
          'Guardar y pasar a Design.',
        ],
        recommendedTool: 'SELECT',
        deliverable: 'Caso de mockup manual preparado con tramo del arco definido.',
      },
      {
        id: 'manual-load-orient',
        title: 'Load scans + orientation',
        objective: 'Cargar upper/lower y orientar el arco para Smile Creator.',
        tasks: [
          'Cargar upper scan primero y lower después.',
          'Rotar para ver oclusal con ligera exposición facial anterior.',
          'Avanzar con la orientación ya estable a Smile Creator.',
        ],
        recommendedTool: 'ROTATE',
        deliverable: 'Escaneos manuales orientados para registro foto ↔ modelo.',
      },
      {
        id: 'manual-photo-registration',
        title: 'Smile Creator registration',
        objective: 'Registrar fotografía retraída, modelo 3D y smile photo sin depender de IA segmentadora.',
        tasks: [
          'Marcar CEJ/cusp tip en la foto retraída y repetir puntos exactos sobre el scan 3D.',
          'Ajustar con rotación y dragging de balls hasta eliminar ghosting visible.',
          'Cargar la smile photo y validar que el maxilar salga nítido.',
          'Corregir lo posible, entendiendo que ángulos de foto malos no alinean bien.',
        ],
        recommendedTool: 'MEASURE',
        deliverable: 'Registro foto-modelo suficientemente estable para componer la sonrisa.',
      },
      {
        id: 'manual-facial-gauge',
        title: 'Lip / pupil / proportion gauge',
        objective: 'Definir referencias faciales y gauge proporcional para el diseño.',
        tasks: [
          'Trazar labios, pupilas y posicionar proportion gauge.',
          'Centrar la barra vertical en línea media y ajustar la horizontal a incisal edge deseado.',
          'Cambiar a ratio 100-65-50 y poner cant a 0° si aplica.',
          'Alinear distal de cúspide con distal del canino clínicamente objetivo.',
        ],
        recommendedTool: 'MEASURE',
        deliverable: 'Gauge facial funcional para longitud, anchura y simetría del diseño.',
      },
      {
        id: 'manual-arch-placement',
        title: 'Arch positioning + facial wax strategy',
        objective: 'Mover el arco y los dientes para una estrategia principalmente aditiva.',
        tasks: [
          'Rotar el arco hasta que las cúspides sean paralelas al arco natural.',
          'Mover el conjunto apical/facial hasta que CEJ e incisales coincidan con el gauge.',
          'Confirmar 10–11 mm de longitud central y cera predominantemente facial.',
          'Ignorar detalles finos de premolares/cúspides hasta validar el frente anterior.',
        ],
        recommendedTool: 'MOVE',
        deliverable: 'Posición global del waxup lista para refinamiento dental.',
      },
      {
        id: 'manual-tooth-placement',
        title: 'Chain mode + individual movement',
        objective: 'Recolocar dientes uno por uno, asegurar contactos y emergencias naturales.',
        tasks: [
          'Activar Chain Mode y fijar anclas (8–9) antes de mover premolares/caninos.',
          'Usar CTRL para rotar, SHIFT para escalar y CTRL+SHIFT para estirar cuando sea necesario.',
          'Mover cada diente palatino/facial hasta dejar una película fina de wax sobre el natural.',
          'Alinear contactos proximales con embrasures naturales.',
        ],
        recommendedTool: 'MOVE',
        deliverable: 'Posición por diente coherente con mockup additivo de baja preparación.',
      },
      {
        id: 'manual-freeform-occlusion',
        title: 'Free-form + dynamic equalization',
        objective: 'Ajustar espesor, dinámica y contornos finales del mockup.',
        tasks: [
          'Visualizar espesor a 0.3 mm y añadir donde existan zonas pink.',
          'Remover exceso lingual con SHIFT y equilibrar contactos dinámicos a verde uniforme.',
          'Revisar que no queden contactos estáticos en veneers si el caso lo requiere.',
          'Ajustar CEJ y cervicales desde una vista facial recta.',
        ],
        recommendedTool: 'SCULPT',
        deliverable: 'Mockup manual imprimible con dinámica y espesor controlados.',
      },
      {
        id: 'manual-export',
        title: 'Adaptation + export STL',
        objective: 'Completar adaptación final y exportar la escena imprimible.',
        tasks: [
          'Usar free-adapt → approximal y static occlusion según el flujo indicado.',
          'Revisar que ya sean shells y que los merged parts visibles sean los correctos.',
          'Guardar escena como STL con visible objects only y orientación default.',
        ],
        recommendedTool: 'CROP',
        deliverable: 'STL exportado del mockup manual listo para impresión.',
      },
    ],
  },
  {
    id: 'final-veneers',
    title: 'Conversion to final veneers',
    subtitle: 'Transformación del mockup a veneers finales individuales, con adaptación más agresiva y export final.',
    source: 'Exocad Smile Design & Veneers · Conversion to Final Veneers',
    targetOutput: 'Set final de STL individuales de no-prep veneers.',
    stages: [
      {
        id: 'final-reload',
        title: 'Reload pre-hard-cut case',
        objective: 'Reabrir el caso correcto y partir del mockup antes del hard cut.',
        tasks: [
          'Confirmar que el diseño anterior fue exportado antes de convertir a finals.',
          'Reabrir el caso desde el Workspace TlantiCAD y cargar la versión anterior al hard cut.',
          'Entrar a design con el contexto correcto del mockup base.',
        ],
        recommendedTool: 'SELECT',
        deliverable: 'Caso recargado en el punto exacto para iniciar conversión a finales.',
      },
      {
        id: 'final-clean-bottom',
        title: 'Delete virtual waxup bottom',
        objective: 'Eliminar la base virtual previa y resetear la estrategia de inserción para veneers individuales.',
        tasks: [
          'Ir a expert mode y borrar virtual waxup bottom.',
          'Seleccionar de nuevo virtual waxup bottom como entidad editable.',
          'Preparar una nueva orientación de inserción casi facial.',
        ],
        recommendedTool: 'SELECT',
        deliverable: 'Base virtual eliminada y lista para inserción de veneers individuales.',
      },
      {
        id: 'final-insertion-aggressive',
        title: 'Insertion reset + aggressive wax removal',
        objective: 'Definir un path de inserción adecuado y retirar más blockout wax que en el mockup splinted.',
        tasks: [
          'Set insertion from facial view con algo de incisal edge visible.',
          'Aplicar la nueva dirección de inserción.',
          'En Free-Form, remover cera con SHIFT de forma más agresiva pero sensata.',
          'Buscar un modelo más crisp sin comprometer el seat de cada veneer.',
        ],
        recommendedTool: 'SCULPT',
        deliverable: 'Veneers individuales con inserción revisada y reducción de wax optimizada.',
      },
      {
        id: 'final-adapt-shells',
        title: 'Adapt shells to jaw scan',
        objective: 'Convertir definitivamente a shells y adaptar al jaw scan.',
        tasks: [
          'Ir a Wizard → Free → Adapt.',
          'Ejecutar en orden: Approximal → Static Occlusion → Adapt Design Jaw Scan Hard Cut.',
          'No editar diseño después del hard cut final.',
        ],
        recommendedTool: 'CROP',
        deliverable: 'Shells finales adaptadas al jaw scan sin re-trabajo posterior.',
      },
      {
        id: 'final-contact-review',
        title: 'Proximal/contact review',
        objective: 'Verificar contactos proximales e integridad final de cada veneer.',
        tasks: [
          'Revisar proximales y usar adjacents si se requiere.',
          'Buscar contacto teal/blue donde show contact areas lo marque como correcto.',
          'Confirmar que cada veneer individual mantiene integridad estructural.',
        ],
        recommendedTool: 'MEASURE',
        deliverable: 'Set de veneers finales clínicamente revisado para export.',
      },
      {
        id: 'final-export',
        title: 'Final STL export',
        objective: 'Cerrar el flujo y ubicar/exportar los STL individuales finales.',
        tasks: [
          'Avanzar con Next hasta cerrar el caso.',
          'Abrir carpeta del caso en el Workspace TlantiCAD y ubicar los STL finales.',
          'Registrar la ruta de salida para laboratorio / CAD-CAM.',
        ],
        recommendedTool: null,
        deliverable: 'Archivos STL individuales finales disponibles para producción.',
      },
    ],
  },
  {
    id: 'mockup-model',
    title: 'Mockup model',
    subtitle: 'Construcción del modelo de trabajo con digital waxup model y base honeycomb para impresión.',
    source: 'Exocad Smile Design & Veneers · Mockup Model',
    targetOutput: 'Modelo STL del waxup/mockup listo para impresión o inyección.',
    stages: [
      {
        id: 'model-entry',
        title: 'Open Design Model',
        objective: 'Entrar al módulo de model design desde el merge/save o desde el Workspace TlantiCAD.',
        tasks: [
          'Seleccionar Design Model y avanzar.',
          'Escoger Digital Waxup Model como tipo de modelo.',
        ],
        recommendedTool: 'SELECT',
        deliverable: 'Entrada correcta al flujo de diseño del modelo.',
      },
      {
        id: 'model-position-crop',
        title: 'Position + crop base',
        objective: 'Posicionar el modelo entre slice planes y recortar la base a una altura útil.',
        tasks: [
          'Drag del modelo entre planos azul/verde.',
          'Usar CTRL para rotar si hace falta.',
          'Recortar altura/base con el slider para evitar una base demasiado alta.',
        ],
        recommendedTool: 'CROP',
        deliverable: 'Base del modelo posicionada y recortada para manufactura.',
      },
      {
        id: 'model-select-teeth',
        title: 'Select included teeth',
        objective: 'Determinar qué waxed teeth quedan en el modelo final.',
        tasks: [
          'Revisar odontograma y apagar dientes que no deban ir en el modelo.',
          'Evaluar presets como every other tooth para modelos de inyección.',
        ],
        recommendedTool: 'SELECT',
        deliverable: 'Selección de dientes definitiva del modelo de trabajo.',
      },
      {
        id: 'model-antagonist-honeycomb',
        title: 'Antagonist + honeycomb',
        objective: 'Configurar antagonista si aplica y rellenar la base con honeycomb.',
        tasks: [
          'Crear o saltar antagonist model según necesidad del caso.',
          'Elegir honeycomb como attachment/fill.',
          'Escalar honeycomb alrededor de 115% y centrarlo correctamente.',
        ],
        recommendedTool: 'VOXELIZE',
        deliverable: 'Modelo con relleno honeycomb listo para cierre.',
      },
      {
        id: 'model-export',
        title: 'Finalize + export model',
        objective: 'Finalizar el modelo digital waxup y exportarlo como STL visible-only.',
        tasks: [
          'Completar el wizard y dejar visible solo el modelo final.',
          'Guardar escena como STL o ubicar el STL dentro del project folder.',
          'Registrar uso del modelo para mockup, inyección o laboratorio.',
        ],
        recommendedTool: null,
        deliverable: 'Modelo STL final del waxup listo para impresión o duplicación.',
      },
    ],
  },
];

export function getSmileDesignPlaybookById(id: SmileDesignPlaybookId) {
  return SMILE_DESIGN_PLAYBOOKS.find((playbook) => playbook.id === id) ?? SMILE_DESIGN_PLAYBOOKS[0];
}

export function getSmileDesignStageTaskIds(playbook: SmileDesignPlaybook, stageIndex: number) {
  const stage = playbook.stages[stageIndex];
  if (!stage) {
    return [];
  }

  return stage.tasks.map((_, taskIndex) => `${playbook.id}:${stage.id}:task-${taskIndex + 1}`);
}