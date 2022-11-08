//useful code

//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs

use scrypto::prelude::*;

//token fund share token


blueprint! {


    struct Fund {
        //vaults with all funds for the the Fund
        vaults: HashMap<ResourceAddress, Vault>, //all vaults with the different token-rri
        //resource defention for admin stuff
        fund_manager_badge: ResourceAddress,
        //Vault holding the admin badge, allowing the component to mint and burn new share tokens. 
        internal_admin_badge: Vault,
        //keep track of total coins
        total_share_tokens: Decimal,
        //vault with share tokens to the creater of the Fund. 
        share_tokens_vault: Vault,
    }

    impl Fund {

        pub fn instantiate_fund() -> (ComponentAddress, Bucket) {

            //badge used for admin stuff
            let fund_manager_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "fund manager badge")
                .initial_supply(1);

            
            //brukes foreløpig ikke
            //Create a minting authoity badge, that will be kept
            //inside the component to be able to mint
            let internal_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Internal admin badge")
                .metadata("desciption", "Badge that has the auhority to mint and burn share tokens")
                .initial_supply(1);

            //share tokens for showing what ashare a user have of the fund
            let share_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "share tokens")
                .metadata("description", "Tokens used to show what share of the fund you have")
                .mintable(rule!(require(internal_admin_badge.resource_address())),LOCKED)
                .burnable(rule!(require(internal_admin_badge.resource_address())),LOCKED)
                .initial_supply(1000);


            //access rules defined
            let access_rules = AccessRules::new()
                //.method("change_this typisk trade, og ta ut fra share vault", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            //instantiate the component (se the state)
            let mut component = Self {
                fund_manager_badge: fund_manager_badge.resource_address(),
                internal_admin_badge: Vault::with_bucket(internal_admin_badge),
                //xrd_vault: Vault::new(RADIX_TOKEN),
                vaults: HashMap::new(),
                total_share_tokens: dec!(1000),
                share_tokens_vault: Vault::with_bucket(share_tokens) 
            }
            .instantiate();
            component.add_access_check(access_rules);

            (component.globalize(),fund_manager_badge)
                
        }

        pub fn add_token_to_fund(&mut self, fund: Bucket){
            let resource_address=fund.resource_address();

            if !self.vaults.contains_key(&resource_address){
                //create vault
                let key=resource_address;
                let value=Vault::new(resource_address);
                self.vaults.insert(key,value);
            }
            //put funds in the vault with specified resource address.
            self.vaults.get_mut(&resource_address).unwrap().put(fund);
        }

        // pub fn get_vault(&mut self, resource_address: ResourceAddress){
        //      self.vaults.get_mut(&resource_address).unwrap();
        // }


        //need transaction mainfest to test this
        //deposit inn all tokens that is in the pool in the same ratio as the pool.
        pub fn deposit_token_to_fund(&mut self, mut tokens: Vec<Bucket>) -> (Bucket, Vec<Bucket>) {

            //find min_ratio between buckets and vaults to find out the value you should take from each bucket,
            //so there is enough to take, an the ratio in the pool remains the same. The rest should be given back


            //example
            //share tokens=1000
            //tokens in fund 10 btc 20 eth
            //buckets contain 1btc and 2.1eth. -> ratio=0.1for btc and 0.10 for eth. min ratio=0.10
            //it will then exist 11btc 22eth in fund. 1100 of share tokens, and he will get 100 sharetokens. (min ratio*share tokens)
            

            //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
            //!!!!!må sikkert lenge til sånn get_mut og unwrap her som self.vaults!!!!!!!!!!!
            //om ikke så hadde jeg denne istad.
            //let mut ratio=tokens[0].amount()/self.vaults[&tokens[0].resource_address()].amount();

            //en annen feil kan også vare at jeg tar mer fra buckets enn det er pga en rounding error når jeg deler? må dette fikses?
            
            //sikker lage helper function for get amount?
            //self.get_vault(tokens[0].resource_address()).amount();
            //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

            //calculate min_ratio to find out how much you shoudl take from each bucket.
            let mut ratio=tokens[0].amount()/self.vaults.get_mut(&tokens[0].resource_address()).unwrap().amount();
            let mut min_ratio=ratio;
            for token in &tokens{
                ratio=token.amount()/(self.vaults.get_mut(&token.resource_address()).unwrap().amount());
                if ratio<min_ratio{
                    min_ratio=ratio;   
                }
            }
            
            //take from buckets, and put them into the fund.
            for token in tokens.iter_mut(){
                let amount=min_ratio*(self.vaults.get_mut(&token.resource_address()).unwrap().amount());
                self.add_token_to_fund(token.take(amount));
            }

            //mint new sharetokens
            let new_share_tokens=min_ratio*self.total_share_tokens;
            let resource_manager = borrow_resource_manager!(self.share_tokens_vault.resource_address());
            let share_tokens = self
                .internal_admin_badge
                .authorize(|| resource_manager.mint(new_share_tokens));
            self.total_share_tokens += new_share_tokens;

            //return the share tokens, and the rest of the funds that was not depoited into the fund.
            (share_tokens, tokens)
        }

        // pub fn witdraw_from_fund(&mut self) -> Bucket {

        // }
    }
}


//veldig relevant se linje 422 for rounding errors i forhold til å deposite flere tokens
//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs