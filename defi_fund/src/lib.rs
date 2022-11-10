//useful code

//see readme file to for how to make accounts, and call .rtm files
//https://github.com/radixdlt/scrypto-challenges/tree/main/1-exchanges/RaDEX

//i forhold til shareholder token
//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs

//veldig relevant se linje 422 for rounding errors i forhold til å deposite flere tokens
//https://github.com/radixdlt/scrypto-examples/blob/main/core/payment-splitter/src/lib.rs

use scrypto::prelude::*;

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
        //resource address fro the share token
        share_token_resource_address: ResourceAddress,
        //vault with share tokens to the creater of the Fund. 
        share_tokens_vault: Vault,
        //defined deposit fee,
        deposit_fee_percentage: Decimal,

    }

    impl Fund {

        //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        //!!let user put in intial supply here and amount of share tokens he want to be initial supply!!
        //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        pub fn instantiate_fund(token: Bucket, initial_supply_share_tokens: Decimal ) -> (ComponentAddress, Bucket) {

            //badge used for admin stuff
            let fund_manager_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "fund manager badge used for managing the fund, change fee, and collecting fee")
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
                .method("witdraw_collected_fee", rule!(require(fund_manager_badge.resource_address())))
                .default(rule!(allow_all));

            let mut vaults = HashMap::new();
            vaults.insert(token.resource_address(),Vault::new(token.resource_address()));
            vaults.get_mut(&token.resource_address()).unwrap().put(token);

            //instantiate the component (se the state)
            let mut component = Self {
                fund_manager_badge: fund_manager_badge.resource_address(),
                internal_admin_badge: Vault::with_bucket(internal_admin_badge),
                vaults: vaults,
                total_share_tokens: dec!(1000),
                share_token_resource_address: share_tokens.resource_address(),
                share_tokens_vault: Vault::with_bucket(share_tokens),
                deposit_fee_percentage: dec!(0) 
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

        //function for making stuff mor readable
        // pub fn get_vault(&mut self, resource_address: ResourceAddress){
        //      self.vaults.get_mut(&resource_address).unwrap();
        // }


        //need transaction mainfest to test this
        //deposit inn all tokens that is in the pool in the same ratio as the pool.

        //example
        //share tokens=1000
        //tokens in fund 10 btc 20 eth
        //buckets contain 1btc and 2.1eth. -> ratio=0.1for btc and 0.10 for eth. min ratio=0.10
        //it will then exist 11btc 22eth in fund. 1100 of share tokens, and he will get 100 sharetokens. (min ratio*share tokens)

        pub fn deposit_tokens_to_fund(&mut self, mut tokens: Vec<Bucket>) -> (Bucket, Vec<Bucket>) {

            //TODO
            //use assert to check for stuff, for example that correct address etc. Not sure if this is needed. you can test.

            //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
            //!!!!!må sikkert lenge til sånn get_mut og unwrap her som self.vaults!!!!!!!!!!!
            //om ikke så hadde jeg denne istad.
            //let mut ratio=tokens[0].amount()/self.vaults[&tokens[0].resource_address()].amount();

            //en annen feil kan også vare at jeg tar mer fra buckets enn det er pga en rounding error når jeg deler? må dette fikses?
            
            //sikker smart å lage helper function for get vault eventuell get_amount_from_vault? feks:
            //self.get_vault(tokens[0].resource_address()).amount();
            //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

            //calculate min_ratio to find out how much you should take from each bucket.
            //so there is enough to take, an the ratio in the pool remains the same. The rest should be given back
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
            let mut share_tokens = self
                .internal_admin_badge
                .authorize(|| resource_manager.mint(new_share_tokens));
            self.total_share_tokens += new_share_tokens;

            // deposit fee to the fund manager.
            let fee=(self.deposit_fee_percentage/dec!(100))*share_tokens.amount();
            self.share_tokens_vault.put(share_tokens.take(fee));

            //return the share tokens, and the rest of the funds that was not depoited into the fund.
            (share_tokens, tokens)
            //!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1noe feil her. acc2 får ikke share tokens som han skal
        }







        pub fn witdraw_tokens_from_fund(&mut self, share_tokens: Bucket) -> Vec<Bucket> {//-> Bucket {
            assert!(share_tokens.resource_address()==self.share_token_resource_address,"Wrong tokens sent. You need to send share tokens.");
            
            //take fund from vaults and put into a Vec<Bucket>
            let mut tokens = Vec::new();
            let your_share = share_tokens.amount()/self.total_share_tokens;
            for vault in self.vaults.values_mut(){
                tokens.push(vault.take(your_share));
            }

            //burn sharetokens
            self.total_share_tokens -= share_tokens.amount();
            let resource_manager = borrow_resource_manager!(self.share_tokens_vault.resource_address());
            self.internal_admin_badge.authorize(|| resource_manager.burn(share_tokens));

            tokens
        }

        //function that lets the fund manager withdraw a the share tokens he have got from the collected fee from the vault.
        pub fn witdraw_collected_fee(&mut self) -> Bucket{
            self.share_tokens_vault.take_all()
        }

        //function that lets the fund manager change the deposit fee.
        pub fn change_deposit_fee_percentage(&mut self, new_fee: Decimal){
            assert!(new_fee >= dec!(0) && new_fee <= dec!(5),"Fee need to be in range of 0% to 5%.");
            self.deposit_fee_percentage=new_fee;
        }


        //make .rtm files for testing the functions/methods you have made. 
        //https://docs.radixdlt.com/main/scrypto/transaction-manifest/specs.html

        //make a method that let fund manager trade assets on ociswap etc. with the funds in the fund.



    }
}

