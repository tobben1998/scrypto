//stuff to add
//not only xrd, but other crypots aswell to be more general

use scrypto::prelude::*;
use random::Random; 


#[derive(Clone, NonFungibleData, ScryptoSbor)]
pub struct NftData {
    #[mutable]clothes: String,
    #[mutable]eyes: String,
    #[mutable]mouth: String,
    #[mutable]ears: String,
    #[mutable]tail: String,
    #[mutable]hats: String,
    #[mutable]fur: String,
    #[mutable]hand: String,
    #[mutable]background: String,
    #[mutable]key_image_url: Url,
    #[mutable]nft_storage: Url,
}

#[blueprint]
mod nfts {
    // extern_blueprint!(
    //     // "package_tdx_2_1p527rqesssgtadvr23elxrnrt6rw2jnfa5ke8n85ykcxmvjt06cvv6",
    //     "package_sim1p5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycnnzj0hj",
    //     MyRandom as RandomComponent {
    //         fn request_random(&self, address: ComponentAddress, method_name: String, on_error: String,
    //             key: u32, badge_opt: Option<FungibleBucket>, expected_fee: u8) -> u32;

    //     }
    // );
    // const RNG: Global<RandomComponent> = global_component!(
    //     RandomComponent,
    //     // "component_tdx_2_1czzxynn4m4snhattvdf6knlyfs3ss70yufj975uh2mdhp8jes938sd"
    //     "component_sim1cqqqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycnf7v0gx"
    // );

    enable_method_auth! {
        methods {
            put_nonfungibledata => PUBLIC; //=> restrict_to: [OWNER];
            collected_crypto => PUBLIC; //=> restrict_to: [OWNER];
            random_buy => PUBLIC; //
            update_nonfungibledata => PUBLIC; //fails if badge not give through bucket. called by RandomComponent
            on_update_error => PUBLIC; //fails if badge not give through bucket. called by RandomComponent

        }
    }
    struct NftCollection {
        nft_price: Decimal, // the price for an nft
        nft_manager: ResourceManager,//the resource address off all nfts
        nft_id_counter: u32, // A counter for ID generationcoun
        nftdata_vec:Vec<NftData>,
        placeholder_nftdata:NftData,
        collected_crypto: FungibleVault, //A vault that collects all xrd payments
        number_of_nfts: u32,
        admin_badge: ResourceAddress,
        buying_badge_vault: Vault,
    }

