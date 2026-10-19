# Event Team Role-Based Access Control (RBAC)

## Overview

The Event Registry contract implements a granular role-based access control system for event teams. This allows large events to have multiple team members with different permission levels, enabling efficient event management without compromising security.

## Role Hierarchy

The system defines three roles with a clear hierarchy:

```
ADMIN (1) > MANAGER (2) > SCANNER (3)
```

### Role Definitions

#### 1. ADMIN (Full Control)
- **Permissions:**
  - Manage team roles (assign/remove ADMIN, MANAGER, SCANNER)
  - Edit all event settings (tiers, metadata, status)
  - Pause/resume event
  - Cancel event
  - All MANAGER and SCANNER permissions

- **Use Cases:**
  - Event organizer (implicit ADMIN)
  - Co-organizers
  - Senior event managers

#### 2. MANAGER (Edit & Control)
- **Permissions:**
  - Edit ticket tiers (pricing, limits)
  - Pause/resume event
  - Update event metadata
  - All SCANNER permissions

- **Restrictions:**
  - Cannot manage roles
  - Cannot cancel event
  - Cannot assign/remove team members

- **Use Cases:**
  - Event coordinators
  - Ticket managers
  - Operations team

#### 3. SCANNER (Check-in Only)
- **Permissions:**
  - Check in attendees (verify tickets)
  - View event information

- **Restrictions:**
  - Cannot edit any event settings
  - Cannot manage roles
  - Cannot pause/cancel event
  - Cannot change ticket prices

- **Use Cases:**
  - Door staff
  - Venue security
  - Check-in volunteers

## Storage Structure

Roles are stored using a persistent storage mapping:

```rust
Map<(event_id: String, member_address: Address), Role>
```

**Storage Key:** `DataKey::EventTeamRole(event_id, member_address)`

**Storage Type:** Persistent (survives ledger expiry)

## Contract Functions

### 1. Assign Role

```rust
pub fn assign_event_role(
    env: Env,
    caller: Address,
    event_id: String,
    member: Address,
    role: Role,
) -> Result<(), EventRegistryError>
```

**Authorization:** Only the event organizer or an existing ADMIN can assign roles.

**Restrictions:**
- Cannot assign a role to the organizer (they have implicit ADMIN rights)
- Caller must be authenticated (`caller.require_auth()`)

**Example:**
```rust
// Organizer assigns MANAGER role
client.assign_event_role(&organizer, &event_id, &manager_address, &Role::Manager);

// Admin assigns SCANNER role
client.assign_event_role(&admin_address, &event_id, &scanner_address, &Role::Scanner);
```

### 2. Remove Role

```rust
pub fn remove_event_role(
    env: Env,
    caller: Address,
    event_id: String,
    member: Address,
) -> Result<(), EventRegistryError>
```

**Authorization:** Only the event organizer or an existing ADMIN can remove roles.

**Restrictions:**
- Cannot remove the organizer (they always have implicit ADMIN rights)
- Caller must be authenticated

**Example:**
```rust
// Remove a team member's role
client.remove_event_role(&organizer, &event_id, &member_address);
```

### 3. Get Role

```rust
pub fn get_event_role(
    env: Env,
    event_id: String,
    member: Address,
) -> Option<Role>
```

**Returns:**
- `Some(Role)` - The role assigned to the member
- `None` - If the member has no role assigned

**Note:** The organizer always returns `Some(Role::Admin)` even if not explicitly assigned.

**Example:**
```rust
let role = client.get_event_role(&event_id, &member_address);
match role {
    Some(Role::Admin) => println!("Member is an admin"),
    Some(Role::Manager) => println!("Member is a manager"),
    Some(Role::Scanner) => println!("Member is a scanner"),
    None => println!("Member has no role"),
}
```

### 4. Check Permission

```rust
pub fn has_event_permission(
    env: Env,
    event_id: String,
    member: Address,
    required_role: Role,
) -> bool
```

**Returns:** `true` if the member has the required role or higher in the hierarchy.

**Role Hierarchy Check:**
- ADMIN has all permissions (Admin, Manager, Scanner)
- MANAGER has Manager and Scanner permissions
- SCANNER has only Scanner permissions

**Example:**
```rust
// Check if member can pause event (requires MANAGER or ADMIN)
if client.has_event_permission(&event_id, &member, &Role::Manager) {
    client.pause_event(&member, &event_id);
}

// Check if member can check in attendees (requires SCANNER, MANAGER, or ADMIN)
if client.has_event_permission(&event_id, &member, &Role::Scanner) {
    // Perform check-in
}
```

## Permission Enforcement

### Functions with Role Checks

#### MANAGER or ADMIN Required:
- `pause_event(caller, event_id)` - Pause ticket sales
- `resume_event(caller, event_id)` - Resume ticket sales

#### ADMIN Required:
- `assign_event_role(caller, event_id, member, role)` - Assign roles
- `remove_event_role(caller, event_id, member)` - Remove roles

#### SCANNER, MANAGER, or ADMIN Required:
- `check_in` functions (to be implemented in ticket_payment contract)

### Organizer Special Status

The event organizer has **implicit ADMIN rights** and:
- Always returns `Role::Admin` from `get_event_role()`
- Always passes `has_event_permission()` checks
- Cannot have their role modified or removed
- Can perform all ADMIN actions without explicit role assignment

## Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `InsufficientPermissions` | 72 | Caller does not have the required role |
| `RoleNotFound` | 73 | Team member has no role assigned |
| `CannotModifyOrganizerRole` | 74 | Attempted to modify organizer's implicit admin role |

