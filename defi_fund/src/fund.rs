use scrypto::prelude::*;
use crate::radiswap::*;
use crate::defifunds::*;

blueprint! {


    struct Fund {
        fund_name: String, 
        vaults: HashMap<ResourceAddress, Vault>, //where all the tokens in the fund are stored
        fund_manager_badge: ResourceAddress, 
        internal_fund_badge: Vault,
        total_share_tokens: Decimal,
        fees_fund_manager_vault: Vault,
        deposit_fee_fund_manager: Decimal,
        defifunds: ComponentAddress, //defifunds ComponentAddress to get access to whitelist and defifund deposit fee

    }

    impl Fund {

        pub fn instantiate_fund(
            fund_name: String,
            token: Bucket, 
            initial_supply_share_tokens: Decimal,
            defifunds: ComponentAddress
        ) -> (ComponentAddress, Bucket, Bucket) {

            let fund_manager_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Fund manager badge")
                .metadata("desciption", "Badge used for managing the fund.")
                .initial_supply(1);


            let internal_fund_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Internal fund badge")
                .metadata("desciption", "Badge that has the auhority to mint and burn share tokens.")
                .initial_supply(1);


            let share_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "hare tokens")
                .metadata("description", "Tokens used to show what share of the fund you have")
                .mintable(rule!(require(internal_fund_badge.resource_address())),LOCKED)
                .burnable(rule!(require(internal_fund_badge.resource_address())),LOCKED)
                .initial_supply(initial_supply_share_tokens);


            let access_rules = AccessRules::new()
                .method("change_deposit_fee_fund_manager", rule!(require(fund_manager_badge.resource_address())))
                .method("withdraw_collected_fee_fund_manager", rule!(require(fund_manager_badge.resource_address())))
                .method("trade_radiswap", rule!(require(fund_manager_badge.resource_address())))
                .default(rule!(allow_all));

                
                
            let mut vaults = HashMap::new();
            vaults.insert(token.resource_address(),Vault::new(token.resource_address())); // adding a new vault to vaults: vaults<ResourceAddress, Vault>
            vaults.get_mut(&token.resource_address()).unwrap().put(token); //putting tokens in the vault


            let mut component = Self {
                fund_name: fund_name,
                fund_manager_badge: fund_manager_badge.resource_address(),
                internal_fund_badge: Vault::with_bucket(internal_fund_badge),
                vaults: vaults,
                total_share_tokens: initial_supply_share_tokens,
                fees_fund_manager_vault: Vault::new(share_tokens.resource_address()),
                deposit_fee_fund_manager: dec!(0),
                defifunds: defifunds
            }
            .instantiate();
            component.add_access_check(access_rules);

            (component.globalize(),fund_manager_badge, share_tokens)
                
        }


        ////////////////////
        ///helper method////
        //////////////////// 

        fn add_token_to_fund(&mut self, token: Bucket){
            let resource_address=token.resource_address();
            
            //create a new vault if not exsisting
            if !self.vaults.contains_key(&resource_address){
                let key=resource_address;
                let value=Vault::new(resource_address);
                self.vaults.insert(key,value);
            }
            //put token in the vault with specified resource address.
            self.vaults.get_mut(&resource_address).unwrap().put(token);
        }



        
        //////////////////////////
        ///methods for everyone///
        //////////////////////////


        //method for depositing tokens to the fund. You need to deposit each token that exists in the pool.
        //tokens will be taken in the same ratio as the pool has, and the rest of the tokens will be returned back to you.
        pub fn deposit_tokens_to_fund(&mut self, mut tokens: Vec<Bucket>) -> (Bucket, Vec<Bucket>) {

            //calculate min_ratio to find out how much you should take from each bucket,
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

            //mint new sharetokens
            let new_share_tokens=min_ratio*self.total_share_tokens;
            self.total_share_tokens += new_share_tokens;
            let resource_manager = borrow_resource_manager!(self.fees_fund_manager_vault.resource_address());
            let mut share_tokens = self
                .internal_fund_badge
                .authorize(|| resource_manager.mint(new_share_tokens));

            //deposit fee to the fund manager and to defifunds
            let defifunds: DefifundsComponent=self.defifunds.into();

            let fee_fund_manager=(self.deposit_fee_fund_manager/dec!(100))*share_tokens.amount();
            let fee_defifunds=(defifunds.get_defifunds_deposit_fee()/dec!(100))*share_tokens.amount();

            self.fees_fund_manager_vault.put(share_tokens.take(fee_fund_manager));
            defifunds.add_token_to_fee_vaults(share_tokens.take(fee_defifunds));
            
            info!("Returned share tokens: {:?}.", share_tokens.amount());
            info!("share tokens fee: {:?}.", fee_fund_manager+fee_defifunds);
      

            (share_tokens, tokens)
        }



        //method that withdraw tokens from the fund relative to how much sharetokens you put into the method.
        pub fn withdraw_tokens_from_fund(&mut self, share_tokens: Bucket) -> Vec<Bucket> {
            assert!(share_tokens.resource_address()==self.fees_fund_manager_vault.resource_address(),"Wrong tokens sent. You need to send share tokens.");
            
            //take fund from vaults and put into a Vec<Bucket> called tokens
            let mut tokens = Vec::new();
            let your_share = share_tokens.amount()/self.total_share_tokens;
            for vault in self.vaults.values_mut(){
                info!("Withdrew {:?} {:?}.", your_share*vault.amount(), vault.resource_address());
                tokens.push(vault.take(your_share*vault.amount()));
            }

            //burn sharetokens
            self.total_share_tokens -= share_tokens.amount();
            let resource_manager = borrow_resource_manager!(self.fees_fund_manager_vault.resource_address());
            self.internal_fund_badge.authorize(|| resource_manager.burn(share_tokens));
            

            tokens
        }



        //////////////////////////////
        ///methods for fund manager///
        ////////////////////////////// 


        pub fn withdraw_collected_fee_fund_manager(&mut self) -> Bucket{
            info!("Withdrew {:?} sharetokens from vault.", self.fees_fund_manager_vault.amount());
            self.fees_fund_manager_vault.take_all()
        }

        pub fn change_deposit_fee_fund_manager(&mut self, new_fee: Decimal){
            assert!(new_fee >= dec!(0) && new_fee <= dec!(5),"Fee need to be in range of 0% to 5%.");
            self.deposit_fee_fund_manager=new_fee;
            info!("Deposit fee updated to: {:?}%.", self.deposit_fee_fund_manager);

        }

        //This method lets the fund manager trade with all the funds assests on whitelisted pools.
        //token_address is the asset you want to trade from.
        pub fn trade_radiswap(&mut self, token_address: ResourceAddress, amount: Decimal, pool_address: ComponentAddress){
            
            //checks if the pool is whitelisted
            let mut whitelisted=false;
            let defifunds: DefifundsComponent= self.defifunds.into();
            for (&address, &epoch) in defifunds.get_whitelisted_pool_addresses().iter(){
                if address == pool_address && epoch <= Runtime::current_epoch(){
                    whitelisted=true;
                }
            }
            assert!(whitelisted, "Trading pool is not yet whitelisted.");

            //do a trade using radiswap.
            let radiswap: RadiswapComponent = pool_address.into();
            let bucket_before_swap=self.vaults.get_mut(&token_address).unwrap().take(amount);
            let bucket_after_swap=radiswap.swap(bucket_before_swap);
            info!("You traded {:?} {:?} for {:?} {:?}.", amount, token_address, bucket_after_swap.amount(), bucket_after_swap.resource_address());

            self.add_token_to_fund(bucket_after_swap);

        }


    }
}


