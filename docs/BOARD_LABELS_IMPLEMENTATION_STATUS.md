# Board Labels Implementation Status

## Overview
Migration from card-level labels to board-level labels system is **IN PROGRESS**.

## Completed Work ✅

### 1. Architecture & Design (100%)
- ✅ [`BOARD_LABELS_DESIGN.md`](BOARD_LABELS_DESIGN.md:1) - Complete architectural design document
- ✅ Database schema design (board_labels + card_labels junction table)
- ✅ API endpoint specifications
- ✅ Data migration strategy
- ✅ UI/UX specifications

### 2. Database Migration (100%)
- ✅ [`backend/migrations/20250115010904_migrate_to_board_labels.sql`](backend/migrations/20250115010904_migrate_to_board_labels.sql:1)
- Creates `board_labels` table
- Creates `card_labels` junction table
- Migrates existing data with automatic duplicate merging
- **Status**: Ready to run (tables don't exist yet, causing sqlx compile errors)

### 3. Backend Models (100%)
- ✅ [`backend/src/models/label.rs`](backend/src/models/label.rs:1)
  - `BoardLabel` model (replaces `Label`)
  - `CardLabel` model (junction table)
  - Full CRUD operations for both models
- ✅ [`backend/src/models/board.rs`](backend/src/models/board.rs:1)
  - Added `labels` field to `BoardWithRelations`
  - Updated to use `BoardLabel` type
- ✅ [`backend/src/models/mod.rs`](backend/src/models/mod.rs:1) - Updated exports

### 4. Backend Services (100%)
- ✅ [`backend/src/services/board_label_service.rs`](backend/src/services/board_label_service.rs:1)
  - Complete service for board label management
  - Label assignment/unassignment operations
- ✅ [`backend/src/services/mod.rs`](backend/src/services/mod.rs:1) - Updated exports
- ✅ Removed old `label_service.rs`

### 5. SSE Events (100%)
- ✅ [`backend/src/sse/events.rs`](backend/src/sse/events.rs:1)
  - `BoardLabelCreated`
  - `BoardLabelUpdated`
  - `BoardLabelDeleted`
  - `CardLabelAssigned`
  - `CardLabelUnassigned`

## Remaining Work 🚧

### Backend (Critical - Must complete before frontend)

#### 6. Label Handlers (0%)
File: `backend/src/handlers/label_handlers.rs` (needs complete rewrite)

**Required Changes:**
```rust
// New endpoints needed:
// GET    /boards/:boardId/labels          - List board labels
// POST   /boards/:boardId/labels          - Create board label
// PUT    /boards/labels/:labelId          - Update board label
// DELETE /boards/labels/:labelId          - Delete board label
// POST   /cards/:cardId/labels/:labelId   - Assign label to card
// DELETE /cards/:cardId/labels/:labelId   - Unassign label from card
```

**Current Status**: Old handlers still reference deleted `Label` type

#### 7. Router Configuration (0%)
File: `backend/src/handlers/mod.rs`

**Required Changes:**
- Update route configuration for new label endpoints
- Remove old card-level label routes
- Add new board-level label routes
- Add card label assignment routes

#### 8. Database Migration Execution (0%)
**Action Required**: Run the migration to create tables

```bash
cd backend
cargo sqlx migrate run
```

This will eliminate all the "relation does not exist" errors.

### Frontend (Not Started - 0%)

#### 9. TypeScript Types (0%)
File: `frontend/src/lib/types.ts`

**Required Changes:**
```typescript
// Update Label interface to BoardLabel
export interface BoardLabel {
  id: string;
  board_id: string;  // NEW
  name: string;
  color: string;
  created_at: string;
  updated_at: string;  // NEW
}

// Update Board interface
export interface Board {
  // ... existing fields
  labels?: BoardLabel[];  // NEW
}

// Card keeps labels array (now references board labels)
export interface Card {
  // ... existing fields
  labels?: BoardLabel[];  // References board labels
}
```

#### 10. API Client (0%)
File: `frontend/src/lib/api.ts`

**Required Changes:**
- Remove: `createLabel(cardId, name, color)`
- Add: `createBoardLabel(boardId, name, color)`
- Add: `updateBoardLabel(labelId, updates)`
- Add: `deleteBoardLabel(labelId)`
- Add: `getBoardLabels(boardId)`
- Add: `assignLabelToCard(cardId, labelId)`
- Add: `unassignLabelFromCard(cardId, labelId)`

#### 11. Board Store (0%)
File: `frontend/src/store/board-store.ts`

**Required Changes:**
- Add board labels to state
- Add actions for board label CRUD
- Update label assignment actions
- Handle SSE events for board labels

#### 12. use-board Hook (0%)
File: `frontend/src/hooks/use-board.ts`

**Required Changes:**
- Replace `createLabel` with `createBoardLabel`
- Add `updateBoardLabel`
- Add `deleteBoardLabel`
- Add `assignLabelToCard`
- Add `unassignLabelFromCard`

#### 13. Board Label Management Dialog (0%)
File: `frontend/src/components/dialogs/board-labels-dialog.tsx` (NEW)

**Features:**
- List all board labels
- Create new labels (name + color picker)
- Edit existing labels
- Delete labels (with confirmation)
- Show usage count per label

#### 14. Board Header Integration (0%)
File: `frontend/src/components/board/board.tsx`

**Required Changes:**
- Add "Manage Labels" button to board header
- Wire up to Board Label Management Dialog

#### 15. Card Edit Dialog Updates (0%)
File: `frontend/src/components/dialogs/edit-card-dialog.tsx`

**Required Changes:**
- Replace "Manage Labels" section
- Show available board labels as selectable chips
- Click to assign/unassign labels
- Remove manual label creation

#### 16. Update Manage Labels Dialog (0%)
File: `frontend/src/components/dialogs/manage-labels-dialog.tsx`

**Options:**
- **Option A**: Delete this file (replace with board labels dialog)
- **Option B**: Repurpose for card label assignment only

## Current Errors 🐛

All current errors are expected and fall into two categories:

### 1. Database Errors (Expected)
```
error returned from database: relation "board_labels" does not exist
error returned from database: relation "card_labels" does not exist
```
**Cause**: Migration hasn't run yet  
**Fix**: Run `cargo sqlx migrate run` in backend directory

### 2. Import Errors (Expected)
```
unresolved imports: Label, UpdateLabelInput, LabelService
no variant named LabelCreated found for enum SseEvent
```
**Cause**: Old handlers still reference deleted types  
**Fix**: Complete handler rewrite (task #6 above)

## Next Steps

### Immediate Priority (to unblock development):

1. **Run Database Migration**
   ```bash
   cd backend
   cargo sqlx migrate run
   ```

2. **Rewrite Label Handlers**
   - Create new board label endpoints
   - Create card label assignment endpoints
   - Use new `BoardLabelService`

3. **Update Router Configuration**
   - Configure new routes
   - Remove old routes

4. **Test Backend**
   - Verify all endpoints work
   - Test SSE events broadcast correctly

### After Backend is Complete:

5. **Frontend Types & API Client**
6. **Frontend State Management**
7. **UI Components**
8. **Integration Testing**
9. **Documentation**

## Testing Checklist

### Backend Testing
- [ ] Create board label
- [ ] Update board label
- [ ] Delete board label
- [ ] List board labels
- [ ] Assign label to card
- [ ] Unassign label from card
- [ ] SSE events broadcast correctly
- [ ] Migration preserves all existing data
- [ ] Duplicate labels merged correctly

### Frontend Testing
- [ ] Board labels display correctly
- [ ] Can manage labels from board header
- [ ] Can assign labels to cards
- [ ] Can unassign labels from cards
- [ ] Real-time updates work via SSE
- [ ] Labels show on cards correctly
- [ ] Error handling works

## Estimated Remaining Effort

- **Backend**: 2-3 hours (handlers + routing + testing)
- **Frontend**: 4-6 hours (types + stores + components + testing)
- **Total**: 6-9 hours

## Important Notes

1. **Database migration is one-way** - Once run, cannot be easily reversed
2. **Coordinate backend/frontend deployment** - Both must be updated together
3. **Test migration with sample data first** - Validate data preservation
4. **Database errors are normal** until migration runs
5. **Old `label_service.rs` has been deleted** - Use `BoardLabelService` instead

## Files Modified

### Backend
- ✅ `backend/migrations/20250115010904_migrate_to_board_labels.sql`
- ✅ `backend/src/models/label.rs`
- ✅ `backend/src/models/board.rs`
- ✅ `backend/src/models/mod.rs`
- ✅ `backend/src/services/board_label_service.rs`
- ✅ `backend/src/services/mod.rs`
- ✅ `backend/src/sse/events.rs`
- 🚧 `backend/src/handlers/label_handlers.rs` (needs rewrite)
- 🚧 `backend/src/handlers/mod.rs` (needs routing update)

### Frontend
- ⏳ All frontend files pending

### Documentation
- ✅ `BOARD_LABELS_DESIGN.md`
- ✅ `BOARD_LABELS_IMPLEMENTATION_STATUS.md`

## Success Criteria

Implementation will be considered complete when:
- [ ] All tests pass
- [ ] No compile errors
- [ ] Migration runs successfully
- [ ] All CRUD operations work for board labels
- [ ] Label assignment/unassignment works
- [ ] UI allows board-level label management
- [ ] Real-time updates work correctly
- [ ] No data loss from migration
- [ ] Documentation is updated