#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol, symbol_short};

/// On-chain record for a single harvest deposit. Each harvest is owned by
/// the farmer that registered it and tracks how many yield tokens have been
/// minted, sold, and at what price.
#[contracttype]
#[derive(Clone)]
pub struct Harvest {
    pub farmer: Address,
    pub weight_kg: u32,
    pub quality: u32,       // 1 - 100 quality score
    pub total_tokens: u32,  // yield tokens minted against the deposit
    pub sold_tokens: u32,   // yield tokens already purchased by distributors
    pub price_per_token: u32,
}

/// `FarmYieldToken` tokenizes the future sale of an agricultural harvest.
///
/// Flow:
///   1. A farmer calls `deposit_harvest` to register proof of harvest.
///   2. The farmer calls `mint_yield` to issue yield tokens that represent
///      a claim on the eventual sale of the harvest.
///   3. A distributor calls `purchase_yield` to buy yield tokens; the
///      payment is tracked on-chain as proceeds owed to the farmer.
///   4. The farmer calls `claim_proceeds` to collect the accumulated
///      payment after distributors have bought the yield tokens.
#[contract]
pub struct FarmYieldToken;

#[contractimpl]
impl FarmYieldToken {
    /// Register a proof of harvest on-chain. The farmer must authorize
    /// the call. Returns the `harvest_id` that was stored.
    pub fn deposit_harvest(
        env: Env,
        farmer: Address,
        harvest_id: u32,
        weight_kg: u32,
        quality: u32,
    ) -> u32 {
        farmer.require_auth();

        if weight_kg == 0 {
            panic!("weight must be positive");
        }
        if quality == 0 || quality > 100 {
            panic!("quality must be between 1 and 100");
        }

        let key = symbol_short!("harvests");
        let mut harvests: Map<u32, Harvest> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));

        if harvests.contains_key(harvest_id) {
            panic!("harvest_id already registered");
        }

        let record = Harvest {
            farmer: farmer.clone(),
            weight_kg,
            quality,
            total_tokens: 0,
            sold_tokens: 0,
            price_per_token: 0,
        };

        harvests.set(harvest_id, record);
        env.storage().instance().set(&key, &harvests);

        harvest_id
    }

    /// Mint `tokens` yield tokens against a previously deposited harvest
    /// and set the per-token sale price. Only the original depositing
    /// farmer may mint, and each harvest may only be minted once.
    /// Returns the number of tokens minted.
    pub fn mint_yield(
        env: Env,
        farmer: Address,
        harvest_id: u32,
        tokens: u32,
        price_per_token: u32,
    ) -> u32 {
        farmer.require_auth();

        if tokens == 0 {
            panic!("tokens must be positive");
        }
        if price_per_token == 0 {
            panic!("price_per_token must be positive");
        }

        let key = symbol_short!("harvests");
        let mut harvests: Map<u32, Harvest> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));

        let mut record = match harvests.get(harvest_id) {
            Some(r) => r,
            None => panic!("harvest not found"),
        };

        if record.farmer != farmer {
            panic!("only the depositing farmer can mint yield");
        }
        if record.total_tokens > 0 {
            panic!("yield already minted for this harvest");
        }

        record.total_tokens = tokens;
        record.price_per_token = price_per_token;
        harvests.set(harvest_id, record);
        env.storage().instance().set(&key, &harvests);

        tokens
    }

    /// Distributor purchases `tokens` yield tokens of a harvest. The
    /// distributor must authorize the call. The corresponding sale
    /// proceeds (`price_per_token * tokens`) are credited to the farmer
    /// for later claiming. Returns the proceeds amount in contract
    /// accounting units (no real XLM is moved).
    pub fn purchase_yield(
        env: Env,
        distributor: Address,
        harvest_id: u32,
        tokens: u32,
    ) -> u32 {
        distributor.require_auth();

        if tokens == 0 {
            panic!("tokens must be positive");
        }

        let key = symbol_short!("harvests");
        let mut harvests: Map<u32, Harvest> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));

        let mut record = match harvests.get(harvest_id) {
            Some(r) => r,
            None => panic!("harvest not found"),
        };

        if record.total_tokens == 0 {
            panic!("yield has not been minted yet");
        }
        if record.sold_tokens + tokens > record.total_tokens {
            panic!("not enough yield tokens available");
        }

        record.sold_tokens += tokens;
        let amount = record.price_per_token * tokens;
        let farmer = record.farmer.clone();
        harvests.set(harvest_id, record);
        env.storage().instance().set(&key, &harvests);

        // Credit proceeds to the farmer.
        let pkey = symbol_short!("proceeds");
        let mut proceeds: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&pkey)
            .unwrap_or_else(|| Map::new(&env));

        let current = proceeds.get(farmer.clone()).unwrap_or(0);
        proceeds.set(farmer, current + amount);
        env.storage().instance().set(&pkey, &proceeds);

        amount
    }

    /// Farmer claims all accumulated sale proceeds and resets the
    /// balance to zero. The farmer must authorize the call. Returns
    /// the amount claimed.
    pub fn claim_proceeds(env: Env, farmer: Address) -> u32 {
        farmer.require_auth();

        let pkey = symbol_short!("proceeds");
        let mut proceeds: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&pkey)
            .unwrap_or_else(|| Map::new(&env));

        let amount = proceeds.get(farmer.clone()).unwrap_or(0);
        if amount == 0 {
            panic!("no proceeds to claim");
        }

        proceeds.set(farmer, 0);
        env.storage().instance().set(&pkey, &proceeds);

        amount
    }

    /// View helper: returns the number of yield tokens still available
    /// for purchase on a given harvest (total_tokens - sold_tokens).
    pub fn get_harvest(env: Env, harvest_id: u32) -> u32 {
        let key = symbol_short!("harvests");
        let harvests: Map<u32, Harvest> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));

        let record = match harvests.get(harvest_id) {
            Some(r) => r,
            None => panic!("harvest not found"),
        };

        record.total_tokens - record.sold_tokens
    }

    /// View helper: returns `true` if the harvest has had yield minted
    /// and all minted tokens have been purchased by distributors.
    pub fn is_sold_out(env: Env, harvest_id: u32) -> bool {
        let key = symbol_short!("harvests");
        let harvests: Map<u32, Harvest> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));

        let record = match harvests.get(harvest_id) {
            Some(r) => r,
            None => panic!("harvest not found"),
        };

        record.total_tokens > 0 && record.sold_tokens >= record.total_tokens
    }
}
