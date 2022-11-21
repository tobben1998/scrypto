
use scrypto::prelude::*;
use crate::fund::*;

blueprint! {


    struct Defifunds {
        funds: Vec<ComponentAddress>, //all funds in the application
        defifunds_admin_badge: ResourceAddress,
        whitelisted_pool_addresses: Vec<ComponentAddress>,
        defifunds_deposit_fee: Decimal

    }

    impl Defifunds {

        pub fn instantiate_defifunds() -> (ComponentAddress, Bucket) {

            let defifunds_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "defifunds_admin_badge")
                .metadata("desciption", "Badge used for the admin stuff")
                .initial_supply(1);

            let access_rules = AccessRules::new()
                .method("new_pool_to_whitelist_all", rule!(require(defifunds_admin_badge.resource_address())))
                .method("change_deposit_fee_defifunds_all", rule!(require(defifunds_admin_badge.resource_address())))
                .method("withdraw_collected_fee_defifunds_all", rule!(require(defifunds_admin_badge.resource_address())))
                .default(rule!(allow_all));


            let mut component = Self {
                funds: Vec::new(),
                defifunds_admin_badge: defifunds_admin_badge.resource_address(),
                whitelisted_pool_addresses: Vec::new(),
                defifunds_deposit_fee: dec!(0)
            }
            .instantiate();
            component.add_access_check(access_rules);

            (component.globalize(), defifunds_admin_badge)
                
        }

        ////////////////////////////
        ///functions for everyone///
        //////////////////////////// 
        
        pub fn new_fund(&mut self, token: Bucket, initial_supply_share_tokens: Decimal) -> (Bucket, Bucket){
            
            let (fund, fund_manager_badge, share_tokens)=FundComponent::instantiate_fund(
                token,
                initial_supply_share_tokens,
                self.defifunds_admin_badge, //the resourceaddress of defifund admin
                self.defifunds_deposit_fee,
                self.whitelisted_pool_addresses.clone()
            )
            .into();
            self.funds.push(fund.into());

            (fund_manager_badge, share_tokens)
        }


        pub fn get_fund_addresses(&mut self) -> Vec<ComponentAddress>{
            let mut vec: Vec<ComponentAddress> = Vec::new();
            for fund in self.funds.iter_mut(){
                vec.push(*fund)
            }
            vec
        }



        //////////////////////////////////
        ///functions for defifund admin///
        ////////////////////////////////// 

        pub fn new_pool_to_whitelist_all(&mut self, pool_address: ComponentAddress){
            self.whitelisted_pool_addresses.push(pool_address);
            for fund in self.funds.iter_mut(){
                Into::<FundComponent>::into(*fund).new_pool_to_whitelist(pool_address);
            }
        }

        pub fn change_deposit_fee_defifunds_all(&mut self, new_fee: Decimal){
            self.defifunds_deposit_fee=new_fee;
            for fund in self.funds.iter_mut(){
                Into::<FundComponent>::into(*fund).change_deposit_fee_defifunds(new_fee);
            }
        }

        pub fn withdraw_collected_fee_defifunds_all(&mut self) -> Vec<Bucket>{
            let mut vec: Vec<Bucket> = Vec::new();
            for fund in self.funds.iter_mut(){
                vec.push(Into::<FundComponent>::into(*fund).withdraw_collected_fee_defifunds());
            }
            vec
        }



        //TODO 
        //need to change the commands for calling transactions


    }
}

