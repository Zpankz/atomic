import { useEffect, useRef, useState } from 'react';
import { useUIStore } from '../../stores/ui';
import { getGlobalCanvas, type CanvasAtomPosition } from '../../lib/api';
import Graph from 'graphology';
import Sigma from 'sigma';

function stringToHue(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  return Math.abs(hash % 360);
}

function hueToRgb(hue: number, s = 50, l = 55): string {
  const h = hue / 360;
  const sat = s / 100;
  const lig = l / 100;
  const a = sat * Math.min(lig, 1 - lig);
  const f = (n: number) => {
    const k = (n + h * 12) % 12;
    return lig - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
  };
  const r = Math.round(f(0) * 255);
  const g = Math.round(f(8) * 255);
  const b = Math.round(f(4) * 255);
  return `rgb(${r},${g},${b})`;
}

export function SigmaCanvas() {
  const openDrawer = useUIStore(s => s.openDrawer);
  const containerRef = useRef<HTMLDivElement>(null);
  const sigmaRef = useRef<Sigma | null>(null);
  const [atoms, setAtoms] = useState<CanvasAtomPosition[] | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Fetch global canvas data
  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    setError(null);

    getGlobalCanvas()
      .then((data) => {
        if (!cancelled) {
          setAtoms(data);
          setIsLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message || 'Failed to load canvas');
          setIsLoading(false);
        }
      });

    return () => { cancelled = true; };
  }, []);

  // Create Sigma graph when atoms are loaded
  useEffect(() => {
    const container = containerRef.current;
    if (!container || !atoms || atoms.length === 0) return;

    // Clean up previous instance
    if (sigmaRef.current) {
      sigmaRef.current.kill();
      sigmaRef.current = null;
    }

    const graph = new Graph();

    // Scale positions to a reasonable coordinate space
    const scale = 500;
    for (const atom of atoms) {
      const hue = atom.primary_tag ? stringToHue(atom.primary_tag) : 220;
      graph.addNode(atom.atom_id, {
        x: atom.x * scale,
        y: atom.y * scale,
        size: 4,
        color: hueToRgb(hue),
        label: atom.title || atom.atom_id.substring(0, 8),
      });
    }

    const sigma = new Sigma(graph, container, {
      renderLabels: true,
      labelRenderedSizeThreshold: 8,
      labelSize: 12,
      labelColor: { color: '#b0b0b0' },
      defaultEdgeColor: '#333',
      defaultNodeColor: '#666',
      minCameraRatio: 0.01,
      maxCameraRatio: 10,
      stagePadding: 40,
    });

    sigmaRef.current = sigma;

    // Click handler
    sigma.on('clickNode', ({ node }) => {
      openDrawer('viewer', node);
    });

    return () => {
      sigma.kill();
      sigmaRef.current = null;
    };
  }, [atoms, openDrawer]);

  return (
    <div className="flex flex-col h-full w-full">
      <div className="flex-1 relative overflow-hidden bg-[#1a1a1a]">
        {isLoading && (
          <div className="absolute inset-0 flex items-center justify-center z-10">
            <div className="flex items-center gap-2 text-[var(--color-text-secondary)]">
              <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              <span className="text-sm">Computing layout...</span>
            </div>
          </div>
        )}

        {error && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center text-[var(--color-text-secondary)]">
              <p className="text-lg mb-2">Error loading canvas</p>
              <p className="text-sm">{error}</p>
            </div>
          </div>
        )}

        {!isLoading && atoms && atoms.length === 0 && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center text-[var(--color-text-secondary)]">
              <p className="text-lg mb-2">No atoms with embeddings</p>
              <p className="text-sm">Create some atoms and wait for embeddings to generate</p>
            </div>
          </div>
        )}

        <div
          ref={containerRef}
          className="w-full h-full"
          style={{ minHeight: 200 }}
        />
      </div>
    </div>
  );
}
