use std::str::FromStr;
use solana_program::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;

pub const TICK_ARRAY_SEED: &str = "tick_array";
const REWARD_NUM: usize = 3;
const TICK_ARRAY_SIZE_USIZE: usize = 60;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TickState {
    pub tick: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_0_x64: u128,
    pub fee_growth_outside_1_x64: u128,
    pub reward_growths_outside_x64: [u128; REWARD_NUM],
    pub padding: [u32; 13],
}

impl TickState {
    pub const LEN: usize = 4  // tick
        + 16                 // liquidity_net
        + 16                 // liquidity_gross
        + 16                 // fee_growth_outside_0_x64
        + 16                 // fee_growth_outside_1_x64
        + (REWARD_NUM * 16)  // reward_growths_outside_x64
        + (13 * 4);          // padding (bytes)
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct TickArrayState {
    pub pool_id: Pubkey,
    pub start_tick_index: i32,
    pub ticks: [TickState; TICK_ARRAY_SIZE_USIZE],
    pub initialized_tick_count: u8,
    pub recent_epoch: u64,
    pub padding: [u8; 107],
}

impl TickArrayState {
    pub const LEN: usize = 32  // pool_id
        + 4                    // start_tick_index
        + (TICK_ARRAY_SIZE_USIZE * TickState::LEN)
        + 1                    // initialized_tick_count
        + 8                    // recent_epoch
        + 107;                 // padding

    pub fn from_account_data(data: &[u8]) -> Result<Self, String> {
        if data.len() < Self::LEN {
            return Err(format!("account data too short: {} < {}", data.len(), Self::LEN));
        }
        let mut pos = 0usize;

        // pool_id (32)
        let pool_id = {
            let bytes: [u8; 32] = data[pos..pos + 32].try_into().unwrap();
            pos += 32;
            Pubkey::new_from_array(bytes)
        };

        // start_tick_index (4, LE)
        let start_tick_index = {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&data[pos..pos + 4]);
            pos += 4;
            i32::from_le_bytes(arr)
        };

        // ticks (60 entries)
        let mut ticks_vec: Vec<TickState> = Vec::with_capacity(TICK_ARRAY_SIZE_USIZE);
        for _ in 0..TICK_ARRAY_SIZE_USIZE {
            // tick (i32)
            let mut tick_arr = [0u8; 4];
            tick_arr.copy_from_slice(&data[pos..pos + 4]);
            let tick = i32::from_le_bytes(tick_arr);
            pos += 4;

            // liquidity_net (i128)
            let mut liq_net_arr = [0u8; 16];
            liq_net_arr.copy_from_slice(&data[pos..pos + 16]);
            let liquidity_net = i128::from_le_bytes(liq_net_arr);
            pos += 16;

            // liquidity_gross (u128)
            let mut liq_gross_arr = [0u8; 16];
            liq_gross_arr.copy_from_slice(&data[pos..pos + 16]);
            let liquidity_gross = u128::from_le_bytes(liq_gross_arr);
            pos += 16;

            // fee_growth_outside_0_x64 (u128)
            let mut fee0_arr = [0u8; 16];
            fee0_arr.copy_from_slice(&data[pos..pos + 16]);
            let fee_growth_outside_0_x64 = u128::from_le_bytes(fee0_arr);
            pos += 16;

            // fee_growth_outside_1_x64 (u128)
            let mut fee1_arr = [0u8; 16];
            fee1_arr.copy_from_slice(&data[pos..pos + 16]);
            let fee_growth_outside_1_x64 = u128::from_le_bytes(fee1_arr);
            pos += 16;

            // reward_growths_outside_x64 ([u128; 3])
            let mut rewards = [0u128; REWARD_NUM];
            for r in rewards.iter_mut() {
                let mut b = [0u8; 16];
                b.copy_from_slice(&data[pos..pos + 16]);
                *r = u128::from_le_bytes(b);
                pos += 16;
            }

            // padding ([u32; 13])
            let mut padding = [0u32; 13];
            for p in padding.iter_mut() {
                let mut b = [0u8; 4];
                b.copy_from_slice(&data[pos..pos + 4]);
                *p = u32::from_le_bytes(b);
                pos += 4;
            }

            ticks_vec.push(TickState {
                tick,
                liquidity_net,
                liquidity_gross,
                fee_growth_outside_0_x64,
                fee_growth_outside_1_x64,
                reward_growths_outside_x64: rewards,
                padding,
            });
        }
        let ticks: [TickState; TICK_ARRAY_SIZE_USIZE] = ticks_vec
            .try_into()
            .map_err(|_| "internal: failed to build fixed-size ticks array".to_string())?;

        // initialized_tick_count (u8)
        let initialized_tick_count = data[pos];
        pos += 1;

        // recent_epoch (u64)
        let mut epoch_arr = [0u8; 8];
        epoch_arr.copy_from_slice(&data[pos..pos + 8]);
        let recent_epoch = u64::from_le_bytes(epoch_arr);
        pos += 8;

        // padding (107 bytes)
        let mut pad107 = [0u8; 107];
        pad107.copy_from_slice(&data[pos..pos + 107]);
        pos += 107;

        Ok(TickArrayState {
            pool_id,
            start_tick_index,
            ticks,
            initialized_tick_count,
            recent_epoch,
            padding: pad107,
        })
    }
}

#[test]
fn test_tick_array_state(){
    let pool_id = Pubkey::from_str("FNmhYF6BxcrtnF37BRNDDg2NU1PctaouTu98pAjToHGe").unwrap();
    let start_tick_index = -71340 ;

    let pub_key = key(pool_id, start_tick_index);
    println!("pub_key: {}", pub_key);
}

pub fn key(pool_id: Pubkey, start_tick_index: i32) -> Pubkey {
    Pubkey::find_program_address(
        &[
            TICK_ARRAY_SEED.as_bytes(),
            pool_id.as_ref(),
            &start_tick_index.to_be_bytes(),
        ],
        &practice::id(),
    )
    .0
}

#[test]
fn test_tick_array_state_2(){
    // get account info
    let rpc_client = RpcClient::new(
        "https://api.devnet.solana.com".to_string(),
    );
    let public_key = Pubkey::from_str("6kMhRzX7qmnQ8rYPd235g2bf73BBTHM9fjvVhNtuPmS1").unwrap();
    let account = rpc_client.get_account(&public_key).unwrap();
    println!("lamports: {} owner: {} data_len: {}", account.lamports, account.owner, account.data.len());

    // Decode TickArrayState
    let ta = TickArrayState::from_account_data(&account.data).unwrap();
    println!(
        "start_tick_index: {} initialized_tick_count: {} recent_epoch: {}",
        ta.start_tick_index,
        ta.initialized_tick_count,
        ta.recent_epoch
    );
    println!(
        "first tick: tick={} liquidity_net={} liquidity_gross={}",
        ta.ticks[0].tick,
        ta.ticks[0].liquidity_net,
        ta.ticks[0].liquidity_gross
    );
    println!(
        "second tick: tick={} liquidity_net={} liquidity_gross={}",
        ta.ticks[1].tick,
        ta.ticks[1].liquidity_net,
        ta.ticks[1].liquidity_gross
    );
}