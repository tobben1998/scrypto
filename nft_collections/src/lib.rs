//stuff to add
//not only xrd, but other crypots aswell to be more general
//delte the first inital supply, just need a way to create a new vault withou the bucket fucntion, cant figure out new beacuse need a resource address

use scrypto::prelude::*;


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
    enable_method_auth! {
        methods {
            buy_nft => PUBLIC;
            mint_nft => PUBLIC; //=> restrict_to: [OWNER];
            collected_crypto => PUBLIC; //=> restrict_to: [OWNER];
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



        //buy from the component, make this random and so you can't se output
        pub fn buy_nft(
            &mut self,
            key: NonFungibleLocalId,
            mut payment: FungibleBucket,
        ) -> (NonFungibleBucket, FungibleBucket) {

            //let remaining_nfts=self.nfts.non_fungible_local_ids(self.number_of_nfts); //get all ids remaining in the vault
            //let key=take a random of these nfts

            self.collected_crypto.put(payment.take(self.nft_price)); // get paid

            // Take the requested NFT
            let nft = self.nfts.take_non_fungible(&key);

            // Return the NFT and change
            (nft, payment)
        }


        pub fn mint_nft(&mut self, nftdata: NftData){

            let nft_bucket = self.nft_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.nft_id_counter),nftdata,
/*                 NftData {
                    clothes: "hoody with headset",
                    eyes: "happy with glasses",
                    mouth: smile with ball,
                    ears: normal with earpods,
                    tail: normal,
                    hats: bucket,
                    fur: yellow,
                    hand: coffe,
                    background: blue,
                    nft_storage: Url::of("https://google.com"),
                    key_image_url: Url::of("https://pyro-public.s3.eu-central-1.amazonaws.com/collections/1/JPG_640px/Pyro_2.jpg")
                }, */
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
    }
}