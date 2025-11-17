"use client";

import { useBoardStore } from "@/store/board-store";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Badge } from "@/components/ui/badge";
import { Filter, X } from "lucide-react";
import { cn } from "@/lib/utils";

export function LabelFilter() {
  const board = useBoardStore((state) => state.board);
  const selectedLabelFilter = useBoardStore(
    (state) => state.selectedLabelFilter
  );
  const toggleLabelFilter = useBoardStore((state) => state.toggleLabelFilter);
  const clearLabelFilter = useBoardStore((state) => state.clearLabelFilter);

  if (!board || !board.labels || board.labels.length === 0) {
    return null;
  }

  const hasActiveFilters = selectedLabelFilter.length > 0;

  return (
    <div className="flex items-center gap-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button
            variant={hasActiveFilters ? "default" : "outline"}
            size="sm"
            className="gap-2"
          >
            <Filter className="h-4 w-4" />
            Filter by Label
            {hasActiveFilters && (
              <Badge variant="secondary" className="ml-1 px-1.5 py-0.5 text-xs">
                {selectedLabelFilter.length}
              </Badge>
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-64 p-3" align="start">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h4 className="font-medium text-sm">Filter by Labels</h4>
              {hasActiveFilters && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={clearLabelFilter}
                  className="h-auto p-1 text-xs"
                >
                  Clear all
                </Button>
              )}
            </div>
            <div className="space-y-1.5">
              {board.labels.map((label) => {
                const isSelected = selectedLabelFilter.includes(label.id);
                return (
                  <button
                    key={label.id}
                    onClick={() => toggleLabelFilter(label.id)}
                    className={cn(
                      "w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors",
                      "hover:bg-accent",
                      isSelected && "bg-accent"
                    )}
                  >
                    <div
                      className="w-4 h-4 rounded-sm flex-shrink-0"
                      style={{ backgroundColor: label.color }}
                    />
                    <span className="flex-1 text-left truncate">
                      {label.name}
                    </span>
                    {isSelected && (
                      <div className="w-4 h-4 rounded-sm bg-primary flex items-center justify-center flex-shrink-0">
                        <svg
                          className="w-3 h-3 text-primary-foreground"
                          fill="none"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="2"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                        >
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      </div>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        </PopoverContent>
      </Popover>

      {hasActiveFilters && (
        <div className="flex items-center gap-1.5 flex-wrap">
          {selectedLabelFilter.map((labelId) => {
            const label = board.labels?.find((l) => l.id === labelId);
            if (!label) return null;

            return (
              <Badge
                key={labelId}
                variant="secondary"
                className="gap-1.5 pr-1"
                style={{
                  backgroundColor: `${label.color}20`,
                  borderColor: label.color,
                }}
              >
                <div
                  className="w-2 h-2 rounded-full"
                  style={{ backgroundColor: label.color }}
                />
                <span className="text-xs">{label.name}</span>
                <button
                  onClick={() => toggleLabelFilter(labelId)}
                  className="hover:bg-accent rounded-sm p-0.5"
                >
                  <X className="h-3 w-3" />
                </button>
              </Badge>
            );
          })}
        </div>
      )}
    </div>
  );
}
