use scrypto::prelude::*;
use crate::defifunds::*;

external_component! {
    BeakerfiComponentTarget {
        fn swap(
            &mut self,
            input: Bucket,
            output: ResourceAddress,
            slippage: Decimal,
            original_rate: Decimal
        ) -> Bucket;
    }
}

external_component! {
    BeakerfiPoolComponentTarget {
        fn swap(
            &mut self,
            input: Bucket,
            slippage: Decimal,
            original_rate: Decimal
        ) -> Bucket;
    }
}

#[blueprint]
mod fund_module{


    struct Fund {
        fund_name: String,
        short_description: String,
        image_link: String,
        website_link: String,
        vaults: HashMap<ResourceAddress, Vault>, //where all the tokens in the fund are stored
        fund_manager_badge: ResourceAddress, 
        internal_fund_badge: Vault,
        share_token: ResourceAddress,
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
            defifunds: ComponentAddress,
            short_description: String,
            image_link: String,
            website_link: String

        ) -> (ComponentAddress, Bucket, Bucket) {

            let fund_manager_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", format!("{} manager badge", fund_name))
                .metadata("description", format!("Badge used for managing {}.", fund_name))
                .mint_initial_supply(1);


            let internal_fund_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Internal fund badge")
                .metadata("description", "Badge that has the auhority to mint and burn share tokens.")
                .mint_initial_supply(1);


            let share_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("{} share tokens", fund_name))
                .metadata("description", format!("Tokens used to show what share of {} you have", fund_name))
                .mintable(rule!(require(internal_fund_badge.resource_address())), AccessRule::DenyAll)
                .burnable(rule!(require(internal_fund_badge.resource_address())), AccessRule::DenyAll)
                .mint_initial_supply(initial_supply_share_tokens);


            let access_rules = AccessRules::new()
                .method("change_deposit_fee_fund_manager", rule!(require(fund_manager_badge.resource_address())), AccessRule::DenyAll)
                .method("withdraw_collected_fee_fund_manager", rule!(require(fund_manager_badge.resource_address())), AccessRule::DenyAll)
                .method("trade_beakerfi", rule!(require(fund_manager_badge.resource_address())), AccessRule::DenyAll)
                .method("change_short_description", rule!(require(fund_manager_badge.resource_address())), AccessRule::DenyAll)
                .method("change_image_link", rule!(require(fund_manager_badge.resource_address())), AccessRule::DenyAll)
                .method("change_website_link", rule!(require(fund_manager_badge.resource_address())), AccessRule::DenyAll)
                .default(rule!(allow_all), AccessRule::DenyAll);

                
                
            let mut vaults = HashMap::new();
            vaults.insert(token.resource_address(),Vault::new(token.resource_address())); // adding a new vault to vaults: vaults<ResourceAddress, Vault>
            vaults.get_mut(&token.resource_address()).unwrap().put(token); //putting tokens in the vault


