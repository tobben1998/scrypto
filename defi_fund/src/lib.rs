use scrypto::prelude::*;

//token fund share token


blueprint! {


    struct Fund {
        vaults: HashMap<ResourceAddress, Vault>, //all vaults with the different token-rri
        //xrd_vault: Vault,
        //For share tokens
        //resource defention for admin stuff
        admin_badge: ResourceAddress,
        //Vault holding the admin badge, allowing the component to mint new share tokens. 
        minting_authority: Vault,
        //keep track of total coins
        share_tokens_minted: u64,
    }

    impl Fund {

        pub fn instantiate_fund() -> (ComponentAddress, Bucket) {

            //badge used for admin stuff
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "admin badge")
                .initial_supply(1);

            //Create a minting authoity badge, that will be kept
            //inside the component to be able to mint
            let minting_authority: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "share minter authority")
                .metadata("desciption", "Badge that has theauhority to mint share tokens")
                .initial_supply(1);


            //access rules defined
            // tro kanskje jeg ikke trenger denne her
            let access_rules = AccessRules::new()
                //.method("change_this", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            //instantiate the component (se the state)
            let mut component = Self {
                admin_badge: admin_badge.resource_address(),
                minting_authority: Vault::with_bucket(minting_authority),
                //xrd_vault: Vault::new(RADIX_TOKEN),
                vaults: HashMap::new(),
                share_tokens_minted: 0,
            }
            .instantiate();
            component.add_access_check(access_rules);

            (component.globalize(),admin_badge)
                
        }

        pub fn test_add_token_fund(&mut self, fund: Bucket){
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

        // pub fn deposit_xrd_to_fund(&mut self, mut xrd: Bucket) -> Bucket {
        //     //bruke ociswap her slik at det blir likt forhold mellom tokensene.
        //     //eventuelt krev at bruker setter inn alle tokens her
        //     //let xrd_payment = xrd.take(all); //take all hwo to
        //     Vec<>
        //     self.xrd_vault.put(xrd);
        //     xrd
        // }

        //calculate how much of each
        // pub fn calculate(&mut self) -> HashMap<String, Decimal>{

        // }
        // pub fn witdraw_from_fund(&mut self) -> Bucket {

        // }
    }
}


//veldig relevant se linje 422 for rounding errors i forhold til å deposite flere tokens
//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs