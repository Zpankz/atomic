import { useState, MouseEvent } from 'react';
import { TagWithCount } from '../../stores/tags';

interface TagNodeProps {
  tag: TagWithCount;
  level: number;
  selectedTagId: string | null;
  onSelect: (tagId: string) => void;
  onContextMenu: (e: MouseEvent, tag: TagWithCount) => void;
}

export function TagNode({ tag, level, selectedTagId, onSelect, onContextMenu }: TagNodeProps) {
  const [isExpanded, setIsExpanded] = useState(true);
  const hasChildren = tag.children && tag.children.length > 0;
  const isSelected = selectedTagId === tag.id;

  const handleToggle = (e: MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(!isExpanded);
  };

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    onContextMenu(e, tag);
  };

  return (
    <div>
      <div
        className={`flex items-center gap-1 px-2 py-1.5 rounded-md cursor-pointer transition-colors ${
          isSelected
            ? 'bg-[#7c3aed]/20 text-[#dcddde]'
            : 'text-[#888888] hover:bg-[#2d2d2d] hover:text-[#dcddde]'
        }`}
        style={{ paddingLeft: `${8 + level * 16}px` }}
        onClick={() => onSelect(tag.id)}
        onContextMenu={handleContextMenu}
      >
        {hasChildren ? (
          <button
            onClick={handleToggle}
            className="w-4 h-4 flex items-center justify-center text-[#888888] hover:text-[#dcddde]"
          >
            <svg
              className={`w-3 h-3 transition-transform ${isExpanded ? 'rotate-90' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </button>
        ) : (
          <span className="w-4" />
        )}
        <span className="flex-1 truncate text-sm">{tag.name}</span>
        <span className="text-xs text-[#666666] tabular-nums">{tag.atom_count}</span>
      </div>
      {hasChildren && isExpanded && (
        <div>
          {tag.children.map((child) => (
            <TagNode
              key={child.id}
              tag={child}
              level={level + 1}
              selectedTagId={selectedTagId}
              onSelect={onSelect}
              onContextMenu={onContextMenu}
            />
          ))}
        </div>
      )}
    </div>
  );
}