    impl NftCollection {
        pub fn instantiate_component(
            name: String,
            description: String,
            tags: Vec<String>,
            icon_url: String,
            info_url: String,
            royalty: Decimal, //NB!! what is the standar for this. not include this, but have allowances instead?
            number_of_nfts: u32,
            price: Decimal,
            placeholder_nftdata: NftData
        ) -> (Global<NftCollection>, FungibleBucket) {

            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => format!("{} admin badge", name), locked;
                    }))
                .mint_initial_supply(1);  
            
            // controls actual buying. should be recallable, non-transferable, etc, but omitted for simplicity
            let nft_buying_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => format!("Buying badge for {}", name), locked;
                    }
                ))
                .mint_initial_supply(1000)
                .into();

            let nft =
                ResourceBuilder::new_integer_non_fungible::<NftData>(OwnerRole::Fixed(
                    rule!(require(admin_badge.resource_address()))
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(allow_all); //this component for update rule
                    non_fungible_data_updater_updater => rule!(allow_all); //this component for update rule
                ))
                .metadata(metadata!(
                    init {
                        "name" => name, locked;
                        "description" => description, locked;
                        "tags" => tags, updatable; 
                        "icon_url" => Url::of(icon_url), updatable;
                        "info_url" => Url::of(info_url), updatable;
                        "royalty" => royalty, locked; //NB!!!!what is the standard, just as metadata, or withdraw rules?
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(allow_all); //require admin badge on this for later
                    minter_updater => rule!(allow_all); //require admin badge on this for later
                ))
                .create_with_no_initial_supply();

            // Instantiate our component
            let component = Self {
                nft_price: price,
                nft_manager: nft,
                nft_id_counter: 0,
                nftdata_vec: Vec::new(),
                placeholder_nftdata: placeholder_nftdata,
                collected_crypto: FungibleVault::new(XRD),
                number_of_nfts: number_of_nfts,
                admin_badge: admin_badge.resource_address(),
                buying_badge_vault: Vault::with_bucket(nft_buying_badge),
            }
            .instantiate()
            .prepare_to_globalize(
                OwnerRole::Fixed(
                    rule!(require(admin_badge.resource_address())
                )
            ))
            .globalize();
            (component, admin_badge)
        }


        
        //puts the data for nfts into a vec
        pub fn put_nonfungibledata(&mut self, nftdata: NftData){
            if self.number_of_nfts>=self.nftdata_vec.len().try_into().unwrap(){ //only "number_of_nfts" is made. // usize compiles to u32
                self.nftdata_vec.push(nftdata);
            }
        }



        pub fn collected_crypto(&mut self) -> FungibleBucket {
            self.collected_crypto.take_all()
        }


        //buys an nft with placeholder metadata that will be changed by component it self.
        pub fn random_buy(&mut self, mut payment: FungibleBucket,) -> (FungibleBucket, NonFungibleBucket) {

            //consume payment
            self.collected_crypto.put(payment.take(self.nft_price));

            //paramters for request random
            let address = Runtime::global_component().address(); //this comp address
            let key = self.nft_id_counter; //dont use it for anything
            let method_name: String = "update_nonfungibledata".into(); //name on my method
            let on_error: String = "on_update_error".into(); //name on my method
            let badge = self.buying_badge_vault.take(Decimal::ONE); //badge used to protect method calls.
            let expected_fee = 6u8; // 6 cents = 1 XRD
            
            //requests a random number, and this method calls do_buy or abort_buy.
            //let _callback_id = RNG.request_random(address, method_name, on_error, key, Some(badge.as_fungible()), expected_fee);
            
            //mint the nft with placeholder metdadata
            let nft_bucket = self.nft_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.nft_id_counter.into()),self.placeholder_nftdata.clone(),
            ).as_non_fungible();


            //only possible to mint "number_of_nfts"
            self.nft_id_counter += 1;
            if self.nft_id_counter == self.number_of_nfts{ //only x is mintable 0,...,x-1
                self.nft_manager.set_mintable(AccessRule::DenyAll);
                self.nft_manager.lock_mintable();
            }
        
            (payment, nft_bucket)
        }
       

        // called by a RandomWatcher off-ledger service (through [RandomComponent]).
        pub fn update_nonfungibledata(&mut self, id: u32, badge: FungibleBucket, random_seed: Vec<u8>){

            //returns the buying badge. Fails if wrong badge
            assert!(badge.amount() == Decimal::ONE);
            self.buying_badge_vault.put(badge.into());

            //getting the random_number from the random seed
            let mut random: Random = Random::new(&random_seed);
            let random_number = random.roll::<usize>(self.nftdata_vec.len());//0...x-1 //usize compiles to u32

            //takes the nftdata
            let nft_data=self.nftdata_vec.swap_remove(random_number);
            
            //updates the data on the nft
            let u64id: u64=id.into();
            let nft_id: NonFungibleLocalId = u64id.into();
            self.nft_manager.update_non_fungible_data(&nft_id,"clothes",nft_data.clothes);
            self.nft_manager.update_non_fungible_data(&nft_id,"eyes",nft_data.eyes);
            self.nft_manager.update_non_fungible_data(&nft_id,"mouth",nft_data.mouth);
            self.nft_manager.update_non_fungible_data(&nft_id,"ears",nft_data.ears);
            self.nft_manager.update_non_fungible_data(&nft_id,"tail",nft_data.tail);
            self.nft_manager.update_non_fungible_data(&nft_id,"hats",nft_data.hats);
            self.nft_manager.update_non_fungible_data(&nft_id,"fur",nft_data.fur);
            self.nft_manager.update_non_fungible_data(&nft_id,"hand",nft_data.hand);
            self.nft_manager.update_non_fungible_data(&nft_id,"background",nft_data.background);
            self.nft_manager.update_non_fungible_data(&nft_id,"key_image_url",nft_data.key_image_url);
            self.nft_manager.update_non_fungible_data(&nft_id,"nft_storage",nft_data.nft_storage);

        }    


        // called by a RandomWatcher off-ledger service (through [RandomComponent]).
        pub fn on_update_error(&mut self, id: u32, badge: FungibleBucket){

            //returns the buying badge. Fails if wrong badge
            assert!(badge.amount() == Decimal::ONE);
            self.buying_badge_vault.put(badge.into());

            //TODO handle the error of an nft not beeing update. give informantion to , and try again
            //make another method that can be called to request a random number again and update call the update_metadat
        }





        //fail function. If a method and nft have metadata field="pending" then i can
        //requst random and update the nft with that.     



            
    }
}