/** Outcome of converting a document into a Markdown note. */
export interface ConvertOutcome {
  source_path: string;
  output_path: string;
  /** Derived image directory, when at least one image was written. */
  image_dir: string | null;
  title: string | null;
  image_count: number;
  warnings: string[];
}

/** Conversion state of a document, for the "converted" / "stale" badge. */
export interface DocumentStatus {
  source_path: string;
  converted: boolean;
  /** The source was modified after its converted note. */
  stale: boolean;
  output_path: string | null;
}
