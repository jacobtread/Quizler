// Module for handling file uploading and downloading

/**
 * Triggers the upload of a single file returning
 * the promise to when the file is provided
 *
 * Null will be returned if nothing was uploaded
 *
 * @returns The promise to the file
 */
export function acceptUpload(
  accept: string | undefined = undefined
): Promise<File | null> {
  // Create temporary file input
  const element: HTMLInputElement = document.createElement("input");
  element.type = "file";
  element.hidden = true;

  if (accept !== undefined) element.accept = accept;

  document.body.appendChild(element);
  return new Promise((resolve, reject) => {
    // Handle file upload
    element.onchange = () => {
      if (element.files === null) return reject();
      const file: File | null = element.files.item(0);
      resolve(file);
    };

    // Handle closing without upload
    element.oncancel = () => resolve(null);

    element.click();

    // Remove the temp element
    document.body.removeChild(element);
  });
}

/**
 * Triggers the upload of many files returning
 * the promise to when the files are provided
 *
 * Null will be returned if nothing was uploaded
 *
 * @returns The promise to the file list
 */
export function acceptUploadMany(
  accept: string | undefined = undefined
): Promise<FileList | null> {
  // Create temporary file input
  const element: HTMLInputElement = document.createElement("input");
  element.type = "file";
  element.multiple = true;
  element.hidden = true;

  if (accept !== undefined) element.accept = accept;

  return new Promise((resolve, reject) => {
    // Append temp element
    document.body.appendChild(element);

    // Handle file upload
    element.onchange = () => {
      if (element.files === null) return reject();
      resolve(element.files);
    };

    // Handle closing without upload
    element.oncancel = () => resolve(null);
    element.click();

    // Remove the temp element
    document.body.removeChild(element);
  });
}

/**
 * Triggers a file download for a file with the
 * provided file name containing the blob data
 *
 * Creates a temporary link element with a object URL and
 * clicks it to begin the download
 *
 * @param fileName The name for the file download
 * @param blob     The blob file contents
 */
export function startDownload(fileName: string, blob: Blob) {
  const element: HTMLAnchorElement = document.createElement("a");
  element.download = fileName.replace(/[ ^/]/g, "_");
  element.href = URL.createObjectURL(blob);
  element.dataset.downloadurl = [
    "application/json",
    element.download,
    element.href
  ].join(":");
  element.style.display = "none";
  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
}
