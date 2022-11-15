//useful code

//see readme file to for how to make accounts, and call .rtm files
//https://github.com/radixdlt/scrypto-challenges/tree/main/1-exchanges/RaDEX

//i forhold til shareholder token
//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs

//veldig relevant se linje 422 for rounding errors i forhold til å deposite flere tokens
//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs

//docs fro creating transactions
//https://docs.radixdlt.com/main/scrypto/transaction-manifest/specs.html

//would use the external package I know what blueprint ociswap will use, but just use the internal for now with radiswap.
//https://github.com/radixdlt/scrypto-examples/tree/main/core/cross-blueprint-call

//for the trading fucntion on Radiswap 
//https://github.com/radixdlt/scrypto-challenges/blob/main/3-lending/degenfi/src/degenfi.rs
//line 419-427

use scrypto::prelude::*;
use crate::radiswap::*;

blueprint! {


    struct Fund {
        //vaults with all funds for the the Fund
        vaults: HashMap<ResourceAddress, Vault>,
        //resource defention for admin stuff
        fund_manager_badge: ResourceAddress,
        //Vault holding the admin badge, allowing the component to mint and burn new share tokens. 
        internal_admin_badge: Vault,
        //keep track of total coins
        total_share_tokens: Decimal,
        //vault with share tokens to the creater of the Fund. 
        share_tokens_vault: Vault,
        //defined deposit fee,
        deposit_fee_percentage: Decimal,
        // whitelisted pools to trade with.
        whitelisted_pool_addresses: Vec<ComponentAddress>

    }

    impl Fund {

        pub fn instantiate_fund(token: Bucket, initial_supply_share_tokens: Decimal ) -> (ComponentAddress, Bucket, Bucket) {

            //badge used for admin stuff
            let fund_manager_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "fund manager badge")
                .metadata("desciption", "Badge used for managing the fund, change fee and collecting fees")
                .initial_supply(1);


            //internal badge used for minting and burning share tokens
            let internal_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Internal admin badge")
                .metadata("desciption", "Badge that has the auhority to mint and burn share tokens")
                .initial_supply(1);

            //share tokens for showing what share a user have of the fund
            let share_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "share tokens")
                .metadata("description", "Tokens used to show what share of the fund you have")
                .mintable(rule!(require(internal_admin_badge.resource_address())),LOCKED)
                .burnable(rule!(require(internal_admin_badge.resource_address())),LOCKED)
                .initial_supply(initial_supply_share_tokens);

            //access rules defined
            let access_rules = AccessRules::new()
                .method("change_deposit_fee_percentage", rule!(require(fund_manager_badge.resource_address())))
                .method("withdraw_collected_fee", rule!(require(fund_manager_badge.resource_address())))
                .method("trade_radiswap", rule!(require(fund_manager_badge.resource_address())))
                .default(rule!(allow_all));

            let mut vaults = HashMap::new();
            vaults.insert(token.resource_address(),Vault::new(token.resource_address()));
            vaults.get_mut(&token.resource_address()).unwrap().put(token);

            //instantiate the component (se the state)
            let mut component = Self {
                fund_manager_badge: fund_manager_badge.resource_address(),
                internal_admin_badge: Vault::with_bucket(internal_admin_badge),
                vaults: vaults,
                total_share_tokens: initial_supply_share_tokens,
                share_tokens_vault: Vault::new(share_tokens.resource_address()),
                deposit_fee_percentage: dec!(0),
                whitelisted_pool_addresses: Vec::new()
            }
            .instantiate();
            component.add_access_check(access_rules);

            (component.globalize(),fund_manager_badge, share_tokens)
                
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

        
        //function for depositing tokens to the fund. You need to deposit each token that excist in the pool.
        //tokens will be taken in the same ratio as the pool has, and the rest of the tokens will be returned back to you.
        pub fn deposit_tokens_to_fund(&mut self, mut tokens: Vec<Bucket>) -> (Bucket, Vec<Bucket>) {

            //calculate min_ratio to find out how much you should take from each bucket.
            //so there is enough to take, an the ratio in the pool remains the same. The rest will be given back
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

            info!("min_ratio (tokens_deposited/(tokens_in_vault): {:?}", min_ratio);
            info!("total share tokens before: {:?}", self.total_share_tokens);

            //mint new sharetokens
            let new_share_tokens=min_ratio*self.total_share_tokens;
            self.total_share_tokens += new_share_tokens;
            let resource_manager = borrow_resource_manager!(self.share_tokens_vault.resource_address());
            let mut share_tokens = self
                .internal_admin_badge
                .authorize(|| resource_manager.mint(new_share_tokens));

            //deposit fee to the fund manager.
            let fee=(self.deposit_fee_percentage/dec!(100))*share_tokens.amount();
            self.share_tokens_vault.put(share_tokens.take(fee));
            
            info!("returned share tokens: {:?}", share_tokens.amount());
            info!("share tokens fee: {:?}", fee);
            info!("total share tokens after: {:?}", self.total_share_tokens);
            

            (share_tokens, tokens)
        }




        //function that witdraw tokens from the fund relative to how much sharetokens you put into the function.
        pub fn witdraw_tokens_from_fund(&mut self, share_tokens: Bucket) -> Vec<Bucket> {

            assert!(share_tokens.resource_address()==self.share_tokens_vault.resource_address(),"Wrong tokens sent. You need to send share tokens.");
            
            //take fund from vaults and put into a Vec<Bucket>
            let mut tokens = Vec::new();
            let your_share = share_tokens.amount()/self.total_share_tokens;
            for vault in self.vaults.values_mut(){
                info!("witdrew {:?} {:?}", your_share*vault.amount(), vault.resource_address());
                tokens.push(vault.take(your_share*vault.amount()));
            }

            //burn sharetokens
            self.total_share_tokens -= share_tokens.amount();
            let resource_manager = borrow_resource_manager!(self.share_tokens_vault.resource_address());
            self.internal_admin_badge.authorize(|| resource_manager.burn(share_tokens));
            

            tokens
        }

        //function that lets the fund manager withdraw a the share tokens he have got from the collected fee from the vault.
        pub fn withdraw_collected_fee(&mut self) -> Bucket{
            info!("witdrew {:?} sharetokens from vault.", self.share_tokens_vault.amount());
            self.share_tokens_vault.take_all()
        }

        //function that lets the fund manager change the deposit fee.
        pub fn change_deposit_fee_percentage(&mut self, new_fee: Decimal){
            assert!(new_fee >= dec!(0) && new_fee <= dec!(5),"Fee need to be in range of 0% to 5%.");
            self.deposit_fee_percentage=new_fee;
            info!("Deposit fee updated to: {:?}%", self.deposit_fee_percentage);

        }

        //This function lets the fund manager trade with all the funds assest on whitelisted pools.
        //token_address is the asset you want to trade from.
        pub fn trade_radiswap(&mut self, token_address: ResourceAddress, amount: Decimal, pool_address: ComponentAddress){
            //check that the pool_address is whitelisted
            assert!(self.whitelisted_pool_addresses.iter().any(|&i| i==pool_address));

            //do a trade using radiswap.
            let radiswap: RadiswapComponent = pool_address.into();
            let bucket_before_swap=self.vaults.get_mut(&token_address).unwrap().take(amount);
            let bucket_after_swap=radiswap.swap(bucket_before_swap);

            self.add_token_to_fund(bucket_after_swap);

        }

        //whitelisted pool addresses //only owner of defiFunds should be able to change this.
        pub fn new_whitelisted_pool(&mut self, pool_address: ComponentAddress){
            self.whitelisted_pool_addresses.push(pool_address);
        }


        //libs

        //funds. keep track of all funds
        //fund   (this will make use of radiswap to trade)
        //a swapping . forexample radiswap. 
        //



    }
}

