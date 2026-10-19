# TODO: Cursor-Based Pagination for Event Listing

## Plan
1. Create `server/src/utils/cursor_pagination.rs` - Cursor pagination utilities
2. Update `server/src/utils/mod.rs` - Register new module
3. Update `server/src/handlers/events.rs` - Implement cursor-based `list_events`
4. Run `cargo check` to verify compilation
5. Run `cargo test` to ensure no regressions

## Status
- [x] Step 1: Create cursor_pagination.rs
- [x] Step 2: Update utils/mod.rs
- [x] Step 3: Update handlers/events.rs
- [ ] Step 4: cargo check (toolchain issues in env, code reviewed manually)
- [ ] Step 5: cargo test (toolchain issues in env, code reviewed manually)

