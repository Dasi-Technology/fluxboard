/**
 * Cursor component for displaying remote user cursors.
 *
 * Shows an SVG cursor icon with the user's color and username label.
 */

import React, { useEffect, useState } from "react";

export interface CursorProps {
  userId: number;
  username: string;
  color: [number, number, number];
  x: number; // Normalized 0-1
  y: number; // Normalized 0-1
  containerRef: React.RefObject<HTMLElement>;
}

/**
 * Convert RGB array to CSS color string
 */
function rgbToString(color: [number, number, number]): string {
  return `rgb(${color[0]}, ${color[1]}, ${color[2]})`;
}

/**
 * Remote user cursor component.
 *
 * Features:
 * - SVG cursor icon with user color
 * - Username label with background
 * - Smooth position transitions
 * - Positioning relative to container bounds
 */
export function Cursor({
  userId,
  username,
  color,
  x,
  y,
  containerRef,
}: CursorProps) {
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isVisible, setIsVisible] = useState(false);

  // Update position when coordinates or container changes
  useEffect(() => {
    if (!containerRef.current) {
      setIsVisible(false);
      return;
    }

    const container = containerRef.current;
    const rect = container.getBoundingClientRect();

    // Calculate absolute position within container
    const pixelX = x * rect.width;
    const pixelY = y * rect.height;

    // Check if cursor is within visible bounds (with some margin)
    const margin = 50;
    const visible =
      pixelX >= -margin &&
      pixelX <= rect.width + margin &&
      pixelY >= -margin &&
      pixelY <= rect.height + margin;

    setIsVisible(visible);
    setPosition({ x: pixelX, y: pixelY });
  }, [x, y, containerRef]);

  if (!isVisible) {
    return null;
  }

  const colorString = rgbToString(color);

  return (
    <div
      className="absolute pointer-events-none z-50 transition-all duration-100 ease-out"
      style={{
        left: 0,
        top: 0,
        transform: `translate(${position.x}px, ${position.y}px)`,
      }}
    >
      {/* Cursor SVG - Inverted */}
      <svg
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{
          filter: "drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2))",
          transform: "scaleX(-1)",
        }}
      >
        <path
          d="M5.65376 12.3673L8.97526 15.6888L10.8632 21.2522C11.0405 21.7948 11.7654 21.8936 12.0808 21.4208L20.4276 8.5978C20.7365 8.13513 20.3755 7.52295 19.8111 7.66884L5.24447 11.3641C4.68445 11.5095 4.62645 12.2633 5.65376 12.3673Z"
          fill={colorString}
          stroke="white"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>

      {/* Username label */}
      <div
        className="ml-6 -mt-2 px-2 py-1 rounded text-xs font-medium text-white whitespace-nowrap"
        style={{
          backgroundColor: colorString,
          boxShadow: "0 2px 4px rgba(0, 0, 0, 0.2)",
        }}
      >
        {username}
      </div>
    </div>
  );
}
