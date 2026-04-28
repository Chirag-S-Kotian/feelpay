import { Maximize2, X, ZoomIn, ZoomOut } from 'lucide-react';
import { useMemo, useState } from 'react';

export type DocumentViewResponse = {
  document_id: string;
  document_type: string;
  content_type: string;
  content: string;
  page_count?: number | null;
  metadata: {
    file_name: string;
    file_size: number;
    status: string;
    created_at: string;
  };
};

type Props = {
  document: DocumentViewResponse | null;
  onClose: () => void;
};

export default function DocumentViewer({ document, onClose }: Props) {
  const [zoom, setZoom] = useState(100);
  const [fullscreen, setFullscreen] = useState(false);
  const src = useMemo(() => {
    if (!document) return '';
    return `data:${document.content_type};base64,${document.content}`;
  }, [document]);

  if (!document) return null;

  const isPdf = document.content_type.includes('pdf');
  const isImage = /^image\/(jpeg|png|webp|gif)$/.test(document.content_type);

  return (
    <div className={`modal-backdrop ${fullscreen ? 'fullscreen' : ''}`}>
      <section className="document-viewer" aria-label="Document viewer">
        <header className="viewer-toolbar">
          <div>
            <strong>{document.metadata.file_name}</strong>
            <span>{document.document_type}</span>
          </div>
          <div className="viewer-actions">
            <button onClick={() => setZoom(z => Math.max(25, z - 25))} title="Zoom out">
              <ZoomOut size={18} />
            </button>
            <input
              aria-label="Zoom"
              type="range"
              min="25"
              max="400"
              step="25"
              value={zoom}
              onChange={event => setZoom(Number(event.target.value))}
            />
            <button onClick={() => setZoom(z => Math.min(400, z + 25))} title="Zoom in">
              <ZoomIn size={18} />
            </button>
            <button onClick={() => setFullscreen(v => !v)} title="Fullscreen">
              <Maximize2 size={18} />
            </button>
            <button onClick={onClose} title="Close">
              <X size={18} />
            </button>
          </div>
        </header>
        <div className="viewer-body">
          {isPdf && (
            <iframe
              title={document.metadata.file_name}
              src={src}
              style={{ width: `${zoom}%`, height: `${zoom}%` }}
            />
          )}
          {isImage && <img src={src} alt={document.metadata.file_name} style={{ width: `${zoom}%` }} />}
          {!isPdf && !isImage && <p>Unsupported preview type.</p>}
        </div>
      </section>
    </div>
  );
}
