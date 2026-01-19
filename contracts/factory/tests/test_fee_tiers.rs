mod common;

use soroban_sdk::Env;

#[test]
fn test_default_fee_tiers() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Check 0.05% tier
    let tier_5 = client.get_fee_tier(&5);
    assert!(tier_5.is_some());
    let tier = tier_5.unwrap();
    assert_eq!(tier.fee_bps, 5);
    assert_eq!(tier.tick_spacing, 10);
    assert_eq!(tier.enabled, true);
    
    // Check 0.30% tier
    let tier_30 = client.get_fee_tier(&30);
    assert!(tier_30.is_some());
    let tier = tier_30.unwrap();
    assert_eq!(tier.fee_bps, 30);
    assert_eq!(tier.tick_spacing, 60);
    assert_eq!(tier.enabled, true);
    
    // Check 1.00% tier
    let tier_100 = client.get_fee_tier(&100);
    assert!(tier_100.is_some());
    let tier = tier_100.unwrap();
    assert_eq!(tier.fee_bps, 100);
    assert_eq!(tier.tick_spacing, 200);
    assert_eq!(tier.enabled, true);
}

#[test]
fn test_add_custom_fee_tier() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Add custom 0.15% tier
    client.set_fee_tier(&15, &30, &true);
    
    let tier = client.get_fee_tier(&15);
    assert!(tier.is_some());
    let tier = tier.unwrap();
    assert_eq!(tier.fee_bps, 15);
    assert_eq!(tier.tick_spacing, 30);
    assert_eq!(tier.enabled, true);
}

#[test]
fn test_disable_fee_tier() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Disable 30bps tier
    client.set_fee_tier(&30, &60, &false);
    
    let tier = client.get_fee_tier(&30);
    assert!(tier.is_some());
    assert_eq!(tier.unwrap().enabled, false);
}

#[test]
fn test_modify_fee_tier_spacing() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    // Change tick spacing for 30bps tier
    client.set_fee_tier(&30, &120, &true);
    
    let tier = client.get_fee_tier(&30);
    assert!(tier.is_some());
    assert_eq!(tier.unwrap().tick_spacing, 120);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_invalid_tick_spacing_zero() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    client.set_fee_tier(&50, &0, &true); // Should fail: InvalidTickSpacing
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_invalid_tick_spacing_negative() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    client.set_fee_tier(&50, &-10, &true); // Should fail: InvalidTickSpacing
}

#[test]
fn test_nonexistent_fee_tier() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _) = common::setup_factory(&env);
    
    let tier = client.get_fee_tier(&999);
    assert!(tier.is_none());
}