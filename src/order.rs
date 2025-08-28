use bincode::{Decode, Encode};
use borsh::{BorshDeserialize, BorshSerialize};
use rand::{Rng, RngCore};

/// A simple place-order like struct for demonstration purposes.
#[derive(Debug, Clone, Decode, Encode, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Order {
    pub id: u64,
    pub market: MarketId,
    pub side: Side,
    pub price: f64, // Would be better to have a specific type for price (maybe fixed point depending on market)
    pub quantity: f64, // Would be better to have a specific type for quantity (maybe fixed point depending on base asset)
}

impl Order {
    pub fn new(id: u64, market: MarketId, side: Side, price: f64, quantity: f64) -> Self {
        Self {
            id,
            market,
            side,
            price,
            quantity,
        }
    }

    /// Create a random order
    pub fn random_order() -> Self {
        let mut rng = rand::rng();

        let side = if rng.random_bool(0.5) {
            Side::Bid
        } else {
            Side::Ask
        };

        Self::new(
            rng.next_u64(),
            MarketId::SolUsd,
            side,
            rng.random::<f64>(),
            rng.random::<f64>(),
        )
    }
}

#[derive(Debug, Clone, Decode, Encode, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum MarketId {
    SolUsd,
}

#[derive(Debug, Clone, Decode, Encode, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Side {
    Bid,
    Ask,
}
