//beakerfi addresses
//component_tdx_b_1qtmzcwxvcvv7audnqgpchnxqphfmhvvujefwfs3a6auqyps8e3




use scrypto::prelude::*;
use crate::fund::*;


blueprint! {


    struct Defifunds {
        funds: Vec<ComponentAddress>, //all funds in the dapp
        defifunds_admin_badge: ResourceAddress,
        whitelisted_pool_addresses: HashMap<ComponentAddress, u64>, //whitelist valid from epoch <u64>
        defifunds_deposit_fee: Decimal,
        fee_vaults: HashMap<ResourceAddress, Vault>,
        set_component_badge: ResourceAddress, //used for the work around
        component_address: Option<ComponentAddress>, //component address of self. A work around for 0.7.0
        price_oracle: Option<ComponentAddress>,
        beakerfi: ComponentAddress
    }

    impl Defifunds {

        pub fn instantiate_defifunds(beakerfi: ComponentAddress) -> (ComponentAddress, Bucket) {

            let defifunds_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "defifunds admin badge")
                .metadata("desciption", "Badge used for the admin stuff")
                .initial_supply(1);

            //used for workaround for 0.7.0 to get it selves component address
            let set_component_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "set component badge")
                .metadata("desciption", "used in 0.7.0 because not possible to get it selves component address")
                .burnable(rule!(allow_all), AccessRule::DenyAll)
                .initial_supply(1);

            let access_rules = AccessRules::new()
                .method("new_pool_to_whitelist", rule!(require(defifunds_admin_badge.resource_address())), AccessRule::DenyAll)
                .method("remove_pool_from_whitelist", rule!(require(defifunds_admin_badge.resource_address())), AccessRule::DenyAll)
                .method("change_deposit_fee_defifunds", rule!(require(defifunds_admin_badge.resource_address())), AccessRule::DenyAll)
                .method("withdraw_collected_fee_defifunds", rule!(require(defifunds_admin_badge.resource_address())), AccessRule::DenyAll)
                .method("withdraw_collected_fee_defifunds_all", rule!(require(defifunds_admin_badge.resource_address())), AccessRule::DenyAll)
                .default(rule!(allow_all), AccessRule::DenyAll);

            let mut component = Self {
                funds: Vec::new(),
                defifunds_admin_badge: defifunds_admin_badge.resource_address(),
                whitelisted_pool_addresses: HashMap::new(),
                defifunds_deposit_fee: dec!(1),
                fee_vaults: HashMap::new(),
                set_component_badge: set_component_badge.resource_address(),
                component_address: None,
                price_oracle:None,
                beakerfi: beakerfi
            }
            .instantiate();
            component.add_access_check(access_rules);
            let globalized_component=component.globalize();
            let defifunds_component: DefifundsGlobalComponentRef = globalized_component.into();
            defifunds_component.set_address(globalized_component, set_component_badge); //workaround to get component address.

            (globalized_component, defifunds_admin_badge)
                
        }

        //helper method for 0.7.0 to set component address. Can only be used when instatiating, because no one get the badge.
        pub fn set_address(&mut self, address: ComponentAddress, badge: Bucket){
            assert_eq!(badge.resource_address(), self.set_component_badge, "The badge is only accasable when instantiating a new component, so no need to call this method.");
            self.component_address = Some(address);
            badge.burn();
        }

        pub fn set_oracle_address(&mut self, address: ComponentAddress){
            self.price_oracle = Some(address);
        }

        //fund make use of this method to deposit the fee to the correct vault
        //if other people decide to use this method it is just free money to the defifunds admin :D
        pub fn add_token_to_fee_vaults(&mut self, token: Bucket){
            let resource_address=token.resource_address();
            
            if !self.fee_vaults.contains_key(&resource_address){
                let key=resource_address;
                let value=Vault::new(resource_address);
                self.fee_vaults.insert(key,value);
            }

            self.fee_vaults.get_mut(&resource_address).unwrap().put(token);
        }

        //////////////////////////
        ///methods for everyone///
        //////////////////////////
        
        pub fn new_fund(&mut self, 
            fund_name: String, 
            token: Bucket, 
            initial_supply_share_tokens: Decimal, 
            short_description: String,
            image_link: String,
            website_link: String
        ) -> (Bucket, Bucket){
            let (fund, fund_manager_badge, share_tokens)=FundComponent::instantiate_fund(
                fund_name,
                token,
                initial_supply_share_tokens,
                self.component_address.unwrap(), //component address of Defifunds
                short_description,
                image_link,
                website_link
            )
            .into();
            self.funds.push(fund.into());

            (fund_manager_badge, share_tokens)
        }

        pub fn get_fund_addresses(&mut self) -> Vec<ComponentAddress>{
            self.funds.clone()
        }

        pub fn get_dex_address(&mut self) -> ComponentAddress{
            self.beakerfi.clone()
        }

        pub fn get_defifunds_deposit_fee(&mut self) -> Decimal{
            self.defifunds_deposit_fee
        }

        pub fn get_whitelisted_pool_addresses(&mut self) -> HashMap<ComponentAddress, u64>{
            self.whitelisted_pool_addresses.clone()
        }



        ////////////////////////////////
        ///methods for defifund admin///
        ////////////////////////////////

        pub fn new_pool_to_whitelist(&mut self, pool_address: ComponentAddress){
            self.whitelisted_pool_addresses.insert(pool_address, Runtime::current_epoch());//+300); //will only be valid after 300 epochs 7days ish. changed since set-current-epoch is not working in 0.7.0
        }

        pub fn remove_pool_from_whitelist(&mut self, pool_address: ComponentAddress){
            self.whitelisted_pool_addresses.remove(&pool_address);
        }

        pub fn change_deposit_fee_defifunds(&mut self, new_fee: Decimal){
            self.defifunds_deposit_fee=new_fee;
        }

        pub fn withdraw_collected_fee_defifunds(&mut self, address: ResourceAddress) -> Bucket{
            self.fee_vaults.get_mut(&address).unwrap().take_all()
        }
        pub fn withdraw_collected_fee_defifunds_all(&mut self) -> Vec<Bucket>{
            let mut tokens = Vec::new();
            for vault in self.fee_vaults.values_mut(){
                tokens.push(vault.take_all());
            }
            tokens
        }



    }
}

