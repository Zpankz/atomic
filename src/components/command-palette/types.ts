import { ComponentType, SVGProps } from 'react';

export type CommandCategory = 'navigation' | 'atoms' | 'tags' | 'wiki' | 'utility';

export type PaletteMode = 'commands' | 'search' | 'tags';

export interface Command {
  id: string;
  label: string;
  category: CommandCategory;
  keywords: string[];          // Additional search terms for fuzzy matching
  shortcut?: string;           // Display hint (e.g., "⌘N")
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  action: () => void | Promise<void>;
  isEnabled?: () => boolean;   // Conditional availability
}

export interface CommandGroup {
  category: CommandCategory;
  label: string;
  commands: Command[];
}

export interface FuzzyMatch {
  command: Command;
  score: number;
  matches: number[];  // Indices of matched characters in label
}

export interface SemanticSearchResult {
  id: string;
  content: string;
  source_url: string | null;
  created_at: string;
  updated_at: string;
  embedding_status: string;
  tagging_status: string;
  tags: Array<{
    id: string;
    name: string;
    parent_id: string | null;
    created_at: string;
  }>;
  similarity_score: number;
  matching_chunk_content: string;
  matching_chunk_index: number;
}

export interface TagWithCount {
  id: string;
  name: string;
  parent_id: string | null;
  created_at: string;
  atom_count: number;
  children?: TagWithCount[];
}
