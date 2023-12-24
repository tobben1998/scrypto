//stuff to add
//not only xrd, but other crypots aswell to be more general

use scrypto::prelude::*;
use random::Random; 


#[derive(NonFungibleData, ScryptoSbor)]
pub struct NftData {
    clothes: String,
    eyes: String,
    mouth: String,
    ears: String,
    tail: String,
    hats: String,
    fur: String,
    hand: String,
    background: String,
    key_image_url: Url,
    nft_storage: Url,
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
            buy_nft => PUBLIC;
            mint_nft => PUBLIC; //=> restrict_to: [OWNER];
            collected_crypto => PUBLIC; //=> restrict_to: [OWNER];
            request_random_buy => PUBLIC; //
            do_buy => PUBLIC; //fails if badge not give through bucket. called by RandomComponent
            abort_buy => PUBLIC; //fails if badge not give through bucket. called by RandomComponent
            //send_nft => PUBLIC;
        }
    }
    struct NftCollection {
        nfts: NonFungibleVault, //a vault that holds all the nfts
        nft_price: Decimal, // the price for an nft
        nft_manager: ResourceManager,//the resource address off all nfts
        nft_id_counter: u64, // A counter for ID generationcoun
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
            price: Decimal
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
                nfts: scrypto::prelude::NonFungibleVault(nft.create_empty_vault()),
                nft_price: price,
                nft_manager: nft,
                nft_id_counter: 0,
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


        pub fn mint_nft(&mut self, nftdata: NftData){
            let nft_bucket = self.nft_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.nft_id_counter),nftdata,
            ).as_non_fungible();
            self.nfts.put(nft_bucket);

            self.nft_id_counter += 1;
            if self.nft_id_counter >= self.number_of_nfts.into(){ //only x is mintable 0,...,x-1
                self.nft_manager.set_mintable(AccessRule::DenyAll);
                self.nft_manager.lock_mintable();
            }
        }

        pub fn collected_crypto(&mut self) -> FungibleBucket {
            self.collected_crypto.take_all()
        }

        //buys a specific NFT //Should be delete.
        pub fn buy_nft(
            &mut self,
            key: NonFungibleLocalId,
            mut payment: FungibleBucket,
        ) -> (NonFungibleBucket, FungibleBucket) {

            //consume payment
            self.collected_crypto.put(payment.take(self.nft_price));

            // Take the requested NFT
            let nft = self.nfts.take_non_fungible(&key);

            // Return the NFT and change
            (nft, payment)
        }

        pub fn request_random_buy(&mut self, mut payment: FungibleBucket,) -> FungibleBucket{

            //consume payment
            self.collected_crypto.put(payment.take(self.nft_price));

            //paramters for request random
            let address = Runtime::global_component().address(); //this comp address
            let key:u32 = 0; //dont use it for anything
            let method_name: String = "do_buy".into(); //name on my method
            let on_error: String = "abort_buy".into(); //name on my method
            let badge = self.buying_badge_vault.take(Decimal::ONE); //badge used to protect method calls.
            // How much you would expect the callback to cost, cents (e.g. test on Stokenet).
            // It helps to avoid a sharp increase in royalties during the first few invocations of `request_random()`
            // but is completely optional.
            let expected_fee = 6u8; // 6 cents = 1 XRD
            
            //requests a random number, and this method calls do_buy or abort_buy.
            //let _callback_id = RNG.request_random(address, method_name, on_error, key, Some(badge.as_fungible()), expected_fee);
            
            payment
        }  

        // called by a RandomWatcher off-ledger service (through [RandomComponent]).
        pub fn abort_buy(&mut self, _nothing: u32, badge: FungibleBucket){

            //returns the buying badge. Fails if wrong badge
            assert!(badge.amount() == Decimal::ONE);
            self.buying_badge_vault.put(badge.into());
        }


        // called by a RandomWatcher off-ledger service (through [RandomComponent]).
        pub fn do_buy(&mut self, address: ComponentAddress, badge: FungibleBucket, random_seed: Vec<u8>){

            //returns the buying badge. Fails if wrong badge
            assert!(badge.amount() == Decimal::ONE);
            self.buying_badge_vault.put(badge.into());

            //getting the nft_id from the random seed
            let mut random: Random = Random::new(&random_seed);
            let nft_id = random.roll::<u64>(self.number_of_nfts.into());//0...x-1


            //1. take nft from vault with nft_id

            //2. call send_nft()

            //3. Take the optional<Bucket> and puts it back into the vault
            // can also use deposit_or_abort, but then I wont get my badge back.
        }    

        //trying to send the nft, but returns it if not depositable
        pub fn send_nft(bucket:NonFungibleBucket, address:ComponentAddress)->Option<NonFungibleBucket>{
            let comp: Global<AnyComponent> = Global::from(address);
            let bucket: Option<NonFungibleBucket> = comp.call::<(NonFungibleBucket,Option<ResourceOrNonFungible>),_>("try_deposit_or_refund", &(bucket, None));

            bucket
        }


            
    }
}