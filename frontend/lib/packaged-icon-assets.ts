type ImportMetaWithGlob = ImportMeta & {
  glob: (
    pattern: string | string[],
    options: { eager: true; import: 'default'; query: '?url' },
  ) => Record<string, string>;
};

const packagedIconUrls = (import.meta as ImportMetaWithGlob).glob([
  '../../../packages/icons/{tlanticad,bite-splint,telescope,workflow,connector,freeforming,worktype,screw,abutment,palette,images/materials,cad-tools}/**/*.{svg,png,jpg,jpeg,webp}',
], {
  eager: true,
  import: 'default',
  query: '?url',
}) as Record<string, string>;

const graphicsIconUrls: Record<string, string> = {
  'Graphics/newcast3dstl.svg': new URL('../../../packages/icons/Graphics/newcast3dstl.svg', import.meta.url).href,
  'Graphics/newcast3dfromdicom.svg': new URL('../../../packages/icons/Graphics/newcast3dfromdicom.svg', import.meta.url).href,
  'Graphics/importimages.svg': new URL('../../../packages/icons/Graphics/importimages.svg', import.meta.url).href,
  'Graphics/standarddocs.svg': new URL('../../../packages/icons/Graphics/standarddocs.svg', import.meta.url).href,
  'Graphics/implant.svg': new URL('../../../packages/icons/Graphics/implant.svg', import.meta.url).href,
  'Graphics/smiledesign3d.svg': new URL('../../../packages/icons/Graphics/smiledesign3d.svg', import.meta.url).href,
  'Graphics/nemofab.svg': new URL('../../../packages/icons/Graphics/nemofab.svg', import.meta.url).href,
  'Graphics/nemofab3d.svg': new URL('../../../packages/icons/Graphics/nemofab3d.svg', import.meta.url).href,
  'Graphics/Añadir-Material.svg': new URL('../../../packages/icons/Graphics/Añadir-Material.svg', import.meta.url).href,
  'Graphics/Quitar-Material.svg': new URL('../../../packages/icons/Graphics/Quitar-Material.svg', import.meta.url).href,
  'Graphics/Suavizar-Material.svg': new URL('../../../packages/icons/Graphics/Suavizar-Material.svg', import.meta.url).href,
  'Graphics/Extrusion.svg': new URL('../../../packages/icons/Graphics/Extrusion.svg', import.meta.url).href,
  'Graphics/Esculpir-Dialogo.svg': new URL('../../../packages/icons/Graphics/Esculpir-Dialogo.svg', import.meta.url).href,
  'Graphics/Seleccionar Lasso.svg': new URL('../../../packages/icons/Graphics/Seleccionar Lasso.svg', import.meta.url).href,
  'Graphics/zoom_more.svg': new URL('../../../packages/icons/Graphics/zoom_more.svg', import.meta.url).href,
  'Graphics/Rotation1.svg': new URL('../../../packages/icons/Graphics/Rotation1.svg', import.meta.url).href,
};

function normalizeIconPath(path: string) {
  return path
    .replace(/^\/+/, '')
    .replace(/^icons\//, '')
    .replace(/\\/g, '/');
}

export function resolvePackagedIconUrl(path: string): string | null {
  if (!path || /^(?:https?:|data:|blob:|asset:)/i.test(path)) {
    return path || null;
  }

  const normalized = normalizeIconPath(path);
  return graphicsIconUrls[normalized] ?? packagedIconUrls[`../../../packages/icons/${normalized}`] ?? null;
}

export function resolvePackagedIconUrlOr(path: string, fallback = path): string {
  return resolvePackagedIconUrl(path) ?? fallback;
}
