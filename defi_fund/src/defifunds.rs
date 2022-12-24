use scrypto::prelude::*;
use crate::fund::*;

blueprint! {


    struct Defifunds {
        funds: Vec<ComponentAddress>, //all funds in the dapp
        defifunds_admin_badge: ResourceAddress,
        whitelisted_pool_addresses: HashMap<ComponentAddress, U64>, //whitelist valid from epoch <u64>
        defifunds_deposit_fee: Decimal
    }

    impl Defifunds {

        pub fn instantiate_defifunds() -> (ComponentAddress, Bucket) {

            let defifunds_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "defifunds admin badge")
                .metadata("desciption", "Badge used for the admin stuff")
                .initial_supply(1);

            let access_rules = AccessRules::new()
                .method("new_pool_to_whitelist_all", rule!(require(defifunds_admin_badge.resource_address())),AccessRule::DenyAll)
                .method("remove_pool_from_whitelist_all", rule!(require(defifunds_admin_badge.resource_address())),AccessRule::DenyAll)
                .method("change_deposit_fee_admin_all", rule!(require(defifunds_admin_badge.resource_address())),AccessRule::DenyAll)
                .method("withdraw_collected_fee_admin_all", rule!(require(defifunds_admin_badge.resource_address())),AccessRule::DenyAll)
                .default(rule!(allow_all),AccessRule::DenyAll);

            let mut component = Self {
                funds: Vec::new(),
                defifunds_admin_badge: defifunds_admin_badge.resource_address(),
                whitelisted_pool_addresses: HashMap::new(),
                defifunds_deposit_fee: dec!(1)
            }
            .instantiate();
            component.add_access_check(access_rules);

            (component.globalize(), defifunds_admin_badge)
                
        }

        //////////////////////////
        ///methods for everyone///
        //////////////////////////
        
        pub fn new_fund(&mut self, fund_name: String, token: Bucket, deposit_fee: Decimal, initial_supply_share_tokens: Decimal) -> (Bucket, Bucket){
            let (fund, fund_manager_badge, share_tokens)=FundComponent::instantiate_fund(
                fund_name,
                token,
                initial_supply_share_tokens,
                deposit_fee,

                self.whitelisted_pool_addresses.clone(),
                self.defifunds_admin_badge.clone(),
                self.defifunds_deposit_fee.clone()

            )
            .into();
            self.funds.push(fund.into());

            (fund_manager_badge, share_tokens)
        }

        pub fn get_fund_addresses(&mut self) -> Vec<ComponentAddress>{
            self.funds.clone()
        }

        pub fn get_defifunds_deposit_fee(&mut self) -> Decimal{
            self.defifunds_deposit_fee
        }

        pub fn get_whitelisted_pool_addresses(&mut self) -> HashMap<ComponentAddress, U64>{
            self.whitelisted_pool_addresses.clone()
        }



        /////////////////////////////////
        ///methods for defifunds admin///
        /////////////////////////////////

        pub fn new_pool_to_whitelist_all(&mut self, pool_address: ComponentAddress){
            self.whitelisted_pool_addresses.insert(pool_address, (Runtime::current_epoch()+300).into()); //will only be valid after 300 epochs 7days ish.
            for &fund1 in self.funds.iter(){
                let fund: FundGlobalComponentRef=fund1.into();
                fund.new_pool_to_whitelist(pool_address);
            }
        }

        pub fn remove_pool_from_whitelist_all(&mut self, pool_address: ComponentAddress){
            self.whitelisted_pool_addresses.remove(&pool_address);
            for &fund1 in self.funds.iter(){
                let fund: FundGlobalComponentRef=fund1.into();
                fund.remove_pool_from_whitelist(pool_address)
            }
        }

        pub fn change_deposit_fee_admin_all(&mut self, new_fee: Decimal){
            assert!(new_fee >= dec!(0) && new_fee <= dec!(5),"Fee need to be in range of 0% to 5%.");
            self.defifunds_deposit_fee=new_fee;
            for &fund1 in self.funds.iter(){
                let fund: FundGlobalComponentRef=fund1.into();
                fund.change_deposit_fee_admin(new_fee);
            }
        }

        pub fn withdraw_collected_fee_admin_all(&mut self) -> Vec<Bucket>{
            let mut tokens= Vec::new();
            for &fund1 in self.funds.iter(){
                let fund: FundGlobalComponentRef=fund1.into();
                tokens.push(fund.withdraw_collected_fee_admin());
            }
            tokens
        }
    }
}

