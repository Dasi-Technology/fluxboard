/**
 * Active users component for displaying presence information.
 *
 * Shows a list of currently active users with their avatars and colors.
 */

import React from "react";
import type { User } from "@/hooks/use-presence";

export interface ActiveUsersProps {
  users: Map<number, User>;
  presenceCount: number;
}

/**
 * Convert RGB array to CSS color string
 */
function rgbToString(color: [number, number, number]): string {
  return `rgb(${color[0]}, ${color[1]}, ${color[2]})`;
}

/**
 * Get initials from username
 */
function getInitials(username: string): string {
  const parts = username.trim().split(/\s+/);
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

/**
 * Active users list component.
 *
 * Features:
 * - Compact avatar display with colored borders
 * - Tooltip with full username on hover
 * - Total presence count
 * - Responsive layout
 */
export function ActiveUsers({ users, presenceCount }: ActiveUsersProps) {
  const userArray = Array.from(users.values());

  if (presenceCount === 0) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 z-40 flex items-center gap-2 bg-white rounded-lg shadow-lg border border-gray-200 px-3 py-2">
      {/* Active users avatars */}
      <div className="flex items-center -space-x-2">
        {userArray.slice(0, 5).map((user) => {
          const colorString = rgbToString(user.color);
          const initials = getInitials(user.username);

          return (
            <div
              key={user.userId}
              className="group relative"
              title={user.username}
            >
              <div
                className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-semibold text-white border-2 border-white shadow-sm transition-transform hover:scale-110 hover:z-10"
                style={{
                  backgroundColor: colorString,
                }}
              >
                {initials}
              </div>

              {/* Tooltip */}
              <div className="absolute top-full left-1/2 -translate-x-1/2 mt-2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity">
                {user.username}
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-b-4 border-transparent border-b-gray-900"></div>
              </div>
            </div>
          );
        })}

        {/* Show "+N" if more than 5 users */}
        {userArray.length > 5 && (
          <div className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-semibold bg-gray-200 text-gray-700 border-2 border-white shadow-sm">
            +{userArray.length - 5}
          </div>
        )}
      </div>

      {/* Presence count */}
      <div className="flex items-center gap-1.5 pl-2 border-l border-gray-200">
        <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
        <span className="text-sm font-medium text-gray-700">
          {presenceCount} {presenceCount === 1 ? "viewer" : "viewers"}
        </span>
      </div>
    </div>
  );
}
