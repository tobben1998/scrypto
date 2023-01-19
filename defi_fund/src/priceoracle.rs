use scrypto::prelude::*;

blueprint! {
    struct Priceoracle {
        prices: HashMap<ResourceAddress, Decimal>,
    }

    impl Priceoracle {
        pub fn instantiate_priceoracle() -> ComponentAddress{
            let component = Self{
                prices: HashMap::new()
            }
            .instantiate()
            .globalize();
            component
        }

        pub fn set_price(&mut self,token_address: ResourceAddress, price: Decimal){
            if self.prices.contains_key(&token_address){
                *self.prices.get_mut(&token_address).unwrap() = price;
            }
            else{
                self.prices.insert(token_address, price);
            }
        }

        pub fn get_price(&self, token_address: ResourceAddress) -> Decimal{
            *self.prices.get(&token_address).unwrap()
        }
    }
}