            let mut component = Self {
                fund_name: fund_name,
                short_description: short_description,
                image_link: image_link,
                website_link: website_link,
                fund_manager_badge: fund_manager_badge.resource_address(),
                internal_fund_badge: Vault::with_bucket(internal_fund_badge),
                vaults: vaults,
                total_share_tokens: initial_supply_share_tokens,
                share_token: share_tokens.resource_address(),
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
            let defifunds: DefifundsGlobalComponentRef=self.defifunds.into();

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

        //often used in combination with deposit to fund
        pub fn swap_token_for_tokens(&mut self, mut token: Bucket, ratios : Vec<(ResourceAddress, Decimal)>) -> Vec<Bucket>{
            let defifunds: DefifundsGlobalComponentRef=self.defifunds.into();
            let mut dex: BeakerfiComponentTarget = BeakerfiComponentTarget::at(defifunds.get_dex_address());
            
            let mut buckets = Vec::new();
            let token_amount=token.amount();

            for (i, (address, ratio)) in ratios.iter().enumerate(){

                //if last element swap the rest, to not recive dust becacause of rounding errors with decimal.
                if i==ratios.len()-1{
                    buckets.push(dex.swap(token, *address, Decimal::MAX,Decimal::ONE));
                    break;
                }

                let bucket_swapped=token.take((*ratio)*token_amount);
                buckets.push(dex.swap(bucket_swapped, *address, Decimal::MAX,Decimal::ONE)); 
            }
            buckets
        }


        //often used in combination with withdraw from fund
        pub fn swap_tokens_for_token(&mut self, tokens: Vec<Bucket>, token_address: ResourceAddress) -> Bucket{
            let defifunds: DefifundsGlobalComponentRef=self.defifunds.into();
            let mut dex: BeakerfiComponentTarget = BeakerfiComponentTarget::at(defifunds.get_dex_address());

            let mut bucket = Bucket::new(token_address);
            for token in tokens{
                bucket.put(dex.swap(token, token_address, Decimal::MAX,Decimal::ONE));
            }
            bucket
        }



        //I guess these three methods can be done offchain, since you do not need accurate answers here.
        //only the first one of these work here, Need the price orcal cmponent address taklen from a place for the
        //two nect methods
        //(get_token_amounts_get toal_value get_token_ratio)

        // pub fn get_token_amounts(&mut self) -> HashMap<ResourceAddress, Decimal>{
        //     let mut map=HashMap::new();
        //     for (address, vault) in self.vaults.iter_mut(){
        //         map.insert(*address, vault.amount());
        //     }
        //     map
        // }

        // pub fn get_total_value(&mut self) -> Decimal{

        //     let radiswap: RadiswapGlobalComponentRef = pool_address.into();
        //     let priceoracle_component: PriceoracleGlobalComponentRef = globalized_component.into();
        //     let total_value=dec!(0);
        //     for (address, vault) in self.vaults.into_iter(){
        //         total_value+=vault.amount()*PriceoracleComponent::get_price(address)
        //     }
        //     total_value
        // }

        // pub fn get_token_ratio(&mut self) -> HashMap<ResourceAddress, Decimal>{
        //     let map=HashMap::new();
        //     let total_value=self.get_total_value();
        //     for (address, vault) in self.vaults.into_iter(){
        //         let ratio=(vault.amount()*PriceoracleComponent::get_price(address))/(total_value);
        //         map.insert(address, ratio);
        //     map
        // }



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
        pub fn trade_beakerfi(&mut self, token_address: ResourceAddress, amount: Decimal, pool_address: ComponentAddress){
            
            //checks if the pool is whitelisted
            let mut whitelisted=false;
            let defifunds: DefifundsGlobalComponentRef= self.defifunds.into();
            for (&address, &epoch) in defifunds.get_whitelisted_pool_addresses().iter(){
                if address == pool_address && epoch <= Runtime::current_epoch(){
                    whitelisted=true;
                }
            }
            assert!(whitelisted, "Trading pool is not yet whitelisted.");

            //do a trade using beakerfi.
            let mut dexpool: BeakerfiPoolComponentTarget = BeakerfiPoolComponentTarget::at(pool_address);
            let bucket_before_swap=self.vaults.get_mut(&token_address).unwrap().take(amount);
            let bucket_after_swap=dexpool.swap(bucket_before_swap, Decimal::MAX,Decimal::ONE);
            info!("You traded {:?} {:?} for {:?} {:?}.", amount, token_address, bucket_after_swap.amount(), bucket_after_swap.resource_address());

            self.add_token_to_fund(bucket_after_swap);

        }

        pub fn change_short_description(&mut self, short_description: String){
            self.short_description=short_description;
        }

        pub fn change_image_link(&mut self, image_link: String){
            self.image_link=image_link;
        }

        pub fn change_website_link(&mut self, website_link: String){
            self.website_link=website_link;
        }

    }
}


