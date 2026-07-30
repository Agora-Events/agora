use crate::types::{EventRegistrationArgs, Role, TicketTier};
use crate::{EventRegistry, EventRegistryClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

fn setup_test_env() -> (Env, EventRegistryClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EventRegistry, ());
    let client = EventRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let platform_wallet = Address::generate(&env);
    let usdc_token = Address::generate(&env);

    client.initialize(&admin, &platform_wallet, &500, &usdc_token);

    (env, client, admin, platform_wallet, usdc_token)
}

fn create_test_event(
    env: &Env,
    client: &EventRegistryClient,
    organizer: &Address,
    event_id: String,
) {
    let payment_address = Address::generate(env);
    let metadata_cid = String::from_str(
        env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let mut tiers = Map::new(env);
    let tier = TicketTier {
        name: String::from_str(env, "General"),
        price: 1000,
        tier_limit: 100,
        current_sold: 0,
        is_refundable: true,
        auction_config: Vec::new(env),
        loyalty_multiplier: 1,
        max_per_user: 0,
    };
    tiers.set(String::from_str(env, "general"), tier);

    let args = EventRegistrationArgs {
        event_id: event_id.clone(),
        name: String::from_str(env, "Test Event"),
        organizer_address: organizer.clone(),
        payment_address,
        metadata_cid,
        max_supply: 100,
        milestone_plan: None,
        tiers,
        refund_deadline: 0,
        restocking_fee: 0,
        resale_cap_bps: None,
        min_sales_target: None,
        target_deadline: None,
        banner_cid: None,
        tags: None,
        category_ids: None,
        start_time: 0,
        is_private: false,
        end_time: 0,
        transfer_lock_duration: 0,
        accepted_tokens: Vec::new(env),
        use_global_whitelist: true,
        referral_rate_bps: None,
    };

    client.register_event(&args);
}

#[test]
fn test_assign_role_by_organizer() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_001");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Organizer assigns MANAGER role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

    // Verify role was assigned
    let role = client.get_event_role(&event_id, &manager);
    assert_eq!(role, Some(Role::Manager));
}

#[test]
fn test_assign_role_by_admin() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let admin_member = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "event_002");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Organizer assigns ADMIN role to admin_member
    client.assign_event_role(&organizer, &event_id, &admin_member, &Role::Admin);

    // Admin member assigns SCANNER role
    client.assign_event_role(&admin_member, &event_id, &scanner, &Role::Scanner);

    // Verify role was assigned
    let role = client.get_event_role(&event_id, &scanner);
    assert_eq!(role, Some(Role::Scanner));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_assign_role_by_manager_fails() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "event_003");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Organizer assigns MANAGER role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

    // Manager tries to assign SCANNER role - should fail
    client.assign_event_role(&manager, &event_id, &scanner, &Role::Scanner);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_assign_role_by_scanner_fails() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let another_user = Address::generate(&env);
    let event_id = String::from_str(&env, "event_004");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Organizer assigns SCANNER role
    client.assign_event_role(&organizer, &event_id, &scanner, &Role::Scanner);

    // Scanner tries to assign role - should fail
    client.assign_event_role(&scanner, &event_id, &another_user, &Role::Manager);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_cannot_assign_role_to_organizer() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_005");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Try to assign role to organizer - should fail
    client.assign_event_role(&organizer, &event_id, &organizer, &Role::Manager);
}

#[test]
fn test_remove_role_by_organizer() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_006");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign and then remove role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);
    client.remove_event_role(&organizer, &event_id, &manager);

    // Verify role was removed
    let role = client.get_event_role(&event_id, &manager);
    assert_eq!(role, None);
}

#[test]
fn test_organizer_has_implicit_admin_role() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let event_id = String::from_str(&env, "event_007");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Organizer should have implicit ADMIN role
    let role = client.get_event_role(&event_id, &organizer);
    assert_eq!(role, Some(Role::Admin));

    // Organizer should have admin permissions
    let has_permission = client.has_event_permission(&event_id, &organizer, &Role::Admin);
    assert!(has_permission);
}

