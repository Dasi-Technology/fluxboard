/**
 * Board password management using localStorage
 *
 * When a user creates a board, the password is automatically saved to localStorage.
 * This allows the board owner to lock/unlock the board without re-entering the password.
 */

const STORAGE_KEY = "fluxboard_board_passwords";

interface BoardPasswordEntry {
  shareToken: string;
  password: string;
  savedAt: string;
}

/**
 * Get all stored board passwords
 */
function getAllPasswords(): BoardPasswordEntry[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];
    return JSON.parse(stored);
  } catch (error) {
    console.error("Error reading board passwords from localStorage:", error);
    return [];
  }
}

/**
 * Save all board passwords
 */
function saveAllPasswords(passwords: BoardPasswordEntry[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(passwords));
  } catch (error) {
    console.error("Error saving board passwords to localStorage:", error);
  }
}

/**
 * Save a board password to localStorage
 *
 * @param shareToken - The board's share token
 * @param password - The board's password
 */
export function saveBoardPassword(shareToken: string, password: string): void {
  const passwords = getAllPasswords();

  // Remove any existing entry for this board
  const filtered = passwords.filter((entry) => entry.shareToken !== shareToken);

  // Add new entry
  filtered.push({
    shareToken,
    password,
    savedAt: new Date().toISOString(),
  });

  saveAllPasswords(filtered);
}

/**
 * Get a board password from localStorage
 *
 * @param shareToken - The board's share token
 * @returns The board's password, or null if not found
 */
export function getBoardPassword(shareToken: string): string | null {
  const passwords = getAllPasswords();
  const entry = passwords.find((p) => p.shareToken === shareToken);
  return entry?.password || null;
}

/**
 * Check if a board password is stored in localStorage
 *
 * @param shareToken - The board's share token
 * @returns True if the password is stored, false otherwise
 */
export function hasBoardPassword(shareToken: string): boolean {
  return getBoardPassword(shareToken) !== null;
}

/**
 * Remove a board password from localStorage
 *
 * @param shareToken - The board's share token
 */
export function removeBoardPassword(shareToken: string): void {
  const passwords = getAllPasswords();
  const filtered = passwords.filter((entry) => entry.shareToken !== shareToken);
  saveAllPasswords(filtered);
}

/**
 * Clear all stored board passwords
 */
export function clearAllBoardPasswords(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    console.error("Error clearing board passwords from localStorage:", error);
  }
}

/**
 * Check if the user is the owner of a board (has the password)
 *
 * @param shareToken - The board's share token
 * @returns True if the user has the board password, false otherwise
 */
export function isBoardOwner(shareToken: string): boolean {
  return hasBoardPassword(shareToken);
}
