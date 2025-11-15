/**
 * Recent boards localStorage utility
 * Manages the list of recently visited boards with a maximum of 5 entries
 */

const STORAGE_KEY = "fluxboard_recent_boards";
const MAX_RECENT_BOARDS = 5;

/**
 * Interface representing a recently visited board
 */
export interface RecentBoard {
  shareToken: string;
  name: string;
  visitedAt: string;
}

/**
 * Checks if localStorage is available in the current environment
 */
function isLocalStorageAvailable(): boolean {
  try {
    const test = "__localStorage_test__";
    localStorage.setItem(test, test);
    localStorage.removeItem(test);
    return true;
  } catch (e) {
    return false;
  }
}

/**
 * Retrieves the list of recent boards from localStorage
 * @returns Array of recent boards, sorted by most recent first. Returns empty array if none exist or on error.
 */
export function getRecentBoards(): RecentBoard[] {
  if (!isLocalStorageAvailable()) {
    return [];
  }

  try {
    const stored = localStorage.getItem(STORAGE_KEY);

    if (!stored) {
      return [];
    }

    const parsed = JSON.parse(stored);

    // Validate that parsed data is an array
    if (!Array.isArray(parsed)) {
      return [];
    }

    // Validate each item has required properties
    const validated = parsed.filter(
      (item): item is RecentBoard =>
        item &&
        typeof item === "object" &&
        typeof item.shareToken === "string" &&
        typeof item.name === "string" &&
        typeof item.visitedAt === "string"
    );

    // Sort by visitedAt descending (most recent first)
    return validated.sort(
      (a, b) =>
        new Date(b.visitedAt).getTime() - new Date(a.visitedAt).getTime()
    );
  } catch (error) {
    // Handle invalid JSON or other errors
    console.error("Error reading recent boards from localStorage:", error);
    return [];
  }
}

/**
 * Adds a board to the recent boards list
 * If the board already exists (same shareToken), updates its visitedAt timestamp and moves it to the top
 * Keeps only the 5 most recent boards
 * @param board - The board to add with shareToken and name
 */
export function addRecentBoard(board: {
  shareToken: string;
  name: string;
}): void {
  if (!isLocalStorageAvailable()) {
    return;
  }

  // Validate input
  if (!board || !board.shareToken || !board.name) {
    console.error("Invalid board data provided to addRecentBoard");
    return;
  }

  try {
    const recentBoards = getRecentBoards();

    // Remove existing entry with same shareToken (if exists)
    const filteredBoards = recentBoards.filter(
      (b) => b.shareToken !== board.shareToken
    );

    // Create new board entry with current timestamp
    const newBoard: RecentBoard = {
      shareToken: board.shareToken,
      name: board.name,
      visitedAt: new Date().toISOString(),
    };

    // Add new board at the beginning
    const updatedBoards = [newBoard, ...filteredBoards];

    // Keep only the most recent MAX_RECENT_BOARDS entries
    const trimmedBoards = updatedBoards.slice(0, MAX_RECENT_BOARDS);

    // Save to localStorage
    localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmedBoards));
  } catch (error) {
    console.error("Error adding recent board to localStorage:", error);
  }
}

/**
 * Clears all recent boards from localStorage
 */
export function clearRecentBoards(): void {
  if (!isLocalStorageAvailable()) {
    return;
  }

  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    console.error("Error clearing recent boards from localStorage:", error);
  }
}