#[test]
fn test_role_hierarchy_permissions() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let admin_member = Address::generate(&env);
    let manager = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "event_008");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign roles
    client.assign_event_role(&organizer, &event_id, &admin_member, &Role::Admin);
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);
    client.assign_event_role(&organizer, &event_id, &scanner, &Role::Scanner);

    // Test ADMIN permissions
    assert!(client.has_event_permission(&event_id, &admin_member, &Role::Admin));
    assert!(client.has_event_permission(&event_id, &admin_member, &Role::Manager));
    assert!(client.has_event_permission(&event_id, &admin_member, &Role::Scanner));

    // Test MANAGER permissions
    assert!(!client.has_event_permission(&event_id, &manager, &Role::Admin));
    assert!(client.has_event_permission(&event_id, &manager, &Role::Manager));
    assert!(client.has_event_permission(&event_id, &manager, &Role::Scanner));

    // Test SCANNER permissions
    assert!(!client.has_event_permission(&event_id, &scanner, &Role::Admin));
    assert!(!client.has_event_permission(&event_id, &scanner, &Role::Manager));
    assert!(client.has_event_permission(&event_id, &scanner, &Role::Scanner));
}

#[test]
fn test_manager_can_pause_event() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_009");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign MANAGER role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

    // Manager pauses event
    client.pause_event(&manager, &event_id);

    // Verify event is paused
    assert!(client.is_event_paused(&event_id));

    // Manager resumes event
    client.resume_event(&manager, &event_id);

    // Verify event is not paused
    assert!(!client.is_event_paused(&event_id));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_scanner_cannot_pause_event() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "event_010");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign SCANNER role
    client.assign_event_role(&organizer, &event_id, &scanner, &Role::Scanner);

    // Scanner tries to pause event - should fail
    client.pause_event(&scanner, &event_id);
}

#[test]
fn test_multiple_admins_can_manage_roles() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_011");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Organizer assigns two ADMIN roles
    client.assign_event_role(&organizer, &event_id, &admin1, &Role::Admin);
    client.assign_event_role(&organizer, &event_id, &admin2, &Role::Admin);

    // Both admins can assign roles
    client.assign_event_role(&admin1, &event_id, &manager, &Role::Manager);

    // Verify role was assigned
    let role = client.get_event_role(&event_id, &manager);
    assert_eq!(role, Some(Role::Manager));

    // Admin2 can remove the role
    client.remove_event_role(&admin2, &event_id, &manager);

    // Verify role was removed
    let role = client.get_event_role(&event_id, &manager);
    assert_eq!(role, None);
}

// Issue #682: Add event_registry unit test for team_role assignment and permission checks

#[test]
fn test_assign_manager_role() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_manager_test");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign Manager role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

    // Verify Manager role was assigned
    let role = client.get_event_role(&event_id, &manager);
    assert_eq!(role, Some(Role::Manager));
    
    // Verify Manager has Manager permissions
    assert!(client.has_event_permission(&event_id, &manager, &Role::Manager));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_scanner_cannot_edit_tiers() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let scanner = Address::generate(&env);
    let event_id = String::from_str(&env, "event_scanner_test");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign Scanner role
    client.assign_event_role(&organizer, &event_id, &scanner, &Role::Scanner);

    // Scanner tries to update event status - should fail with Unauthorized error
    // Scanners only have permission to scan tickets, not manage events
    client.update_event_status(&event_id, &false);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_manager_cannot_cancel_event() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_cancel_test");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign Manager role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

    // Manager tries to cancel event - should fail with Unauthorized error
    // Only organizer (Admin role) can cancel events
    client.cancel_event(&event_id);
}

#[test]
fn test_revoke_role() {
    let (env, client, _admin, _platform_wallet, _usdc_token) = setup_test_env();

    let organizer = Address::generate(&env);
    let manager = Address::generate(&env);
    let event_id = String::from_str(&env, "event_revoke_test");

    // Add organizer to whitelist
    client.add_organizer(&organizer);

    // Create event
    create_test_event(&env, &client, &organizer, event_id.clone());

    // Assign Manager role
    client.assign_event_role(&organizer, &event_id, &manager, &Role::Manager);

    // Verify Manager has permissions
    assert!(client.has_event_permission(&event_id, &manager, &Role::Manager));

    // Revoke Manager role
    client.remove_event_role(&organizer, &event_id, &manager);

    // Verify role was removed
    let role = client.get_event_role(&event_id, &manager);
    assert_eq!(role, None);

    // Verify Manager no longer has permissions
    assert!(!client.has_event_permission(&event_id, &manager, &Role::Manager));
}
