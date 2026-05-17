import { parseUrl } from '@/src/utils/url';

/**
 * Detects object-storage URIs. TlantiCAD clinical mode is offline-only, so
 * these sources are identified only to return a deterministic blocked error.
 * @param uri
 * @returns
 */
export const isAmazonS3Uri = (uri: string) =>
  parseUrl(uri, window.location.origin).protocol === 's3:';

export type ObjectAvailableCallback = (url: string, name: string) => void;

/**
 * Extracts bucket and prefix from object-storage URIs.
 * @param uri
 * @returns
 */
export const extractBucketAndPrefixFromS3Uri = (uri: string) => {
  const { hostname: bucket, pathname } = parseUrl(uri);
  // drop the leading forward slash
  const objectName = pathname.replace(/^\//, '');
  return [bucket, objectName] as const;
};

/**
 * Remote object-store expansion is intentionally blocked in the clinical
 * desktop. Import DICOM/STL/OBJ/PLY through local paths or Mesh Vault handles.
 *
 * @param s3Uri
 */
export const getObjectsFromS3 = async (
  s3Uri: string,
  _onObjectAvailable: ObjectAvailableCallback = () => {}
) => {
  const [bucket, objPrefix] = extractBucketAndPrefixFromS3Uri(s3Uri);
  throw new Error(`Offline clinical mode blocks remote object-store imports: ${bucket}/${objPrefix}`);
};
