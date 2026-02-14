import { memo, useMemo } from 'react';
import { AtomSummary } from '../../stores/atoms';

interface AtomNodeProps {
  atom: AtomSummary;
  x: number;
  y: number;
  isFaded: boolean;
  isHub?: boolean;
  isHighlighted?: boolean;
  connectionCount?: number;
  onClick: (atomId: string) => void;
  atomId: string;
}

// Generate a consistent color from a string (tag name)
interface TagColor {
  h: number;
  s: number;
  l: number;
  hsl: string;
}

function stringToColor(str: string): TagColor {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }

  // Convert to HSL for better control over saturation and lightness
  const h = Math.abs(hash % 360);
  const s = 50 + (hash % 20); // 50-70% saturation
  const l = 45 + (hash % 10); // 45-55% lightness

  return { h, s, l, hsl: `hsl(${h}, ${s}%, ${l}%)` };
}

function colorWithAlpha(color: TagColor, alpha: number): string {
  return `hsla(${color.h}, ${color.s}%, ${color.l}%, ${alpha})`;
}

export const AtomNode = memo(function AtomNode({
  atom,
  x,
  y,
  isFaded,
  isHub = false,
  isHighlighted = false,
  connectionCount = 0,
  onClick,
  atomId,
}: AtomNodeProps) {
  // Use snippet (already stripped of markdown), truncate to ~50 chars
  const displayContent = useMemo(() => {
    const text = atom.snippet || 'Empty atom';
    return text.length > 50 ? text.substring(0, 47) + '...' : text;
  }, [atom.snippet]);

  // Get color from primary tag
  const tagColor = useMemo(() => {
    if (atom.tags.length === 0) return null;
    return stringToColor(atom.tags[0].name);
  }, [atom.tags]);

  // Calculate node width based on connection count (for hubs)
  const nodeWidth = isHub ? 180 : 160;

  return (
    <div
      className={`absolute cursor-pointer select-none transition-all duration-150 ${
        isFaded ? 'opacity-20 pointer-events-none' : 'opacity-100'
      }`}
      style={{
        left: x,
        top: y,
        transform: 'translate(-50%, -50%)',
        width: `${nodeWidth}px`,
      }}
      onClick={() => onClick(atomId)}
    >
      <div
        className={`
          bg-[var(--color-bg-card)] border rounded-md px-3 py-2
          hover:scale-[1.02] transition-all duration-150
          relative overflow-hidden
          ${isHighlighted
            ? 'border-[var(--color-success)] shadow-[0_0_20px_rgb(var(--color-success-rgb) / 0.5)] animate-pulse ring-2 ring-[var(--color-success)] ring-opacity-50'
            : isHub
            ? 'border-[var(--color-accent)] shadow-[0_0_12px_rgb(var(--color-accent-rgb) / 0.3)]'
            : 'border-[var(--color-border)] hover:border-[var(--color-border-hover)]'}
        `}
      >
        {/* Tag color indicator */}
        {tagColor && (
          <div
            className="absolute left-0 top-0 bottom-0 w-1 rounded-l"
            style={{ backgroundColor: tagColor.hsl }}
          />
        )}

        {/* Hub indicator */}
        {isHub && (
          <div className="absolute top-1 right-1">
            <div
              className="w-2 h-2 rounded-full bg-[var(--color-accent)] animate-pulse"
              title={`Hub: ${connectionCount} connections`}
            />
          </div>
        )}

        <p className={`text-sm text-[var(--color-text-primary)] line-clamp-2 break-words ${isHub ? 'font-medium' : ''}`}>
          {displayContent}
        </p>

        {/* Show tag count indicator */}
        {atom.tags.length > 0 && (
          <div className="flex items-center gap-1 mt-1.5">
            <span
              className="text-[10px] px-1.5 py-0.5 rounded"
              style={{
                backgroundColor: tagColor ? colorWithAlpha(tagColor, 0.35) : 'var(--color-bg-hover)',
                color: 'var(--color-text-primary)'
              }}
            >
              {atom.tags[0].name.length > 12
                ? atom.tags[0].name.substring(0, 10) + '...'
                : atom.tags[0].name}
            </span>
            {atom.tags.length > 1 && (
              <span className="text-[10px] text-[var(--color-text-tertiary)]">
                +{atom.tags.length - 1}
              </span>
            )}
            {/* Show connection count for hubs */}
            {isHub && connectionCount > 0 && (
              <span className="text-[10px] text-[var(--color-accent)] ml-auto">
                {connectionCount}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
});
