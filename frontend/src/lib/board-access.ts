/**
 * Board access control utilities
 *
 * Determines whether a user can edit a board based on:
 * - Whether the board is locked
 * - Whether the user is the board owner (has the password)
 */

import { isBoardOwner } from "./board-passwords";

/**
 * Check if a board is in read-only mode for the current user
 *
 * A board is read-only if:
 * - The board is locked AND
 * - The user is not the owner (doesn't have the password)
 *
 * @param shareToken - The board's share token
 * @param isLocked - Whether the board is locked
 * @returns True if the board is read-only for the current user
 */
export function isBoardReadOnly(
  shareToken: string,
  isLocked: boolean
): boolean {
  if (!isLocked) {
    // Board is not locked, everyone can edit
    return false;
  }

  // Board is locked, only the owner can edit
  return !isBoardOwner(shareToken);
}

/**
 * Check if a user can edit a board
 *
 * @param shareToken - The board's share token
 * @param isLocked - Whether the board is locked
 * @returns True if the user can edit the board
 */
export function canEditBoard(shareToken: string, isLocked: boolean): boolean {
  return !isBoardReadOnly(shareToken, isLocked);
}
