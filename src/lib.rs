use stylus_sdk::stylus_proc::{entrypoint, external, sol_storage};

extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
use crate::serc721::{SERC721Details, SERC721};

mod serc721;

pub struct CustomNFTDetails;
impl SERC721Details for CustomNFTDetails {
    const NAME: &'static str = "StylusNFT";
    const SYMBOL: &'static str = "SNFT";
}

sol_storage! {
    #[entrypoint]
    pub struct CustomNFT {
        #[borrow]
        SERC721<CustomNFTDetails> my_custom_token;
        uint256 counter;
    }
}

#[external]
#[inherit(SERC721<CustomNFTDetails>)]
impl CustomNFT {
    // write your own custom mint burn and token uri functions
}
