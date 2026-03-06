import { useState, useRef, useEffect } from 'react';
import { useDatabasesStore, DatabaseInfo } from '../stores/databases';

export function DatabaseSwitcher() {
  const { databases, activeId, fetchDatabases, switchDatabase, createDatabase, renameDatabase, deleteDatabase } = useDatabasesStore();
  const [isOpen, setIsOpen] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [newName, setNewName] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchDatabases();
  }, [fetchDatabases]);

  // Close dropdown on outside click
  useEffect(() => {
    if (!isOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsOpen(false);
        setIsCreating(false);
        setEditingId(null);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [isOpen]);

  const activeDb = databases.find(d => d.id === activeId);
  const activeName = activeDb?.name ?? 'Database';

  const handleSwitch = async (id: string) => {
    if (id === activeId) return;
    setIsOpen(false);
    await switchDatabase(id);
  };

  const handleCreate = async () => {
    if (!newName.trim()) return;
    await createDatabase(newName.trim());
    setNewName('');
    setIsCreating(false);
  };

  const handleRename = async (id: string) => {
    if (!editName.trim()) return;
    await renameDatabase(id, editName.trim());
    setEditingId(null);
  };

  const handleDelete = async (db: DatabaseInfo) => {
    if (db.is_default) return;
    if (!confirm(`Delete "${db.name}"? This cannot be undone.`)) return;
    await deleteDatabase(db.id);
  };

  if (databases.length === 0) {
    return null;
  }

  return (
    <div className="relative flex-1 min-w-0" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center gap-1.5 px-2 py-1.5 text-xs text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-hover)] rounded transition-colors"
        title={activeName}
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor" className="flex-shrink-0 opacity-60">
          <path d="M8 1L1 4.5 8 8l7-3.5L8 1zM1 8l7 3.5L15 8M1 11.5l7 3.5 7-3.5" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round"/>
        </svg>
        <span className="truncate">{activeName}</span>
        <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor" className="flex-shrink-0 opacity-40">
          <path d="M1 2.5L4 5.5L7 2.5" stroke="currentColor" strokeWidth="1.5" fill="none"/>
        </svg>
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-lg shadow-xl z-50 py-1">
          {databases.map(db => (
            <div
              key={db.id}
              className={`flex items-center gap-2 px-3 py-1.5 text-xs cursor-pointer hover:bg-[var(--color-bg-hover)] ${
                db.id === activeId ? 'text-[var(--color-accent)]' : 'text-[var(--color-text-primary)]'
              }`}
            >
              {editingId === db.id ? (
                <input
                  autoFocus
                  className="flex-1 bg-transparent border border-[var(--color-border)] rounded px-1 py-0.5 text-xs outline-none"
                  value={editName}
                  onChange={e => setEditName(e.target.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') handleRename(db.id);
                    if (e.key === 'Escape') setEditingId(null);
                  }}
                  onBlur={() => setEditingId(null)}
                />
              ) : (
                <>
                  <span
                    className="flex-1 truncate"
                    onClick={() => handleSwitch(db.id)}
                  >
                    {db.name}
                  </span>
                  <button
                    onClick={(e) => { e.stopPropagation(); setEditingId(db.id); setEditName(db.name); }}
                    className="opacity-0 group-hover:opacity-100 hover:text-[var(--color-text-primary)] text-[var(--color-text-tertiary)]"
                    title="Rename"
                  >
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                      <path d="M12.146.854a.5.5 0 0 1 .708 0l2.292 2.292a.5.5 0 0 1 0 .708l-9.5 9.5a.5.5 0 0 1-.168.11l-4 1.5a.5.5 0 0 1-.65-.65l1.5-4a.5.5 0 0 1 .11-.168l9.5-9.5z"/>
                    </svg>
                  </button>
                  {!db.is_default && (
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDelete(db); }}
                      className="opacity-0 group-hover:opacity-100 hover:text-red-400 text-[var(--color-text-tertiary)]"
                      title="Delete"
                    >
                      <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"/>
                        <path fillRule="evenodd" d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1zM4.118 4L4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4H4.118z"/>
                      </svg>
                    </button>
                  )}
                </>
              )}
            </div>
          ))}

          <div className="border-t border-[var(--color-border)] mt-1 pt-1">
            {isCreating ? (
              <div className="px-3 py-1.5">
                <input
                  autoFocus
                  className="w-full bg-transparent border border-[var(--color-border)] rounded px-2 py-1 text-xs outline-none text-[var(--color-text-primary)]"
                  placeholder="Database name..."
                  value={newName}
                  onChange={e => setNewName(e.target.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') handleCreate();
                    if (e.key === 'Escape') { setIsCreating(false); setNewName(''); }
                  }}
                />
              </div>
            ) : (
              <button
                onClick={() => setIsCreating(true)}
                className="w-full text-left px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-text-primary)]"
              >
                + New database
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
