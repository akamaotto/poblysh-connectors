'use client';

import { useState, useMemo } from 'react';
import { Search } from 'lucide-react';

/**
 * SearchableList component.
 * Provides a searchable, filterable list with customizable item rendering.
 */
export interface SearchableListProps<T> {
  items: T[];
  getItemId: (item: T) => string;
  getItemSearchText: (item: T) => string;
  renderItem: (item: T) => React.ReactNode;
  placeholder?: string;
  emptyText?: string;
  noResultsText?: string;
  className?: string;
  searchClassName?: string;
  listClassName?: string;
}

export function SearchableList<T>({
  items,
  getItemId,
  getItemSearchText,
  renderItem,
  placeholder = 'Search...',
  emptyText = 'No items to display',
  noResultsText = 'No results found',
  className = '',
  searchClassName = '',
  listClassName = '',
}: SearchableListProps<T>) {
  const [searchTerm, setSearchTerm] = useState('');

  const filteredItems = useMemo(() => {
    if (!searchTerm.trim()) {
      return items;
    }

    const lowercasedSearchTerm = searchTerm.toLowerCase();
    return items.filter(item =>
      getItemSearchText(item).toLowerCase().includes(lowercasedSearchTerm)
    );
  }, [items, searchTerm, getItemSearchText]);

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Search Input */}
      <div className="relative">
        <label htmlFor="search-input" className="sr-only">
          Search items
        </label>
        <Search
          className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400"
          aria-hidden="true"
        />
        <input
          id="search-input"
          type="search"
          placeholder={placeholder}
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className={`w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black ${searchClassName}`}
          aria-describedby="search-help"
        />
        <div id="search-help" className="sr-only">
          Search through items by typing in the search box
        </div>
      </div>

      {/* Results Count */}
      <div className="text-sm text-gray-600" role="status" aria-live="polite">
        {items.length === 0 ? (
          <span>{emptyText}</span>
        ) : (
          <span>
            Showing {filteredItems.length} of {items.length} items
            {searchTerm && ` matching "${searchTerm}"`}
          </span>
        )}
      </div>

      {/* Filtered Items */}
      {filteredItems.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          {items.length === 0 ? emptyText : noResultsText}
        </div>
      ) : (
        <ul className={`space-y-2 ${listClassName}`} role="list">
          {filteredItems.map(item => (
            <li key={getItemId(item)} role="listitem">
              {renderItem(item)}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}