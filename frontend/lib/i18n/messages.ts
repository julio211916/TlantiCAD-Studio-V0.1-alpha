/**
 * i18n message catalogs (V262 scaffold).
 *
 * Default locale: **es** (Spanish). Three secondary locales: en, ru, pt.
 *
 * Strings are keyed by short slugs (e.g. `app.workspace.title`). Components
 * call `t('key')` from `lib/i18n/useT.ts`. When a key is missing in the
 * active locale we fall back to es so nothing renders blank.
 *
 * Convention:
 *  - Keys are dot-namespaced under the surface they live in
 *    (`app.*`, `dentaldb.*`, `cad.*`, `implant.*`, `dicom.*`, `share.*`, …).
 *  - Sentence case in es; the other locales follow their own conventions.
 *  - Placeholders use `{name}` syntax: `t('cad.tooth.title', { fdi: 16 })`.
 */

export type SupportedLocale = 'es' | 'en' | 'ru' | 'pt';

export const SUPPORTED_LOCALES: SupportedLocale[] = ['es', 'en', 'ru', 'pt'];

export const LOCALE_LABELS: Record<SupportedLocale, string> = {
    es: 'Español',
    en: 'English',
    ru: 'Русский',
    pt: 'Português',
};

export type MessageCatalog = Record<string, string>;