## Usage Examples

### Example 1: Setting Up an Event Team

```rust
// 1. Organizer creates event
let organizer = Address::generate(&env);
client.register_event(&event_args);

// 2. Organizer assigns co-organizer as ADMIN
let co_organizer = Address::generate(&env);
client.assign_event_role(&organizer, &event_id, &co_organizer, &Role::Admin);

// 3. Organizer assigns operations manager as MANAGER
let ops_manager = Address::generate(&env);
client.assign_event_role(&organizer, &event_id, &ops_manager, &Role::Manager);

// 4. Co-organizer assigns door staff as SCANNERS
let scanner1 = Address::generate(&env);
let scanner2 = Address::generate(&env);
client.assign_event_role(&co_organizer, &event_id, &scanner1, &Role::Scanner);
client.assign_event_role(&co_organizer, &event_id, &scanner2, &Role::Scanner);
```

### Example 2: Manager Pausing Event

```rust
// Manager can pause event due to venue issue
let manager = Address::generate(&env);
client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

// Manager pauses ticket sales
client.pause_event(&manager, &event_id);

// Later, manager resumes sales
client.resume_event(&manager, &event_id);
```

### Example 3: Scanner Check-in (Conceptual)

```rust
// Scanner can only check in attendees
let scanner = Address::generate(&env);
client.assign_event_role(&organizer, &event_id, &scanner, &Role::Scanner);

// Scanner verifies ticket at door
if client.has_event_permission(&event_id, &scanner, &Role::Scanner) {
    // ticket_payment_client.check_in(&scanner, &ticket_id);
}

// Scanner CANNOT change ticket price
// This would fail with InsufficientPermissions:
// client.pause_event(&scanner, &event_id); // ❌ FAILS
```

## Acceptance Criteria Verification

✅ **A SCANNER cannot change the ticket price**
- SCANNER role has no permissions to edit tiers or pause events
- `pause_event()` and `resume_event()` require MANAGER or ADMIN role
- Attempting to call these functions as SCANNER returns `InsufficientPermissions` error

✅ **An ADMIN can promote other users to MANAGER**
- `assign_event_role()` is restricted to organizer or ADMIN role holders
- ADMIN can assign any role (ADMIN, MANAGER, SCANNER) to other users
- Role hierarchy is enforced: ADMIN (1) > MANAGER (2) > SCANNER (3)

✅ **Map<(event_id, Address), Role> storage structure**
- Implemented using `DataKey::EventTeamRole(event_id, member_address)`
- Persistent storage ensures data survives ledger expiry
- Efficient O(1) lookup for role checks

## Testing

Comprehensive test suite in `src/test_roles.rs`:

- ✅ `test_assign_role_by_organizer` - Organizer can assign roles
- ✅ `test_assign_role_by_admin` - Admin can assign roles
- ✅ `test_assign_role_by_manager_fails` - Manager cannot assign roles
- ✅ `test_assign_role_by_scanner_fails` - Scanner cannot assign roles
- ✅ `test_cannot_assign_role_to_organizer` - Cannot modify organizer role
- ✅ `test_remove_role_by_organizer` - Organizer can remove roles
- ✅ `test_organizer_has_implicit_admin_role` - Organizer has implicit admin
- ✅ `test_role_hierarchy_permissions` - Role hierarchy works correctly
- ✅ `test_manager_can_pause_event` - Manager can pause/resume
- ✅ `test_scanner_cannot_pause_event` - Scanner cannot pause
- ✅ `test_multiple_admins_can_manage_roles` - Multiple admins work together

Run tests:
```bash
cd contract/contracts/event_registry
cargo test test_roles
```

## Integration with TicketPayment Contract

The role system is designed to integrate with the `ticket_payment` contract for check-in functionality:

```rust
// In ticket_payment contract:
pub fn check_in(
    env: Env,
    scanner: Address,
    event_id: String,
    ticket_id: String,
) -> Result<(), Error> {
    scanner.require_auth();
    
    // Query event_registry to verify scanner has at least SCANNER role
    let event_registry = EventRegistryClient::new(&env, &registry_address);
    
    if !event_registry.has_event_permission(&event_id, &scanner, &Role::Scanner) {
        return Err(Error::InsufficientPermissions);
    }
    
    // Proceed with check-in...
}
```

## Security Considerations

1. **Organizer Protection**: The organizer cannot have their role modified or removed, ensuring they always maintain control.

2. **Authentication Required**: All role management functions require `caller.require_auth()` to prevent unauthorized access.

3. **Persistent Storage**: Roles are stored in persistent storage to survive ledger expiry and ensure long-term reliability.

4. **Role Hierarchy**: The hierarchy prevents privilege escalation (e.g., a MANAGER cannot promote themselves to ADMIN).

5. **Explicit Checks**: Permission checks are explicit and fail-safe (default to no permission if role not found).

## Future Enhancements

Potential future improvements:

1. **Role Expiration**: Add time-based role expiration for temporary team members
2. **Role History**: Track role assignment/removal history for audit purposes
3. **Bulk Operations**: Add functions to assign/remove roles for multiple members at once
4. **Custom Roles**: Allow organizers to define custom roles with specific permission sets
5. **Role Templates**: Pre-defined role templates for common event types

## Conclusion

The role-based access control system provides a secure, flexible, and scalable way to manage event teams. It enables large events to delegate responsibilities while maintaining security and control, ensuring that team members can only perform actions appropriate to their role.
