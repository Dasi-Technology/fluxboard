/**
 * Board access control utilities
 *
 * Determines whether a user can edit a board based on:
 * - Whether the board is locked
 * - Whether the user is the board owner (has the password)
 */

import { isBoardOwner } from "./board-passwords";

/**
 * Error thrown when a board operation is not allowed
 */
export class BoardAccessError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BoardAccessError";
  }
}

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

/**
 * Validate that a board operation is allowed
 *
 * Throws an error if the board is locked and the user doesn't have the password.
 * Use this before making API calls that modify board data.
 *
 * @param shareToken - The board's share token
 * @param isLocked - Whether the board is locked
 * @throws {BoardAccessError} If the operation is not allowed
 *
 * @example
 * ```typescript
 * validateBoardOperation(board.share_token, board.is_locked);
 * await createCard(columnId, title, position, board.share_token);
 * ```
 */
export function validateBoardOperation(
  shareToken: string,
  isLocked: boolean
): void {
  if (isBoardReadOnly(shareToken, isLocked)) {
    throw new BoardAccessError(
      "This board is locked. Only the board owner can make changes."
    );
  }
}