export const MESSAGES: Record<SupportedLocale, MessageCatalog> = {
    es: {
        // App-level
        'app.title': 'TlantiCAD Studio',
        'app.workspace.case-manager': 'Workspace TlantiCAD',
        'app.workspace.cad': 'TlantiCAD Diseño',
        'app.action.switch-to-workspace': 'Ir al Workspace TlantiCAD',
        'app.action.switch-to-cad': 'Ir al diseño CAD',
        'app.action.toggle-theme': 'Cambiar tema',
        'app.action.back-to-workspace': 'Volver al Workspace',

        // DentalDB / Workspace
        'dentaldb.welcome.title': 'Workspace TlantiCAD',
        'dentaldb.welcome.subtitle': 'Datos del caso, indicación y checklist clínica.',
        'dentaldb.sidebar.collapse': 'Plegar barra lateral del Workspace',
        'dentaldb.action.open-folder': 'Abrir carpeta del caso',
        'dentaldb.toolbar.project': 'Proyecto',
        'dentaldb.toolbar.dicom': 'Estudio DICOM',
        'dentaldb.toolbar.stl': 'Archivos STL / 3D',
        'dentaldb.toolbar.images': 'Imágenes 2D',
        'dentaldb.toolbar.documents': 'Documentos',
        'dentaldb.toolbar.notifications': 'Notificaciones',
        'dentaldb.badge': 'Workspace',

        // Patient form
        'patient.add.title': 'Añadir paciente',
        'patient.add.first-name': 'Nombre',
        'patient.add.surname': 'Apellido',
        'patient.add.dob': 'Fecha de nacimiento',
        'patient.add.notes': 'Notas',
        'patient.add.ok': 'OK',
        'patient.add.cancel': 'Cancelar',

        // Import dialog
        'import.dicom.title': 'Importar estudio DICOM',
        'import.stl.title': 'Importar archivos STL / 3D',
        'import.images.title': 'Importar imágenes 2D',
        'import.documents.title': 'Importar documentos',
        'import.action.file': 'Importar archivo(s)',
        'import.action.folder': 'Importar carpeta',
        'import.action.close': 'Cerrar',

        // Implant module
        'implant.module.empty': 'Define los objetivos del implante en el Workspace TlantiCAD primero o configura el modo de implante para desbloquear este panel.',

        // Share / collaboration (P2P only)
        'share.title': 'Compartir caso',
        'share.subtitle': 'Envía a otros equipos con TlantiCAD por AirDrop, Bluetooth o Wi-Fi local.',
        'share.transport.airdrop': 'AirDrop',
        'share.transport.bluetooth': 'Bluetooth',
        'share.transport.lan': 'Wi-Fi (LAN)',
        'share.peers.empty': 'No se detectan equipos cercanos. Asegúrate de tener AirDrop / Bluetooth activo y la misma red Wi-Fi.',
        'share.action.send': 'Enviar',
    },
    en: {
        'app.title': 'TlantiCAD Studio',
        'app.workspace.case-manager': 'TlantiCAD Workspace',
        'app.workspace.cad': 'TlantiCAD Design',
        'app.action.switch-to-workspace': 'Switch to TlantiCAD Workspace',
        'app.action.switch-to-cad': 'Switch to CAD design',
        'app.action.toggle-theme': 'Toggle theme',
        'app.action.back-to-workspace': 'Back to Workspace',

        'dentaldb.welcome.title': 'TlantiCAD Workspace',
        'dentaldb.welcome.subtitle': 'Case data, indication and clinical checklist.',
        'dentaldb.sidebar.collapse': 'Collapse Workspace sidebar',
        'dentaldb.action.open-folder': 'Open case folder',
        'dentaldb.toolbar.project': 'Project',
        'dentaldb.toolbar.dicom': 'DICOM dataset',
        'dentaldb.toolbar.stl': 'STL / 3D files',
        'dentaldb.toolbar.images': '2D images',
        'dentaldb.toolbar.documents': 'Documents',
        'dentaldb.toolbar.notifications': 'Notifications',
        'dentaldb.badge': 'Workspace',

        'patient.add.title': 'Add patient',
        'patient.add.first-name': 'First name',
        'patient.add.surname': 'Surname',
        'patient.add.dob': 'Date of birth',
        'patient.add.notes': 'Notes',
        'patient.add.ok': 'OK',
        'patient.add.cancel': 'Cancel',

        'import.dicom.title': 'Import DICOM study',
        'import.stl.title': 'Import STL / 3D files',
        'import.images.title': 'Import 2D images',
        'import.documents.title': 'Import documents',
        'import.action.file': 'Import file(s)',
        'import.action.folder': 'Import folder',
        'import.action.close': 'Close',

        'implant.module.empty': 'Define implant targets in the TlantiCAD Workspace first or set an implant mode to unlock this panel.',

        'share.title': 'Share case',
        'share.subtitle': 'Send to other TlantiCAD-enabled stations over AirDrop, Bluetooth or local Wi-Fi.',
        'share.transport.airdrop': 'AirDrop',
        'share.transport.bluetooth': 'Bluetooth',
        'share.transport.lan': 'Wi-Fi (LAN)',
        'share.peers.empty': 'No nearby stations detected. Make sure AirDrop / Bluetooth is on and that you share the same Wi-Fi network.',
        'share.action.send': 'Send',
    },
    ru: {
        'app.title': 'TlantiCAD Studio',
        'app.workspace.case-manager': 'Рабочее пространство TlantiCAD',
        'app.workspace.cad': 'CAD-дизайн TlantiCAD',
        'app.action.switch-to-workspace': 'Перейти к рабочему пространству TlantiCAD',
        'app.action.switch-to-cad': 'Перейти к CAD-дизайну',
        'app.action.toggle-theme': 'Сменить тему',
        'app.action.back-to-workspace': 'Назад к рабочему пространству',

        'dentaldb.welcome.title': 'Рабочее пространство TlantiCAD',
        'dentaldb.welcome.subtitle': 'Данные случая, показания и клинический чек-лист.',
        'dentaldb.sidebar.collapse': 'Свернуть боковую панель',
        'dentaldb.action.open-folder': 'Открыть папку случая',
        'dentaldb.toolbar.project': 'Проект',
        'dentaldb.toolbar.dicom': 'Набор DICOM',
        'dentaldb.toolbar.stl': 'STL / 3D-файлы',
        'dentaldb.toolbar.images': '2D изображения',
        'dentaldb.toolbar.documents': 'Документы',
        'dentaldb.toolbar.notifications': 'Уведомления',
        'dentaldb.badge': 'Рабочее',

        'patient.add.title': 'Добавить пациента',
        'patient.add.first-name': 'Имя',
        'patient.add.surname': 'Фамилия',
        'patient.add.dob': 'Дата рождения',
        'patient.add.notes': 'Заметки',
        'patient.add.ok': 'OK',
        'patient.add.cancel': 'Отмена',

        'import.dicom.title': 'Импорт DICOM-исследования',
        'import.stl.title': 'Импорт STL / 3D-файлов',
        'import.images.title': 'Импорт 2D-изображений',
        'import.documents.title': 'Импорт документов',
        'import.action.file': 'Импортировать файл(ы)',
        'import.action.folder': 'Импортировать папку',
        'import.action.close': 'Закрыть',

        'implant.module.empty': 'Сначала определите цели имплантата в рабочем пространстве TlantiCAD или выберите режим имплантата, чтобы разблокировать эту панель.',

        'share.title': 'Поделиться случаем',
        'share.subtitle': 'Отправить на другие станции TlantiCAD через AirDrop, Bluetooth или локальный Wi-Fi.',
        'share.transport.airdrop': 'AirDrop',
        'share.transport.bluetooth': 'Bluetooth',
        'share.transport.lan': 'Wi-Fi (LAN)',
        'share.peers.empty': 'Соседние станции не обнаружены. Убедитесь, что AirDrop / Bluetooth включён и вы в одной Wi-Fi сети.',
        'share.action.send': 'Отправить',
    },
    pt: {
        'app.title': 'TlantiCAD Studio',
        'app.workspace.case-manager': 'Workspace TlantiCAD',
        'app.workspace.cad': 'Desenho CAD TlantiCAD',
        'app.action.switch-to-workspace': 'Ir para o Workspace TlantiCAD',
        'app.action.switch-to-cad': 'Ir para o desenho CAD',
        'app.action.toggle-theme': 'Alternar tema',
        'app.action.back-to-workspace': 'Voltar ao Workspace',

        'dentaldb.welcome.title': 'Workspace TlantiCAD',
        'dentaldb.welcome.subtitle': 'Dados do caso, indicação e checklist clínica.',
        'dentaldb.sidebar.collapse': 'Recolher barra lateral',
        'dentaldb.action.open-folder': 'Abrir pasta do caso',
        'dentaldb.toolbar.project': 'Projeto',
        'dentaldb.toolbar.dicom': 'Estudo DICOM',
        'dentaldb.toolbar.stl': 'Arquivos STL / 3D',
        'dentaldb.toolbar.images': 'Imagens 2D',
        'dentaldb.toolbar.documents': 'Documentos',
        'dentaldb.toolbar.notifications': 'Notificações',
        'dentaldb.badge': 'Workspace',

        'patient.add.title': 'Adicionar paciente',
        'patient.add.first-name': 'Nome',
        'patient.add.surname': 'Sobrenome',
        'patient.add.dob': 'Data de nascimento',
        'patient.add.notes': 'Notas',
        'patient.add.ok': 'OK',
        'patient.add.cancel': 'Cancelar',

        'import.dicom.title': 'Importar estudo DICOM',
        'import.stl.title': 'Importar arquivos STL / 3D',
        'import.images.title': 'Importar imagens 2D',
        'import.documents.title': 'Importar documentos',
        'import.action.file': 'Importar arquivo(s)',
        'import.action.folder': 'Importar pasta',
        'import.action.close': 'Fechar',

        'implant.module.empty': 'Defina os alvos do implante no Workspace TlantiCAD primeiro ou configure um modo de implante para desbloquear este painel.',

        'share.title': 'Compartilhar caso',
        'share.subtitle': 'Envie para outras estações TlantiCAD via AirDrop, Bluetooth ou Wi-Fi local.',
        'share.transport.airdrop': 'AirDrop',
        'share.transport.bluetooth': 'Bluetooth',
        'share.transport.lan': 'Wi-Fi (LAN)',
        'share.peers.empty': 'Nenhuma estação próxima detectada. Verifique se AirDrop / Bluetooth está ligado e que estão na mesma rede Wi-Fi.',
        'share.action.send': 'Enviar',
    },
};

export const DEFAULT_LOCALE: SupportedLocale = 'es';

export function pickInitialLocale(): SupportedLocale {
    if (typeof navigator === 'undefined') return DEFAULT_LOCALE;
    const lang = navigator.language?.toLowerCase() ?? '';
    if (lang.startsWith('en')) return 'en';
    if (lang.startsWith('ru')) return 'ru';
    if (lang.startsWith('pt')) return 'pt';
    return DEFAULT_LOCALE;
}
