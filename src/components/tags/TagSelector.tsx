import { useState } from 'react';
import { TagChip } from './TagChip';
import { Input } from '../ui/Input';
import { useTagsStore, TagWithCount } from '../../stores/tags';
import { Tag } from '../../stores/atoms';

interface TagSelectorProps {
  selectedTags: Tag[];
  onTagsChange: (tags: Tag[]) => void;
}

export function TagSelector({ selectedTags, onTagsChange }: TagSelectorProps) {
  const { tags, createTag } = useTagsStore();
  const [inputValue, setInputValue] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  // Flatten the tag tree for searching
  const flattenTags = (tags: TagWithCount[]): Tag[] => {
    return tags.reduce<Tag[]>((acc, tag) => {
      acc.push({
        id: tag.id,
        name: tag.name,
        parent_id: tag.parent_id,
        created_at: tag.created_at,
      });
      if (tag.children) {
        acc.push(...flattenTags(tag.children));
      }
      return acc;
    }, []);
  };

  const allTags = flattenTags(tags);
  const selectedTagIds = new Set(selectedTags.map((t) => t.id));

  // Filter tags based on input
  const filteredTags = allTags.filter(
    (tag) =>
      !selectedTagIds.has(tag.id) &&
      tag.name.toLowerCase().includes(inputValue.toLowerCase())
  );

  const handleAddTag = (tag: Tag) => {
    onTagsChange([...selectedTags, tag]);
    setInputValue('');
  };

  const handleRemoveTag = (tagId: string) => {
    onTagsChange(selectedTags.filter((t) => t.id !== tagId));
  };

  const handleCreateTag = async () => {
    if (!inputValue.trim() || isCreating) return;
    
    setIsCreating(true);
    try {
      const newTag = await createTag(inputValue.trim());
      onTagsChange([...selectedTags, newTag]);
      setInputValue('');
    } catch (error) {
      console.error('Failed to create tag:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const showCreateOption =
    inputValue.trim() &&
    !allTags.some((t) => t.name.toLowerCase() === inputValue.toLowerCase());

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-[#dcddde]">Tags</label>
      
      {/* Selected tags */}
      {selectedTags.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mb-2">
          {selectedTags.map((tag) => (
            <TagChip
              key={tag.id}
              name={tag.name}
              size="md"
              onRemove={() => handleRemoveTag(tag.id)}
            />
          ))}
        </div>
      )}

      {/* Input */}
      <div className="relative">
        <Input
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          placeholder="Search or create tags..."
          onKeyDown={(e) => {
            if (e.key === 'Enter' && showCreateOption) {
              e.preventDefault();
              handleCreateTag();
            }
          }}
        />

        {/* Dropdown */}
        {inputValue && (filteredTags.length > 0 || showCreateOption) && (
          <div className="absolute z-10 w-full mt-1 bg-[#2d2d2d] border border-[#3d3d3d] rounded-md shadow-lg max-h-48 overflow-y-auto">
            {filteredTags.map((tag) => (
              <button
                key={tag.id}
                onClick={() => handleAddTag(tag)}
                className="w-full px-3 py-2 text-left text-sm text-[#dcddde] hover:bg-[#3d3d3d] transition-colors"
              >
                {tag.name}
              </button>
            ))}
            {showCreateOption && (
              <button
                onClick={handleCreateTag}
                disabled={isCreating}
                className="w-full px-3 py-2 text-left text-sm text-[#7c3aed] hover:bg-[#3d3d3d] transition-colors flex items-center gap-2"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
                Create "{inputValue}"
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

