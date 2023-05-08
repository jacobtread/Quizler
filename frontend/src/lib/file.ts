// Module for handling file uploading and downloading

/**
 * Triggers the upload of a single file returning
 * the promise to when the file is provided
 *
 * Null will be returned if nothing was uploaded
 *
 * @returns The promise to the file
 */
export function acceptUpload(): Promise<File | null> {
  // Create temporary file input
  const element: HTMLInputElement = document.createElement("input");
  element.type = "file";
  element.hidden = true;
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
export function acceptUploadMany(): Promise<FileList | null> {
  // Create temporary file input
  const element: HTMLInputElement = document.createElement("input");
  element.type = "file";
  element.multiple = true;
  element.hidden = true;

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
