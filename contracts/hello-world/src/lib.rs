#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Symbol, String, symbol_short, log};

// Structure to represent an Asset Lease
#[contracttype]
#[derive(Clone)]
pub struct Lease {
    pub lease_id: u64,
    pub asset_name: String,
    pub asset_type: String,       // "physical" or "digital"
    pub owner: String,
    pub lessee: String,
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
    pub is_returned: bool,
    pub amount_paid: i128,        // in XLM
}

// Lease status summary structure
#[contracttype]
#[derive(Clone)]
pub struct LeaseStatus {
    pub total_leases: u64,
    pub active_leases: u64,
    pub completed_leases: u64,
    pub pending_leases: u64, // reserved for future implementation
}

// Storage keys (must be 9 characters max)
const LEASE_COUNTER: Symbol = symbol_short!("LCOUNT"); // Lease count
const LEASE_STATUS: Symbol = symbol_short!("LSTATUS"); // Lease summary

// Enum to map lease IDs to Lease structs
#[contracttype]
pub enum LeaseBook {
    LeaseById(u64),
}

// Contract definition
#[contract]
pub struct AssetLeaseContract;

#[contractimpl]
impl AssetLeaseContract {
    // Create a new lease
    pub fn create_lease(
        env: Env,
        asset_name: String,
        asset_type: String,
        owner: String,
        lessee: String,
        duration_secs: u64,
        amount_paid: i128,
    ) -> u64 {
        let mut lease_count: u64 = env
            .storage()
            .instance()
            .get(&LEASE_COUNTER)
            .unwrap_or(0);
        lease_count += 1;

        let start_time = env.ledger().timestamp();
        let end_time = start_time + duration_secs;

        let lease = Lease {
            lease_id: lease_count,
            asset_name,
            asset_type,
            owner,
            lessee,
            start_time,
            end_time,
            is_active: true,
            is_returned: false,
            amount_paid,
        };

        // Store the new lease
        env.storage()
            .instance()
            .set(&LeaseBook::LeaseById(lease.lease_id), &lease);
        env.storage().instance().set(&LEASE_COUNTER, &lease_count);

        // Update lease status
        let mut status = Self::get_status(env.clone());
        status.total_leases += 1;
        status.active_leases += 1;
        env.storage().instance().set(&LEASE_STATUS, &status);

        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "New Lease Created with ID: {}", lease.lease_id);

        lease.lease_id
    }

    // Mark a lease as completed
    pub fn complete_lease(env: Env, lease_id: u64) {
        let mut lease: Lease = env
            .storage()
            .instance()
            .get(&LeaseBook::LeaseById(lease_id))
            .expect("Lease not found");

        if !lease.is_active || lease.is_returned {
            panic!("Lease is already completed or inactive");
        }

        lease.is_active = false;
        lease.is_returned = true;

        env.storage()
            .instance()
            .set(&LeaseBook::LeaseById(lease_id), &lease);

        let mut status = Self::get_status(env.clone());
        status.active_leases -= 1;
        status.completed_leases += 1;

        env.storage().instance().set(&LEASE_STATUS, &status);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Lease with ID: {} marked as completed", lease_id);
    }

    // Retrieve lease details by ID
    pub fn get_lease(env: Env, lease_id: u64) -> Lease {
        env.storage()
            .instance()
            .get(&LeaseBook::LeaseById(lease_id))
            .expect("Lease not found")
    }

    // Get lease statistics
    pub fn get_status(env: Env) -> LeaseStatus {
        env.storage().instance().get(&LEASE_STATUS).unwrap_or(LeaseStatus {
            total_leases: 0,
            active_leases: 0,
            completed_leases: 0,
            pending_leases: 0, // reserved
        })
    }
}
