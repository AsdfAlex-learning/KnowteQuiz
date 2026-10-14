import type { ConvertOutcome, DocumentStatus } from '../types/docConvert';
import { invoke, isTauri } from './tauri';
import { throwHttpError, parseJsonResponse } from './http';

/**
 * Convert a PDF/DOCX/PPTX/HTML document into a Markdown note next to its
 * source. The produced note and its `.images/` directory are plain files in the
 * vault, so the note tree picks them up on the next scan.
 */
export async function convertDocument(sourcePath: string): Promise<ConvertOutcome> {
  if (isTauri()) {
    return invoke<ConvertOutcome>('convert_document', { sourcePath });
  }
  const res = await fetch('/api/documents/convert', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source_path: sourcePath }),
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ConvertOutcome>(res);
}

/** Whether a document is converted, and whether its note is stale. */
export async function getDocumentStatus(sourcePath: string): Promise<DocumentStatus> {
  if (isTauri()) {
    return invoke<DocumentStatus>('get_document_status', { sourcePath });
  }
  const query = new URLSearchParams({ path: sourcePath });
  const res = await fetch(`/api/documents/status?${query.toString()}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<DocumentStatus>(res);
}
