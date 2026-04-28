import { useState, useEffect } from 'react';
import { apiRequest } from '../hooks/useApi';
import { Trash2, Edit2, Clock, User, AlertCircle } from 'lucide-react';

interface MerchantNote {
  id: string;
  merchant_id: string;
  content: string;
  author_id: string;
  author_name: string;
  created_at: string;
  updated_at: string;
  is_deleted: boolean;
}

interface MerchantNoteHistory {
  id: string;
  note_id: string;
  previous_content: string;
  changed_by: string;
  changed_at: string;
}

interface MerchantNotesProps {
  merchantId: string;
}

export default function MerchantNotes({ merchantId }: MerchantNotesProps) {
  const [notes, setNotes] = useState<MerchantNote[]>([]);
  const [noteHistory, setNoteHistory] = useState<Record<string, MerchantNoteHistory[]>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newNoteContent, setNewNoteContent] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [editingNoteId, setEditingNoteId] = useState<string | null>(null);
  const [editContent, setEditContent] = useState('');
  const [historyLoading, setHistoryLoading] = useState<Record<string, boolean>>({});

  useEffect(() => {
    fetchNotes();
  }, [merchantId]);

  const fetchNotes = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiRequest<MerchantNote[]>(
        'GET',
        `/api/admin/merchants/${merchantId}/notes`
      );
      setNotes(data || []);
    } catch (err) {
      console.error('Failed to fetch notes:', err);
      setError('Failed to load notes');
    } finally {
      setLoading(false);
    }
  };

  const fetchNoteHistory = async (noteId: string) => {
    if (noteHistory[noteId]) return; // Already loaded

    try {
      setHistoryLoading((prev) => ({ ...prev, [noteId]: true }));
      const history = await apiRequest<MerchantNoteHistory[]>(
        'GET',
        `/api/admin/notes/${noteId}/history`
      );
      setNoteHistory((prev) => ({ ...prev, [noteId]: history || [] }));
    } catch (err) {
      console.error('Failed to fetch note history:', err);
    } finally {
      setHistoryLoading((prev) => ({ ...prev, [noteId]: false }));
    }
  };

  const createNote = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newNoteContent.trim() || newNoteContent.length > 5000) return;

    try {
      setSubmitting(true);
      setError(null);
      await apiRequest<MerchantNote>('POST', `/api/admin/merchants/${merchantId}/notes`, {
        content: newNoteContent,
      });
      setNewNoteContent('');
      fetchNotes();
    } catch (err) {
      console.error('Failed to create note:', err);
      setError('Failed to create note');
    } finally {
      setSubmitting(false);
    }
  };

  const updateNote = async (noteId: string) => {
    if (!editContent.trim() || editContent.length > 5000) return;

    try {
      setSubmitting(true);
      setError(null);
      await apiRequest<MerchantNote>(`PUT`, `/api/admin/notes/${noteId}`, {
        content: editContent,
      });
      setEditingNoteId(null);
      setEditContent('');
      fetchNotes();
    } catch (err) {
      console.error('Failed to update note:', err);
      setError('Failed to update note');
    } finally {
      setSubmitting(false);
    }
  };

  const deleteNote = async (noteId: string) => {
    if (!confirm('Are you sure you want to delete this note?')) return;

    try {
      setError(null);
      await apiRequest('DELETE', `/api/admin/notes/${noteId}`);
      fetchNotes();
    } catch (err) {
      console.error('Failed to delete note:', err);
      setError('Failed to delete note');
    }
  };

  const startEditing = (note: MerchantNote) => {
    setEditingNoteId(note.id);
    setEditContent(note.content);
  };

  const cancelEditing = () => {
    setEditingNoteId(null);
    setEditContent('');
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('en-IN', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const renderMarkdown = (content: string) => {
    // Simple markdown rendering: bold, italic, code, links
    let html = content
      // Escape HTML first
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      // Bold: **text** or __text__
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/__(.+?)__/g, '<strong>$1</strong>')
      // Italic: *text* or _text_
      .replace(/\*(.+?)\*/g, '<em>$1</em>')
      .replace(/_(.+?)_/g, '<em>$1</em>')
      // Code: `code`
      .replace(/`(.+?)`/g, '<code style="background: #f3f4f6; padding: 2px 4px; border-radius: 3px; font-family: monospace;">$1</code>')
      // Links: [text](url)
      .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer" style="color: #3b82f6; text-decoration: underline;">$1</a>')
      // Line breaks
      .replace(/\n/g, '<br />');

    return { __html: html };
  };

  if (loading) {
    return (
      <div className="card">
        <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 16 }}>Internal Notes</h3>
        <div style={{ color: 'var(--fp-text-muted)', textAlign: 'center', padding: 20 }}>
          Loading notes...
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 16 }}>Internal Notes</h3>

      {error && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            padding: 12,
            background: '#fef2f2',
            border: '1px solid #fecaca',
            borderRadius: 6,
            color: '#dc2626',
            marginBottom: 16,
          }}
        >
          <AlertCircle size={16} />
          <span>{error}</span>
        </div>
      )}

      {/* Note Creation Form */}
      <form onSubmit={createNote} style={{ marginBottom: 24 }}>
        <div style={{ marginBottom: 8 }}>
          <textarea
            className="form-input"
            placeholder="Add a note... (Markdown supported)"
            value={newNoteContent}
            onChange={(e) => setNewNoteContent(e.target.value)}
            maxLength={5000}
            rows={4}
            style={{ resize: 'vertical', fontFamily: 'inherit' }}
          />
          <div
            style={{
              fontSize: 12,
              color: newNoteContent.length > 4500 ? '#dc2626' : 'var(--fp-text-muted)',
              textAlign: 'right',
              marginTop: 4,
            }}
          >
            {newNoteContent.length}/5000
          </div>
        </div>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={submitting || !newNoteContent.trim()}
        >
          {submitting ? 'Adding...' : 'Add Note'}
        </button>
      </form>

      {/* Notes List */}
      {notes.length === 0 ? (
        <div style={{ color: 'var(--fp-text-muted)', textAlign: 'center', padding: 20 }}>
          No notes yet
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          {notes.map((note) => (
            <div
              key={note.id}
              style={{
                padding: 16,
                background: '#f9fafb',
                borderRadius: 8,
                border: '1px solid #e5e7eb',
              }}
            >
              {editingNoteId === note.id ? (
                // Edit Mode
                <div>
                  <textarea
                    className="form-input"
                    value={editContent}
                    onChange={(e) => setEditContent(e.target.value)}
                    maxLength={5000}
                    rows={4}
                    style={{ resize: 'vertical', marginBottom: 12 }}
                  />
                  <div style={{ display: 'flex', gap: 8 }}>
                    <button
                      className="btn btn-sm btn-primary"
                      onClick={() => updateNote(note.id)}
                      disabled={submitting}
                    >
                      Save
                    </button>
                    <button className="btn btn-sm btn-secondary" onClick={cancelEditing}>
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                // View Mode
                <>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'flex-start',
                      marginBottom: 8,
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                      <User size={14} style={{ color: 'var(--fp-text-muted)' }} />
                      <span style={{ fontWeight: 500, fontSize: 14 }}>{note.author_name}</span>
                      <span style={{ color: 'var(--fp-text-muted)', fontSize: 13 }}>
                        {formatDate(note.created_at)}
                      </span>
                      {note.updated_at !== note.created_at && (
                        <span
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: 4,
                            color: 'var(--fp-text-muted)',
                            fontSize: 12,
                          }}
                        >
                          (edited)
                        </span>
                      )}
                    </div>
                    <div style={{ display: 'flex', gap: 4 }}>
                      <button
                        className="btn btn-sm btn-secondary"
                        title="Edit note"
                        onClick={() => startEditing(note)}
                        style={{ padding: '4px 8px' }}
                      >
                        <Edit2 size={14} />
                      </button>
                      <button
                        className="btn btn-sm btn-danger"
                        title="Delete note"
                        onClick={() => deleteNote(note.id)}
                        style={{ padding: '4px 8px' }}
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>

                  <div
                    style={{
                      fontSize: 14,
                      lineHeight: 1.6,
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-word',
                    }}
                    dangerouslySetInnerHTML={renderMarkdown(note.content)}
                  />

                  {/* Edit History on Hover */}
                  <div
                    style={{
                      marginTop: 12,
                      paddingTop: 12,
                      borderTop: '1px solid #e5e7eb',
                    }}
                  >
                    <button
                      className="btn btn-sm btn-secondary"
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 6,
                        fontSize: 12,
                        color: 'var(--fp-text-muted)',
                      }}
                      onClick={() => fetchNoteHistory(note.id)}
                      onMouseEnter={() => fetchNoteHistory(note.id)}
                    >
                      <Clock size={12} />
                      {historyLoading[note.id] ? 'Loading history...' : 'View edit history'}
                    </button>

                    {(noteHistory[note.id]?.length ?? 0) > 0 && (
                      <div
                        style={{
                          marginTop: 12,
                          padding: 12,
                          background: '#fff',
                          borderRadius: 6,
                          border: '1px solid #e5e7eb',
                        }}
                      >
                        <div style={{ fontSize: 12, fontWeight: 600, marginBottom: 8 }}>
                          Edit History
                        </div>
                        {noteHistory[note.id]?.map((history) => (
                          <div
                            key={history.id}
                            style={{
                              padding: '8px 0',
                              borderBottom: '1px solid #f3f4f6',
                              fontSize: 13,
                            }}
                          >
                            <div style={{ color: 'var(--fp-text-muted)', marginBottom: 4 }}>
                              {formatDate(history.changed_at)}
                            </div>
                            <div
                              style={{
                                background: '#fef3c7',
                                padding: 8,
                                borderRadius: 4,
                                fontSize: 12,
                                whiteSpace: 'pre-wrap',
                              }}
                            >
                              {history.previous_content}
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
