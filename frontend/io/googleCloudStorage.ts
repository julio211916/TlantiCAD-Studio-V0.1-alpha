import { parseUrl } from '@/src/utils/url';

export interface GcsObject {
  kind: string;
  id: string;
  selfLink: string;
  mediaLink: string;
  name: string;
  bucket: string;
  size: string;
}

/**
 * Detects `gs://` uri.
 * @param uri
 * @returns
 */
export const isGoogleCloudStorageUri = (uri: string) =>
  parseUrl(uri, window.location.origin).protocol === 'gs:';

/**
 * Extracts bucket and prefix from `gs://` URIs
 * @param uri
 * @returns
 */
export const extractBucketAndPrefixFromGsUri = (uri: string) => {
  const { hostname: bucket, pathname } = parseUrl(uri);
  // drop the leading forward slash
  const objectName = pathname.replace(/^\//, '');
  return [bucket, objectName] as const;
};

export type ObjectAvailableCallback = (object: GcsObject) => void;

/**
 * Gets all objects from a given gs:// URI.
 *
 * Remote object-store expansion is intentionally blocked in the clinical
 * desktop. Import DICOM/STL/OBJ/PLY through local paths or Mesh Vault handles.
 *
 * @param gsUri
 * @param onObjectAvailable
 * @returns
 */
export const getObjectsFromGsUri = async (
  gsUri: string,
  _onObjectAvailable: ObjectAvailableCallback = () => {}
) => {
  const [bucketName, objPrefix] = extractBucketAndPrefixFromGsUri(gsUri);
  throw new Error(`Offline clinical mode blocks remote object-store imports: ${bucketName}/${objPrefix}`);
};
